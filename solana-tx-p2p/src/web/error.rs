use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use snafu::Snafu;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("{source}"))]
    Service { source: crate::service::error::Error },
}

impl From<crate::service::error::Error> for Error {
    fn from(source: crate::service::error::Error) -> Self { Self::Service { source } }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Self::Service { source } => source.into_response(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub message: String,
}
