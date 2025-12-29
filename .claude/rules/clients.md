# Clients 계층 규칙

Client 계층은 **외부 API 및 서비스 호출**을 담당합니다.

## 계층 역할
- 외부 API 호출 (Gemini, Qdrant 등)
- Trait 기반 추상화로 구현체 교체 가능
- ClientError로 에러 처리
- Mock 구현으로 테스트 용이성 제공
- **비즈니스 로직 금지 (Service 계층에서 처리)**

---

## 디렉토리 구조

```
src/clients/
├── errors.rs              # ClientError 정의
├── mod.rs                 # 모든 클라이언트 re-export
└── gemini/                # Gemini API 클라이언트
    ├── traits.rs          # Embedder, TextGenerator 인터페이스
    ├── client.rs          # GeminiClient 실제 구현
    ├── mock.rs            # MockGeminiClient 테스트용
    ├── tests.rs           # 연결 테스트 (#[ignore])
    └── mod.rs
```

---

## Trait 기반 추상화

**설명**: Client는 Trait으로 추상화하여 구현체를 쉽게 교체할 수 있도록 한다.

**좋은 예시**:
```rust
// src/clients/gemini/traits.rs
use crate::clients::ClientError;

#[async_trait::async_trait]
pub trait Embedder: Send + Sync {
    async fn embed(&self, text: &str) -> Result<Vec<f32>, ClientError>;
    fn dimension(&self) -> usize;
}

#[async_trait::async_trait]
pub trait TextGenerator: Send + Sync {
    async fn generate(
        &self,
        prompt: &str,
        context: Vec<String>,
    ) -> Result<String, ClientError>;
}
```

**나쁜 예시**:
```rust
// Trait 없이 구체 타입에 의존
pub struct AssistService {
    gemini_client: GeminiClient,  // ❌ 구체 타입에 의존
}
```

**이유**: Trait을 사용하면 Mock으로 교체 가능하고, 나중에 다른 AI 제공자로 쉽게 변경할 수 있다.

---

## Client 구현 규칙

### 실제 구현 (GeminiClient)

```rust
// src/clients/gemini/client.rs
use super::traits::{Embedder, TextGenerator};
use crate::clients::ClientError;

#[derive(Clone)]
pub struct GeminiClient {
    api_key: String,
    client: reqwest::Client,
}

impl GeminiClient {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait::async_trait]
impl Embedder for GeminiClient {
    async fn embed(&self, text: &str) -> Result<Vec<f32>, ClientError> {
        let response = self.client
            .post(format!("{}?key={}", EMBEDDING_API_URL, self.api_key))
            .json(&request_body)
            .send()
            .await
            .map_err(|e| ClientError::Network(format!("Failed to send request: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(ClientError::GeminiApi(format!(
                "API request failed with status {}: {}",
                status, error_text
            )));
        }

        let embed_response: EmbedResponse = response.json().await
            .map_err(|e| ClientError::ParseError(format!("Failed to parse: {}", e)))?;

        Ok(embed_response.embedding.values)
    }

    fn dimension(&self) -> usize {
        768
    }
}
```

### Mock 구현 (테스트용)

```rust
// src/clients/gemini/mock.rs
use super::traits::{Embedder, TextGenerator};
use crate::clients::ClientError;

#[derive(Clone)]
pub struct MockGeminiClient {
    pub embedding_dimension: usize,
}

impl MockGeminiClient {
    pub fn new() -> Self {
        Self {
            embedding_dimension: 768,
        }
    }
}

#[async_trait::async_trait]
impl Embedder for MockGeminiClient {
    async fn embed(&self, text: &str) -> Result<Vec<f32>, ClientError> {
        // 테스트용 더미 벡터 생성
        let hash = text.len() as f32;
        let vector: Vec<f32> = (0..self.embedding_dimension)
            .map(|i| (hash + i as f32) / 1000.0)
            .collect();
        Ok(vector)
    }

    fn dimension(&self) -> usize {
        self.embedding_dimension
    }
}
```

---

## ClientError 사용 규칙

**설명**: Client는 ServiceError가 아닌 ClientError를 반환한다.

**좋은 예시**:
```rust
// src/clients/errors.rs
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("Gemini API error: {0}")]
    GeminiApi(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("JSON parsing error: {0}")]
    ParseError(String),

    #[error("Qdrant error: {0}")]
    Qdrant(String),
}

// ServiceError에 From 구현 (src/errors/service_error.rs)
impl From<ClientError> for ServiceError {
    fn from(err: ClientError) -> Self {
        match err {
            ClientError::GeminiApi(msg) => ServiceError::GeminiApi(msg),
            ClientError::Network(msg) => ServiceError::GeminiApi(msg),
            ClientError::ParseError(msg) => ServiceError::GeminiApi(msg),
            ClientError::Qdrant(msg) => ServiceError::Qdrant(msg),
        }
    }
}
```

**나쁜 예시**:
```rust
// Client가 ServiceError를 직접 반환
impl Embedder for GeminiClient {
    async fn embed(&self, text: &str) -> Result<Vec<f32>, ServiceError> {
        // ❌ 계층 분리 위반
    }
}
```

**이유**: Client는 독립적이어야 하며, Service 계층에 의존하면 안 된다.

