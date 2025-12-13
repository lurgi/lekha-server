# Models 계층 규칙

이 디렉토리는 **DTO (Data Transfer Object)와 Domain Model 전용**입니다.

## Models의 책임

✅ **정의할 것:**
- API Request/Response DTO
- Domain Model (비즈니스 개념)
- Enum 타입
- Validation 로직

❌ **정의하지 말 것:**
- Entity (entities/에 정의)
- 비즈니스 로직 (services/에 정의)

---

## 파일 구조

```
src/models/
├── mod.rs           # 모든 모델 re-export
├── dto/
│   ├── mod.rs
│   ├── user.rs      # User 관련 DTO
│   ├── survey.rs    # Survey 관련 DTO
│   └── common.rs    # 공통 DTO (Pagination 등)
└── domain/
    ├── mod.rs
    └── survey.rs    # Survey 도메인 모델 (필요 시)
```

---

## DTO (Data Transfer Object)

### Request DTO

**입력 검증이 핵심:**

```rust
use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateUserRequest {
    #[validate(length(min = 3, max = 20, message = "Username must be 3-20 characters"))]
    pub username: String,

    #[validate(email(message = "Invalid email format"))]
    pub email: String,

    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateUserRequest {
    #[validate(length(min = 3, max = 20))]
    pub username: Option<String>,

    #[validate(url)]
    pub avatar_url: Option<String>,

    #[validate(length(max = 500))]
    pub bio: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}
```

**커스텀 검증:**
```rust
use validator::{Validate, ValidationError};

fn validate_password_strength(password: &str) -> Result<(), ValidationError> {
    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_digit(10));

    if has_uppercase && has_lowercase && has_digit {
        Ok(())
    } else {
        Err(ValidationError::new("password_weak"))
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateUserRequest {
    pub username: String,

    #[validate(custom = "validate_password_strength")]
    pub password: String,
}
```

### Response DTO

**민감 정보 제외가 핵심:**

```rust
use serde::Serialize;
use chrono::NaiveDateTime;

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub avatar_url: Option<String>,
    pub created_at: NaiveDateTime,
    // password_hash는 절대 포함 안 함!
}

#[derive(Debug, Serialize)]
pub struct SurveyResponse {
    pub id: i32,
    pub title: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub question_count: usize,
    pub response_count: usize,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Serialize)]
pub struct SurveyDetailResponse {
    pub id: i32,
    pub title: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub questions: Vec<QuestionResponse>,  // 중첩 DTO
    pub created_at: NaiveDateTime,
}
```

---

## Entity → DTO 변환

### From 트레잇 구현

```rust
use crate::entities::user;

impl From<user::Model> for UserResponse {
    fn from(user: user::Model) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            avatar_url: user.avatar_url,
            created_at: user.created_at,
            // password_hash는 의도적으로 제외
        }
    }
}
```

**여러 Entity 조합:**
```rust
use crate::entities::{survey, question};

impl SurveyDetailResponse {
    pub fn from_entities(survey: survey::Model, questions: Vec<question::Model>) -> Self {
        Self {
            id: survey.id,
            title: survey.title,
            description: survey.description,
            is_active: survey.is_active,
            questions: questions.into_iter().map(QuestionResponse::from).collect(),
            created_at: survey.created_at,
        }
    }
}
```

---

## 공통 DTO

### Pagination

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    #[serde(default = "default_page")]
    pub page: u32,

    #[serde(default = "default_limit")]
    pub limit: u32,
}

fn default_page() -> u32 { 1 }
fn default_limit() -> u32 { 20 }

impl PaginationQuery {
    pub fn offset(&self) -> u32 {
        (self.page - 1) * self.limit
    }
}

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub pagination: PaginationMeta,
}

#[derive(Debug, Serialize)]
pub struct PaginationMeta {
    pub current_page: u32,
    pub per_page: u32,
    pub total: u64,
    pub total_pages: u32,
}

