# 기능 추가 워크플로우

**중요: 각 단계마다 개발자와 검토하고 피드백을 받은 후 다음 단계로 진행한다.**

---

## 🎯 핵심 원칙

**한 번에 하나의 계층만 구현하고, 반드시 검토 후 다음 단계로 진행한다.**

---

## 1. 요구사항 분석 단계

**개발자와 함께:**
- **API 스펙 정의**: 엔드포인트, HTTP 메서드, 요청/응답 형식
- **필요한 데이터 파악**: 어떤 데이터를 저장하고 조회할지
- **비즈니스 로직 파악**: 유효성 검증, 권한 체크, 중복 확인 등

**✅ 체크포인트**: 개발자와 요구사항 합의 완료

---

## 2. 설계 단계

**개발자와 함께:**
- **DB 스키마 설계**: Entity 필드, 타입, 제약조건, 관계 정의
- **DTO 설계**: Request DTO (검증 규칙 포함), Response DTO (민감 정보 제외)
- **에러 케이스 정의**: 발생 가능한 에러와 HTTP 상태 코드

**✅ 체크포인트**: 개발자와 설계 합의 완료

---

## 3. 구현 단계 (Bottom-up, 단계별 검토 필수)

### Step 1: Entity + Migration 작성

1. **Entity 구현**
   - `src/entities/` 에 Entity 정의
   - 필드, 타입, 관계, 제약조건 포함

2. **✋ STOP - 검토 단계**
   - 개발자에게 Entity 코드 보여주기
   - 피드백 받기 및 수정

3. **Migration 구현**
   - `migration/src/` 에 마이그레이션 작성
   - Entity와 정확히 일치하는 스키마

4. **✋ STOP - 검토 단계**
   - 개발자에게 Migration 코드 보여주기
   - 피드백 받기 및 수정

5. **개발자가 직접 실행**
   - `cargo run -p migration up` 실행은 **개발자가 직접** 수행
   - Claude는 실행하지 않음

**✅ 체크포인트**: Entity + Migration 완료, DB 스키마 생성 확인

---

### Step 2: Repository 구현

1. **Repository 구현**
   - `src/repositories/` 에 Repository 구현
   - CRUD 메서드 작성 (find_*, create, update, delete)

2. **✋ STOP - 검토 단계**
   - 개발자에게 Repository 코드 보여주기
   - 필요한 메서드가 모두 있는지 확인
   - 피드백 받기 및 수정

**✅ 체크포인트**: Repository 완료, 개발자 승인

---

### Step 3: Service 구현

**중요: Service는 4단계로 나누어 구현하고, 각 단계마다 검토를 받는다.**

1. **DTO 구현**
   - `src/models/` 에 Request/Response DTO 작성
   - Request DTO: 요청 데이터 구조
   - Response DTO: 응답 데이터 구조 (Entity → DTO 변환)

2. **✋ STOP - DTO 검토**
   - 개발자에게 DTO 코드 보여주기
   - 필드와 타입이 적절한지 확인
   - 피드백 받기 및 수정

3. **ServiceError 구현**
   - `src/errors/` 에 Service 에러 타입 작성
   - 비즈니스 에러 정의 (NotFound, Unauthorized 등)
   - HTTP 상태 코드 매핑

4. **✋ STOP - Error 검토**
   - 개발자에게 Error 코드 보여주기
   - 에러 케이스가 충분한지 확인
   - 피드백 받기 및 수정

5. **Service 로직 구현**
   - `src/services/` 에 Service 구현
   - Repository를 조합하여 비즈니스 로직 작성

6. **✋ STOP - Service 검토**
   - 개발자에게 Service 코드 보여주기
   - 비즈니스 로직이 요구사항과 맞는지 확인
   - 피드백 받기 및 수정

7. **Test 구현**
   - Service 단위 테스트 작성
   - 정상 케이스, 에러 케이스 모두 포함

8. **✋ STOP - Test 검토**
   - 개발자에게 Test 코드 보여주기
   - 테스트 케이스가 충분한지 확인
   - 피드백 받기 및 수정

**✅ 체크포인트**: Service + 테스트 완료, 개발자 승인

---

### Step 4: Handler 구현

1. **Handler 구현**
   - `src/handlers/` 에 Handler 구현
   - DTO 검증, Service 호출, 응답 반환
   - 라우터에 엔드포인트 등록

2. **✋ STOP - 검토 단계**
   - 개발자에게 Handler 코드 보여주기
   - API 스펙과 맞는지 확인
   - 피드백 받기 및 수정

**✅ 체크포인트**: Handler 완료, 개발자 승인

---

## 4. 테스트 및 검증

**개발자와 함께:**
- Service 테스트 실행 (`cargo test`)
- API 수동 테스트 (curl, Postman 등)
- 버그 발견 시 즉시 수정

**✅ 체크포인트**: 모든 테스트 통과, 기능 완성

---

## ⚠️ 중요 규칙

1. **절대 여러 단계를 한 번에 구현하지 않는다**
2. **각 단계마다 반드시 개발자 검토를 받는다**
3. **검토 없이 다음 단계로 진행하지 않는다**
4. **마이그레이션 실행은 개발자가 직접 한다**
5. **개발자의 피드백을 우선시한다**
