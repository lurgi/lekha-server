use sea_orm::*;
use std::sync::Arc;

use crate::entities::refresh_token::{self, Entity as RefreshToken};

#[derive(Clone)]
pub struct RefreshTokenRepository {
    db: Arc<DatabaseConnection>,
}

impl RefreshTokenRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    /// Refresh Token 생성
    pub async fn create(
        &self,
        user_id: i32,
        token_hash: String,
        expires_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<refresh_token::Model, DbErr> {
        let now = chrono::Utc::now().naive_utc();

        let refresh_token = refresh_token::ActiveModel {
            user_id: Set(user_id),
            token_hash: Set(token_hash),
            expires_at: Set(expires_at.naive_utc()),
            created_at: Set(now),
            ..Default::default()
        };

        refresh_token.insert(self.db.as_ref()).await
    }

    /// Token Hash로 조회
    pub async fn find_by_token_hash(
        &self,
        token_hash: &str,
    ) -> Result<Option<refresh_token::Model>, DbErr> {
        RefreshToken::find()
            .filter(refresh_token::Column::TokenHash.eq(token_hash))
            .one(self.db.as_ref())
            .await
    }

    /// User ID로 모든 토큰 조회
    pub async fn find_by_user_id(
        &self,
        user_id: i32,
    ) -> Result<Vec<refresh_token::Model>, DbErr> {
        RefreshToken::find()
            .filter(refresh_token::Column::UserId.eq(user_id))
            .all(self.db.as_ref())
            .await
    }

    /// Token Hash로 삭제 (로그아웃)
    pub async fn delete_by_token_hash(&self, token_hash: &str) -> Result<DeleteResult, DbErr> {
        RefreshToken::delete_many()
            .filter(refresh_token::Column::TokenHash.eq(token_hash))
            .exec(self.db.as_ref())
            .await
    }

    /// User ID로 모든 토큰 삭제 (모든 디바이스 로그아웃)
    pub async fn delete_by_user_id(&self, user_id: i32) -> Result<DeleteResult, DbErr> {
        RefreshToken::delete_many()
            .filter(refresh_token::Column::UserId.eq(user_id))
            .exec(self.db.as_ref())
            .await
    }

    /// 만료된 토큰 정리 (크론잡/스케줄러에서 호출)
    pub async fn delete_expired(&self) -> Result<DeleteResult, DbErr> {
        let now = chrono::Utc::now().naive_utc();

        RefreshToken::delete_many()
            .filter(refresh_token::Column::ExpiresAt.lt(now))
            .exec(self.db.as_ref())
            .await
    }
}
