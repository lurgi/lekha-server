pub mod auth;
pub mod health_handler;
pub mod memo_handler;

use crate::{
    clients::Embedder,
    repositories::QdrantRepo,
    services::memo_service::MemoService,
};
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
}

pub fn create_router(
    db: Arc<DatabaseConnection>,
    qdrant_repo: Arc<dyn QdrantRepo>,
    embedder: Arc<dyn Embedder>,
) -> Router {
    let memo_service = Arc::new(MemoService::new(db.clone(), qdrant_repo, embedder));

    let app_state = AppState { db, memo_service };

    Router::new()
        .route("/api/health", get(health_handler::health_check))
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
