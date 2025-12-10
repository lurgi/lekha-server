# SeaORM Migrations

SeaORM에서는 **Rust 코드로 마이그레이션을 작성**합니다. SQL 몰라도 됩니다!

## 마이그레이션 실행

```bash
# 프로젝트 루트에서
cd migration

# 마이그레이션 실행 (up)
DATABASE_URL="postgres://inklings_user:inklings_dev_password@localhost:5432/inklings_db" cargo run

# 또는 .env 사용
cargo run
```

## 마이그레이션 되돌리기

```bash
DATABASE_URL="postgres://inklings_user:inklings_dev_password@localhost:5432/inklings_db" cargo run -- down
```

## 새 마이그레이션 추가

1. `migration/src/` 에 새 파일 생성:
   ```
   m20241210_000002_create_posts_table.rs
   ```

2. `migration/src/lib.rs`의 `migrations()` 함수에 추가:
   ```rust
   vec![
       Box::new(m20241210_000001_create_users_table::Migration),
       Box::new(m20241210_000002_create_posts_table::Migration),
   ]
   ```

## 예시: Posts 테이블 추가

`migration/src/m20241210_000002_create_posts_table.rs`:

```rust
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Posts::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Posts::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Posts::Title).string().not_null())
                    .col(ColumnDef::new(Posts::Content).text().not_null())
                    .col(ColumnDef::new(Posts::UserId).integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-posts-user_id")
                            .from(Posts::Table, Posts::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Posts::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Posts {
    Table,
    Id,
    Title,
    Content,
    UserId,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}
```

## SQL을 몰라도 됩니다!

모든 것이 Rust 코드로 작성됩니다:
- `Table::create()` - CREATE TABLE
- `ColumnDef::new()` - 컬럼 정의
- `ForeignKey::create()` - 외래키
- `Index::create()` - 인덱스

타입 안전성까지 보장됩니다! ✅
