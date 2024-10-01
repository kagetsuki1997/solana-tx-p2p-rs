mod controller;
mod error;

use std::net::SocketAddr;

use axum::Extension;
use snafu::ResultExt;

pub use self::{controller::ApiDoc, error::ErrorResponse};
use crate::{
    app_state::AppState,
    error::{Result, RunWebServerSnafu},
    ShutdownSignal,
};

/// # Errors
/// * if server error
pub async fn run<S>(
    socket_address: SocketAddr,
    app_state: S,
    mut shutdown_signal: ShutdownSignal,
) -> Result<()>
where
    S: AppState + Clone + Send + Sync + 'static,
{
    let make_service = self::controller::apis::<S>(&app_state)?
        .layer(Extension(app_state))
        .into_make_service_with_connect_info::<SocketAddr>();

    axum::Server::bind(&socket_address)
        .serve(make_service)
        .with_graceful_shutdown(shutdown_signal.wait())
        .await
        .context(RunWebServerSnafu)?;

    tracing::info!("API server is shutdown gracefully");
    Ok(())
}
