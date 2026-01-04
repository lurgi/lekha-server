use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use sea_orm::DbErr;
use thiserror::Error;

use crate::clients::ClientError;

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("Memo not found")]
    MemoNotFound,

    #[error("User not found")]
    UserNotFound,

    #[error("Unauthorized: you don't have permission to access this memo")]
    Unauthorized,

    #[error("Gemini API error: {0}")]
    GeminiApi(String),

    #[error("Qdrant error: {0}")]
    Qdrant(String),

    #[error("Failed to generate JWT token")]
    TokenGenerationFailed,

    #[error("Invalid or expired token")]
    InvalidToken,

    #[error("Missing JWT secret configuration")]
    MissingJwtSecret,

    #[error("Refresh token not found or invalid")]
    RefreshTokenNotFound,

    #[error("Refresh token expired")]
    RefreshTokenExpired,

    #[error("Database error: {0}")]
    Database(#[from] DbErr),
}

impl IntoResponse for ServiceError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::MemoNotFound => (StatusCode::NOT_FOUND, self.to_string()),
            Self::UserNotFound => (StatusCode::NOT_FOUND, self.to_string()),
            Self::Unauthorized => (StatusCode::FORBIDDEN, self.to_string()),
            Self::GeminiApi(_) => (
                StatusCode::BAD_GATEWAY,
                "External AI service error".to_string(),
            ),
            Self::Qdrant(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Vector database error".to_string(),
            ),
            Self::TokenGenerationFailed => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to generate authentication token".to_string(),
            ),
            Self::InvalidToken => (StatusCode::UNAUTHORIZED, self.to_string()),
            Self::MissingJwtSecret => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Server configuration error".to_string(),
            ),
            Self::RefreshTokenNotFound => (StatusCode::UNAUTHORIZED, self.to_string()),
            Self::RefreshTokenExpired => (StatusCode::UNAUTHORIZED, self.to_string()),
            Self::Database(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ),
        };

        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}

impl From<ClientError> for ServiceError {
    fn from(err: ClientError) -> Self {
        match err {
            ClientError::GeminiApi(msg) => ServiceError::GeminiApi(msg),
            ClientError::Network(msg) => ServiceError::GeminiApi(msg),
            ClientError::ParseError(msg) => ServiceError::GeminiApi(msg),
            ClientError::Qdrant(msg) => ServiceError::Qdrant(msg),
        }
    }
}
