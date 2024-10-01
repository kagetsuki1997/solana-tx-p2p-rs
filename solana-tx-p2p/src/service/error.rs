use std::borrow::Cow;

use axum::{
    body,
    response::{IntoResponse, Response},
};
use http::{header, StatusCode};
use snafu::{Backtrace, Snafu};
use tonic::Status;

use crate::{error::fmt_backtrace_with_source, web::ErrorResponse};

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
        "Can not spawn async task `{name}`{}",
        fmt_backtrace_with_source(backtrace, source)
    ))]
    Spawn { name: Cow<'static, str>, source: std::io::Error, backtrace: Backtrace },
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