---

## Service에서 Client 사용

**설명**: Service는 Trait을 의존성으로 받아서 Client를 사용한다.

**좋은 예시**:
```rust
use std::sync::Arc;
use crate::clients::gemini::{Embedder, TextGenerator};

pub struct AssistService {
    embedder: Arc<dyn Embedder>,           // Trait 의존
    text_generator: Arc<dyn TextGenerator>, // Trait 의존
    qdrant_repo: QdrantRepository,
    memo_repo: MemoRepository,
}

impl AssistService {
    pub fn new(
        embedder: Arc<dyn Embedder>,
        text_generator: Arc<dyn TextGenerator>,
        qdrant_repo: QdrantRepository,
        memo_repo: MemoRepository,
    ) -> Self {
        Self {
            embedder,
            text_generator,
            qdrant_repo,
            memo_repo,
        }
    }

    pub async fn assist(&self, user_id: i32, prompt: &str) -> Result<String, ServiceError> {
        // ClientError는 From 트레잇 덕분에 자동 변환됨
        let vector = self.embedder.embed(prompt).await?;
        //                                            ^ ClientError → ServiceError 자동 변환

        let similar_memo_ids = self.qdrant_repo.search(user_id, vector).await?;
        let memos = self.memo_repo.find_by_ids(similar_memo_ids).await?;

        let result = self.text_generator.generate(prompt, memos).await?;
        Ok(result)
    }
}
```

**나쁜 예시**:
```rust
pub struct AssistService {
    gemini_client: GeminiClient,  // ❌ 구체 타입에 의존
}

impl AssistService {
    pub async fn assist(&self, prompt: &str) -> Result<String, ServiceError> {
        // Mock으로 교체 불가능
        let vector = self.gemini_client.embed(prompt).await
            .map_err(|e| ServiceError::GeminiApi(e.to_string()))?;  // ❌ 수동 변환
    }
}
```

**이유**: Trait을 사용하면 테스트 시 Mock으로 교체 가능하고, From 트레잇 덕분에 에러 변환이 자동으로 된다.

---

## 테스트 규칙

**설명**: Client는 연결 테스트만 작성하고, `#[ignore]` 플래그를 사용한다.

**좋은 예시**:
```rust
// src/clients/gemini/tests.rs
use super::*;

#[tokio::test]
#[ignore]  // 실제 API 호출이므로 기본적으로 실행 안 함
async fn test_real_gemini_embedding() {
    dotenv::dotenv().ok();
    let api_key = std::env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY not set");

    let client = GeminiClient::new(api_key);
    let result = client.embed("안녕하세요").await;

    assert!(result.is_ok(), "Embedding failed: {:?}", result.err());
    let vector = result.unwrap();
    assert_eq!(vector.len(), 768);

    println!("✅ Gemini Embedding API 연결 성공!");
}
```

**나쁜 예시**:
```rust
// 생성형 AI의 결과를 검증하는 단위 테스트
#[tokio::test]
async fn test_gemini_generation_content() {
    let result = client.generate("사랑", vec![]).await.unwrap();
    assert_eq!(result, "예상된 정확한 결과");  // ❌ 생성 결과는 매번 다름
}
```

**이유**:
- 외부 API는 연결만 확인
- 생성형 AI는 결과가 비결정적이므로 정확한 검증 불가능
- `#[ignore]`로 CI에서 실행 방지 (API 키 필요, 비용 발생)

---

## mod.rs 구조

```rust
// src/clients/mod.rs
pub mod errors;
pub mod gemini;

pub use errors::ClientError;
```

```rust
// src/clients/gemini/mod.rs
mod client;
mod mock;
mod traits;

#[cfg(test)]
mod tests;

pub use client::GeminiClient;
pub use mock::MockGeminiClient;
pub use traits::{Embedder, TextGenerator};
```

---

## 금지 사항

### ❌ Client에 비즈니스 로직 작성 금지
```rust
// 나쁜 예
impl GeminiClient {
    pub async fn embed_and_validate(&self, text: &str, max_length: usize) -> Result<Vec<f32>> {
        // ❌ 비즈니스 로직 (검증)은 Service에서
        if text.len() > max_length {
            return Err(...);
        }
        self.embed(text).await
    }
}
```

### ❌ Client에서 DB 접근 금지
```rust
// 나쁜 예
impl GeminiClient {
    pub async fn embed_and_save(&self, text: &str, db: &DatabaseConnection) -> Result<()> {
        // ❌ DB 작업은 Repository에서
        let vector = self.embed(text).await?;
        // save to db...
    }
}
```

### ❌ ServiceError 직접 반환 금지
```rust
// 나쁜 예
impl Embedder for GeminiClient {
    async fn embed(&self, text: &str) -> Result<Vec<f32>, ServiceError> {
        // ❌ ClientError를 사용해야 함
    }
}
```

---

## Client는 순수한 외부 호출 계층

Client는 **외부 API/서비스 호출**만 담당합니다.
- ✅ Trait 기반 추상화
- ✅ ClientError 반환
- ✅ Mock 구현 제공
- ❌ 비즈니스 로직 없음
- ❌ DB 접근 없음
- ❌ 다른 계층 의존 없음

**모든 로직은 Service 계층에서!**
