use std::sync::Arc;

use axum::Json;
use axum::Router;
use axum::debug_handler;
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::put;
use chrono::NaiveDate;
use core_ng::task;
use core_ng::web::error::HttpError;
use serde::Deserialize;
use serde::Serialize;

use crate::AppState;
use crate::service::upload_archive;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new().route("/upload", put(upload))
}

#[derive(Serialize, Deserialize, Debug)]
struct UploadRequest {
    date: NaiveDate,
}

#[debug_handler]
async fn upload(state: State<Arc<AppState>>, Json(request): Json<UploadRequest>) -> Result<StatusCode, HttpError> {
    task::spawn_action("upload", async move { upload_archive(request.date, &state).await });
    Ok(StatusCode::NO_CONTENT)
}
