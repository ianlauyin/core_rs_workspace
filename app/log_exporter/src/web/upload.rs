use std::sync::Arc;

use axum::Json;
use axum::Router;
use axum::debug_handler;
use axum::http::StatusCode;
use axum::routing::put;
use core_ng::web::error::HttpError;
use serde::Deserialize;
use serde::Serialize;

use crate::AppState;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new().route("/upload", put(upload))
}

#[derive(Serialize, Deserialize, Debug)]
struct UploadRequest {}

#[debug_handler]
async fn upload(Json(request): Json<UploadRequest>) -> Result<StatusCode, HttpError> {
    println!("{request:?}");
    Ok(StatusCode::NO_CONTENT)
}
