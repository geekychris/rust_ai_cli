use async_trait::async_trait;
use std::path::PathBuf;
use llm_base::Model;
use llm_llama::Llama;
use crate::error::{Result, Error};
use super::{Llm, LlmConfig, LlmResponse};

pub struct LlamaLlm {
    config: LlmConfig,
    model: Llama,
}

impl LlamaLlm {
    pub fn new(config: LlmConfig) -> Result<Self> {
        let model_path = PathBuf::from(&config.model_path);
        let model = Llama::load(
            &model_path, 
            llm_base::ModelParameters::default(),
            |_| {}
        ).map_err(|e| Error::Llm(format!("Failed to load model: {}", e)))?;
            
        Ok(Self { config, model })
    }
}

#[async_trait]
impl Llm for LlamaLlm {
    fn name(&self) -> &str {
        "llama"
    }
    
    async fn generate(&self, prompt: &str) -> Result<LlmResponse> {
        let mut output = String::new();
        let mut session = self.model.start_session(Default::default());
        
        let inference_parameters = llm_base::InferenceParameters {
            temperature: self.config.temperature,
            ..Default::default()
        };
        
        let inference_request = llm_base::InferenceRequest {
            prompt: prompt.into(),
            parameters: Some(&inference_parameters),
            play_back_previous_tokens: false,
            maximum_token_count: None
        };
        
        session.infer(
            &self.model,
            &mut rand::thread_rng(),
            &inference_request,
            &mut Default::default(),
            |t| {
                output.push_str(&t.to_string());
                Ok::<(), std::io::Error>(())
            },
        ).map_err(|e| Error::Llm(format!("Inference error: {}", e)))?;
            
        Ok(LlmResponse {
            text: output,
        })
    }
    
    fn config(&self) -> &LlmConfig {
        &self.config
    }
}
