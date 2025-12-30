use crate::repositories::QdrantRepo;
use async_trait::async_trait;
use sea_orm::DbErr;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct MockQdrantRepository {
    memos: Arc<Mutex<HashMap<i32, (i32, Vec<f32>)>>>,
}

impl MockQdrantRepository {
    pub fn new() -> Self {
        Self {
            memos: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl QdrantRepo for MockQdrantRepository {
    async fn upsert_memo(
        &self,
        memo_id: i32,
        user_id: i32,
        vector: Vec<f32>,
    ) -> Result<(), DbErr> {
        self.memos
            .lock()
            .unwrap()
            .insert(memo_id, (user_id, vector));
        Ok(())
    }

    async fn search_similar(
        &self,
        user_id: i32,
        _query_vector: Vec<f32>,
        limit: u64,
    ) -> Result<Vec<i32>, DbErr> {
        let memos = self.memos.lock().unwrap();
        let memo_ids: Vec<i32> = memos
            .iter()
            .filter(|(_, (uid, _))| *uid == user_id)
            .map(|(memo_id, _)| *memo_id)
            .take(limit as usize)
            .collect();
        Ok(memo_ids)
    }

    async fn delete_memo(&self, memo_id: i32) -> Result<(), DbErr> {
        self.memos.lock().unwrap().remove(&memo_id);
        Ok(())
    }
}
