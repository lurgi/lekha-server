# Handlers 계층 규칙

이 디렉토리는 **HTTP 요청/응답 처리 전용**입니다.

## Handler의 책임

✅ **해야 할 일:**
- HTTP 요청 파싱 (JSON, Query, Path 파라미터)
- 입력 검증 (DTO validation)
- Service 호출
- HTTP 응답 반환

❌ **하지 말아야 할 일:**
- 비즈니스 로직 작성
- DB 직접 접근
- 복잡한 데이터 변환

---

## 파일 명명 규칙

```
src/handlers/
├── mod.rs              # 모든 handler re-export
├── user_handler.rs     # User 관련 엔드포인트
├── survey_handler.rs   # Survey 관련 엔드포인트
└── health_handler.rs   # Health check
```

**규칙:**
- 파일명: `{리소스명}_handler.rs`
- 모듈명: `user_handler` (snake_case)

---

## Handler 함수 작성 규칙

### 1. 함수 시그니처

**기본 패턴:**
```rust
use axum::{extract::State, Json};
use crate::{models::dto::*, errors::AppError, services::UserService};

pub async fn create_user(
    State(user_service): State<UserService>,
    Json(req): Json<CreateUserRequest>,
) -> Result<Json<UserResponse>, AppError> {
    req.validate()?;  // 검증
    let user = user_service.create_user(req).await?;
    Ok(Json(user))
}
```

**Path 파라미터:**
```rust
use axum::extract::Path;

pub async fn get_user(
    State(user_service): State<UserService>,
    Path(user_id): Path<i32>,
) -> Result<Json<UserResponse>, AppError> {
    let user = user_service.get_user(user_id).await?;
    Ok(Json(user))
}
```

**Query 파라미터:**
```rust
use axum::extract::Query;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Pagination {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_limit")]
    pub limit: u32,
}

fn default_page() -> u32 { 1 }
fn default_limit() -> u32 { 20 }

pub async fn list_users(
    State(user_service): State<UserService>,
    Query(pagination): Query<Pagination>,
) -> Result<Json<Vec<UserResponse>>, AppError> {
    let users = user_service.list_users(pagination.page, pagination.limit).await?;
    Ok(Json(users))
}
```

**복합 추출자:**
```rust
use axum::extract::{State, Path, Json};

pub async fn update_user(
    State(user_service): State<UserService>,
    Path(user_id): Path<i32>,
    Json(req): Json<UpdateUserRequest>,
) -> Result<Json<UserResponse>, AppError> {
    req.validate()?;
    let user = user_service.update_user(user_id, req).await?;
    Ok(Json(user))
}
```

---

## 2. 상태 코드 명시

**성공 응답 상태 코드:**
```rust
use axum::http::StatusCode;

// 201 Created
pub async fn create_user(
    State(user_service): State<UserService>,
    Json(req): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<UserResponse>), AppError> {
    req.validate()?;
    let user = user_service.create_user(req).await?;
    Ok((StatusCode::CREATED, Json(user)))
}

// 204 No Content
pub async fn delete_user(
    State(user_service): State<UserService>,
    Path(user_id): Path<i32>,
) -> Result<StatusCode, AppError> {
    user_service.delete_user(user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
```

---

## 3. 입력 검증

**validator 사용:**
```rust
use validator::Validate;

pub async fn create_user(
    State(user_service): State<UserService>,
    Json(req): Json<CreateUserRequest>,
) -> Result<Json<UserResponse>, AppError> {
    req.validate()?;  // ⚠️ 필수! Service 호출 전 검증
    let user = user_service.create_user(req).await?;
    Ok(Json(user))
}
```

**커스텀 검증:**
```rust
pub async fn upload_file(
    State(file_service): State<FileService>,
    Json(req): Json<UploadFileRequest>,
) -> Result<Json<FileResponse>, AppError> {
    // 파일 크기 검증
    if req.size > 10 * 1024 * 1024 {
        return Err(AppError::InvalidInput("File too large".into()));
    }

    let file = file_service.upload(req).await?;
    Ok(Json(file))
}
```

---

## 4. 에러 처리

