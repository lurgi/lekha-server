# Errors 계층 규칙

이 디렉토리는 **커스텀 에러 타입 정의 전용**입니다.

## 에러 계층 구조

```
AppError (Handler 계층)
  ↑
ServiceError (Service 계층)
  ↑
DbErr (Repository 계층 - SeaORM 기본 제공)
```

**각 계층은 자신의 에러 타입을 가지며, 상위 계층으로 변환됩니다.**

---

## 파일 구조

```
src/errors/
├── mod.rs           # 모든 에러 타입 re-export
├── app_error.rs     # Handler 계층 에러 (HTTP 응답)
└── service_error.rs # Service 계층 에러 (비즈니스 로직)
```

---

## Service 계층 에러

**비즈니스 규칙 위반을 표현:**

```rust
// src/errors/service_error.rs
use thiserror::Error;
use sea_orm::DbErr;

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Forbidden")]
    Forbidden,

    #[error("Email already exists")]
    EmailAlreadyExists,

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Token expired")]
    TokenExpired,

    #[error("Database error")]
    Database(#[from] DbErr),

    #[error("External API error: {0}")]
    ExternalApi(String),

    #[error("Internal error: {0}")]
    Internal(String),
}
```

**사용 예시:**
```rust
impl UserService {
    pub async fn create_user(&self, req: CreateUserRequest) -> Result<UserResponse, ServiceError> {
        // 중복 검사
        if self.user_repo.find_by_email(&req.email).await?.is_some() {
            return Err(ServiceError::EmailAlreadyExists);
        }

        // ...
    }

    pub async fn get_user(&self, user_id: i32) -> Result<UserResponse, ServiceError> {
        let user = self.user_repo
            .find_by_id(user_id)
            .await?
            .ok_or(ServiceError::NotFound(format!("User {} not found", user_id)))?;

        Ok(UserResponse::from(user))
    }
}
```

---

## Handler 계층 에러 (AppError)

**HTTP 응답으로 변환:**

```rust
// src/errors/app_error.rs
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

use super::ServiceError;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Service error")]
    Service(#[from] ServiceError),

    #[error("Validation error")]
    Validation(#[from] validator::ValidationErrors),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Not found")]
    NotFound,

    #[error("Internal server error")]
    InternalServerError,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            // Service 에러 변환
            AppError::Service(ServiceError::NotFound(msg)) => {
                (StatusCode::NOT_FOUND, msg)
            }
            AppError::Service(ServiceError::InvalidInput(msg)) => {
                (StatusCode::BAD_REQUEST, msg)
            }
            AppError::Service(ServiceError::Conflict(msg)) => {
                (StatusCode::CONFLICT, msg)
            }
            AppError::Service(ServiceError::EmailAlreadyExists) => {
                (StatusCode::CONFLICT, "Email already exists".to_string())
            }
            AppError::Service(ServiceError::Unauthorized) => {
                (StatusCode::UNAUTHORIZED, "Unauthorized".to_string())
            }
            AppError::Service(ServiceError::Forbidden) => {
                (StatusCode::FORBIDDEN, "Forbidden".to_string())
            }
            AppError::Service(ServiceError::InvalidCredentials) => {
                (StatusCode::UNAUTHORIZED, "Invalid credentials".to_string())
            }
            AppError::Service(ServiceError::TokenExpired) => {
                (StatusCode::UNAUTHORIZED, "Token expired".to_string())
            }
            AppError::Service(ServiceError::Database(_)) => {
                tracing::error!("Database error: {:?}", self);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            }
            AppError::Service(ServiceError::ExternalApi(msg)) => {
                tracing::error!("External API error: {}", msg);
                (StatusCode::BAD_GATEWAY, "External service unavailable".to_string())
            }
            AppError::Service(ServiceError::Internal(msg)) => {
                tracing::error!("Internal error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            }

            // Validation 에러
            AppError::Validation(errors) => {
                let error_messages: Vec<String> = errors
                    .field_errors()
                    .iter()
                    .flat_map(|(field, errors)| {
                        errors.iter().map(move |error| {
                            format!("{}: {}", field, error.message.as_ref().unwrap_or(&"validation failed".into()))
                        })
                    })
                    .collect();

                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "error": "Validation failed",
                        "details": error_messages
                    }))
                ).into_response();
            }

            // 직접 정의 에러
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()),
            AppError::NotFound => (StatusCode::NOT_FOUND, "Not found".to_string()),
            AppError::InternalServerError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            }
        };

        let body = Json(json!({
            "error": error_message
        }));

        (status, body).into_response()
    }
}
```

