pub mod error;
mod peer;
mod round_robin_election;
mod solana;

use std::time::Duration;

use axum::async_trait;
use rand::random;
use solana_sdk::transaction::Transaction;
use tokio::{sync::mpsc, time};

pub use self::{
    error::Result,
    peer::{DefaultPeerService, PeerWorker, PeerWorkerInboundEvent, PeerWorkerInstruction},
    round_robin_election::{RRElectionWorker, RRElectionWorkerType},
    solana::{RelayerInboundEvent, SignerInboundEvent, SolanaRelayer, SolanaSigner},
};
use crate::ShutdownSignal;

#[async_trait]
pub trait PeerService {
    async fn discovery_peers(&self) -> Result<Vec<String>>;

    async fn list_signed_messages(&self) -> Result<Vec<Transaction>>;

    async fn list_relayed_transactions(&self) -> Result<Vec<String>>;
}

/// Trigger message
pub async fn start_message_trigger(
    message_duration: Option<Duration>,
    mut shutdown_signal: ShutdownSignal,
    peer_worker_inbound_sender: mpsc::Sender<PeerWorkerInboundEvent>,
) -> Result<()> {
    let message_duration = message_duration.unwrap_or_else(|| {
        let duration = random::<u8>() % 10 + 5;
        Duration::from_secs(duration.into())
    });

    tracing::info!("Message duration: {}", message_duration.as_secs());

    let mut message_timer = time::interval(message_duration);
    message_timer.reset();
    loop {
        tokio::select! {
            () = shutdown_signal.wait() => break,
            _ = message_timer.tick() => (),
        }

        if let Err(err) =
            peer_worker_inbound_sender.send(PeerWorkerInboundEvent::MessageTrigger).await
        {
            tracing::error!("Fail to trigger message: {err}");
            break;
        }
    }

    tracing::warn!("Message trigger task stopped!");
    Ok(())
}

/// Trigger heartbeat
pub async fn start_heartbeat_trigger(
    heartbeat_duration: Duration,
    mut shutdown_signal: ShutdownSignal,
    peer_worker_inbound_sender: mpsc::Sender<PeerWorkerInboundEvent>,
) -> Result<()> {
    let mut heartbeat_timer = time::interval(heartbeat_duration);
    heartbeat_timer.reset();
    loop {
        tokio::select! {
            () = shutdown_signal.wait() => break,
            _ = heartbeat_timer.tick() => (),
        }

        // send heartbeat to peer worker
        if let Err(err) =
            peer_worker_inbound_sender.send(PeerWorkerInboundEvent::HeartbeatTrigger).await
        {
            tracing::error!("Fail to trigger heartbeat: {err}");
            break;
        }
    }

    tracing::warn!("Heartbeat trigger task stopped!");
    Ok(())
}
