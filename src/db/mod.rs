use sea_orm::{Database, DatabaseConnection, DbErr};

pub async fn create_connection(database_url: &str) -> Result<DatabaseConnection, DbErr> {
    Database::connect(database_url).await
}

pub async fn test_connection(db: &DatabaseConnection) -> Result<(), DbErr> {
    db.ping().await?;
    Ok(())
}
