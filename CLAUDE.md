# Inklings Server - 프로젝트 규칙

## 코딩 스타일

### 주석 규칙
- **불필요한 주석은 작성하지 않는다**
- 코드 자체로 의미가 명확하면 주석 불필요
- 주석이 필요한 경우:
  - 복잡한 비즈니스 로직
  - 왜 이렇게 구현했는지 (Why)
  - 외부 API나 복잡한 알고리즘
- 주석이 불필요한 경우:
  - 코드가 하는 일을 그대로 반복 (What)
  - 함수명/변수명으로 충분히 설명 가능한 경우

**나쁜 예:**
```rust
// 사용자 ID를 가져옴
let user_id = get_user_id();

// 사용자를 생성함
let user = create_user();
```

**좋은 예:**
```rust
let user_id = get_user_id();
let user = create_user();

// 외부 결제 API는 3번까지 재시도 필요 (API 문서 참고)
let payment_result = retry_payment_api(3).await?;
```

### Rust 코딩 표준
- 함수명: `snake_case`
- 상수명: `SCREAMING_SNAKE_CASE`
- 타입명: `PascalCase`
- `unwrap()` 사용 금지 - `?` 또는 `Result` 타입 사용
- 모든 public 함수/struct는 `///` 문서화 주석 작성

## 프로젝트 구조

### 기술 스택
- **언어:** Rust
- **데이터베이스:** PostgreSQL
- **ORM:** SeaORM (SQL 작성 불필요)
- **Async Runtime:** Tokio

### 디렉토리 구조
```
inklings-server/
├── src/
│   ├── main.rs              # 엔트리 포인트
│   ├── handlers/            # HTTP 요청/응답 처리 (Axum)
│   ├── services/            # 비즈니스 로직
│   ├── repositories/        # 데이터베이스 접근 계층
│   ├── models/              # DTO, Domain Model
│   ├── errors/              # 커스텀 에러 타입
│   ├── db/                  # 데이터베이스 연결
│   └── entities/            # SeaORM Entity 모델
└── migration/               # 데이터베이스 마이그레이션 (Rust 코드)
```

### 3계층 아키텍처 (Handlers → Services → Repositories)

**계층별 역할:**
- **Handlers**: HTTP 요청 수신, DTO 검증, 응답 반환
- **Services**: 비즈니스 로직, 트랜잭션 관리
- **Repositories**: 데이터베이스 CRUD 작업

**규칙:**
- Handler는 Service를 호출만 한다 (직접 DB 접근 금지)
- Service는 여러 Repository를 조합할 수 있다
- Repository는 순수 DB 작업만 수행 (비즈니스 로직 금지)

## SeaORM 사용 규칙

### Entity 정의
- `src/entities/` 에 모델 정의
- SQL 작성 불필요, Rust 코드로 정의
- `#[derive(DeriveEntityModel)]` 사용

### 마이그레이션
- `migration/src/` 에 Rust 코드로 작성
- SQL 파일 작성 금지
- 새 마이그레이션은 `m<timestamp>_<name>.rs` 형식

### CRUD 작업
- SeaORM의 ActiveModel 패턴 사용
- Raw SQL 쿼리 지양
- 타입 안전성 최대한 활용

## 에러 처리
- `unwrap()`, `expect()` 사용 금지 (프로덕션 코드)
- `Result` 타입과 `?` 연산자 사용
- `anyhow::Result` 활용

## 환경 변수
- `.env` 파일 사용 (git 무시)
- `.env.example` 템플릿 유지
- 민감 정보 절대 하드코딩 금지

## 개발 워크플로우
1. 기능 구현 전 Entity 정의
2. 마이그레이션 작성 및 실행
3. 비즈니스 로직 구현
4. 테스트 작성

---

# 상세 코딩 규칙 (Rust/Axum/SeaORM)

## 1. Handlers 계층 규칙

### Handler 함수 시그니처
**설명**: Axum handler는 추출자(extractor)를 통해 요청을 받고, `Result<impl IntoResponse>`를 반환한다.

