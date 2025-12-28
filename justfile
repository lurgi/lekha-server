# Inklings Server - justfile

# 기본 레시피 목록 보기
default:
    @just --list

# ========================================
# Docker Compose 명령어
# ========================================

# 모든 서비스 시작 (PostgreSQL + Qdrant)
docker-up:
    docker-compose up -d
    @echo "✅ All services started!"
    @echo "   PostgreSQL: localhost:5432"
    @echo "   Qdrant: localhost:6333"

# 모든 서비스 종료
docker-down:
    docker-compose down
    @echo "✅ All services stopped!"

# 서비스 로그 확인
docker-logs:
    docker-compose logs -f

# 실행 중인 서비스 확인
docker-ps:
    docker-compose ps

# 서비스 재시작
docker-restart:
    docker-compose restart

# PostgreSQL만 시작
postgres-up:
    docker-compose up -d postgres
    @echo "✅ PostgreSQL started on port 5432"

# Qdrant만 시작
qdrant-up:
    docker-compose up -d qdrant
    @echo "✅ Qdrant started on port 6333"

# 모든 서비스 종료 + 볼륨 삭제 (데이터 초기화)
docker-clean:
    docker-compose down -v
    @echo "✅ All services stopped and volumes removed!"

# ========================================
# 개발 서버
# ========================================

# 개발 서버 실행
dev:
    cargo run

# 프로덕션 빌드
build:
    cargo build --release

# 테스트 실행
test:
    cargo test

# 마이그레이션 실행
migrate:
    cargo run -p migration up

# 마이그레이션 되돌리기
migrate-down:
    cargo run -p migration down

# 마이그레이션 상태 확인
migrate-status:
    cargo run -p migration status

# 테스트 DB 설정 (Docker)
setup-test-db:
    @echo "Setting up test database with Docker..."
    -docker stop inklings_test_postgres 2>/dev/null
    -docker rm inklings_test_postgres 2>/dev/null
    docker run -d \
      --name inklings_test_postgres \
      -e POSTGRES_USER=inklings_user \
      -e POSTGRES_PASSWORD=inklings_dev_password \
      -e POSTGRES_DB=inklings_test_db \
      -p 5433:5432 \
      postgres:15
    @echo "Waiting for PostgreSQL to start..."
    @sleep 3
    @echo "Running migrations..."
    DATABASE_URL=postgres://inklings_user:inklings_dev_password@localhost:5433/inklings_test_db cargo run -p migration up
    @echo ""
    @echo "✅ Test database setup complete!"
    @echo "   Container: inklings_test_postgres"
    @echo "   Port: 5433"
    @echo "   Database: inklings_test_db"

# 테스트 DB 삭제
teardown-test-db:
    @echo "Removing test database container..."
    -docker stop inklings_test_postgres
    -docker rm inklings_test_postgres
    @echo "✅ Test database removed!"

# 코드 포맷팅
fmt:
    cargo fmt

# 코드 포맷팅 체크
fmt-check:
    cargo fmt -- --check

# Clippy 린트
lint:
    cargo clippy -- -D warnings

# 전체 CI 체크 (포맷, 린트, 테스트)
ci: fmt-check lint test
    @echo "✅ All CI checks passed!"

# 의존성 업데이트 확인
outdated:
    cargo outdated

# 프로젝트 클린
clean:
    cargo clean
    -docker stop inklings_test_postgres
    -docker rm inklings_test_postgres
