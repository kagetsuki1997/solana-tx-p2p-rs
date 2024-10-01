use std::{fmt, io, net::SocketAddr};

use snafu::{Backtrace, Snafu};

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("{source}"))]
    Io { source: io::Error },

    #[snafu(display("{source}"))]
    Service { source: crate::service::error::Error },

    #[snafu(display(
        "Fail to bind address `{socket_address}`{}",
        fmt_backtrace_with_source(backtrace, source)
    ))]
    BindAddress { socket_address: SocketAddr, source: hyper::Error, backtrace: Backtrace },

    #[snafu(display(
        "Fail to build gRPC reflection service{}",
        fmt_backtrace_with_source(backtrace, source)
    ))]
    BuildGrpcReflectionService { source: tonic_reflection::server::Error, backtrace: Backtrace },

    #[snafu(display(
        "Fail to build gRPC server with tls{}",
        fmt_backtrace_with_source(backtrace, source)
    ))]
    BuildGrpcTlsServer { source: tonic::transport::Error, backtrace: Backtrace },

    #[snafu(display(
        "Error occurs when running gRPC server{}",
        fmt_backtrace_with_source(backtrace, source)
    ))]
    RunGrpcServer { source: tonic::transport::Error, backtrace: Backtrace },

    #[snafu(display(
        "Error occurs when running metrics server{}",
        fmt_backtrace_with_source(backtrace, source)
    ))]
    RunMetricsServer { source: hyper::Error, backtrace: Backtrace },

    #[snafu(display(
        "Error occurs when running web server{}",
        fmt_backtrace_with_source(backtrace, source)
    ))]
    RunWebServer { source: hyper::Error, backtrace: Backtrace },
}

impl From<io::Error> for Error {
    fn from(source: io::Error) -> Self { Self::Io { source } }
}

impl From<crate::service::error::Error> for Error {
    fn from(source: crate::service::error::Error) -> Self { Self::Service { source } }
}

#[inline]
#[must_use]
pub fn fmt_backtrace(backtrace: &Backtrace) -> String {
    if cfg!(feature = "backtrace") {
        format!("\n{backtrace}")
    } else {
        String::new()
    }
}

#[inline]
#[must_use]
pub fn fmt_backtrace_with_source(backtrace: &Backtrace, source: impl fmt::Display) -> String {
    format!("{}{}", fmt_backtrace(backtrace), fmt_source(source))
}

#[inline]
#[must_use]
pub fn fmt_source(source: impl fmt::Display) -> String { format!(". Caused by: {source}") }
