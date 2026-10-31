use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

/// API 공통 에러 (doc/09_api.md §1 에러 형식).
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{message}")]
    Http {
        status: StatusCode,
        code: &'static str,
        message: String,
    },
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl AppError {
    fn http(status: StatusCode, code: &'static str, message: impl Into<String>) -> Self {
        Self::Http {
            status,
            code,
            message: message.into(),
        }
    }
    pub fn bad_request(code: &'static str, m: impl Into<String>) -> Self {
        Self::http(StatusCode::BAD_REQUEST, code, m)
    }
    pub fn unauthorized(code: &'static str, m: impl Into<String>) -> Self {
        Self::http(StatusCode::UNAUTHORIZED, code, m)
    }
    pub fn forbidden(code: &'static str, m: impl Into<String>) -> Self {
        Self::http(StatusCode::FORBIDDEN, code, m)
    }
    pub fn not_found(code: &'static str, m: impl Into<String>) -> Self {
        Self::http(StatusCode::NOT_FOUND, code, m)
    }
    pub fn too_many(code: &'static str, m: impl Into<String>) -> Self {
        Self::http(StatusCode::TOO_MANY_REQUESTS, code, m)
    }
    pub fn unavailable(code: &'static str, m: impl Into<String>) -> Self {
        Self::http(StatusCode::SERVICE_UNAVAILABLE, code, m)
    }
}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        Self::Internal(e.into())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            Self::Http {
                status,
                code,
                message,
            } => (status, code, message),
            Self::Internal(e) => {
                tracing::error!(error = ?e, "internal error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL",
                    "서버 오류가 발생했습니다.".to_owned(),
                )
            }
        };
        let body = json!({ "error": { "code": code, "message": message, "status": status.as_u16() } });
        (status, Json(body)).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;
