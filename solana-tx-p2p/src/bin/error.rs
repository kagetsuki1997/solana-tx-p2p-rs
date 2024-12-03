use std::borrow::Cow;

use http::Uri;
use snafu::{Backtrace, Snafu};
use solana_tx_p2p::fmt_backtrace_with_source;

/// Result type alias for the CLI.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Error type for the CLI.
#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("{source}"))]
    Io { source: std::io::Error },

    #[snafu(display(
        "Fail to parse filtering directives{}",
        fmt_backtrace_with_source(backtrace, source)
    ))]
    ParseFilteringDirectives {
        source: tracing_subscriber::filter::ParseError,
        backtrace: Backtrace,
    },

    #[snafu(display(
        "Can not initialize async runtime{}",
        fmt_backtrace_with_source(backtrace, source)
    ))]
    InitializeAsyncRuntime { source: std::io::Error, backtrace: Backtrace },

    #[snafu(display(
        "Can not spawn async task `{name}`{}",
        fmt_backtrace_with_source(backtrace, source)
    ))]
    Spawn { name: Cow<'static, str>, source: std::io::Error, backtrace: Backtrace },

    #[snafu(display(
        "Can not connect grpc channel for `{endpoint}`{}",
        fmt_backtrace_with_source(backtrace, source)
    ))]
    ConnectGrpcChannel { endpoint: Uri, source: tonic::transport::Error, backtrace: Backtrace },

    #[snafu(display(
        "Can not create solana client{}",
        fmt_backtrace_with_source(backtrace, source)
    ))]
    CreateSolanaClient { source: solana_tx_p2p::service::error::Error, backtrace: Backtrace },
}
