mod db;
mod handlers;

use anyhow::Result;
use std::{env::var, sync::Arc};
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    println!("DEBUG: main() started");  // 디버그용
    dotenv::dotenv().ok();
    println!("DEBUG: dotenv loaded");  // 디버그용
    tracing_subscriber::fmt::init();
    println!("DEBUG: tracing initialized");  // 디버그용

    info!("Starting Inklings Server...");

    let database_url = var("DATABASE_URL").expect("DATABASE_URL must be set in .env file");
    let host = var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = var("SERVER_PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("{}:{}", host, port);

    let db = Arc::new(db::create_connection(&database_url).await?);

    let app = handlers::create_router(db);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|_| panic!("Failed to bind to {}", addr));

    if host == "127.0.0.1" {
        info!("Server is running on http://localhost:{}", port);
    }

    axum::serve(listener, app).await?;

    Ok(())
}
