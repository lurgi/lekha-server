use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 1. users 테이블의 password_hash 컬럼을 nullable로 변경
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .modify_column(ColumnDef::new(Users::PasswordHash).string().null())
                    .to_owned(),
            )
            .await?;

        // 2. oauth_accounts 테이블 생성
        manager
            .create_table(
                Table::create()
                    .table(OAuthAccounts::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(OAuthAccounts::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(OAuthAccounts::UserId).integer().not_null())
                    .col(
                        ColumnDef::new(OAuthAccounts::Provider)
                            .string_len(20)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(OAuthAccounts::ProviderUserId)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(OAuthAccounts::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(OAuthAccounts::UpdatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-oauth_accounts-user_id")
                            .from(OAuthAccounts::Table, OAuthAccounts::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // 3. oauth_accounts 테이블 인덱스 생성
        manager
            .create_index(
                Index::create()
                    .name("idx-oauth_accounts-user_id")
                    .table(OAuthAccounts::Table)
                    .col(OAuthAccounts::UserId)
                    .to_owned(),
            )
            .await?;

        // 4. provider + provider_user_id 복합 유니크 인덱스
        manager
            .create_index(
                Index::create()
                    .name("idx-oauth_accounts-provider-provider_user_id")
                    .table(OAuthAccounts::Table)
                    .col(OAuthAccounts::Provider)
                    .col(OAuthAccounts::ProviderUserId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 1. oauth_accounts 테이블 삭제
        manager
            .drop_table(Table::drop().table(OAuthAccounts::Table).to_owned())
            .await?;

        // 2. users 테이블의 password_hash를 다시 not null로 변경
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .modify_column(ColumnDef::new(Users::PasswordHash).string().not_null())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
    PasswordHash,
}

#[derive(DeriveIden)]
enum OAuthAccounts {
    Table,
    Id,
    UserId,
    Provider,
    ProviderUserId,
    CreatedAt,
    UpdatedAt,
}
