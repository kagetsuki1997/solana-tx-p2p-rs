use std::{fmt::Display, sync::Arc, time::Duration};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::{
    sync::{mpsc, RwLock},
    time,
    time::{error::Elapsed, timeout, Instant},
};

use crate::{
    service::{peer::PeerWorkerInboundEvent, Result},
    ShutdownSignal,
};

#[derive(Debug)]
pub enum ElectionWorkerInboundEvent {
    LeaderSyncInfo(LeaderSyncInfo),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderSyncInfo {
    leader: String,
    next_round_time: DateTime<Utc>,
}

enum Action {
    Stop,
    LeaderHeartbeat(std::result::Result<Option<()>, Elapsed>),
    NextRound,
    Inbound(Option<ElectionWorkerInboundEvent>),
}

#[derive(Debug, PartialEq, Eq)]
pub enum RRElectionWorkerType {
    Relayer,
    Signer,
}

impl Display for RRElectionWorkerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Relayer => write!(f, "Relayer"),
            Self::Signer => write!(f, "Signer"),
        }
    }
}

pub struct RRElectionWorker {
    r#type: RRElectionWorkerType,
    current_leader: Arc<RwLock<String>>,
    peers: Arc<RwLock<Vec<String>>>,

    heartbeat_timeout: Duration,
    round_interval: Duration,
    inbound_receiver: mpsc::Receiver<ElectionWorkerInboundEvent>,
    leader_heartbeat_receiver: mpsc::Receiver<()>,

    peer_worker_inbound_sender: mpsc::Sender<PeerWorkerInboundEvent>,
}

impl RRElectionWorker {
    #[must_use]
    pub fn new(
        r#type: RRElectionWorkerType,
        current_leader: Arc<RwLock<String>>,
        heartbeat_timeout: Duration,
        round_interval: Duration,
        inbound_receiver: mpsc::Receiver<ElectionWorkerInboundEvent>,
        leader_heartbeat_receiver: mpsc::Receiver<()>,
        peers: Arc<RwLock<Vec<String>>>,
        peer_worker_inbound_sender: mpsc::Sender<PeerWorkerInboundEvent>,
    ) -> Self {
        Self {
            r#type,
            current_leader,
            heartbeat_timeout,
            round_interval,
            inbound_receiver,
            leader_heartbeat_receiver,
            peers,
            peer_worker_inbound_sender,
        }
    }

    pub async fn start(mut self, mut shutdown_signal: ShutdownSignal) -> Result<()> {
        let mut next_round_time = Instant::now() + self.round_interval;

        loop {
            let action = tokio::select! {
                () = shutdown_signal.wait() => Action::Stop,
                result = self.inbound_receiver.recv() => Action::Inbound(result),
                _ = time::sleep_until(next_round_time) => Action::NextRound,
                result = timeout(self.heartbeat_timeout, self.leader_heartbeat_receiver.recv()) => Action::LeaderHeartbeat(result),
            };

            match action {
                Action::Stop => break,
                Action::LeaderHeartbeat(result) => match result {
                    Err(_) => {
                        tracing::warn!(
                            "Receive leader heatbeat timeout, elect next {} leader",
                            self.r#type
                        );
                        next_round_time = self.elect_next_leader().await;
                    }
                    Ok(None) => {
                        tracing::warn!("`leader_heartbeat_receiver` of {} is closed", self.r#type);
                        break;
                    }
                    Ok(Some(())) => {
                        tracing::debug!("Receive heartbeat, {}", self.r#type);
                    }
                },
                Action::Inbound(result) => match result {
                    None => {
                        tracing::warn!(
                            "`election_worker_inbound_receiver` of {} is closed",
                            self.r#type
                        );
                        break;
                    }
                    Some(ElectionWorkerInboundEvent::LeaderSyncInfo(leader_info)) => {
                        tracing::info!("Receive {} leader sync info: {leader_info:?}", self.r#type);
                        {
                            *self.current_leader.write().await = leader_info.leader;
                            next_round_time = Instant::now()
                                + (leader_info.next_round_time - Utc::now())
                                    .to_std()
                                    .unwrap_or_default();
                        }
                    }
                },
                Action::NextRound => {
                    next_round_time = self.elect_next_leader().await;
                }
            }
        }

        tracing::warn!("RRElectionWorker stopped.");

        Ok(())
    }

    async fn elect_next_leader(&mut self) -> Instant {
        let next_round_time = Instant::now() + self.round_interval;
        let next_round_datetime = Utc::now() + self.round_interval;
        let peers = self.peers.read().await;
        let peers_len = peers.len();
        let mut current_leader = self.current_leader.write().await;

        // check current leader
        let mut found = false;
        for (idx, peer) in peers.iter().enumerate() {
            if *peer == *current_leader {
                let idx = (idx + 1) % peers_len;
                *current_leader = peers[idx].clone();
                found = true;
                break;
            }
        }

        // elect first peer as leader if current leader not found
        if !found {
            *current_leader = peers.first().expect("Peers must not be empty").clone();
        }

        tracing::info!(
            "Elect `{}` as {} leader for next round until {next_round_datetime}",
            *current_leader,
            self.r#type
        );

        let leader_info =
            LeaderSyncInfo { leader: current_leader.clone(), next_round_time: next_round_datetime };
        let event = if self.r#type == RRElectionWorkerType::Relayer {
            PeerWorkerInboundEvent::RelayerSyncInfo(leader_info)
        } else {
            PeerWorkerInboundEvent::SignerSyncInfo(leader_info)
        };

        if let Err(err) = self.peer_worker_inbound_sender.send(event).await {
            tracing::error!("Fail to send {} leader sync info: {err}", self.r#type);
        }

        next_round_time
    }
}
