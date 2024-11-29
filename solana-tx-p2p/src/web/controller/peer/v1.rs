use axum::{Extension, Json};
use solana_sdk::transaction::Transaction;

use crate::{
    app_state::AppState, model::TransactionForUtoipa, service::PeerService, web::error::Result,
};

/// Discovery peers
#[utoipa::path(
    post,
    path = "/api/v1/peer/discovery",
    responses(
        (status = 200, body = Vec<String>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn discovery<S>(Extension(app_state): Extension<S>) -> Result<Json<Vec<String>>>
where
    S: AppState + Clone + Send + Sync + 'static,
{
    let peers = app_state.peer_service().discovery_peers().await?;

    Ok(Json(peers))
}

/// List signed messages
#[utoipa::path(
    post,
    path = "/api/v1/peer/signed-message",
    responses(
        (status = 200, body = Vec<TransactionForUtoipa>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn list_signed_messages<S>(
    Extension(app_state): Extension<S>,
) -> Result<Json<Vec<Transaction>>>
where
    S: AppState + Clone + Send + Sync + 'static,
{
    let signed_messages = app_state.peer_service().list_signed_messages().await?;

    Ok(Json(signed_messages))
}

/// List relayed transactions
#[utoipa::path(
    post,
    path = "/api/v1/peer/relayed-transaction",
    responses(
        (status = 200, body = Vec<String>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn list_relayed_transactions<S>(
    Extension(app_state): Extension<S>,
) -> Result<Json<Vec<String>>>
where
    S: AppState + Clone + Send + Sync + 'static,
{
    let signatures = app_state.peer_service().list_relayed_transactions().await?;

    Ok(Json(signatures))
}
