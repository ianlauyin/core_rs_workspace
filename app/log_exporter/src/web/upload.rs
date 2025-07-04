use std::sync::Arc;

use anyhow::Context;
use axum::Json;
use axum::Router;
use axum::debug_handler;
use axum::http::StatusCode;
use axum::routing::get;
use axum::routing::put;
use core_ng::web::error::HttpError;
use serde::Deserialize;
use serde::Serialize;

use crate::AppState;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new().route("/upload", put(upload)).route("/test", get(test))
}

#[derive(Serialize, Deserialize, Debug)]
struct UploadRequest {}

#[debug_handler]
async fn upload(Json(request): Json<UploadRequest>) -> Result<StatusCode, HttpError> {
    println!("{request:?}");
    Ok(StatusCode::NO_CONTENT)
}

#[debug_handler]
async fn test() -> Result<StatusCode, HttpError> {
    let _ = Option::None.context("test empty")?;
    Ok(StatusCode::NO_CONTENT)
}
