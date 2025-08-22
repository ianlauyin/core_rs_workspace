use std::fmt::Display;

use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Response;
use serde::Deserialize;
use serde::Serialize;

use crate::exception;
use crate::exception::Exception;
use crate::exception::Severity;
use crate::log;
use crate::web::body::Json;

pub type HttpResult<T> = Result<T, HttpError>;

pub const ERROR_CODE_BAD_REQUEST: &str = "BAD_REQUEST";
pub const ERROR_CODE_NOT_FOUND: &str = "NOT_FOUND";

#[derive(Debug)]
pub struct HttpError {
    status_code: StatusCode,
    body: HttpErrorBody,
}

#[derive(Debug, Serialize, Deserialize)]
struct HttpErrorBody {
    severity: Severity,
    code: Option<String>,
    message: String,
}

impl IntoResponse for HttpError {
    fn into_response(self) -> Response {
        (self.status_code, Json(self.body)).into_response()
    }
}

impl<E> From<E> for HttpError
where
    E: Into<Exception>,
{
    fn from(err: E) -> Self {
        let exception: Exception = err.into();
        log::log_exception(&exception);

        let status_code = exception
            .code
            .as_deref()
            .map(|code| match code {
                ERROR_CODE_BAD_REQUEST => StatusCode::BAD_REQUEST,
                exception::ERROR_CODE_VALIDATION_ERROR => StatusCode::BAD_REQUEST,
                ERROR_CODE_NOT_FOUND => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

        Self {
            status_code,
            body: HttpErrorBody {
                severity: exception.severity,
                code: exception.code,
                message: exception.message,
            },
        }
    }
}

impl Display for HttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{self:?}"))
    }
}
