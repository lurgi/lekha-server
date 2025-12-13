# Repositories 계층 규칙

이 디렉토리는 **데이터베이스 접근 전용**입니다.

## Repository의 책임

✅ **해야 할 일:**
- SeaORM을 사용한 CRUD 작업
- 복잡한 쿼리 작성 (조인, 집계 등)
- 트랜잭션 지원 메서드 제공

❌ **하지 말아야 할 일:**
- 비즈니스 로직 작성
- DTO 변환 (Entity만 반환)
- HTTP 관련 처리

---

## 파일 명명 규칙

```
src/repositories/
├── mod.rs                  # 모든 repository re-export
├── user_repository.rs      # User Entity 접근
├── survey_repository.rs    # Survey Entity 접근
└── question_repository.rs  # Question Entity 접근
```

**규칙:** Entity 1개 = Repository 1개

---

## Repository 구조체 정의

```rust
use sea_orm::*;
use crate::entities::user::{self, Entity as User};
use std::sync::Arc;

#[derive(Clone)]
pub struct UserRepository {
    db: Arc<DatabaseConnection>,
}

impl UserRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }
}
```

---

## CRUD 메서드 명명 규칙

### 조회 (Read)

```rust
impl UserRepository {
    // 단일 조회: find_by_{컬럼명}
    pub async fn find_by_id(&self, id: i32) -> Result<Option<user::Model>, DbErr> {
        User::find_by_id(id).one(self.db.as_ref()).await
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<user::Model>, DbErr> {
        User::find()
            .filter(user::Column::Email.eq(email))
            .one(self.db.as_ref())
            .await
    }

    // 여러 개 조회: find_by_{조건}, list_{조건}
    pub async fn find_by_role(&self, role: UserRole) -> Result<Vec<user::Model>, DbErr> {
        User::find()
            .filter(user::Column::Role.eq(role))
            .all(self.db.as_ref())
            .await
    }

    // 전체 조회: list_all
    pub async fn list_all(&self) -> Result<Vec<user::Model>, DbErr> {
        User::find().all(self.db.as_ref()).await
    }

    // 페이지네이션: list_paginated
    pub async fn list_paginated(
        &self,
        page: u32,
        page_size: u32,
    ) -> Result<(Vec<user::Model>, u64), DbErr> {
        let offset = (page - 1) * page_size;

        let users = User::find()
            .order_by_desc(user::Column::CreatedAt)
            .limit(page_size as u64)
            .offset(offset as u64)
            .all(self.db.as_ref())
            .await?;

        let total = User::find().count(self.db.as_ref()).await?;

        Ok((users, total))
    }

    // 존재 여부 확인: exists_by_{조건}
    pub async fn exists_by_email(&self, email: &str) -> Result<bool, DbErr> {
        let count = User::find()
            .filter(user::Column::Email.eq(email))
            .count(self.db.as_ref())
            .await?;
        Ok(count > 0)
    }
}
```

### 생성 (Create)

```rust
use chrono::Utc;

impl UserRepository {
    // 기본 생성: create
    pub async fn create(
        &self,
        username: String,
        email: String,
        password_hash: String,
    ) -> Result<user::Model, DbErr> {
        let now = Utc::now().naive_utc();

        let active_model = user::ActiveModel {
            username: Set(username),
            email: Set(email),
            password_hash: Set(password_hash),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        active_model.insert(self.db.as_ref()).await
    }

    // 트랜잭션용: create_with_txn
    pub async fn create_with_txn(
        &self,
        txn: &DatabaseTransaction,
        username: String,
        email: String,
        password_hash: String,
    ) -> Result<user::Model, DbErr> {
        let now = Utc::now().naive_utc();

        let active_model = user::ActiveModel {
            username: Set(username),
            email: Set(email),
            password_hash: Set(password_hash),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        active_model.insert(txn).await
    }

    // 대량 생성: create_many
    pub async fn create_many(
        &self,
        users: Vec<(String, String, String)>,
    ) -> Result<(), DbErr> {
        let now = Utc::now().naive_utc();

        let active_models: Vec<user::ActiveModel> = users
            .into_iter()
            .map(|(username, email, password_hash)| user::ActiveModel {
                username: Set(username),
                email: Set(email),
                password_hash: Set(password_hash),
                created_at: Set(now),
                updated_at: Set(now),
                ..Default::default()
            })
            .collect();

        User::insert_many(active_models)
            .exec(self.db.as_ref())
            .await?;

        Ok(())
    }
}
```

