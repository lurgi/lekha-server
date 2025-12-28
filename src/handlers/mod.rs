pub mod auth;
pub mod health_handler;
pub mod memo_handler;
pub mod user_handler;

use crate::services::{memo_service::MemoService, user_service::UserService};
use axum::{
    routing::{delete, get, patch, post, put},
    Router,
};
use sea_orm::DatabaseConnection;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<DatabaseConnection>,
    pub memo_service: Arc<MemoService>,
    pub user_service: Arc<UserService>,
}

pub fn create_router(db: Arc<DatabaseConnection>) -> Router {
    let memo_service = Arc::new(MemoService::new(db.clone()));
    let user_service = Arc::new(UserService::new(db.clone()));

    let app_state = AppState {
        db,
        memo_service,
        user_service,
    };

    Router::new()
        .route("/api/health", get(health_handler::health_check))
        .route("/api/users/oauth-login", post(user_handler::oauth_login))
        .nest(
            "/api/memos",
            Router::new()
                .route("/", post(memo_handler::create_memo))
                .route("/", get(memo_handler::list_memos))
                .route("/:id", get(memo_handler::get_memo))
                .route("/:id", put(memo_handler::update_memo))
                .route("/:id", delete(memo_handler::delete_memo))
                .route("/:id/pin", patch(memo_handler::toggle_pin)),
        )
        .with_state(app_state)
}
