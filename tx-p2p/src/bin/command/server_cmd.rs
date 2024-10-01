use std::{
    net::{IpAddr, Ipv6Addr, SocketAddr},
    process,
};

use clap::Args;
use futures_util::future::TryFutureExt;
use snafu::ResultExt;
use tokio::{runtime::Runtime, task::JoinSet};
use tx_p2p::{grpc, metrics, web, DefaultAppState, SignalHandleBuilder};

use crate::{
    env, error,
    error::{Error, Result},
    tracing::init_tracing,
};

const APP_NAME: &str = "Transaction-relaying Peer-to-peer Node Server";

#[derive(Args, Debug)]
pub struct ApiConfig {
    #[clap(
        name = "api-address",
        long,
        env = env::API_ADDRESS,
        default_value_t = IpAddr::V6(Ipv6Addr::LOCALHOST)
    )]
    address: IpAddr,

    #[clap(
        name = "api-port",
        long,
        env = env::API_PORT,
        default_value_t = 8007
    )]
    port: u16,
}

impl ApiConfig {
    const fn socket_address(&self) -> SocketAddr { SocketAddr::new(self.address, self.port) }
}

#[derive(Args, Debug)]
pub struct GrpcConfig {
    #[arg(
        name = "grpc-address",
        long,
        env = env::GRPC_ADDRESS,
        default_value_t = IpAddr::V6(Ipv6Addr::LOCALHOST)
    )]
    address: IpAddr,

    #[arg(
        name = "grpc-port",
        long,
        env = env::GRPC_PORT,
        default_value_t = 50051
    )]
    port: u16,
}

impl GrpcConfig {
    const fn socket_address(&self) -> SocketAddr { SocketAddr::new(self.address, self.port) }
}

#[derive(Args, Debug)]
pub struct MetricsConfig {
    #[arg(
        name = "metrics-address",
        long,
        env = env::METRICS_ADDRESS,
        default_value_t = IpAddr::V6(Ipv6Addr::LOCALHOST)
    )]
    address: IpAddr,

    #[arg(
        name = "metrics-port",
        long,
        env = env::METRICS_PORT,
        default_value_t = 9090
    )]
    port: u16,
}

impl MetricsConfig {
    const fn socket_address(&self) -> SocketAddr { SocketAddr::new(self.address, self.port) }
}

#[derive(Args, Debug)]
pub struct TlsConfig {
    #[arg(
        name = "tls-cert",
        long,
        env = env::TLS_CERT
    )]
    cert: Option<String>,

    #[arg(
        name = "tls-key",
        long,
        env = env::TLS_KEY
    )]
    key: Option<String>,

    #[arg(
        name = "tls-ca",
        long,
        env = env::TLS_CA
    )]
    ca: Option<String>,
}

#[derive(Args, Debug)]
pub struct ServerCmd {
    #[command(flatten)]
    pub api: ApiConfig,

    #[command(flatten)]
    pub grpc: GrpcConfig,

    #[command(flatten)]
    pub metrics: MetricsConfig,

    #[command(flatten)]
    pub tls: TlsConfig,
}

impl ServerCmd {
    /// Run the server
    // FIXME: clippy::significant_drop_tightening: clippy bug
    #[allow(clippy::significant_drop_tightening, clippy::too_many_lines)]
    pub fn run(self) -> Result<()> {
        let Self { api, grpc, metrics, tls } = self;
        Runtime::new().context(error::InitializeAsyncRuntimeSnafu)?.block_on(async {
            let _handle = init_tracing("debug,hyper=info,tower=info")?;

            tracing::debug!("{APP_NAME} starting");
            tracing::info!("Process ID: {}", process::id());

            let mut join_set = JoinSet::<tx_p2p::Result<()>>::new();

            tracing::info!("Initializing shutdown signal handler");
            let shutdown_signal_handler = SignalHandleBuilder::new(None).start();
            let shutdown_signal = shutdown_signal_handler.shutdown_signal();

            tracing::info!("Initializing app state");
            let app_state = DefaultAppState::new();

            tracing::info!("Initializing metrics server");
            join_set
                .build_task()
                .name("metrics server")
                .spawn(
                    metrics::run(
                        metrics.socket_address(),
                        app_state.clone(),
                        shutdown_signal.clone(),
                    )
                    .err_into(),
                )
                .context(error::SpawnSnafu { name: "metrics server".to_string() })?;

            tracing::info!("Initializing web api server");
            join_set
                .build_task()
                .name("web api server")
                .spawn(
                    web::run(api.socket_address(), app_state.clone(), shutdown_signal.clone())
                        .err_into(),
                )
                .context(error::SpawnSnafu { name: "web api server".to_string() })?;

            tracing::info!("Initializing gRPC server");
            join_set
                .build_task()
                .name("gRPC Server")
                .spawn(
                    grpc::run(grpc.socket_address(), app_state, tls.cert, tls.key, shutdown_signal)
                        .err_into(),
                )
                .context(error::SpawnSnafu { name: "gRPC server".to_string() })?;

            while let Some(result) = join_set.join_next().await {
                match result {
                    Ok(Ok(())) => {}
                    Ok(Err(err)) => tracing::error!("Server error: {err}"),
                    Err(err) => tracing::error!("Join error: {err}"),
                }
            }

            shutdown_signal_handler.stop();

            Result::<_, Error>::Ok(())
        })?;

        tracing::info!("{APP_NAME} shutdown complete");

        Ok(())
    }
}