**좋은 예시**:
```rust
use axum::{Json, extract::State};
use crate::models::dto::{CreateUserRequest, UserResponse};
use crate::errors::AppError;

pub async fn create_user(
    State(service): State<UserService>,
    Json(req): Json<CreateUserRequest>,
) -> Result<Json<UserResponse>, AppError> {
    let user = service.create_user(req).await?;
    Ok(Json(user))
}
```

**나쁜 예시**:
```rust
pub async fn create_user(req: CreateUserRequest) -> UserResponse {
    // Axum 추출자 미사용
    // 에러 처리 없음
}
```

**이유**: Axum의 타입 안전한 추출자를 사용하면 런타임 에러를 컴파일 타임에 잡을 수 있다.

### State 사용 규칙
**설명**: AppState는 Arc로 감싸진 공유 상태를 포함하며, Service 인스턴스를 저장한다.

**좋은 예시**:
```rust
use axum::Router;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub user_service: UserService,
    pub survey_service: SurveyService,
}

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/users", post(handlers::create_user))
        .with_state(state)
}
```

**나쁜 예시**:
```rust
pub struct AppState {
    pub db: DatabaseConnection, // Service를 건너뛰고 DB 직접 노출
}
```

**이유**: Handler가 DB를 직접 접근하면 비즈니스 로직이 분산되고 테스트가 어려워진다.

### DTO 검증
**설명**: 입력 DTO는 `validator` crate로 검증하거나, 타입 시스템으로 불가능한 상태를 방지한다.

**좋은 예시**:
```rust
use serde::Deserialize;
use validator::Validate;

#[derive(Deserialize, Validate)]
pub struct CreateUserRequest {
    #[validate(length(min = 3, max = 20))]
    pub username: String,

    #[validate(email)]
    pub email: String,

    #[validate(length(min = 8))]
    pub password: String,
}

// Handler에서
pub async fn create_user(
    State(service): State<UserService>,
    Json(req): Json<CreateUserRequest>,
) -> Result<Json<UserResponse>, AppError> {
    req.validate()?; // 검증 실패 시 자동으로 400 에러
    let user = service.create_user(req).await?;
    Ok(Json(user))
}
```

**나쁜 예시**:
```rust
pub struct CreateUserRequest {
    pub username: String, // 검증 없음
    pub email: String,
}
```

**이유**: 입력 검증을 Handler에서 하면 Service 계층은 항상 유효한 데이터를 받는다고 가정할 수 있다.

---

## 2. Services 계층 규칙

### Service 구조체 정의
**설명**: Service는 필요한 Repository들을 필드로 가지며, 비즈니스 로직을 구현한다.

**좋은 예시**:
```rust
use crate::repositories::UserRepository;
use sea_orm::DatabaseConnection;
use std::sync::Arc;

#[derive(Clone)]
pub struct UserService {
    user_repo: UserRepository,
}

impl UserService {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self {
            user_repo: UserRepository::new(db),
        }
    }

    pub async fn create_user(&self, req: CreateUserRequest) -> Result<UserResponse, ServiceError> {
        // 비즈니스 로직: 중복 검사
        if self.user_repo.find_by_email(&req.email).await?.is_some() {
            return Err(ServiceError::EmailAlreadyExists);
        }

        // 비즈니스 로직: 비밀번호 해싱
        let password_hash = hash_password(&req.password)?;

        let user = self.user_repo.create(req.username, req.email, password_hash).await?;
        Ok(UserResponse::from(user))
    }
}
```

**나쁜 예시**:
```rust
impl UserService {
    pub async fn create_user(&self, req: CreateUserRequest) -> Result<UserResponse> {
        // Repository를 거치지 않고 직접 DB 접근
        let user = Entity::insert(active_model).exec(&self.db).await?;
        Ok(user.into())
    }
}
```

**이유**: Repository를 통하면 데이터 접근 로직을 재사용할 수 있고, 테스트 시 Mock이 가능하다.

### 트랜잭션 처리
**설명**: 여러 DB 작업이 원자적으로 실행되어야 할 때는 트랜잭션을 사용한다.

