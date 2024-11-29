use std::borrow::Cow;

use axum::{
    body,
    response::{IntoResponse, Response},
};
use http::{header, StatusCode};
use snafu::{Backtrace, Snafu};
use tokio::sync::{
    mpsc::error::SendError as MpscSendError, oneshot::error::RecvError as OneshotRecvError,
};
use tonic::Status;

use crate::{
    error::fmt_backtrace_with_source, service::PeerWorkerInboundEvent, web::ErrorResponse,
};

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display(
        "Fail to create swarm with tcp{}",
        fmt_backtrace_with_source(backtrace, source)
    ))]
    SwarmWithTcp { source: libp2p::noise::Error, backtrace: Backtrace },

    #[snafu(display(
        "Fail to send peer worker instruction `{instruction}`{}",
        fmt_backtrace_with_source(backtrace, source)
    ))]
    SendPeerWorkerInstruction {
        instruction: Cow<'static, str>,
        source: MpscSendError<PeerWorkerInboundEvent>,
        backtrace: Backtrace,
    },

    #[snafu(display("Fail to list peers`{}", fmt_backtrace_with_source(backtrace, source)))]
    ListPeers { source: OneshotRecvError, backtrace: Backtrace },

    #[snafu(display(
        "Fail to list signed messages`{}",
        fmt_backtrace_with_source(backtrace, source)
    ))]
    ListSignedMessages { source: OneshotRecvError, backtrace: Backtrace },

    #[snafu(display(
        "Fail to list relayed transactions`{}",
        fmt_backtrace_with_source(backtrace, source)
    ))]
    ListRelayedTransactions { source: OneshotRecvError, backtrace: Backtrace },
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let body = {
            let body = ErrorResponse { message: self.to_string() };
            body::boxed(body::Full::from(
                serde_json::to_vec(&body).expect("`ErrorResponse` is valid JSON"),
            ))
        };

        match self {
            _ => Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .header(header::CONTENT_TYPE, "application/json")
                .body(body)
                .expect("It should be a valid `Response`"),
        }
    }
}

impl From<Error> for Status {
    fn from(error: Error) -> Self { Self::internal(error.to_string()) }
}
