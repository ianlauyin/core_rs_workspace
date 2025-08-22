use std::sync::Arc;

use axum::Router;
use axum::debug_handler;
use axum::routing::post;
use framework::exception::Exception;
use framework::validation_error;
use framework::web::body::Json;
use framework::web::error::HttpResult;
use serde::Deserialize;
use serde::Serialize;

use crate::AppState;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new().route("/hello", post(hello))
}

#[derive(Debug, Deserialize)]
struct HelloRequest {
    message: String,
}

impl HelloRequest {
    fn validate(&self) -> Result<(), Exception> {
        if self.message.len() > 10 {
            return Err(validation_error!(message = "message len must less than 10"));
        }
        Ok(())
    }
}

#[derive(Debug, Serialize)]
struct HelloResponse {
    message: String,
}

#[debug_handler]
async fn hello(Json(request): Json<HelloRequest>) -> HttpResult<Json<HelloResponse>> {
    request.validate()?;

    Ok(Json(HelloResponse {
        message: request.message,
    }))
}
