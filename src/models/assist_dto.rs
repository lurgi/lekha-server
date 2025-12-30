use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct AssistRequest {
    pub prompt: String,

    #[serde(default = "default_limit")]
    pub limit: u64,
}

fn default_limit() -> u64 {
    5
}

#[derive(Debug, Serialize)]
pub struct AssistResponse {
    pub suggestion: String,
    pub similar_memos: Vec<SimilarMemo>,
}

#[derive(Debug, Serialize)]
pub struct SimilarMemo {
    pub id: i32,
    pub content: String,
    pub created_at: NaiveDateTime,
}