### 수정 (Update)

```rust
impl UserRepository {
    // 단일 수정: update
    pub async fn update(
        &self,
        id: i32,
        username: Option<String>,
        email: Option<String>,
    ) -> Result<user::Model, DbErr> {
        let user = self.find_by_id(id).await?
            .ok_or(DbErr::RecordNotFound("User not found".into()))?;

        let mut active_model: user::ActiveModel = user.into();

        if let Some(username) = username {
            active_model.username = Set(username);
        }
        if let Some(email) = email {
            active_model.email = Set(email);
        }
        active_model.updated_at = Set(Utc::now().naive_utc());

        active_model.update(self.db.as_ref()).await
    }

    // 특정 필드만 수정: update_{필드명}
    pub async fn update_password(
        &self,
        id: i32,
        new_password_hash: String,
    ) -> Result<user::Model, DbErr> {
        let user = self.find_by_id(id).await?
            .ok_or(DbErr::RecordNotFound("User not found".into()))?;

        let mut active_model: user::ActiveModel = user.into();
        active_model.password_hash = Set(new_password_hash);
        active_model.updated_at = Set(Utc::now().naive_utc());

        active_model.update(self.db.as_ref()).await
    }

    // 대량 수정
    pub async fn update_role_by_ids(
        &self,
        ids: Vec<i32>,
        new_role: UserRole,
    ) -> Result<u64, DbErr> {
        User::update_many()
            .col_expr(user::Column::Role, Expr::value(new_role))
            .col_expr(user::Column::UpdatedAt, Expr::value(Utc::now().naive_utc()))
            .filter(user::Column::Id.is_in(ids))
            .exec(self.db.as_ref())
            .await
            .map(|result| result.rows_affected)
    }
}
```

### 삭제 (Delete)

```rust
impl UserRepository {
    // 단일 삭제: delete
    pub async fn delete(&self, id: i32) -> Result<DeleteResult, DbErr> {
        User::delete_by_id(id).exec(self.db.as_ref()).await
    }

    // 조건부 삭제: delete_by_{조건}
    pub async fn delete_by_email(&self, email: &str) -> Result<DeleteResult, DbErr> {
        User::delete_many()
            .filter(user::Column::Email.eq(email))
            .exec(self.db.as_ref())
            .await
    }

    // 대량 삭제
    pub async fn delete_by_ids(&self, ids: Vec<i32>) -> Result<u64, DbErr> {
        User::delete_many()
            .filter(user::Column::Id.is_in(ids))
            .exec(self.db.as_ref())
            .await
            .map(|result| result.rows_affected)
    }

    // Soft Delete (실제로는 update)
    pub async fn soft_delete(&self, id: i32) -> Result<user::Model, DbErr> {
        let user = self.find_by_id(id).await?
            .ok_or(DbErr::RecordNotFound("User not found".into()))?;

        let mut active_model: user::ActiveModel = user.into();
        active_model.deleted_at = Set(Some(Utc::now().naive_utc()));

        active_model.update(self.db.as_ref()).await
    }
}
```

---

## 관계(Relation) 쿼리

### 1:N 관계

```rust
use crate::entities::{user, survey};

impl SurveyRepository {
    // Survey와 User 함께 조회
    pub async fn find_with_user(&self, id: i32) -> Result<Option<(survey::Model, Option<user::Model>)>, DbErr> {
        Survey::find_by_id(id)
            .find_also_related(User)
            .one(self.db.as_ref())
            .await
    }

    // User의 모든 Survey 조회
    pub async fn find_by_user_id(&self, user_id: i32) -> Result<Vec<survey::Model>, DbErr> {
        Survey::find()
            .filter(survey::Column::UserId.eq(user_id))
            .order_by_desc(survey::Column::CreatedAt)
            .all(self.db.as_ref())
            .await
    }
}
```

### 중첩 관계

```rust
use crate::entities::{survey, question};

impl SurveyRepository {
    // Survey + Questions 함께 조회
    pub async fn find_with_questions(
        &self,
        survey_id: i32,
    ) -> Result<(survey::Model, Vec<question::Model>), DbErr> {
        let survey = Survey::find_by_id(survey_id)
            .one(self.db.as_ref())
            .await?
            .ok_or(DbErr::RecordNotFound("Survey not found".into()))?;

        let questions = survey
            .find_related(Question)
            .order_by_asc(question::Column::Order)
            .all(self.db.as_ref())
            .await?;

        Ok((survey, questions))
    }
}
```

