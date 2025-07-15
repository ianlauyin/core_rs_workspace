use std::sync::Arc;

use axum::Router;
use axum::debug_handler;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::http::HeaderValue;
use axum::http::header;
use axum::routing::options;
use framework::exception;
use framework::exception::Severity;
use framework::web::error::HttpError;
use framework::web::error::HttpResult;

use crate::AppState;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new().route("/event/{app}", options(event_options))
}

#[debug_handler]
async fn event_options(_state: State<Arc<AppState>>, headers: HeaderMap) -> HttpResult<HeaderMap> {
    let mut response_headers = HeaderMap::new();

    let origin = headers.get(header::ORIGIN).ok_or_else(|| {
        HttpError::forbidden(exception!(
            severity = Severity::Warn,
            code = "FORDIDDEN",
            message = "access denied"
        ))
    })?;
    response_headers.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, origin.clone());

    response_headers.insert(
        header::ACCESS_CONTROL_ALLOW_METHODS,
        HeaderValue::from_static("POST, PUT, OPTIONS"),
    );
    response_headers.insert(
        header::ACCESS_CONTROL_ALLOW_HEADERS,
        HeaderValue::from_static("Accept, Content-Type"),
    );
    response_headers.insert(
        header::ACCESS_CONTROL_ALLOW_CREDENTIALS,
        HeaderValue::from_static("true"),
    );

    Ok(response_headers)
}
