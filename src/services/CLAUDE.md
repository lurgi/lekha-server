# Services 계층 규칙

이 디렉토리는 **비즈니스 로직 전용**입니다.

## Service의 책임

✅ **해야 할 일:**
- 비즈니스 규칙 구현
- 여러 Repository 조합
- 트랜잭션 관리
- 도메인 이벤트 처리
- 외부 API 호출

❌ **하지 말아야 할 일:**
- HTTP 요청/응답 직접 처리
- 직접 SQL 작성
- Entity를 직접 반환 (DTO 변환 필수)

---

## 파일 명명 규칙

```
src/services/
├── mod.rs              # 모든 service re-export
├── user_service.rs     # User 비즈니스 로직
├── survey_service.rs   # Survey 비즈니스 로직
└── auth_service.rs     # 인증/인가 로직
```

---

## Service 구조체 정의

### 기본 패턴

```rust
use crate::repositories::{UserRepository, SurveyRepository};
use sea_orm::DatabaseConnection;
use std::sync::Arc;

#[derive(Clone)]
pub struct UserService {
    db: Arc<DatabaseConnection>,
    user_repo: UserRepository,
    survey_repo: SurveyRepository,  // 다른 Repository도 사용 가능
}

impl UserService {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self {
            db: db.clone(),
            user_repo: UserRepository::new(db.clone()),
            survey_repo: SurveyRepository::new(db),
        }
    }
}
```

**왜 db 필드가 필요한가?**
- 트랜잭션 시작 시 필요: `self.db.begin().await?`

---

## 비즈니스 로직 패턴

### 1. CRUD + 비즈니스 규칙

```rust
use crate::models::dto::*;
use crate::errors::ServiceError;

impl UserService {
    pub async fn create_user(&self, req: CreateUserRequest) -> Result<UserResponse, ServiceError> {
        // 1. 비즈니스 규칙: 중복 검사
        if self.user_repo.find_by_email(&req.email).await?.is_some() {
            return Err(ServiceError::EmailAlreadyExists);
        }

        // 2. 비즈니스 규칙: 비밀번호 해싱
        let password_hash = bcrypt::hash(&req.password, bcrypt::DEFAULT_COST)
            .map_err(|_| ServiceError::InvalidInput("Password hashing failed".into()))?;

        // 3. Repository 호출
        let user = self.user_repo
            .create(req.username, req.email, password_hash)
            .await?;

        // 4. Entity → DTO 변환
        Ok(UserResponse::from(user))
    }

    pub async fn get_user(&self, user_id: i32) -> Result<UserResponse, ServiceError> {
        let user = self.user_repo
            .find_by_id(user_id)
            .await?
            .ok_or(ServiceError::NotFound(format!("User {} not found", user_id)))?;

        Ok(UserResponse::from(user))
    }

    pub async fn update_user(
        &self,
        user_id: i32,
        req: UpdateUserRequest,
    ) -> Result<UserResponse, ServiceError> {
        // 권한 검증, 중복 체크 등 비즈니스 로직
        let user = self.user_repo.update(user_id, req.username).await?;
        Ok(UserResponse::from(user))
    }

    pub async fn delete_user(&self, user_id: i32) -> Result<(), ServiceError> {
        self.user_repo.delete(user_id).await?;
        Ok(())
    }
}
```

---

### 2. 여러 Repository 조합

```rust
impl UserService {
    pub async fn get_user_with_surveys(&self, user_id: i32) -> Result<UserWithSurveysResponse, ServiceError> {
        // 1. User 조회
        let user = self.user_repo
            .find_by_id(user_id)
            .await?
            .ok_or(ServiceError::NotFound(format!("User {} not found", user_id)))?;

        // 2. User의 Surveys 조회
        let surveys = self.survey_repo
            .find_by_user_id(user_id)
            .await?;

        // 3. 조합된 응답 생성
        Ok(UserWithSurveysResponse {
            user: UserResponse::from(user),
            surveys: surveys.into_iter().map(SurveyResponse::from).collect(),
        })
    }
}
```

---

### 3. 트랜잭션 처리

**여러 작업이 원자적으로 실행되어야 할 때:**

```rust
use sea_orm::TransactionTrait;

impl SurveyService {
    pub async fn create_survey_with_questions(
        &self,
        user_id: i32,
        req: CreateSurveyRequest,
    ) -> Result<SurveyResponse, ServiceError> {
        // 트랜잭션 시작
        let txn = self.db.begin().await?;

        // 1. Survey 생성
        let survey = self.survey_repo
            .create_with_txn(&txn, user_id, &req.title, &req.description)
            .await?;

        // 2. Questions 생성
        for question_req in req.questions {
            self.question_repo
                .create_with_txn(&txn, survey.id, &question_req.text, question_req.question_type)
                .await?;
        }

        // 3. 커밋 (실패 시 자동 롤백)
        txn.commit().await?;

        // 4. 생성된 Survey + Questions 조회
        let (survey, questions) = self.survey_repo
            .find_with_questions(survey.id)
            .await?;

        Ok(SurveyResponse {
            id: survey.id,
            title: survey.title,
            questions: questions.into_iter().map(QuestionResponse::from).collect(),
        })
    }
}
```