**Result 타입 사용:**
```rust
// ✅ 좋은 예
pub async fn get_user(
    State(user_service): State<UserService>,
    Path(user_id): Path<i32>,
) -> Result<Json<UserResponse>, AppError> {
    let user = user_service.get_user(user_id).await?;  // ? 사용
    Ok(Json(user))
}

// ❌ 나쁜 예
pub async fn get_user(
    State(user_service): State<UserService>,
    Path(user_id): Path<i32>,
) -> Json<UserResponse> {
    let user = user_service.get_user(user_id).await.unwrap();  // panic 위험!
    Json(user)
}
```

---

## 5. Router 등록 (mod.rs)

**모듈 구조:**
```rust
// src/handlers/mod.rs
mod user_handler;
mod survey_handler;
mod health_handler;

use axum::{routing::*, Router};
use crate::AppState;

pub fn create_router(state: AppState) -> Router {
    Router::new()
        // Health check
        .route("/health", get(health_handler::health_check))

        // User routes
        .route("/users", post(user_handler::create_user))
        .route("/users", get(user_handler::list_users))
        .route("/users/:id", get(user_handler::get_user))
        .route("/users/:id", put(user_handler::update_user))
        .route("/users/:id", delete(user_handler::delete_user))

        // Survey routes
        .route("/surveys", post(survey_handler::create_survey))
        .route("/surveys/:id", get(survey_handler::get_survey))

        .with_state(state)
}
```

**RESTful 라우팅 규칙:**
```
POST   /users           → create_user
GET    /users           → list_users
GET    /users/:id       → get_user
PUT    /users/:id       → update_user
PATCH  /users/:id       → partial_update_user
DELETE /users/:id       → delete_user

POST   /users/:id/surveys          → create_user_survey (중첩 리소스)
GET    /users/:id/surveys          → list_user_surveys
```

---

## 6. 인증/인가 (미들웨어)

**JWT 토큰 추출:**
```rust
use axum::{extract::State, http::Request, middleware::Next, response::Response};

pub async fn auth_middleware<B>(
    State(auth_service): State<AuthService>,
    mut req: Request<B>,
    next: Next<B>,
) -> Result<Response, AppError> {
    let token = req.headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or(AppError::Unauthorized)?;

    let user_id = auth_service.verify_token(token).await?;
    req.extensions_mut().insert(user_id);

    Ok(next.run(req).await)
}

// 사용
pub async fn get_my_profile(
    Extension(user_id): Extension<i32>,  // 미들웨어에서 주입
    State(user_service): State<UserService>,
) -> Result<Json<UserResponse>, AppError> {
    let user = user_service.get_user(user_id).await?;
    Ok(Json(user))
}
```

---

## 7. 금지 사항

### ❌ Handler에 비즈니스 로직 작성 금지
```rust
// 나쁜 예
pub async fn create_user(
    State(repo): State<UserRepository>,  // ❌ Repository 직접 사용
    Json(req): Json<CreateUserRequest>,
) -> Result<Json<UserResponse>, AppError> {
    // ❌ 비즈니스 로직이 Handler에 있음
    if repo.find_by_email(&req.email).await?.is_some() {
        return Err(AppError::Conflict("Email exists".into()));
    }

    let password_hash = bcrypt::hash(&req.password, 10)?;
    let user = repo.create(req.username, req.email, password_hash).await?;

    Ok(Json(user.into()))
}
```

**올바른 예:**
```rust
pub async fn create_user(
    State(user_service): State<UserService>,  // ✅ Service 사용
    Json(req): Json<CreateUserRequest>,
) -> Result<Json<UserResponse>, AppError> {
    req.validate()?;
    let user = user_service.create_user(req).await?;  // ✅ 로직은 Service에
    Ok(Json(user))
}
```

---

## Handler는 얇게 유지

Handler는 **HTTP와 비즈니스 로직을 연결하는 얇은 레이어**여야 합니다.

**이상적인 Handler (5줄 이내):**
```rust
pub async fn create_user(
    State(service): State<UserService>,
    Json(req): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<UserResponse>), AppError> {
    req.validate()?;
    let user = service.create_user(req).await?;
    Ok((StatusCode::CREATED, Json(user)))
}
```

**복잡한 로직은 모두 Service로!**
