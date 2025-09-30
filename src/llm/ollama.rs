use std::sync::Arc;
use async_trait::async_trait;
use serde_json::json;
use reqwest::Client;
use crate::error::{Result, Error};
use crate::prompt::PromptTemplate;
use crate::tools::ToolRegistry;
use super::{Llm, LlmConfig, LlmResponse};

pub struct OllamaLlm {
    config: LlmConfig,
    client: Client,
    tools: Arc<ToolRegistry>,
    prompt_template: PromptTemplate,
}

impl OllamaLlm {
    pub fn new(config: LlmConfig, tools: Arc<ToolRegistry>) -> Result<Self> {
        let prompt_template = PromptTemplate::new(tools.clone());
        Ok(Self { 
            config,
            client: Client::new(),
            tools,
            prompt_template,
        })
    }
}

#[async_trait]
impl Llm for OllamaLlm {
    fn name(&self) -> &str {
        "ollama"
    }
    
    async fn generate(&self, user_prompt: &str) -> Result<LlmResponse> {
        let prompt = self.prompt_template.format(user_prompt)?;
        // Default to localhost:11434 if model_path doesn't specify a host
        let base_url = if self.config.model_path.contains("://") {
            self.config.model_path.clone()
        } else {
            format!("http://localhost:11434/api/generate")
        };

        let response = self.client
            .post(&base_url)
            .json(&json!({
                "model": self.config.model_path,
                "prompt": prompt,
                "stream": false,
                "temperature": self.config.temperature,
            }))
            .send()
            .await
            .map_err(|e| Error::Llm(format!("Ollama request failed: {}", e)))?;

        let json = response
            .json::<serde_json::Value>()
            .await
            .map_err(|e| Error::Llm(format!("Failed to parse Ollama response: {}", e)))?;

        let response_text = json["response"].as_str()
            .ok_or_else(|| Error::Llm("Missing 'response' field in Ollama response".into()))?
            .to_string();

        // Handle potential tool usage
        let mut current_response = response_text;
        let mut final_response = String::new();
        let mut iterations = 0;
        const MAX_ITERATIONS: i32 = 5;

        loop {
            if iterations >= MAX_ITERATIONS {
                break;
            }

            if let Some((result, needs_more)) = self.prompt_template.handle_response(&current_response).await? {
                if !final_response.is_empty() {
                    final_response.push_str("\n");
                }
                final_response.push_str(&result);

                if !needs_more {
                    break;
                }

                // Get follow-up from Ollama
                let follow_up = format!("{}\n\nTool output: {}\n\nContinue with your response:", prompt, result);
                let response = self.client
                    .post(&base_url)
                    .json(&json!({
                        "model": self.config.model_path,
                        "prompt": follow_up,
                        "stream": false,
                        "temperature": self.config.temperature,
                    }))
                    .send()
                    .await
                    .map_err(|e| Error::Llm(format!("Ollama request failed: {}", e)))?;

                let json = response
                    .json::<serde_json::Value>()
                    .await
                    .map_err(|e| Error::Llm(format!("Failed to parse Ollama response: {}", e)))?;

                current_response = json["response"].as_str()
                    .ok_or_else(|| Error::Llm("Missing 'response' field in Ollama response".into()))?
                    .to_string();
            } else {
                break;
            }

            iterations += 1;
        }

        Ok(LlmResponse {
            text: if !final_response.is_empty() { final_response } else { current_response },
        })
    }
    
    fn config(&self) -> &LlmConfig {
        &self.config
    }
}