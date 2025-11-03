
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use sqlx::Error as SqlxError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("bad request: {0}")]
    BadRequest(&'static str),
    #[error("unauthorized")]
    Unauthorized,
    #[error("forbidden")]
    Forbidden,
    #[error("not found")]
    NotFound,
    #[error("internal error: {0}")]
    Internal(&'static str),
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}

#[derive(Serialize)]
struct ErrorBody<'a> {
    code: u16,
    error_code: &'a str,
    message: &'a str,
    data: Option<()>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_code, msg) = match self {
            AppError::BadRequest(m) => (StatusCode::BAD_REQUEST, "00-01-00", m),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "01-01-00", "未授权"),
            AppError::Forbidden => (StatusCode::FORBIDDEN, "01-02-00", "禁止访问"),
            AppError::NotFound => (StatusCode::NOT_FOUND, "04-04-00", "未找到"),
            AppError::Internal(_) | AppError::Anyhow(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "05-00-00",
                "服务器内部错误",
            ),
        };
        let body = Json(ErrorBody {
            code: status.as_u16(),
            error_code,
            message: msg,
            data: None,
        });
        (status, body).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;

impl From<SqlxError> for AppError {
    fn from(value: SqlxError) -> Self {
        tracing::error!(error = ?value, "sqlx error");
        AppError::Internal("database error")
    }
}
