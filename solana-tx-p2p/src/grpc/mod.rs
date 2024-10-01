mod peer;

use std::net::SocketAddr;

use axum::extract::FromRef;
use http::Request;
use hyper::Body;
use snafu::ResultExt;
use tonic::{
    codegen::CompressionEncoding,
    transport::{Identity, ServerTlsConfig},
    Code,
};
use tower_http::{classify::GrpcFailureClass, trace::TraceLayer};
use tracing::Span;

use crate::{
    app_state::AppState,
    error,
    error::Result,
    grpc::peer::v1::PeerService,
    proto::{peer::v1::PeerServiceServer, FILE_DESCRIPTOR_SET},
    ShutdownSignal,
};

/// # Errors
/// * if server error
pub async fn run<S>(
    socket_address: SocketAddr,
    app_state: S,
    tls_cert: Option<String>,
    tls_key: Option<String>,
    mut shutdown_signal: ShutdownSignal,
) -> Result<()>
where
    S: AppState,
{
    let reflection = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
        .build()
        .context(error::BuildGrpcReflectionServiceSnafu)?;

    let server_builder = if let (Some(tls_cert), Some(tls_key)) = (tls_cert, tls_key) {
        tracing::info!("Build gRPC server with TLS");
        tonic::transport::Server::builder()
            .tls_config(ServerTlsConfig::new().identity(Identity::from_pem(&tls_cert, &tls_key)))
            .context(error::BuildGrpcTlsServerSnafu)?
    } else {
        tonic::transport::Server::builder()
    };

    server_builder
        .layer(
            TraceLayer::new_for_grpc()
                .on_request(|request: &Request<Body>, _span: &Span| {
                    tracing::info!("request: {}", request.uri().path());
                })
                .on_failure(|error, _latency, _span: &Span| {
                    if let GrpcFailureClass::Code(code) = error {
                        let code = Code::from_i32(code.into());
                        tracing::error!("{}", code.description());
                    } else {
                        tracing::error!("{error}");
                    }
                }),
        )
        .add_service(
            PeerServiceServer::new(PeerService::new(S::PeerService::from_ref(&app_state)))
                .accept_compressed(CompressionEncoding::Gzip)
                .send_compressed(CompressionEncoding::Gzip),
        )
        .add_service(reflection)
        .serve_with_shutdown(socket_address, shutdown_signal.wait())
        .await
        .context(error::RunGrpcServerSnafu)?;

    tracing::info!("gRPC server is shutdown gracefully");
    Ok(())
}