**Repository에 트랜잭션 메서드 추가:**
```rust
// repositories/survey_repository.rs
impl SurveyRepository {
    // 일반 메서드
    pub async fn create(&self, user_id: i32, title: &str) -> Result<survey::Model, DbErr> {
        // ...
        active_model.insert(self.db.as_ref()).await
    }

    // 트랜잭션용 메서드
    pub async fn create_with_txn(
        &self,
        txn: &DatabaseTransaction,
        user_id: i32,
        title: &str,
        description: &str,
    ) -> Result<survey::Model, DbErr> {
        // ...
        active_model.insert(txn).await  // db 대신 txn 사용
    }
}
```

---

### 4. 복잡한 비즈니스 로직

```rust
impl SurveyService {
    pub async fn submit_survey_response(
        &self,
        user_id: i32,
        survey_id: i32,
        req: SubmitSurveyRequest,
    ) -> Result<SurveyResponseSubmission, ServiceError> {
        // 1. Survey 존재 및 활성 상태 확인
        let survey = self.survey_repo
            .find_by_id(survey_id)
            .await?
            .ok_or(ServiceError::NotFound("Survey not found".into()))?;

        if !survey.is_active {
            return Err(ServiceError::InvalidInput("Survey is closed".into()));
        }

        // 2. 중복 응답 방지
        if self.response_repo.exists(user_id, survey_id).await? {
            return Err(ServiceError::Conflict("Already submitted".into()));
        }

        // 3. 필수 질문 답변 검증
        let questions = self.question_repo.find_by_survey_id(survey_id).await?;
        let required_question_ids: Vec<i32> = questions
            .iter()
            .filter(|q| q.is_required)
            .map(|q| q.id)
            .collect();

        let answered_question_ids: Vec<i32> = req.answers
            .iter()
            .map(|a| a.question_id)
            .collect();

        for required_id in required_question_ids {
            if !answered_question_ids.contains(&required_id) {
                return Err(ServiceError::InvalidInput(
                    format!("Required question {} not answered", required_id)
                ));
            }
        }

        // 4. 트랜잭션으로 응답 저장
        let txn = self.db.begin().await?;

        let response = self.response_repo
            .create_with_txn(&txn, user_id, survey_id)
            .await?;

        for answer in req.answers {
            self.answer_repo
                .create_with_txn(&txn, response.id, answer.question_id, &answer.value)
                .await?;
        }

        txn.commit().await?;

        Ok(SurveyResponseSubmission {
            response_id: response.id,
            submitted_at: response.created_at,
        })
    }
}
```

---

## 에러 처리

### ServiceError 정의

```rust
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

    #[error("Email already exists")]
    EmailAlreadyExists,

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Database error")]
    Database(#[from] DbErr),

    #[error("External API error: {0}")]
    ExternalApi(String),
}
```

---

## 외부 API 호출

```rust
use reqwest::Client;

impl NotificationService {
    pub async fn send_email(
        &self,
        to: &str,
        subject: &str,
        body: &str,
    ) -> Result<(), ServiceError> {
        let client = Client::new();

        let response = client
            .post("https://api.sendgrid.com/v3/mail/send")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&serde_json::json!({
                "personalizations": [{"to": [{"email": to}]}],
                "from": {"email": "noreply@example.com"},
                "subject": subject,
                "content": [{"type": "text/plain", "value": body}]
            }))
            .send()
            .await
            .map_err(|e| ServiceError::ExternalApi(e.to_string()))?;

        if !response.status().is_success() {
            return Err(ServiceError::ExternalApi("Email send failed".into()));
        }

        Ok(())
    }
}
```

---

## mod.rs 구조

```rust
// src/services/mod.rs
mod user_service;
mod survey_service;
mod auth_service;

pub use user_service::UserService;
pub use survey_service::SurveyService;
pub use auth_service::AuthService;

use sea_orm::DatabaseConnection;
use std::sync::Arc;

#[derive(Clone)]
pub struct Services {
    pub user: UserService,
    pub survey: SurveyService,
    pub auth: AuthService,
}

impl Services {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self {
            user: UserService::new(db.clone()),
            survey: SurveyService::new(db.clone()),
            auth: AuthService::new(db),
        }
    }
}
```

---

## 테스트

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::mock;

    // Mock Repository
    mock! {
        UserRepo {
            async fn find_by_email(&self, email: &str) -> Result<Option<user::Model>, DbErr>;
            async fn create(&self, username: String, email: String, password_hash: String) -> Result<user::Model, DbErr>;
        }
    }

    #[tokio::test]
    async fn test_create_user_duplicate_email() {
        // Mock 설정: 이메일 중복
        let mut mock_repo = MockUserRepo::new();
        mock_repo
            .expect_find_by_email()
            .returning(|_| Ok(Some(user::Model {
                id: 1,
                email: "test@example.com".into(),
                // ...
            })));

        // Service 생성 (실제로는 mock_repo 주입 필요)
        // let service = UserService::new_with_repo(mock_repo);

        // 테스트
        // let result = service.create_user(req).await;
        // assert!(matches!(result, Err(ServiceError::EmailAlreadyExists)));
    }
}
```

---

## Service는 비즈니스 로직의 중심

- **단일 책임**: 하나의 도메인 영역만 담당
- **재사용성**: Handler뿐 아니라 CLI, 백그라운드 작업에서도 사용
- **테스트 가능**: Repository를 Mock하여 단위 테스트
- **트랜잭션 경계**: Service 메서드가 트랜잭션 단위
