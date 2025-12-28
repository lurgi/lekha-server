use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use sea_orm::DbErr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("Memo not found")]
    MemoNotFound,

    #[error("User not found")]
    UserNotFound,

    #[error("Unauthorized: you don't have permission to access this memo")]
    Unauthorized,

    #[error("Database error: {0}")]
    Database(#[from] DbErr),
}

impl IntoResponse for ServiceError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::MemoNotFound => (StatusCode::NOT_FOUND, self.to_string()),
            Self::UserNotFound => (StatusCode::NOT_FOUND, self.to_string()),
            Self::Unauthorized => (StatusCode::FORBIDDEN, self.to_string()),
            Self::Database(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ),
        };

        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}