**좋은 예시**:
```rust
use sea_orm::TransactionTrait;

impl SurveyService {
    pub async fn create_survey_with_questions(
        &self,
        req: CreateSurveyRequest,
    ) -> Result<SurveyResponse, ServiceError> {
        let txn = self.db.begin().await?;

        // 설문 생성
        let survey = self.survey_repo.create_with_txn(&txn, req.title).await?;

        // 질문들 생성
        for q in req.questions {
            self.question_repo.create_with_txn(&txn, survey.id, q).await?;
        }

        txn.commit().await?;
        Ok(SurveyResponse::from(survey))
    }
}
```

**나쁜 예시**:
```rust
// 트랜잭션 없이 여러 작업 수행
let survey = self.survey_repo.create(req.title).await?;
for q in req.questions {
    self.question_repo.create(survey.id, q).await?; // 중간에 실패하면 부분 삽입
}
```

**이유**: 트랜잭션 없이는 중간에 실패할 경우 데이터 불일치가 발생한다.

### 에러 변환
**설명**: Repository 에러를 Service 에러로, Service 에러를 AppError로 변환한다.

**좋은 예시**:
```rust
#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("Email already exists")]
    EmailAlreadyExists,

    #[error("User not found")]
    UserNotFound,

    #[error("Database error: {0}")]
    Database(#[from] DbErr),
}

impl IntoResponse for ServiceError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::EmailAlreadyExists => (StatusCode::CONFLICT, self.to_string()),
            Self::UserNotFound => (StatusCode::NOT_FOUND, self.to_string()),
            Self::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal error".to_string()),
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}
```

**나쁜 예시**:
```rust
// 모든 에러를 String으로 처리
pub async fn create_user(&self, req: CreateUserRequest) -> Result<UserResponse, String> {
    // 에러 종류를 구분할 수 없음
}
```

**이유**: 타입화된 에러는 각 에러에 맞는 HTTP 상태 코드와 메시지를 반환할 수 있다.

---

## 3. Repositories 계층 규칙

### Repository 메서드 명명 규칙
**설명**: CRUD 작업은 일관된 네이밍을 따른다.

- 조회: `find_*`, `find_by_*`, `list_*`
- 생성: `create`, `insert`
- 수정: `update`, `update_*`
- 삭제: `delete`, `delete_by_*`

**좋은 예시**:
```rust
use sea_orm::*;
use crate::entities::user::{self, Entity as User};

#[derive(Clone)]
pub struct UserRepository {
    db: Arc<DatabaseConnection>,
}

impl UserRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    pub async fn find_by_id(&self, id: i32) -> Result<Option<user::Model>, DbErr> {
        User::find_by_id(id).one(self.db.as_ref()).await
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<user::Model>, DbErr> {
        User::find()
            .filter(user::Column::Email.eq(email))
            .one(self.db.as_ref())
            .await
    }

    pub async fn list_all(&self) -> Result<Vec<user::Model>, DbErr> {
        User::find().all(self.db.as_ref()).await
    }

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

    pub async fn update(&self, id: i32, username: Option<String>) -> Result<user::Model, DbErr> {
        let user = self.find_by_id(id).await?
            .ok_or(DbErr::RecordNotFound("User not found".into()))?;

        let mut active_model: user::ActiveModel = user.into();

        if let Some(username) = username {
            active_model.username = Set(username);
        }
        active_model.updated_at = Set(Utc::now().naive_utc());

        active_model.update(self.db.as_ref()).await
    }

    pub async fn delete(&self, id: i32) -> Result<DeleteResult, DbErr> {
        User::delete_by_id(id).exec(self.db.as_ref()).await
    }
}
```

**나쁜 예시**:
```rust
impl UserRepository {
    pub async fn get_user(&self, id: i32) -> ... { } // find_by_id로 통일
    pub async fn new_user(&self, ...) -> ... { } // create로 통일
    pub async fn remove(&self, id: i32) -> ... { } // delete로 통일
}
```

**이유**: 일관된 네이밍은 코드 가독성을 높이고, 어떤 작업을 하는지 명확히 알 수 있다.

