use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // refresh_token 테이블을 refresh_tokens로 이름 변경
        manager
            .rename_table(
                Table::rename()
                    .table(RefreshToken::Table, RefreshTokens::Table)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 롤백: refresh_tokens를 refresh_token으로 되돌림
        manager
            .rename_table(
                Table::rename()
                    .table(RefreshTokens::Table, RefreshToken::Table)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum RefreshToken {
    Table,
}

#[derive(DeriveIden)]
enum RefreshTokens {
    Table,
}
