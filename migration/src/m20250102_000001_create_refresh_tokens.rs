use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // refresh_tokens 테이블 생성
        manager
            .create_table(
                Table::create()
                    .table(RefreshToken::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(RefreshToken::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(RefreshToken::UserId).integer().not_null())
                    .col(
                        ColumnDef::new(RefreshToken::TokenHash)
                            .string()
                            .string_len(255)
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(RefreshToken::ExpiresAt)
                            .timestamp()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(RefreshToken::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-refresh_tokens-user_id")
                            .from(RefreshToken::Table, RefreshToken::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // user_id 인덱스 생성
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-refresh_tokens-user_id")
                    .table(RefreshToken::Table)
                    .col(RefreshToken::UserId)
                    .to_owned(),
            )
            .await?;

        // token_hash 인덱스 생성
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-refresh_tokens-token_hash")
                    .table(RefreshToken::Table)
                    .col(RefreshToken::TokenHash)
                    .to_owned(),
            )
            .await?;

        // expires_at 인덱스 생성
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-refresh_tokens-expires_at")
                    .table(RefreshToken::Table)
                    .col(RefreshToken::ExpiresAt)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(RefreshToken::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum RefreshToken {
    Table,
    Id,
    UserId,
    TokenHash,
    ExpiresAt,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}
