use async_trait::async_trait;
use snafu::ResultExt;
use solana_sdk::transaction::Transaction;
use tokio::sync::{mpsc, oneshot};

use crate::service::{
    error, error::Result, PeerService, PeerWorkerInboundEvent, PeerWorkerInstruction,
};

#[derive(Clone)]
pub struct DefaultPeerService {
    peer_worker_inbound_sender: mpsc::Sender<PeerWorkerInboundEvent>,
}

impl DefaultPeerService {
    #[inline]
    #[must_use]
    pub const fn new(peer_worker_inbound_sender: mpsc::Sender<PeerWorkerInboundEvent>) -> Self {
        Self { peer_worker_inbound_sender }
    }
}

#[async_trait]
impl PeerService for DefaultPeerService {
    async fn discovery_peers(&self) -> Result<Vec<String>> {
        let (sender, receiver) = oneshot::channel();

        let instruction = PeerWorkerInstruction::ListPeers(sender);
        self.peer_worker_inbound_sender
            .send(PeerWorkerInboundEvent::Instruction(instruction))
            .await
            .context(error::SendPeerWorkerInstructionSnafu { instruction: "ListPeers" })?;

        let peers = receiver.await.context(error::ListPeersSnafu)?;

        Ok(peers)
    }

    async fn list_signed_messages(&self) -> Result<Vec<Transaction>> {
        let (sender, receiver) = oneshot::channel();

        let instruction = PeerWorkerInstruction::ListSignedMessages(sender);
        self.peer_worker_inbound_sender
            .send(PeerWorkerInboundEvent::Instruction(instruction))
            .await
            .context(error::SendPeerWorkerInstructionSnafu { instruction: "ListSignedMessages" })?;

        let signed_messages = receiver.await.context(error::ListSignedMessagesSnafu)?;

        Ok(signed_messages)
    }

    async fn list_relayed_transactions(&self) -> Result<Vec<String>> {
        let (sender, receiver) = oneshot::channel();

        let instruction = PeerWorkerInstruction::ListRelayedTransactions(sender);
        self.peer_worker_inbound_sender
            .send(PeerWorkerInboundEvent::Instruction(instruction))
            .await
            .context(error::SendPeerWorkerInstructionSnafu {
                instruction: "ListRelayedTransactions",
            })?;

        let relayed_transactions = receiver.await.context(error::ListRelayedTransactionsSnafu)?;

        Ok(relayed_transactions)
    }
}
