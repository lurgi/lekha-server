use sea_orm::DatabaseConnection;
use std::sync::Arc;

use crate::{
    errors::ServiceError,
    models::{OAuthLoginRequest, UserResponse},
    repositories::{OAuthAccountRepository, UserRepository},
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
    ) -> Result<UserResponse, ServiceError> {
        if let Some(oauth_account) = self
            .oauth_repo
            .find_by_provider_and_id(&req.provider, &req.provider_user_id)
            .await?
        {
            let user = self
                .user_repo
                .find_by_id(oauth_account.user_id)
                .await?
                .ok_or(ServiceError::UserNotFound)?;

            return Ok(UserResponse::from(user));
        }

        let user = if let Some(existing_user) = self.user_repo.find_by_email(&req.email).await? {
            existing_user
        } else {
            self.user_repo
                .create(req.username.clone(), req.email.clone(), None)
                .await?
        };

        self.oauth_repo
            .create(user.id, req.provider, req.provider_user_id)
            .await?;

        Ok(UserResponse::from(user))
    }
}
