use std::fmt::Display;

use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Response;
use tracing::error;
use tracing::warn;

use crate::exception::Exception;

pub type HttpResult<T> = Result<T, HttpError>;

#[derive(Debug)]
pub enum HttpError {
    NotFound(String),
    InternalError(Exception),
}

impl IntoResponse for HttpError {
    fn into_response(self) -> Response {
        match self {
            HttpError::InternalError(error) => {
                error!(
                    error_code = "INTERNAL_ERROR",
                    backtrace = format!("{error}"),
                    "{}",
                    error.message
                );
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Internal Error: {}", error.message),
                )
                    .into_response()
            }
            HttpError::NotFound(error) => {
                warn!(error_code = "NOT_FOUND", "Not Found: {error}");
                (StatusCode::NOT_FOUND, format!("Not Found: {error}")).into_response()
            }
        }
    }
}

impl<E> From<E> for HttpError
where
    E: Into<Exception>,
{
    fn from(err: E) -> Self {
        Self::InternalError(err.into())
    }
}

impl Display for HttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{self:?}"))
    }
}
