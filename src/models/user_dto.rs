use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::entities::{oauth_account::OAuthProvider, user};

#[derive(Debug, Deserialize, Clone)]
pub struct OAuthLoginRequest {
    pub provider: OAuthProvider,
    pub provider_user_id: String,
    pub email: String,
    pub username: String,
}

#[derive(Debug, Serialize, Clone, PartialEq)]
pub struct UserResponse {
    pub id: i32,
    pub username: String,
    pub email: String,
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
