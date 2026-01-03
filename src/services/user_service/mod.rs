use sea_orm::DatabaseConnection;
use std::sync::Arc;

use crate::{
    errors::ServiceError,
    models::{AuthResponse, OAuthLoginRequest, UserResponse},
    repositories::{OAuthAccountRepository, UserRepository},
    services::TokenService,
};

#[derive(Clone)]
pub struct UserService {
    user_repo: UserRepository,
    oauth_repo: OAuthAccountRepository,
    token_service: TokenService,
}

impl UserService {
    pub fn new(db: Arc<DatabaseConnection>) -> Result<Self, ServiceError> {
        Ok(Self {
            user_repo: UserRepository::new(db.clone()),
            oauth_repo: OAuthAccountRepository::new(db.clone()),
            token_service: TokenService::new(db)?,
        })
    }

    pub async fn oauth_login(
        &self,
        req: OAuthLoginRequest,
    ) -> Result<(AuthResponse, String, String), ServiceError> {
        let user = if let Some(oauth_account) = self
            .oauth_repo
            .find_by_provider_and_id(&req.provider, &req.provider_user_id)
            .await?
        {
            self.user_repo
                .find_by_id(oauth_account.user_id)
                .await?
                .ok_or(ServiceError::UserNotFound)?
        } else {
            let user = if let Some(existing_user) =
                self.user_repo.find_by_email(&req.email).await?
            {
                existing_user
            } else {
                self.user_repo
                    .create(req.username.clone(), req.email.clone(), None)
                    .await?
            };

            self.oauth_repo
                .create(user.id, req.provider, req.provider_user_id)
                .await?;

            user
        };

        let access_token = self.token_service.generate_access_token(user.id)?;
        let refresh_token = self.token_service.generate_refresh_token(user.id).await?;

        Ok((
            AuthResponse {
                user: UserResponse::from(user),
            },
            access_token,
            refresh_token,
        ))
    }

    /// Access Token 재발급 (Refresh Token Rotation)
    pub async fn refresh_tokens(
        &self,
        refresh_token: &str,
    ) -> Result<(String, String, i32), ServiceError> {
        let (access_token, user_id) = self
            .token_service
            .refresh_access_token(refresh_token)
            .await?;

        let new_refresh_token = self.token_service.generate_refresh_token(user_id).await?;

        self.token_service
            .revoke_refresh_token(refresh_token)
            .await?;

        Ok((access_token, new_refresh_token, user_id))
    }

    /// 로그아웃
    pub async fn logout(&self, refresh_token: &str) -> Result<(), ServiceError> {
        self.token_service.revoke_refresh_token(refresh_token).await
    }

    /// 모든 디바이스 로그아웃
    pub async fn logout_all(&self, user_id: i32) -> Result<(), ServiceError> {
        self.token_service
            .revoke_all_refresh_tokens(user_id)
            .await
    }
}

#[cfg(test)]
mod tests;
