# Entities 계층 규칙

이 디렉토리는 **SeaORM Entity 정의 전용**입니다.

## Entity 작성 규칙

### 1. 파일 구조
- 파일명: `{테이블명_단수형}.rs` (예: `user.rs`, `survey.rs`, `question.rs`)
- 각 파일은 하나의 Entity만 정의
- `mod.rs`에서 모든 Entity를 re-export

### 2. 필수 구성 요소

모든 Entity 파일은 다음 순서로 구성:

```rust
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

// 1. Model 정의
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "테이블명")]
pub struct Model {
    // 필드 정의
}

// 2. Relation 정의
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    // 관계 정의
}

// 3. ActiveModelBehavior 구현
impl ActiveModelBehavior for ActiveModel {}
```

### 3. 필드 정의 규칙

**타임스탬프 필드는 필수:**
```rust
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,

    // ... 다른 필드들 ...

    pub created_at: DateTime,  // 필수
    pub updated_at: DateTime,  // 필수
}
```

**인덱스가 필요한 필드는 명시:**
```rust
#[sea_orm(unique)]
pub email: String,

#[sea_orm(indexed)]
pub user_id: i32,
```

**NULL 허용 필드는 Option 사용:**
```rust
pub bio: Option<String>,
pub avatar_url: Option<String>,
```

### 4. Relation 정의 규칙

**One-to-Many (1:N):**
```rust
// users 테이블 (부모)
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::survey::Entity")]
    Surveys,
}

impl Related<super::survey::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Surveys.def()
    }
}

// surveys 테이블 (자식)
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::Id"
    )]
    User,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}
```

**Many-to-Many (N:M):**
```rust
// 중간 테이블을 통한 관계 정의
impl Related<super::tag::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Tags.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::survey_tag::Relation::Survey.def().rev())
    }
}
```

### 5. Enum 필드 규칙

**DB Enum은 별도 파일로 분리하지 않고 Entity 파일 내 정의:**
```rust
use sea_orm::entity::prelude::*;

#[derive(Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "String(Some(20))")]
pub enum UserRole {
    #[sea_orm(string_value = "admin")]
    Admin,
    #[sea_orm(string_value = "user")]
    User,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    pub role: UserRole,  // Enum 사용
}
```

### 6. 보안 규칙

**민감 정보는 Serialize에서 제외:**
```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    pub id: i32,
    pub email: String,

    #[serde(skip_serializing)]  // JSON 변환 시 제외
    pub password_hash: String,

    #[serde(skip_serializing)]
    pub api_secret: Option<String>,
}
```

**또는 Serialize를 아예 derive하지 않음:**
```rust
// Entity는 직접 API 응답으로 사용 금지!
// DTO 변환 강제
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]  // Serialize 제거
#[sea_orm(table_name = "users")]
pub struct Model {
    pub password_hash: String,  // 실수로도 노출 불가
}
```

### 7. mod.rs 규칙

**모든 Entity를 명시적으로 나열:**
```rust
// src/entities/mod.rs
pub mod user;
pub mod survey;
pub mod question;
pub mod answer;

pub use user::Entity as User;
pub use survey::Entity as Survey;
pub use question::Entity as Question;
pub use answer::Entity as Answer;
```

---

## 금지 사항

### ❌ Entity에 비즈니스 로직 작성 금지
```rust
// 나쁜 예
impl Model {
    pub fn is_admin(&self) -> bool {
        self.role == UserRole::Admin  // ❌ 비즈니스 로직은 Service에
    }
}
```

### ❌ Entity에서 DB 접근 금지
```rust
// 나쁜 예
impl Model {
    pub async fn save(&self, db: &DatabaseConnection) -> Result<()> {
        // ❌ DB 작업은 Repository에
    }
}
```

### ❌ 외래 키 제약조건 정의 금지
```rust
// 나쁜 예 - Entity에서 제약조건 정의하지 말 것
// 제약조건은 마이그레이션 파일에서만 정의
```

---

## Entity는 순수한 데이터 모델

Entity는 **DB 스키마의 Rust 표현**일 뿐입니다.
- ✅ 필드 정의만
- ✅ 관계 정의만
- ❌ 로직 없음
- ❌ DB 접근 없음

**모든 로직은 Service/Repository 계층에서!**
