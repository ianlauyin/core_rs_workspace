use axum::Json;
use axum::Router;
use axum::debug_handler;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::routing::get;
use axum::routing::put;
use core_ng::web::error::HttpError;
use serde::Deserialize;
use serde::Serialize;
use tracing::warn;

use crate::ApiState;

pub fn routes() -> Router<ApiState> {
    Router::new()
        .route("/upload", put(upload))
        .route("/customer/{id}", get(get_customer))
}

#[derive(Serialize, Deserialize, Debug)]
struct UploadRequest {}

#[debug_handler]
async fn upload(Json(request): Json<UploadRequest>) -> Result<StatusCode, HttpError> {
    println!("{request:?}");
    Ok(StatusCode::NO_CONTENT)
}

#[debug_handler]
async fn get_customer(Path(id): Path<String>) -> Json<UploadRequest> {
    warn!("testm, id={id}");
    Json::from(UploadRequest {})
}