### 집계 쿼리

```rust
impl SurveyRepository {
    // Survey별 응답 수 조회
    pub async fn count_responses(&self, survey_id: i32) -> Result<u64, DbErr> {
        use crate::entities::response::{self, Entity as Response};

        Response::find()
            .filter(response::Column::SurveyId.eq(survey_id))
            .count(self.db.as_ref())
            .await
    }

    // 사용자별 Survey 수
    pub async fn count_by_user(&self, user_id: i32) -> Result<u64, DbErr> {
        Survey::find()
            .filter(survey::Column::UserId.eq(user_id))
            .count(self.db.as_ref())
            .await
    }
}
```

---

## 복잡한 쿼리

### 여러 조건 조합

```rust
use sea_orm::{Condition, QueryOrder, QuerySelect};

impl SurveyRepository {
    pub async fn find_by_filters(
        &self,
        user_id: Option<i32>,
        is_active: Option<bool>,
        keyword: Option<String>,
    ) -> Result<Vec<survey::Model>, DbErr> {
        let mut query = Survey::find();

        // 동적 조건 추가
        if let Some(user_id) = user_id {
            query = query.filter(survey::Column::UserId.eq(user_id));
        }

        if let Some(is_active) = is_active {
            query = query.filter(survey::Column::IsActive.eq(is_active));
        }

        if let Some(keyword) = keyword {
            query = query.filter(
                Condition::any()
                    .add(survey::Column::Title.contains(&keyword))
                    .add(survey::Column::Description.contains(&keyword))
            );
        }

        query
            .order_by_desc(survey::Column::CreatedAt)
            .all(self.db.as_ref())
            .await
    }
}
```

### 커스텀 SELECT

```rust
impl UserRepository {
    // 특정 컬럼만 조회
    pub async fn find_emails(&self) -> Result<Vec<String>, DbErr> {
        let users: Vec<user::Model> = User::find()
            .select_only()
            .column(user::Column::Email)
            .into_model()
            .all(self.db.as_ref())
            .await?;

        Ok(users.into_iter().map(|u| u.email).collect())
    }
}
```

---

## Raw SQL (최후의 수단)

**SeaORM으로 표현 불가능한 복잡한 쿼리만 사용:**

```rust
use sea_orm::FromQueryResult;

#[derive(Debug, FromQueryResult)]
struct SurveyStats {
    survey_id: i32,
    response_count: i64,
    avg_completion_time: f64,
}

impl SurveyRepository {
    pub async fn get_stats(&self, survey_id: i32) -> Result<SurveyStats, DbErr> {
        SurveyStats::find_by_statement(Statement::from_sql_and_values(
            DbBackend::Postgres,
            r#"
            SELECT
                s.id as survey_id,
                COUNT(r.id) as response_count,
                AVG(EXTRACT(EPOCH FROM (r.completed_at - r.created_at))) as avg_completion_time
            FROM surveys s
            LEFT JOIN responses r ON r.survey_id = s.id
            WHERE s.id = $1
            GROUP BY s.id
            "#,
            vec![survey_id.into()],
        ))
        .one(self.db.as_ref())
        .await?
        .ok_or(DbErr::RecordNotFound("Survey not found".into()))
    }
}
```

**주의:** Raw SQL은 타입 안전성을 잃으므로 최소화!

---

## mod.rs 구조

```rust
// src/repositories/mod.rs
mod user_repository;
mod survey_repository;
mod question_repository;
mod answer_repository;

pub use user_repository::UserRepository;
pub use survey_repository::SurveyRepository;
pub use question_repository::QuestionRepository;
pub use answer_repository::AnswerRepository;
```

---

## 테스트

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::Database;

    #[tokio::test]
    async fn test_create_user() {
        let db = Database::connect("sqlite::memory:").await.unwrap();
        // 마이그레이션 실행
        // ...

        let repo = UserRepository::new(Arc::new(db));

        let user = repo.create(
            "testuser".into(),
            "test@example.com".into(),
            "hash".into(),
        ).await.unwrap();

        assert_eq!(user.username, "testuser");
    }
}
```

---

## Repository는 얇은 DB 레이어

- **순수 CRUD**: 비즈니스 로직 없이 데이터 접근만
- **재사용 가능**: Service에서 자유롭게 조합
- **타입 안전**: SeaORM의 쿼리 빌더 최대 활용
- **Entity 반환**: DTO 변환은 Service에서
