# 아키텍처 규칙

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

---

## 3계층 아키텍처 (Handlers → Services → Repositories)

### 계층별 역할
- **Handlers**: HTTP 요청 수신, DTO 검증, 응답 반환
- **Services**: 비즈니스 로직, 트랜잭션 관리
- **Repositories**: 데이터베이스 CRUD 작업

### 규칙
- Handler는 Service를 호출만 한다 (직접 DB 접근 금지)
- Service는 여러 Repository를 조합할 수 있다
- Repository는 순수 DB 작업만 수행 (비즈니스 로직 금지)

### 계층별 상세 규칙
- **[Handler 계층](./handlers.md)**: HTTP 처리, DTO 검증, State 사용
- **[Service 계층](./services.md)**: 비즈니스 로직, 트랜잭션, 에러 변환
- **[Repository 계층](./repositories.md)**: CRUD 작업, 관계 처리, 네이밍 규칙

---

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

---

## 에러 처리

### 기본 원칙
- `unwrap()`, `expect()` 사용 금지 (프로덕션 코드)
- `Result` 타입과 `?` 연산자 사용
- `anyhow::Result` 활용

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

## 모델/타입 규칙

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

## 환경 변수
- `.env` 파일 사용 (git 무시)
- `.env.example` 템플릿 유지
- 민감 정보 절대 하드코딩 금지
