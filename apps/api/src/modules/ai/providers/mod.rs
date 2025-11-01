pub mod openai;
pub mod anthropic;
pub mod local;

use async_trait::async_trait;

use crate::utils::error::AppResult;
use super::model::{ChatRequest, ChatResponse};

#[async_trait]
pub trait AiProvider: Send + Sync {
    async fn chat(&self, request: &ChatRequest) -> AppResult<ChatResponse>;
    async fn generate_embedding(&self, text: &str, model: Option<String>) -> AppResult<Vec<f32>>;
    fn provider_name(&self) -> &str;
}
