use axum::{Extension, Json};

use crate::{app_state::AppState, model::Peer, service::PeerService, web::error::Result};

/// Discovery peers
#[utoipa::path(
    post,
    path = "/api/v1/peer/discovery",
    responses(
        (status = 200, body = Vec<Peer>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn discovery<S>(Extension(app_state): Extension<S>) -> Result<Json<Vec<Peer>>>
where
    S: AppState + Clone + Send + Sync + 'static,
{
    let peers = app_state.peer_service().discovery().await?;

    Ok(Json(peers))
}
