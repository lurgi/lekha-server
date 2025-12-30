pub use sea_orm_migration::prelude::*;

mod m20241210_000001_create_users_table;
mod m20241220_000001_create_memos_table;
mod m20241222_000001_add_oauth_accounts;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20241210_000001_create_users_table::Migration),
            Box::new(m20241220_000001_create_memos_table::Migration),
            Box::new(m20241222_000001_add_oauth_accounts::Migration),
        ]
    }
}
