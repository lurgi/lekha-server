pub mod assist_handler;
pub mod auth;
pub mod auth_handler;
pub mod health_handler;
pub mod memo_handler;
pub mod user_handler;

use crate::{
    clients::{Embedder, TextGenerator},
    openapi::ApiDoc,
    repositories::QdrantRepo,
    services::{
        assist_service::AssistService, memo_service::MemoService, user_service::UserService,
    },
};
use axum::{
    http::{header, HeaderValue, Method},
    routing::{delete, get, patch, post, put},
    Router,
};
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use tower_cookies::CookieManagerLayer;
use tower_http::cors::CorsLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<DatabaseConnection>,
    pub memo_service: Arc<MemoService>,
    pub assist_service: Arc<AssistService>,
    pub user_service: Arc<UserService>,
}

pub fn create_router(
    db: Arc<DatabaseConnection>,
    qdrant_repo: Arc<dyn QdrantRepo>,
    embedder: Arc<dyn Embedder>,
    text_generator: Arc<dyn TextGenerator>,
) -> Router {
    let memo_service = Arc::new(MemoService::new(
        db.clone(),
        qdrant_repo.clone(),
        embedder.clone(),
    ));

    let assist_service = Arc::new(AssistService::new(
        db.clone(),
        qdrant_repo,
        embedder,
        text_generator,
    ));

    let user_service = Arc::new(
        UserService::new(db.clone()).expect("Failed to initialize UserService"),
    );

    let app_state = AppState {
        db,
        memo_service,
        assist_service,
        user_service,
    };

    let openapi = ApiDoc::openapi();

    let frontend_url = std::env::var("FRONTEND_URL")
        .unwrap_or_else(|_| "http://localhost:3000".to_string());

    let cors = CorsLayer::new()
        .allow_origin(
            frontend_url
                .parse::<HeaderValue>()
                .expect("Invalid FRONTEND_URL"),
        )
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
        ])
        .allow_credentials(true)
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]);

    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", openapi))
        .route("/api/health", get(health_handler::health_check))
        .route("/api/assist", post(assist_handler::assist))
        .route("/api/users/oauth-login", post(user_handler::oauth_login))
        .route("/api/auth/refresh", post(auth_handler::refresh))
        .route("/api/auth/logout", post(auth_handler::logout))
        .route("/api/auth/logout-all", delete(auth_handler::logout_all))
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
        .layer(CookieManagerLayer::new())
        .layer(cors)
        .with_state(app_state)
}
