use chrono::Utc;
use sea_orm::*;
use std::sync::Arc;

use crate::entities::oauth_account::{self, Entity as OAuthAccount, OAuthProvider};

#[derive(Clone)]
pub struct OAuthAccountRepository {
    db: Arc<DatabaseConnection>,
}

impl OAuthAccountRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    pub async fn find_by_provider_and_id(
        &self,
        provider: &OAuthProvider,
        provider_user_id: &str,
    ) -> Result<Option<oauth_account::Model>, DbErr> {
        OAuthAccount::find()
            .filter(oauth_account::Column::Provider.eq(provider.clone()))
            .filter(oauth_account::Column::ProviderUserId.eq(provider_user_id))
            .one(self.db.as_ref())
            .await
    }

    pub async fn find_by_user_id(
        &self,
        user_id: i32,
    ) -> Result<Vec<oauth_account::Model>, DbErr> {
        OAuthAccount::find()
            .filter(oauth_account::Column::UserId.eq(user_id))
            .all(self.db.as_ref())
            .await
    }

    pub async fn create(
        &self,
        user_id: i32,
        provider: OAuthProvider,
        provider_user_id: String,
    ) -> Result<oauth_account::Model, DbErr> {
        let now = Utc::now().naive_utc();

        let active_model = oauth_account::ActiveModel {
            user_id: Set(user_id),
            provider: Set(provider),
            provider_user_id: Set(provider_user_id),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        active_model.insert(self.db.as_ref()).await
    }

    pub async fn delete(&self, id: i32) -> Result<DeleteResult, DbErr> {
        OAuthAccount::delete_by_id(id).exec(self.db.as_ref()).await
    }
}
