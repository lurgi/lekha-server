mod db;

use anyhow::Result;
use std::env::var;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();

    info!("Starting Inklings Server with SeaORM...");

    let database_url = var("DATABASE_URL").expect("DATABASE_URL must be set in .env file");

    info!("Connecting to database...");
    let db = db::create_connection(&database_url).await?;

    match db::test_connection(&db).await {
        Ok(_) => info!("Database connection successful!"),
        Err(e) => {
            error!("Failed to connect to database: {}", e);
            return Err(e.into());
        }
    }

    info!("Inklings Server is running with SeaORM!");

    Ok(())
}
