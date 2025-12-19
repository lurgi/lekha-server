---
paths: src/services/**/*.rs
---

# Services 계층 규칙

Service 계층은 비즈니스 로직을 구현하고, Repository를 조합하며, 트랜잭션을 관리합니다.

## 계층 역할
- 비즈니스 로직 구현 (중복 검사, 권한 확인, 상태 검증 등)
- 여러 Repository 조합
- 트랜잭션 관리
- 데이터 변환 (Entity → DTO)
- **직접 DB 접근 금지 (Repository를 통해서만)**

---

## Service 구조체 정의

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

---

## 트랜잭션 처리

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

---

## 에러 처리

**설명**: Repository 에러를 Service 에러로 변환하여 비즈니스 의미를 부여한다.

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

## 비즈니스 로직 구현 가이드

Service 계층에서 구현해야 할 비즈니스 로직:

1. **중복 검사**: 이메일, username 등 유니크 필드 확인
2. **권한/인가**: 사용자가 해당 작업을 수행할 권한이 있는지
3. **상태 검증**: 주문 취소 가능 상태인지, 활성화된 사용자인지 등
4. **데이터 변환/계산**: 가격 계산, 포인트 적립, 비밀번호 해싱 등
5. **도메인 규칙 강제**: "게시글은 작성자만 수정 가능" 등

---

## 테스트 필수

Service 계층은 **반드시 단위 테스트를 작성**해야 합니다.

- 정상 케이스
- 비즈니스 로직 분기별 검증
- 에러 케이스 (NotFound, Conflict 등)
- 민감 정보 노출 여부 (password_hash 등)

자세한 내용은 [testing.md](./testing.md)를 참조하세요.
