use std::net::SocketAddr;

use axum::{response::IntoResponse, routing, routing::Router};
use hyper::server::conn::AddrIncoming;
use snafu::ResultExt;

use crate::{app_state::AppState, error, error::Result, ShutdownSignal};

/// # Errors
/// * if server error
pub async fn run<S>(
    socket_address: SocketAddr,
    app_state: S,
    mut shutdown_signal: ShutdownSignal,
) -> Result<()>
where
    S: AppState,
{
    let make_service = Router::new()
        .route("/startz", routing::get(startz))
        .route("/livez", routing::get(livez))
        .route("/readyz", routing::get(readyz))
        .with_state(app_state)
        .into_make_service_with_connect_info::<SocketAddr>();

    let acceptor = {
        let mut incoming = AddrIncoming::bind(&socket_address)
            .context(error::BindAddressSnafu { socket_address })?;
        incoming.set_nodelay(true);
        incoming
    };
    axum::Server::builder(acceptor)
        .serve(make_service)
        .with_graceful_shutdown(shutdown_signal.wait())
        .await
        .context(error::RunMetricsServerSnafu)?;

    tracing::info!("Metrics server is shutdown gracefully");
    Ok(())
}

// SAFETY: clippy::unused_async: axum requires such function signature
#[allow(clippy::unused_async)]
async fn startz() -> impl IntoResponse { "TODO" }

// SAFETY: clippy::unused_async: axum requires such function signature
#[allow(clippy::unused_async)]
async fn livez() -> impl IntoResponse { "TODO" }

// SAFETY: clippy::unused_async: axum requires such function signature
#[allow(clippy::unused_async)]
async fn readyz() -> impl IntoResponse { "TODO" }