---

## 에러 변환 흐름

```rust
// Repository → Service
impl UserRepository {
    pub async fn find_by_id(&self, id: i32) -> Result<Option<user::Model>, DbErr> {
        User::find_by_id(id).one(self.db.as_ref()).await
        // DbErr 반환
    }
}

// Service: DbErr → ServiceError (자동 변환 - #[from] 덕분)
impl UserService {
    pub async fn get_user(&self, user_id: i32) -> Result<UserResponse, ServiceError> {
        let user = self.user_repo.find_by_id(user_id).await?;  // ? 사용 시 자동 변환
        // ...
        Ok(UserResponse::from(user))
    }
}

// Handler: ServiceError → AppError (자동 변환)
pub async fn get_user(
    State(service): State<UserService>,
    Path(user_id): Path<i32>,
) -> Result<Json<UserResponse>, AppError> {
    let user = service.get_user(user_id).await?;  // ? 사용 시 자동 변환
    Ok(Json(user))
}
```

---

## 에러 로깅

**민감 정보는 로그에만, 사용자에게는 일반 메시지:**

```rust
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::Service(ServiceError::Database(ref db_err)) => {
                // 상세 에러는 로그에만
                tracing::error!("Database error: {:?}", db_err);
                // 사용자에게는 일반 메시지
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            }
            // ...
        };
        // ...
    }
}
```

---

## 커스텀 에러 추가

### 도메인별 Service 에러

```rust
#[derive(Debug, Error)]
pub enum SurveyServiceError {
    #[error("Survey not found")]
    SurveyNotFound,

    #[error("Survey is closed")]
    SurveyClosed,

    #[error("Required question not answered: {0}")]
    RequiredQuestionMissing(i32),

    #[error("Invalid question type")]
    InvalidQuestionType,

    #[error(transparent)]
    Base(#[from] ServiceError),  // 기본 ServiceError 위임
}

// AppError로 변환
impl From<SurveyServiceError> for AppError {
    fn from(err: SurveyServiceError) -> Self {
        match err {
            SurveyServiceError::SurveyNotFound => {
                AppError::Service(ServiceError::NotFound("Survey not found".into()))
            }
            SurveyServiceError::SurveyClosed => {
                AppError::Service(ServiceError::InvalidInput("Survey is closed".into()))
            }
            SurveyServiceError::RequiredQuestionMissing(id) => {
                AppError::Service(ServiceError::InvalidInput(
                    format!("Required question {} not answered", id)
                ))
            }
            SurveyServiceError::InvalidQuestionType => {
                AppError::Service(ServiceError::InvalidInput("Invalid question type".into()))
            }
            SurveyServiceError::Base(service_err) => AppError::Service(service_err),
        }
    }
}
```

---

## 에러 테스트

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_error_display() {
        let err = ServiceError::NotFound("User 123 not found".into());
        assert_eq!(err.to_string(), "Resource not found: User 123 not found");
    }

    #[tokio::test]
    async fn test_app_error_response() {
        let err = AppError::Service(ServiceError::EmailAlreadyExists);
        let response = err.into_response();

        // 상태 코드 검증
        assert_eq!(response.status(), StatusCode::CONFLICT);
    }
}
```

---

## mod.rs 구조

```rust
// src/errors/mod.rs
mod app_error;
mod service_error;

pub use app_error::AppError;
pub use service_error::ServiceError;
```

---

## 에러 처리 체크리스트

### ServiceError
- [ ] `#[derive(Debug, Error)]` 추가
- [ ] 각 에러 케이스에 명확한 메시지
- [ ] `#[from] DbErr` 자동 변환 구현
- [ ] 비즈니스 규칙 위반을 명확히 표현

### AppError
- [ ] `IntoResponse` 구현
- [ ] 각 ServiceError를 적절한 HTTP 상태 코드로 매핑
- [ ] 민감 정보는 로그에만, 사용자에게는 일반 메시지
- [ ] Validation 에러 처리
- [ ] JSON 응답 형식 일관성 유지

### 로깅
- [ ] 500 에러는 반드시 로그 기록
- [ ] 스택 트레이스 포함 (Debug 모드)
- [ ] 민감 정보 마스킹
