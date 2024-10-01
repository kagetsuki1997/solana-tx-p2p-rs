mod peer;

use axum::Router;
use http::Request;
use hyper::Body;
use tower_http::{classify::ServerErrorsFailureClass, trace::TraceLayer};
use tracing::Span;
use utoipa::OpenApi;

use crate::{app_state::AppState, error::Result, model::Peer};

pub fn apis<S>(_app_state: &S) -> Result<Router>
where
    S: AppState + Clone + Send + Sync + 'static,
{
    Ok(Router::new().nest(
        "/api",
        Router::new().merge(self::peer::v1::<S>()).layer(
            TraceLayer::new_for_http()
                .on_request(|request: &Request<Body>, _span: &Span| {
                    let request_url = request.uri();
                    tracing::info!(
                        "request: {}, query: {}",
                        request_url.path(),
                        request_url.query().unwrap_or_default()
                    );
                })
                .on_failure(|error, _latency, _span: &Span| {
                    if let ServerErrorsFailureClass::StatusCode(code) = error {
                        tracing::error!("{code}");
                    } else {
                        tracing::error!("{error}");
                    }
                }),
        ),
    ))
}

#[derive(OpenApi)]
#[openapi(paths(peer::v1::discovery,), components(schemas(Peer,)))]
pub struct ApiDoc;
