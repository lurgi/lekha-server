---
paths: src/**/*_test.rs, tests/**/*.rs
---

# 테스트 규칙

## 테스트 작성 기준

### Service 테스트 (필수)

**✅ 반드시 테스트해야 하는 경우:**

1. **비즈니스 로직이 있는 경우**
   - 중복 검사 (이메일, username 등)
   - 권한/인가 확인
   - 상태 검증 (이미 완료된 주문인지, 활성화된 사용자인지 등)
   - 데이터 변환/계산 (가격 계산, 포인트 적립 등)

2. **여러 Repository를 조합하는 경우**
   - 설문 + 질문 생성처럼 여러 엔티티를 다루는 경우

3. **트랜잭션을 사용하는 경우**
   - 중간에 실패하면 롤백되는지 확인

4. **조건 분기가 있는 경우**
   - if/else, match 등의 분기 로직

5. **도메인 규칙을 강제하는 경우**
   - "게시글은 작성자만 수정 가능"
   - "주문은 결제 완료 상태에서만 취소 가능"

**🔍 테스트해야 할 케이스:**

회원가입 예시:
- ✅ 정상 회원가입 성공
- ✅ 이메일 중복 시 EmailAlreadyExists 에러
- ✅ 비밀번호가 해싱되어 저장되는지
- ✅ password_hash가 UserResponse에 노출되지 않는지

---

### Repository 테스트 (조건부)

**✅ 테스트가 필요한 "복잡한 쿼리" 기준:**

1. **2개 이상의 테이블 JOIN**
   ```rust
   find_with_related(Question).find_with_related(Response)
   ```

2. **복잡한 필터 조건 (3개 이상 AND/OR 조합)**
   ```rust
   .filter(user::Column::Active.eq(true))
   .filter(user::Column::CreatedAt.gt(date))
   .filter(user::Column::Role.eq(UserRole::Admin))
   ```

3. **집계/그룹화 쿼리**
   ```rust
   .select_only()
   .column_as(user::Column::Id.count(), "count")
   .group_by(user::Column::Role)
   ```

4. **페이지네이션 + 정렬 + 필터 조합**

5. **Raw SQL 사용하는 경우**

**❌ 테스트 생략 가능 (단순 쿼리):**
- `find_by_id()`, `find_by_email()` 같은 단순 조회
- 단순 `create()`, `update()`, `delete()`

**테스트 방법:**
- `sqlite::memory:` 사용한 인메모리 DB 테스트
- 또는 `testcontainers` 사용한 PostgreSQL 컨테이너 테스트

---

### Handler 테스트
- **기본적으로 작성하지 않음**
- Service 테스트로 비즈니스 로직 커버
- 필요시 통합 테스트만 작성

---

## 단위 테스트 위치

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

---

## 통합 테스트 구조

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
