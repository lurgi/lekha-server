use super::traits::{Embedder, TextGenerator};
use crate::errors::ServiceError;

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

impl Default for MockGeminiClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl Embedder for MockGeminiClient {
    async fn embed(&self, text: &str) -> Result<Vec<f32>, ServiceError> {
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

#[async_trait::async_trait]
impl TextGenerator for MockGeminiClient {
    async fn generate(
        &self,
        prompt: &str,
        context: Vec<String>,
    ) -> Result<String, ServiceError> {
        let mut result = format!("AI 제안 (prompt: {})\n\n", prompt);

        if !context.is_empty() {
            result.push_str("참고한 메모:\n");
            for (i, memo) in context.iter().enumerate() {
                result.push_str(&format!("- 메모 {}: {}\n", i + 1, memo));
            }
        }

        result.push_str("\n생성된 글쓰기 제안입니다.");

        Ok(result)
    }
}
