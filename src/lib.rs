pub mod clients;
pub mod db;
pub mod entities;
pub mod errors;
pub mod handlers;
pub mod models;
pub mod openapi;
pub mod repositories;
pub mod services;
pub mod test_utils;
pub mod utils;

use anyhow::Result;
use std::{env::var, sync::Arc};
use tracing::info;

pub async fn run() -> Result<()> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();

    info!("Starting Inklings Server...");

    let database_url = var("DATABASE_URL").expect("DATABASE_URL must be set in .env file");
    let qdrant_url = var("QDRANT_URL").expect("QDRANT_URL must be set in .env file");
    let gemini_api_key = var("GEMINI_API_KEY").expect("GEMINI_API_KEY must be set in .env file");
    let host = var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = var("SERVER_PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("{}:{}", host, port);

    let db = Arc::new(db::create_connection(&database_url).await?);

    let qdrant_repo = Arc::new(
        repositories::QdrantRepository::new(qdrant_url)
            .await
            .expect("Failed to initialize Qdrant repository"),
    );

    let gemini_client = Arc::new(clients::GeminiClient::new(gemini_api_key));

    let app = handlers::create_router(
        db,
        qdrant_repo,
        gemini_client.clone() as Arc<dyn clients::Embedder>,
        gemini_client as Arc<dyn clients::TextGenerator>,
    );

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|_| panic!("Failed to bind to {}", addr));

    if host == "127.0.0.1" {
        info!("Server is running on http://localhost:{}/swagger-ui", port);
    }

    axum::serve(listener, app).await?;

    Ok(())
}
