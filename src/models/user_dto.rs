use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::entities::{oauth_account::OAuthProvider, user};

#[derive(Debug, Deserialize, Clone, ToSchema)]
pub struct OAuthLoginRequest {
    pub provider: OAuthProvider,
    #[schema(example = "google_123456789")]
    pub provider_user_id: String,
    #[schema(example = "user@example.com")]
    pub email: String,
    #[schema(example = "홍길동")]
    pub username: String,
}

#[derive(Debug, Serialize, Clone, PartialEq, ToSchema)]
pub struct UserResponse {
    #[schema(example = 1)]
    pub id: i32,
    #[schema(example = "홍길동")]
    pub username: String,
    #[schema(example = "user@example.com")]
    pub email: String,
    #[schema(example = "2024-01-15T10:30:00")]
    pub created_at: NaiveDateTime,
}

impl From<user::Model> for UserResponse {
    fn from(user: user::Model) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            created_at: user.created_at,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AuthResponse {
    pub user: UserResponse,
    #[schema(example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")]
    pub access_token: String,
    #[schema(example = 86400)]
    pub expires_in: i64,
}
