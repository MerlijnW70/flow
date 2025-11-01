use std::sync::Arc;

use crate::config::AiConfig;
use crate::utils::error::{AppError, AppResult};

use super::model::{AiProvider as AiProviderEnum, ChatRequest, ChatResponse, EmbeddingRequest, EmbeddingResponse};
use super::providers::{
    anthropic::AnthropicProvider,
    openai::OpenAIProvider,
    local::LocalProvider,
    AiProvider,
};

pub struct AiService {
    openai: Option<Arc<OpenAIProvider>>,
    anthropic: Option<Arc<AnthropicProvider>>,
    local: Option<Arc<LocalProvider>>,
}

impl AiService {
    pub fn new(config: AiConfig) -> Self {
        let openai = config.openai_api_key.map(|key| {
            Arc::new(OpenAIProvider::new(key, config.default_model.clone()))
        });

        let anthropic = config.anthropic_api_key.map(|key| {
            Arc::new(AnthropicProvider::new(
                key,
                "claude-3-5-sonnet-20241022".to_string(),
            ))
        });

        // For local provider, you would typically load from a model path
        let local = Some(Arc::new(LocalProvider::new(
            "./models/local-model.gguf".to_string(),
        )));

        Self {
            openai,
            anthropic,
            local,
        }
    }

    fn get_provider(&self, provider: &AiProviderEnum) -> AppResult<Arc<dyn AiProvider>> {
        match provider {
            AiProviderEnum::Openai => self
                .openai
                .clone()
                .ok_or_else(|| AppError::Configuration("OpenAI API key not configured".to_string()))
                .map(|p| p as Arc<dyn AiProvider>),
            AiProviderEnum::Anthropic => self
                .anthropic
                .clone()
                .ok_or_else(|| AppError::Configuration("Anthropic API key not configured".to_string()))
                .map(|p| p as Arc<dyn AiProvider>),
            AiProviderEnum::Local => self
                .local
                .clone()
                .ok_or_else(|| AppError::Configuration("Local model not configured".to_string()))
                .map(|p| p as Arc<dyn AiProvider>),
        }
    }

    pub async fn chat(&self, request: ChatRequest) -> AppResult<ChatResponse> {
        let provider = self.get_provider(&request.provider)?;
        provider.chat(&request).await
    }

    pub async fn generate_embedding(&self, request: EmbeddingRequest) -> AppResult<EmbeddingResponse> {
        // Default to OpenAI for embeddings
        let provider = self
            .openai
            .clone()
            .ok_or_else(|| AppError::Configuration("OpenAI API key required for embeddings".to_string()))?;

        let embedding = provider.generate_embedding(&request.text, request.model.clone()).await?;

        Ok(EmbeddingResponse {
            embedding: embedding.clone(),
            model: request.model.unwrap_or_else(|| "text-embedding-3-small".to_string()),
            dimensions: embedding.len(),
        })
    }
}