### 관계(Relation) 처리
**설명**: SeaORM의 `find_also_related`, `find_with_related`를 사용해 관계 데이터를 로드한다.

**좋은 예시**:
```rust
use crate::entities::{survey, question};

impl SurveyRepository {
    pub async fn find_with_questions(&self, survey_id: i32) -> Result<(survey::Model, Vec<question::Model>), DbErr> {
        let survey = Survey::find_by_id(survey_id)
            .one(self.db.as_ref())
            .await?
            .ok_or(DbErr::RecordNotFound("Survey not found".into()))?;

        let questions = survey
            .find_related(Question)
            .all(self.db.as_ref())
            .await?;

        Ok((survey, questions))
    }
}
```

**나쁜 예시**:
```rust
// N+1 쿼리 문제
pub async fn find_with_questions(&self, survey_id: i32) -> Result<...> {
    let survey = self.find_by_id(survey_id).await?;
    let questions = Question::find()
        .filter(question::Column::SurveyId.eq(survey_id))
        .all(self.db.as_ref())
        .await?; // 별도 쿼리
    Ok((survey, questions))
}
```

**이유**: SeaORM의 Relation을 사용하면 JOIN을 활용해 효율적인 쿼리를 생성한다.

---

## 4. 모델/타입 규칙

### Entity vs DTO 분리
**설명**: Entity는 DB 스키마를 나타내고, DTO는 API 입출력을 나타낸다.

**좋은 예시**:
```rust
// src/entities/user.rs - Entity (DB 스키마)
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub username: String,
    pub email: String,
    pub password_hash: String, // 절대 외부 노출 금지
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

// src/models/dto.rs - DTO (API 입출력)
#[derive(Serialize)]
pub struct UserResponse {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub created_at: DateTime,
    // password_hash는 노출 안 함
}

impl From<user::Model> for UserResponse {
    fn from(user: user::Model) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            created_at: user.created_at,
        }
    }
}
```

**나쁜 예시**:
```rust
// Entity를 직접 반환 (password_hash 노출 위험)
pub async fn get_user(id: i32) -> Json<user::Model> {
    // ...
}
```

**이유**: Entity를 직접 노출하면 민감 정보가 유출되고, API 스펙이 DB 스키마에 종속된다.

### Enum 사용 규칙
**설명**: DB Enum은 SeaORM의 `DeriveActiveEnum`을 사용한다.

**좋은 예시**:
```rust
use sea_orm::entity::prelude::*;

#[derive(Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "String(Some(20))")]
pub enum UserRole {
    #[sea_orm(string_value = "admin")]
    Admin,
    #[sea_orm(string_value = "user")]
    User,
    #[sea_orm(string_value = "guest")]
    Guest,
}
```

**나쁜 예시**:
```rust
// String으로 처리 (타입 안전성 없음)
pub struct Model {
    pub role: String, // "admin", "user", "guest" - 오타 위험
}
```

**이유**: Enum을 사용하면 컴파일 타임에 잘못된 값을 방지할 수 있다.

---

## 5. 고급 에러 처리 규칙

### 커스텀 에러 타입 계층
**설명**: `thiserror`로 각 계층의 에러를 정의하고, `From` 트레잇으로 변환한다.

**좋은 예시**:
```rust
use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use thiserror::Error;

// Repository 계층 에러 (이미 DbErr 사용)

// Service 계층 에러
#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Database error")]
    Database(#[from] DbErr),
}

// Handler 계층 에러 (최종 HTTP 응답)
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Service error")]
    Service(#[from] ServiceError),

    #[error("Validation error")]
    Validation(#[from] validator::ValidationErrors),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::Service(ServiceError::NotFound(msg)) => {
                (StatusCode::NOT_FOUND, msg)
            }
            AppError::Service(ServiceError::InvalidInput(msg)) => {
                (StatusCode::BAD_REQUEST, msg)
            }
            AppError::Validation(_) => {
                (StatusCode::BAD_REQUEST, "Validation failed".to_string())
            }
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "Internal error".to_string()),
        };

        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}
```