impl PaginationMeta {
    pub fn new(current_page: u32, per_page: u32, total: u64) -> Self {
        let total_pages = ((total as f64) / (per_page as f64)).ceil() as u32;
        Self {
            current_page,
            per_page,
            total,
            total_pages,
        }
    }
}
```

**사용 예시:**
```rust
// Handler에서
pub async fn list_users(
    State(service): State<UserService>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<PaginatedResponse<UserResponse>>, AppError> {
    let (users, total) = service.list_users_paginated(pagination.page, pagination.limit).await?;

    Ok(Json(PaginatedResponse {
        data: users,
        pagination: PaginationMeta::new(pagination.page, pagination.limit, total),
    }))
}
```

### 에러 응답

```rust
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl ErrorResponse {
    pub fn new(error: &str, message: &str) -> Self {
        Self {
            error: error.to_string(),
            message: message.to_string(),
            details: None,
        }
    }

    pub fn with_details(error: &str, message: &str, details: serde_json::Value) -> Self {
        Self {
            error: error.to_string(),
            message: message.to_string(),
            details: Some(details),
        }
    }
}
```

---

## Domain Model (필요 시)

**Entity와 분리된 비즈니스 개념:**

```rust
// Entity는 DB 스키마 그대로
// Domain Model은 비즈니스 로직 포함

pub struct Survey {
    pub id: i32,
    pub title: String,
    pub questions: Vec<Question>,
    pub status: SurveyStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SurveyStatus {
    Draft,
    Active,
    Closed,
}

impl Survey {
    pub fn can_accept_responses(&self) -> bool {
        self.status == SurveyStatus::Active
    }

    pub fn is_complete(&self) -> bool {
        self.questions.iter().all(|q| q.is_valid())
    }
}

pub struct Question {
    pub id: i32,
    pub text: String,
    pub question_type: QuestionType,
    pub is_required: bool,
}

impl Question {
    pub fn is_valid(&self) -> bool {
        !self.text.is_empty() && self.text.len() <= 500
    }
}
```

**언제 Domain Model을 사용하나?**
- Entity가 빈약한 데이터 홀더일 때
- 복잡한 비즈니스 규칙이 있을 때
- 여러 Entity를 조합한 개념이 필요할 때

**대부분의 경우 Entity + Service로 충분!**

---

## Enum 타입

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum QuestionType {
    ShortText,
    LongText,
    SingleChoice,
    MultipleChoice,
    Rating,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SortOrder {
    Asc,
    Desc,
}
```

---

## 중첩 DTO

```rust
#[derive(Debug, Deserialize, Validate)]
pub struct CreateSurveyRequest {
    #[validate(length(min = 3, max = 200))]
    pub title: String,

    #[validate(length(max = 1000))]
    pub description: Option<String>,

    #[validate(length(min = 1, message = "At least one question required"))]
    #[validate]
    pub questions: Vec<CreateQuestionRequest>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateQuestionRequest {
    #[validate(length(min = 3, max = 500))]
    pub text: String,

    pub question_type: QuestionType,

    pub is_required: bool,

    #[serde(default)]
    pub options: Vec<String>,  // SingleChoice, MultipleChoice일 때 사용
}
```

---

## Builder 패턴 (복잡한 DTO)

```rust
pub struct SurveyResponseBuilder {
    id: i32,
    title: String,
    description: Option<String>,
    is_active: bool,
    questions: Vec<QuestionResponse>,
    response_count: Option<usize>,
    created_at: NaiveDateTime,
}

impl SurveyResponseBuilder {
    pub fn new(survey: survey::Model) -> Self {
        Self {
            id: survey.id,
            title: survey.title,
            description: survey.description,
            is_active: survey.is_active,
            questions: Vec::new(),
            response_count: None,
            created_at: survey.created_at,
        }
    }

    pub fn with_questions(mut self, questions: Vec<question::Model>) -> Self {
        self.questions = questions.into_iter().map(QuestionResponse::from).collect();
        self
    }

    pub fn with_response_count(mut self, count: usize) -> Self {
        self.response_count = Some(count);
        self
    }

    pub fn build(self) -> SurveyDetailResponse {
        SurveyDetailResponse {
            id: self.id,
            title: self.title,
            description: self.description,
            is_active: self.is_active,
            questions: self.questions,
            response_count: self.response_count.unwrap_or(0),
            created_at: self.created_at,
        }
    }
}

// 사용
let response = SurveyResponseBuilder::new(survey)
    .with_questions(questions)
    .with_response_count(10)
    .build();
```

---

## mod.rs 구조

```rust
// src/models/mod.rs
pub mod dto;
pub mod domain;

// 자주 사용하는 타입 re-export
pub use dto::{
    user::*,
    survey::*,
    common::{PaginatedResponse, PaginationQuery, ErrorResponse},
};
```

```rust
// src/models/dto/mod.rs
pub mod user;
pub mod survey;
pub mod common;
```

---

## DTO 작성 체크리스트

### Request DTO
- [ ] `#[derive(Deserialize, Validate)]` 추가
- [ ] 모든 필드에 적절한 검증 추가
- [ ] 민감 정보는 별도 DTO로 분리

### Response DTO
- [ ] `#[derive(Serialize)]` 추가
- [ ] 민감 정보 (password_hash 등) 제외
- [ ] Entity → DTO 변환 `From` 트레잇 구현
- [ ] 날짜/시간은 `NaiveDateTime` 또는 ISO 8601 문자열

### 공통
- [ ] 명확한 네이밍 (`CreateUserRequest`, `UserResponse`)
- [ ] 문서화 주석 (`///`) 추가
- [ ] 필요 시 `Default` 구현
