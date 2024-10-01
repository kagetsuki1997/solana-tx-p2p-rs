use axum::{
    body,
    response::{IntoResponse, Response},
};
use http::{header, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use snafu::{Backtrace, Snafu};

use crate::error::fmt_backtrace_with_source;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("{source}"))]
    Service { source: crate::service::error::Error },

    #[snafu(display("App state extension not set"))]
    AppState,

    #[snafu(display(
        "Fail to consume request body{}",
        fmt_backtrace_with_source(backtrace, source)
    ))]
    ConsumeRequestBody { source: hyper::Error, backtrace: Backtrace },

    #[snafu(display(
        "Fail to deserialize request body{}",
        fmt_backtrace_with_source(backtrace, source)
    ))]
    DeserializeRequestBody { source: serde_json::Error, backtrace: Backtrace },
}

impl From<crate::service::error::Error> for Error {
    fn from(source: crate::service::error::Error) -> Self { Self::Service { source } }
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
            Self::Service { source } => source.into_response(),
            _ => Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .header(header::CONTENT_TYPE, "application/json")
                .body(body)
                .expect("It should be a valid `Response`"),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub message: String,
}
