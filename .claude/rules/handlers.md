---
paths: src/handlers/**/*.rs
---

# Handlers 계층 규칙

Handler 계층은 HTTP 요청을 수신하고, DTO를 검증하며, Service를 호출하여 응답을 반환합니다.

## 계층 역할
- HTTP 요청 수신 및 파싱
- DTO 검증 (validator crate)
- Service 호출
- HTTP 응답 반환
- **직접 DB 접근 금지**

---

## Handler 함수 시그니처

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

---

## State 사용 규칙

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

---

## DTO 검증

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

## 에러 처리

Handler 계층에서는 `AppError`를 사용하여 HTTP 응답으로 자동 변환합니다.

**좋은 예시**:
```rust
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
