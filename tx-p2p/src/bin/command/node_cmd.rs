use std::{io, process, time::Duration};

use clap::Args;
use futures_util::TryFutureExt;
use snafu::ResultExt;
use tokio::{runtime::Runtime, sync::mpsc, task::JoinSet};
use tx_p2p::{PeerWorker, SignalHandleBuilder};

use crate::{
    env,
    error::{self, Error, Result},
    tracing::init_tracing,
};

const APP_NAME: &str = "Transaction-relaying Peer-to-peer Node";

#[derive(Args, Debug)]
pub struct NodeCmd {
    #[arg(
        name = "message-duration",
        long,
        env = env::MESSAGE_DURATION,
    )]
    message_duration: Option<humantime::Duration>,

    #[arg(
        name = "relay-leader-duration",
        long,
        env = env::RELAY_LEADER_DURATION,
        default_value = "60s"
    )]
    relay_leader_duration: humantime::Duration,

    #[arg(
        name = "signing-leader-duration",
        long,
        env = env::SIGNING_LEADER_DURATION,
        default_value = "60s"
    )]
    signing_leader_duration: humantime::Duration,
}

impl NodeCmd {
    /// Run the node
    // FIXME: clippy::significant_drop_tightening: clippy bug
    #[allow(clippy::significant_drop_tightening, clippy::too_many_lines)]
    pub fn run(self) -> Result<()> {
        let Self { message_duration, relay_leader_duration, signing_leader_duration } = self;
        Runtime::new().context(error::InitializeAsyncRuntimeSnafu)?.block_on(async {
            let _handle = init_tracing("debug,hyper=info,tower=info")?;

            tracing::debug!("{APP_NAME} starting");
            tracing::info!("Process ID: {}", process::id());

            let mut join_set = JoinSet::<tx_p2p::Result<()>>::new();

            tracing::info!("Initializing shutdown signal handler");
            let shutdown_signal_handler = SignalHandleBuilder::new(None).start();
            let shutdown_signal = shutdown_signal_handler.shutdown_signal();

            tracing::info!("Initializing PeerWorker");
            let (stdin_sender, stdin_receiver) = mpsc::channel(10);
            let peer_worker = PeerWorker::new(
                message_duration.map(Into::into),
                *relay_leader_duration,
                *signing_leader_duration,
            );
            join_set
                .build_task()
                .name("peer worker")
                .spawn(peer_worker.start(shutdown_signal.clone(), stdin_receiver).err_into())
                .context(error::SpawnSnafu { name: "peer worker".to_string() })?;

            // stdin reader
            let rt = Runtime::new().context(error::InitializeAsyncRuntimeSnafu)?;
            rt.spawn_blocking(move || {
                loop {
                    let mut buffer = String::new();
                    if let Err(err) = std::io::stdin().read_line(&mut buffer) {
                        tracing::error!("Fail to read stdin: {err}");
                        continue;
                    }

                    if let Err(err) = stdin_sender.blocking_send(buffer) {
                        tracing::error!("Fail to send stdin: {err}");
                        break;
                    }
                }

                tracing::warn!("stdin reader thread stopped");
            });

            while let Some(result) = join_set.join_next().await {
                match result {
                    Ok(Ok(())) => {}
                    Ok(Err(err)) => tracing::error!("Server error: {err}"),
                    Err(err) => tracing::error!("Join error: {err}"),
                }
            }

            shutdown_signal_handler.stop();
            rt.shutdown_background();

            Result::<_, Error>::Ok(())
        })?;

        tracing::info!("{APP_NAME} shutdown complete");

        Ok(())
    }
}
