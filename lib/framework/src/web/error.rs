use std::fmt::Display;

use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Response;

use crate::exception::Exception;
use crate::log;

pub type HttpResult<T> = Result<T, HttpError>;

#[derive(Debug)]
pub struct HttpError {
    status_code: StatusCode,
    exception: Exception,
}

impl HttpError {
    pub fn internal_error<E>(error: E) -> Self
    where
        E: Into<Exception>,
    {
        Self {
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            exception: error.into(),
        }
    }

    pub fn not_found<E>(error: E) -> Self
    where
        E: Into<Exception>,
    {
        Self {
            status_code: StatusCode::NOT_FOUND,
            exception: error.into(),
        }
    }

    pub fn forbidden<E>(error: E) -> Self
    where
        E: Into<Exception>,
    {
        Self {
            status_code: StatusCode::FORBIDDEN,
            exception: error.into(),
        }
    }
}

impl IntoResponse for HttpError {
    fn into_response(self) -> Response {
        log::log_exception(&self.exception);
        (self.status_code, self.exception.message).into_response()
    }
}

impl<E> From<E> for HttpError
where
    E: Into<Exception>,
{
    fn from(err: E) -> Self {
        Self::internal_error(err)
    }
}

impl Display for HttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{self:?}"))
    }
}
