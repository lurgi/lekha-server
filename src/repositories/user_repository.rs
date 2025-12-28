use chrono::Utc;
use sea_orm::*;
use std::sync::Arc;

use crate::entities::user::{self, Entity as User};

#[derive(Clone)]
pub struct UserRepository {
    db: Arc<DatabaseConnection>,
}

impl UserRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    pub async fn find_by_id(&self, id: i32) -> Result<Option<user::Model>, DbErr> {
        User::find_by_id(id).one(self.db.as_ref()).await
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<user::Model>, DbErr> {
        User::find()
            .filter(user::Column::Email.eq(email))
            .one(self.db.as_ref())
            .await
    }

    pub async fn find_by_username(&self, username: &str) -> Result<Option<user::Model>, DbErr> {
        User::find()
            .filter(user::Column::Username.eq(username))
            .one(self.db.as_ref())
            .await
    }

    pub async fn create(
        &self,
        username: String,
        email: String,
        password_hash: Option<String>,
    ) -> Result<user::Model, DbErr> {
        let now = Utc::now().naive_utc();

        let active_model = user::ActiveModel {
            username: Set(username),
            email: Set(email),
            password_hash: Set(password_hash),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        active_model.insert(self.db.as_ref()).await
    }

    pub async fn update(
        &self,
        id: i32,
        username: Option<String>,
        email: Option<String>,
    ) -> Result<user::Model, DbErr> {
        let user = self
            .find_by_id(id)
            .await?
            .ok_or(DbErr::RecordNotFound("User not found".into()))?;

        let mut active_model: user::ActiveModel = user.into();

        if let Some(username) = username {
            active_model.username = Set(username);
        }
        if let Some(email) = email {
            active_model.email = Set(email);
        }
        active_model.updated_at = Set(Utc::now().naive_utc());

        active_model.update(self.db.as_ref()).await
    }

    pub async fn delete(&self, id: i32) -> Result<DeleteResult, DbErr> {
        User::delete_by_id(id).exec(self.db.as_ref()).await
    }
}
