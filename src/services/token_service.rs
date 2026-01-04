use chrono::{Duration, Utc};
use sea_orm::DatabaseConnection;
use sha2::{Digest, Sha256};
use std::sync::Arc;

use crate::{errors::ServiceError, repositories::RefreshTokenRepository, utils::jwt};

const ACCESS_TOKEN_EXPIRATION_MINUTES: i64 = 15;
const REFRESH_TOKEN_EXPIRATION_DAYS: i64 = 7;

#[derive(Clone)]
pub struct TokenService {
    refresh_token_repo: RefreshTokenRepository,
    jwt_secret: String,
}

impl TokenService {
    pub fn new(db: Arc<DatabaseConnection>) -> Result<Self, ServiceError> {
        let jwt_secret = std::env::var("JWT_SECRET").map_err(|_| ServiceError::MissingJwtSecret)?;

        Ok(Self {
            refresh_token_repo: RefreshTokenRepository::new(db),
            jwt_secret,
        })
    }

    /// Access Token 생성 (15분)
    pub fn generate_access_token(&self, user_id: i32) -> Result<String, ServiceError> {
        jwt::generate_token(
            user_id,
            &self.jwt_secret,
            ACCESS_TOKEN_EXPIRATION_MINUTES / 60,
        )
        .map_err(|_| ServiceError::TokenGenerationFailed)
    }

    /// Refresh Token 생성 및 DB 저장 (7일)
    pub async fn generate_refresh_token(&self, user_id: i32) -> Result<String, ServiceError> {
        let refresh_token = uuid::Uuid::new_v4().to_string();

        let token_hash = Self::hash_token(&refresh_token);

        let expires_at = Utc::now() + Duration::days(REFRESH_TOKEN_EXPIRATION_DAYS);

        self.refresh_token_repo
            .create(user_id, token_hash, expires_at)
            .await?;

        Ok(refresh_token)
    }

    /// Refresh Token으로 Access Token 재발급
    pub async fn refresh_access_token(
        &self,
        refresh_token: &str,
    ) -> Result<(String, i32), ServiceError> {
        let token_hash = Self::hash_token(refresh_token);

        let stored_token = self
            .refresh_token_repo
            .find_by_token_hash(&token_hash)
            .await?
            .ok_or(ServiceError::RefreshTokenNotFound)?;

        let now = Utc::now().naive_utc();
        if stored_token.expires_at < now {
            self.refresh_token_repo
                .delete_by_token_hash(&token_hash)
                .await?;
            return Err(ServiceError::RefreshTokenExpired);
        }

        let access_token = self.generate_access_token(stored_token.user_id)?;

        Ok((access_token, stored_token.user_id))
    }

    /// Refresh Token 무효화 (로그아웃)
    pub async fn revoke_refresh_token(&self, refresh_token: &str) -> Result<(), ServiceError> {
        let token_hash = Self::hash_token(refresh_token);
        self.refresh_token_repo
            .delete_by_token_hash(&token_hash)
            .await?;
        Ok(())
    }

    /// 모든 디바이스 로그아웃
    pub async fn revoke_all_refresh_tokens(&self, user_id: i32) -> Result<(), ServiceError> {
        self.refresh_token_repo.delete_by_user_id(user_id).await?;
        Ok(())
    }

    /// SHA256 해시 생성
    fn hash_token(token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Access Token 만료 시간 (초)
    pub fn access_token_max_age() -> i64 {
        ACCESS_TOKEN_EXPIRATION_MINUTES * 60
    }

    /// Refresh Token 만료 시간 (초)
    pub fn refresh_token_max_age() -> i64 {
        REFRESH_TOKEN_EXPIRATION_DAYS * 24 * 60 * 60
    }
}
