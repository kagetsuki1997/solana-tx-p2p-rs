use std::{
    io,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use futures_util::{future::BoxFuture, stream::select_all, FutureExt, StreamExt};
use tokio::{
    signal::unix::{signal, SignalKind},
    sync::watch,
    task::JoinHandle,
    time::sleep,
};
use tokio_stream::wrappers::SignalStream;

#[derive(Clone, Copy, Debug)]
pub enum ShutdownState {
    Initial,
    WaitForSignal,
    ShuttingDown,
    Aborting,
}

impl Default for ShutdownState {
    #[inline]
    fn default() -> Self { Self::Initial }
}

#[allow(clippy::copy_iterator)]
impl Iterator for ShutdownState {
    type Item = Self;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match *self {
            Self::Initial => {
                *self = Self::WaitForSignal;
            }
            Self::WaitForSignal => {
                *self = Self::ShuttingDown;
            }
            Self::ShuttingDown => {
                *self = Self::Aborting;
            }
            Self::Aborting => return None,
        }

        Some(*self)
    }
}

#[derive(Clone)]
pub struct ShutdownSignal {
    shutdown_signal_receiver: watch::Receiver<()>,
    stopped: Arc<AtomicBool>,
}

impl ShutdownSignal {
    pub async fn wait(&mut self) {
        if !self.stopped.load(Ordering::Relaxed) {
            drop(self.shutdown_signal_receiver.changed().await);
            self.stopped.store(true, Ordering::Relaxed);
        }
    }
}

pub struct SignalHandler {
    shutdown_signal_receiver: watch::Receiver<()>,
    handle: JoinHandle<Result<(), io::Error>>,
}

impl SignalHandler {
    #[must_use]
    pub fn shutdown_signal(&self) -> ShutdownSignal {
        ShutdownSignal {
            shutdown_signal_receiver: self.shutdown_signal_receiver.clone(),
            stopped: Arc::new(AtomicBool::default()),
        }
    }

    pub fn stop(self) {
        self.handle.abort();

        tracing::info!("Shutdown signal handler stopped");
    }
}

pub struct SignalHandleBuilder {
    custom_signal: Option<BoxFuture<'static, ()>>,
}

impl SignalHandleBuilder {
    #[must_use]
    pub fn new(custom_signal: Option<BoxFuture<'static, ()>>) -> Self { Self { custom_signal } }

    #[must_use]
    pub fn start(self) -> SignalHandler {
        let (shutdown_signal_sender, shutdown_signal_receiver) = watch::channel(());
        let handle = tokio::spawn(Self::handle(shutdown_signal_sender, self.custom_signal));

        SignalHandler { shutdown_signal_receiver, handle }
    }

    async fn handle(
        shutdown_signal_sender: watch::Sender<()>,
        custom_signal: Option<BoxFuture<'static, ()>>,
    ) -> io::Result<()> {
        let mut signals = vec![
            SignalStream::new(signal(SignalKind::terminate())?).boxed(),
            SignalStream::new(signal(SignalKind::interrupt())?).boxed(),
        ];

        if let Some(signal) = custom_signal {
            signals.push(signal.into_stream().boxed());
        }

        let mut signal_stream = select_all(signals);

        let mut state = ShutdownState::default();

        while signal_stream.next().await.is_some() {
            match state.next() {
                None | Some(ShutdownState::Initial) => unreachable!(),
                Some(ShutdownState::WaitForSignal) => {
                    tracing::warn!("Receive UNIX shutdown signal, try to graceful shutdown.");

                    if let Err(err) = shutdown_signal_sender.send(()) {
                        tracing::warn!("Failed to send shutdown signal: {err}");
                    }
                }
                Some(ShutdownState::ShuttingDown) => {
                    tracing::warn!(
                        "Another shutdown signal is received, force exit in 200 milliseconds"
                    );

                    sleep(Duration::from_millis(200)).await;
                    tracing::warn!("Force exit this process");
                    std::process::exit(1);
                }
                Some(ShutdownState::Aborting) => {
                    tracing::error!(
                        "Could not shut down this process gracefully, abort this process"
                    );
                    std::process::abort();
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use futures_util::FutureExt;
    use tokio::time::{interval, sleep, timeout};

    use super::SignalHandleBuilder;

    #[tokio::test]
    #[cfg_attr(miri, ignore)]
    async fn test_custom_signal() {
        let fut = async {
            sleep(Duration::from_secs(1)).await;
        };

        let handler = {
            let builder = SignalHandleBuilder::new(Some(fut.boxed()));
            builder.start()
        };

        let mut signal = handler.shutdown_signal();

        let mut timer = interval(Duration::from_secs(5));
        timer.tick().await;
        loop {
            tokio::select! {
                () = signal.wait() => break,
                _ = timer.tick() => {}
            }
        }

        assert!(timeout(Duration::from_secs(2), signal.wait()).await.is_ok(), "timeout");

        handler.stop();
    }
}
