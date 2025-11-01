use async_trait::async_trait;

use crate::utils::error::{AppError, AppResult};
use super::super::model::{ChatRequest, ChatResponse};

pub struct LocalProvider {
    model_path: String,
}

impl LocalProvider {
    pub fn new(model_path: String) -> Self {
        Self { model_path }
    }
}

#[async_trait]
impl super::AiProvider for LocalProvider {
    async fn chat(&self, request: &ChatRequest) -> AppResult<ChatResponse> {
        // Placeholder for local model inference
        // In production, you would use llama.cpp, Candle, or ONNX Runtime here
        // For now, we'll return a mock response

        tracing::warn!("Local AI provider is not fully implemented yet");

        Ok(ChatResponse {
            response: format!(
                "Local model response (mock): Received message: {}",
                request.message
            ),
            provider: "local".to_string(),
            model: request.model.clone()
                .unwrap_or_else(|| "local-model".to_string()),
            tokens_used: None,
        })
    }

    async fn generate_embedding(&self, text: &str, _model: Option<String>) -> AppResult<Vec<f32>> {
        // Placeholder for local embedding generation
        tracing::warn!("Local embedding generation is not fully implemented yet");

        // Return a mock embedding vector
        Ok(vec![0.1; 384]) // Mock 384-dimensional embedding
    }

    fn provider_name(&self) -> &str {
        "local"
    }
}

// TODO: Implement actual local model inference using one of:
// 1. llama-cpp-rs for llama.cpp bindings
// 2. candle-core for Rust-native ML
// 3. ort (ONNX Runtime) for ONNX models
//
// Example with llama.cpp (pseudo-code):
// ```
// use llama_cpp_rs::LlamaModel;
//
// let model = LlamaModel::load_from_file(&self.model_path)?;
// let response = model.generate(&request.message, params)?;
// ```
