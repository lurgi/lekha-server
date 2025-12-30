pub mod errors;
pub mod gemini;

pub use errors::ClientError;
pub use gemini::{Embedder, GeminiClient, TextGenerator};
