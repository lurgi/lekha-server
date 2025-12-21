---
contextPaths:
  - path: "src/handlers/**/*.rs"
    rules:
      - ".claude/rules/handlers.md"
  - path: "src/services/**/*.rs"
    rules:
      - ".claude/rules/services.md"
  - path: "src/repositories/**/*.rs"
    rules:
      - ".claude/rules/repositories.md"
  - path: "src/entities/**/*.rs"
    rules:
      - ".claude/rules/entities.md"
  - path: "src/models/**/*.rs"
    rules:
      - ".claude/rules/models.md"
  - path: "src/errors/**/*.rs"
    rules:
      - ".claude/rules/errors.md"
  - path: "**/*_test.rs"
    rules:
      - ".claude/rules/testing.md"
  - path: "tests/**/*.rs"
    rules:
      - ".claude/rules/testing.md"
---

# Inklings Server - 프로젝트 규칙

## 프로젝트 개요

Rust + Axum + SeaORM + PostgreSQL 기반의 3계층 아키텍처 웹 서버

## 📁 규칙 파일 구조

이 프로젝트는 다음 규칙 파일들을 따릅니다. 각 파일은 특정 컨텍스트에만 자동으로 로드됩니다.

- **[코딩 스타일](./rules/coding-style.md)**: 주석 규칙, Rust 표준, Async/Await
- **[아키텍처](./rules/architecture.md)**: 3계층 구조, SeaORM, 에러 처리
- **[핸들러 규칙](./rules/handlers.md)**: Handler 계층 상세 규칙 (`src/handlers/**/*.rs`)
- **[서비스 규칙](./rules/services.md)**: Service 계층 상세 규칙 (`src/services/**/*.rs`)
- **[리포지토리 규칙](./rules/repositories.md)**: Repository 계층 상세 규칙 (`src/repositories/**/*.rs`)
- **[엔티티 규칙](./rules/entities.md)**: Entity 계층 상세 규칙 (`src/entities/**/*.rs`)
- **[모델 규칙](./rules/models.md)**: Models/DTO 계층 상세 규칙 (`src/models/**/*.rs`)
- **[에러 규칙](./rules/errors.md)**: Errors 계층 상세 규칙 (`src/errors/**/*.rs`)
- **[워크플로우](./rules/workflow.md)**: 기능 추가 단계별 프로세스
- **[테스트](./rules/testing.md)**: Service/Repository/Handler 테스트 기준 (테스트 파일)

---

## Claude Code 작업 규칙

### 명령어 실행 규칙
- **마이그레이션은 절대 실행하지 않는다** (`cargo run -p migration` 금지)
- **데이터를 변경하는 명령어는 실행하지 않는다** (git push, npm install 등)
- **읽기 전용 명령어는 실행 가능** (cargo test, cargo build, git status 등)
- 사용자가 명령어를 직접 실행하고 결과를 공유할 수도 있다

### Git 작업 규칙
- **절대 사용자 승인 없이 `git push`를 실행하지 않는다**
- 커밋은 사용자가 명시적으로 요청했을 때만 수행
- Push 전에 반드시 사용자에게 변경 사항을 확인받는다

### 사고 과정 (Thinking Process)
- **복잡한 문제 해결 시 Sequential Thinking MCP를 사용하여 단계적으로 사고한다**
- **코드베이스 분석 시 Context7 MCP를 활용하여 맥락을 파악한다**
- 문제를 작은 단위로 나누어 접근한다
- 가정을 명확히 하고, 불확실한 부분은 사용자에게 질문한다

---

## 빠른 참조

### 기술 스택
- **언어:** Rust
- **웹 프레임워크:** Axum
- **데이터베이스:** PostgreSQL
- **ORM:** SeaORM
- **Async Runtime:** Tokio

### 3계층 아키텍처
```
Handler (HTTP) → Service (비즈니스 로직) → Repository (DB 접근)
```

### 구현 순서 (Bottom-up)
```
1. Entity + Migration
2. Repository
3. Service (+ 필수 테스트)
4. Handler
```

### 핵심 원칙
- ❌ `unwrap()` 사용 금지 → ✅ `?` 연산자 사용
- ❌ Entity 직접 노출 금지 → ✅ DTO 사용
- ❌ Handler에서 DB 직접 접근 금지 → ✅ Service 호출
- ❌ 불필요한 주석 금지 → ✅ 코드로 의미 표현

---

**자세한 규칙은 [`.claude/rules/`](./rules/) 폴더를 참조하세요.**
