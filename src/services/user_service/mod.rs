use sea_orm::DatabaseConnection;
use std::sync::Arc;

use crate::{
    errors::ServiceError,
    models::{AuthResponse, OAuthLoginRequest, UserResponse},
    repositories::{OAuthAccountRepository, UserRepository},
    utils::jwt,
};

#[derive(Clone)]
pub struct UserService {
    user_repo: UserRepository,
    oauth_repo: OAuthAccountRepository,
}

impl UserService {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self {
            user_repo: UserRepository::new(db.clone()),
            oauth_repo: OAuthAccountRepository::new(db),
        }
    }

    pub async fn oauth_login(
        &self,
        req: OAuthLoginRequest,
    ) -> Result<AuthResponse, ServiceError> {
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

        let jwt_secret = std::env::var("JWT_SECRET")
            .map_err(|_| ServiceError::MissingJwtSecret)?;

        let expiration_hours = std::env::var("JWT_EXPIRATION_HOURS")
            .unwrap_or_else(|_| "24".to_string())
            .parse::<i64>()
            .unwrap_or(24);

        let access_token = jwt::generate_token(user.id, &jwt_secret, expiration_hours)
            .map_err(|_| ServiceError::TokenGenerationFailed)?;

        let expires_in = expiration_hours * 3600;

        Ok(AuthResponse {
            user: UserResponse::from(user),
            access_token,
            expires_in,
        })
    }
}

#[cfg(test)]
mod tests;
