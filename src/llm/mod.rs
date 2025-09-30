use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::error::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub model_path: String,
    pub context_size: usize,
    pub temperature: f32,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            model_path: String::new(),
            context_size: 2048,
            temperature: 0.7,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmResponse {
    pub text: String,
}

#[async_trait]
pub trait Llm: Send + Sync {
    fn name(&self) -> &str;
    async fn generate(&self, prompt: &str) -> Result<LlmResponse>;
    fn config(&self) -> &LlmConfig;
}

pub mod llama;
pub mod ollama;
pub use llama::LlamaLlm;
pub use ollama::OllamaLlm;
