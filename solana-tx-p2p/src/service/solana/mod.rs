mod relayer;
mod signer;

use std::sync::Arc;

use snafu::ResultExt;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig, signature::Keypair as SolanaKeypair, signer::Signer as _,
};

pub use self::{
    relayer::{RelayerInboundEvent, SolanaRelayer},
    signer::{SignerInboundEvent, SolanaSigner},
};
use crate::service::{error, error::Result};

pub async fn create_solana_client(
    rpc_url: &str,
    keypair: Arc<SolanaKeypair>,
) -> Result<Arc<RpcClient>> {
    // Connect to the Solana devnet
    let client = RpcClient::new_with_commitment(rpc_url.to_string(), CommitmentConfig::confirmed());

    // Request airdrop
    tracing::debug!("Request airdrop");
    let airdrop_amount = 1_000_000_000; // 1 SOL
    let signature = client
        .request_airdrop(&keypair.pubkey(), airdrop_amount)
        .await
        .context(error::RequestAirdropSnafu)?;

    // Wait for airdrop confirmation
    tracing::debug!("Wait airdrop confirmation");
    loop {
        let confirmed = client
            .confirm_transaction(&signature)
            .await
            .context(error::ConfirmSolanaTransactionSnafu)?;
        if confirmed {
            break;
        }
    }

    tracing::debug!("Complete airdrop confirmation");

    Ok(Arc::new(client))
}
