use std::{sync::Arc, time::Duration};

use tokio::sync::{mpsc, RwLock};

use crate::{service::Result, ShutdownSignal};

enum Action {
    Stop,
    NextRound,
    Inbound(ElectionEvent),
}

pub enum ElectionEvent {
    LeaderHeartbeat,
    ElectionInfo(ElectionInfo),
}

pub struct ElectionInfo {
    leader: String,
}

enum Role {
    Leader,
    Follower,
}

pub struct RRElectionWorker {
    peer_id: String,
    heartbeat_timeout: Duration,
    round_interval: Duration,
    inbound_receiver: mpsc::Receiver<ElectionEvent>,
    outbound_sender: mpsc::Sender<ElectionEvent>,
    peers: Arc<RwLock<Vec<String>>>,
}

impl RRElectionWorker {
    #[must_use]
    pub fn new(
        peer_id: String,
        heartbeat_timeout: Duration,
        round_interval: Duration,
        inbound_receiver: mpsc::Receiver<ElectionEvent>,
        outbound_sender: mpsc::Sender<ElectionEvent>,
        peers: Arc<RwLock<Vec<String>>>,
    ) -> Self {
        Self {
            peer_id,
            heartbeat_timeout,
            round_interval,
            inbound_receiver,
            outbound_sender,
            peers,
        }
    }

    pub async fn start(self, mut shutdown_signal: ShutdownSignal) -> Result<()> {
        let mut role = Role::Follower;

        loop {
            let action = tokio::select! {
                () = shutdown_signal.wait() =>Action::Stop
            };
        }

        Ok(())
    }
}
