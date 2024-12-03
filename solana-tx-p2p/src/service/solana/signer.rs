use std::sync::Arc;

use bytes::Bytes;
use libp2p::PeerId;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    instruction::Instruction, pubkey::Pubkey, signature::Keypair as SolanaKeypair,
    signer::Signer as _, transaction::Transaction,
};
use tokio::sync::{mpsc, RwLock};

use crate::{
    service::{PeerWorkerInboundEvent, Result},
    ShutdownSignal,
};

pub enum SignerInboundEvent {
    RawMessage(Bytes),
}

enum Action {
    Stop,
    Inbound(Option<SignerInboundEvent>),
}

pub struct SolanaSigner {
    peer_id: PeerId,
    signer: Arc<RwLock<String>>,
    keypair: Arc<SolanaKeypair>,

    peer_worker_inbound_sender: mpsc::Sender<PeerWorkerInboundEvent>,

    program_id: Pubkey,
    client: Arc<RpcClient>,

    inbound_receiver: mpsc::Receiver<SignerInboundEvent>,
}

impl SolanaSigner {
    #[must_use]
    pub const fn new(
        peer_id: PeerId,
        signer: Arc<RwLock<String>>,
        keypair: Arc<SolanaKeypair>,
        peer_worker_inbound_sender: mpsc::Sender<PeerWorkerInboundEvent>,
        program_id: Pubkey,
        client: Arc<RpcClient>,
        inbound_receiver: mpsc::Receiver<SignerInboundEvent>,
    ) -> Self {
        Self {
            peer_id,
            signer,
            keypair,
            peer_worker_inbound_sender,
            program_id,
            client,
            inbound_receiver,
        }
    }

    /// # Panics
    ///
    /// * fail to request airdrop
    /// * fail to confirm airdrop transaction
    pub async fn start(mut self, mut shutdown_signal: ShutdownSignal) -> Result<()> {
        loop {
            let action = tokio::select! {
                () = shutdown_signal.wait() => Action::Stop,
                result = self.inbound_receiver.recv() => Action::Inbound(result),
            };

            match action {
                Action::Stop => break,
                Action::Inbound(result) => match result {
                    None => {
                        tracing::warn!("`signer_worker_inbound_receiver` is closed",);
                        break;
                    }
                    Some(SignerInboundEvent::RawMessage(raw_message)) => {
                        if *self.signer.read().await == self.peer_id.to_string() {
                            let message = String::from_utf8_lossy(&raw_message);
                            let data = format!("{message}, Signer: {}", self.peer_id);

                            // Create the instruction
                            let instruction = Instruction::new_with_borsh(
                                self.program_id,
                                &data,
                                vec![], // No accounts needed
                            );

                            // Add the instruction to new transaction
                            let mut transaction = Transaction::new_with_payer(
                                &[instruction],
                                Some(&self.keypair.pubkey()),
                            );

                            match self.client.get_latest_blockhash().await {
                                Ok(recent_blockhash) => {
                                    transaction.sign(&[&self.keypair], recent_blockhash);
                                }
                                Err(err) => {
                                    tracing::error!("Fail to get recent blockhash: {err}");
                                    continue;
                                }
                            }

                            // send transaction to p2p network
                            if let Err(err) = self
                                .peer_worker_inbound_sender
                                .send(PeerWorkerInboundEvent::Transaction(transaction))
                                .await
                            {
                                tracing::error!("Fail to send transaction to peer worker: {err}");
                                break;
                            }
                        }
                    }
                },
            }
        }

        tracing::warn!("Solana Signer stopped.");

        Ok(())
    }
}