**나쁜 예시**:
```rust
// 모든 에러를 anyhow::Error로 처리
pub async fn create_user(...) -> Result<UserResponse, anyhow::Error> {
    // HTTP 상태 코드를 구분할 수 없음
}
```

**이유**: 타입화된 에러 계층은 각 상황에 맞는 HTTP 응답을 자동으로 생성할 수 있다.

---

## 6. 테스트 규칙

### 단위 테스트 위치
**설명**: 각 모듈의 하단에 `#[cfg(test)]` 모듈로 작성한다.

**좋은 예시**:
```rust
// src/services/user_service.rs
impl UserService {
    pub async fn create_user(&self, req: CreateUserRequest) -> Result<UserResponse, ServiceError> {
        // ...
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::mock;

    mock! {
        UserRepo {
            async fn find_by_email(&self, email: &str) -> Result<Option<user::Model>, DbErr>;
            async fn create(&self, username: String, email: String, password_hash: String) -> Result<user::Model, DbErr>;
        }
    }

    #[tokio::test]
    async fn test_create_user_success() {
        let mut mock_repo = MockUserRepo::new();
        mock_repo
            .expect_find_by_email()
            .returning(|_| Ok(None)); // 중복 없음

        mock_repo
            .expect_create()
            .returning(|username, email, password_hash| {
                Ok(user::Model {
                    id: 1,
                    username,
                    email,
                    password_hash,
                    created_at: Utc::now().naive_utc(),
                    updated_at: Utc::now().naive_utc(),
                })
            });

        // 테스트 로직
    }
}
```

**나쁜 예시**:
```rust
// 별도의 tests/ 디렉토리에 모든 테스트 작성 (단위 테스트도)
// 모듈과 멀어져 유지보수 어려움
```

**이유**: 단위 테스트는 코드와 가까이 있어야 수정 시 함께 업데이트하기 쉽다.

### 통합 테스트 구조
**설명**: `tests/` 디렉토리에 E2E 테스트 작성.

**좋은 예시**:
```rust
// tests/user_integration_test.rs
use inklings_server::*;
use sea_orm::Database;

#[tokio::test]
async fn test_create_user_e2e() {
    let db = Database::connect("sqlite::memory:").await.unwrap();
    // 마이그레이션 실행
    // API 호출
    // DB 검증
}
```

---

## 7. Async/Await 사용 규칙

### 비동기 함수 정의
**설명**: DB나 외부 API 호출은 모두 `async fn`으로 작성한다.

**좋은 예시**:
```rust
impl UserService {
    pub async fn create_user(&self, req: CreateUserRequest) -> Result<UserResponse, ServiceError> {
        let existing = self.user_repo.find_by_email(&req.email).await?;
        // ...
    }
}
```

**나쁜 예시**:
```rust
// 동기 함수에서 비동기 호출
pub fn create_user(&self, req: CreateUserRequest) -> Result<UserResponse, ServiceError> {
    let existing = self.user_repo.find_by_email(&req.email); // await 불가능
}
```

**이유**: Tokio 런타임에서는 모든 I/O 작업이 비동기여야 한다.

### `?` 연산자 활용
**설명**: `Result`를 반환하는 비동기 함수에서는 `?`로 에러 전파.

**좋은 예시**:
```rust
pub async fn get_user_surveys(&self, user_id: i32) -> Result<Vec<SurveyResponse>, ServiceError> {
    let user = self.user_repo.find_by_id(user_id).await?;
    let surveys = self.survey_repo.find_by_user_id(user_id).await?;
    Ok(surveys.into_iter().map(Into::into).collect())
}
```

**나쁜 예시**:
```rust
pub async fn get_user_surveys(&self, user_id: i32) -> Result<Vec<SurveyResponse>, ServiceError> {
    match self.user_repo.find_by_id(user_id).await {
        Ok(user) => {
            match self.survey_repo.find_by_user_id(user_id).await {
                Ok(surveys) => Ok(surveys.into_iter().map(Into::into).collect()),
                Err(e) => Err(e.into()),
            }
        }
        Err(e) => Err(e.into()),
    }
}
```

**이유**: `?` 연산자는 코드를 간결하게 만들고 가독성을 높인다.
