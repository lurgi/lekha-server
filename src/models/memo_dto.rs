use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::entities::memo;

#[derive(Debug, Deserialize)]
pub struct CreateMemoRequest {
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateMemoRequest {
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct MemoResponse {
    pub id: i32,
    pub user_id: i32,
    pub content: String,
    pub is_pinned: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl From<memo::Model> for MemoResponse {
    fn from(memo: memo::Model) -> Self {
        Self {
            id: memo.id,
            user_id: memo.user_id,
            content: memo.content,
            is_pinned: memo.is_pinned,
            created_at: memo.created_at,
            updated_at: memo.updated_at,
        }
    }
}
