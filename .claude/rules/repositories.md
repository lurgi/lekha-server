---
paths: src/repositories/**/*.rs
---

# Repositories 계층 규칙

Repository 계층은 데이터베이스 CRUD 작업만 수행하며, 비즈니스 로직을 포함하지 않습니다.

## 계층 역할
- 데이터베이스 CRUD 작업
- SeaORM을 사용한 타입 안전 쿼리
- 관계(Relation) 데이터 로드
- **비즈니스 로직 금지**

---

## Repository 메서드 명명 규칙

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

---

## 관계(Relation) 처리

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

## SeaORM 사용 규칙

### ActiveModel 패턴
- SeaORM의 ActiveModel 패턴 사용
- `Set(value)`로 필드 설정
- `..Default::default()`로 나머지 필드 초기화

### Raw SQL 지양
- Raw SQL 쿼리 지양
- SeaORM의 타입 안전 쿼리 빌더 사용
- 타입 안전성 최대한 활용

### 에러 처리
- Repository는 `DbErr`를 그대로 반환
- 비즈니스 의미의 에러 변환은 Service 계층에서

---

## 테스트 작성 기준

**복잡한 쿼리만 테스트 작성**:

1. 2개 이상의 테이블 JOIN
2. 복잡한 필터 조건 (3개 이상 AND/OR 조합)
3. 집계/그룹화 쿼리
4. 페이지네이션 + 정렬 + 필터 조합
5. Raw SQL 사용하는 경우

**단순 쿼리는 테스트 생략 가능**:
- `find_by_id()`, `find_by_email()` 같은 단순 조회
- 단순 `create()`, `update()`, `delete()`

자세한 내용은 [testing.md](./testing.md)를 참조하세요.
