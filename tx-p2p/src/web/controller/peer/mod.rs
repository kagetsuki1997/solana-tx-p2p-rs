pub mod v1;

use axum::{routing, Router};

use crate::app_state::AppState;

pub fn v1<S>() -> Router
where
    S: AppState + Clone + Send + Sync + 'static,
{
    Router::new()
        .nest("/v1/peer", Router::new().route("/discovery", routing::get(v1::discovery::<S>)))
}
