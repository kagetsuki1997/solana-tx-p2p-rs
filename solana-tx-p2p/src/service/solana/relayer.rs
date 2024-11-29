use std::sync::Arc;

use libp2p::PeerId;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig, signature::Keypair as SolanaKeypair, signer::Signer as _,
    transaction::Transaction,
};
use tokio::sync::{mpsc, RwLock};

use crate::{
    service::{PeerWorkerInboundEvent, Result},
    ShutdownSignal,
};

pub enum RelayerInboundEvent {
    Transaction(Transaction),
}

enum Action {
    Stop,
    Inbound(Option<RelayerInboundEvent>),
}

pub struct SolanaRelayer {
    peer_id: PeerId,
    relayer: Arc<RwLock<String>>,
    keypair: Arc<SolanaKeypair>,

    peer_worker_inbound_sender: mpsc::Sender<PeerWorkerInboundEvent>,

    rpc_url: String,

    inbound_receiver: mpsc::Receiver<RelayerInboundEvent>,
}

impl SolanaRelayer {
    #[must_use]
    pub const fn new(
        peer_id: PeerId,
        relayer: Arc<RwLock<String>>,
        keypair: Arc<SolanaKeypair>,
        peer_worker_inbound_sender: mpsc::Sender<PeerWorkerInboundEvent>,
        rpc_url: String,
        inbound_receiver: mpsc::Receiver<RelayerInboundEvent>,
    ) -> Self {
        Self { peer_id, relayer, keypair, peer_worker_inbound_sender, rpc_url, inbound_receiver }
    }

    /// # Panics
    ///
    /// * fail to request airdrop
    /// * fail to confirm airdrop transaction
    pub async fn start(mut self, mut shutdown_signal: ShutdownSignal) -> Result<()> {
        // Connect to the Solana devnet
        let client = RpcClient::new_with_commitment(self.rpc_url, CommitmentConfig::confirmed());

        // Request airdrop
        tracing::debug!("Request airdrop confirmation");
        let airdrop_amount = 1_000_000_000; // 1 SOL
        let signature = client
            .request_airdrop(&self.keypair.pubkey(), airdrop_amount)
            .await
            .expect("Failed to request airdrop");

        // Wait for airdrop confirmation
        tracing::debug!("Wait airdrop confirmation");
        loop {
            let confirmed = client
                .confirm_transaction(&signature)
                .await
                .expect("Failed to confirm transaction");
            if confirmed {
                break;
            }
        }

        tracing::debug!("Complete airdrop confirmation");

        loop {
            let action = tokio::select! {
                () = shutdown_signal.wait() => Action::Stop,
                result = self.inbound_receiver.recv() => Action::Inbound(result),
            };

            match action {
                Action::Stop => break,
                Action::Inbound(result) => match result {
                    None => {
                        tracing::warn!("`relayer_worker_inbound_receiver` is closed",);
                        break;
                    }
                    Some(RelayerInboundEvent::Transaction(transaction)) => {
                        if *self.relayer.read().await == self.peer_id.to_string() {
                            // Send and confirm the transaction
                            match client.send_and_confirm_transaction(&transaction).await {
                                Ok(signature) => {
                                    tracing::info!("Transaction Signature: {}", signature);

                                    // send relayed transaction to p2p network
                                    if let Err(err) = self
                                        .peer_worker_inbound_sender
                                        .send(PeerWorkerInboundEvent::RelayedTransaction(
                                            signature.to_string(),
                                        ))
                                        .await
                                    {
                                        tracing::error!(
                                            "Fail to send relayed transaction to peer worker: \
                                             {err}"
                                        );
                                        break;
                                    }
                                }
                                Err(err) => {
                                    tracing::error!("Fail to send transaction to solana: {}", err);
                                }
                            }
                        }
                    }
                },
            }
        }

        tracing::warn!("Solana Relayer stopped.");

        Ok(())
    }
}
