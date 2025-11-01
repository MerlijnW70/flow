use anthropic_sdk::{Client, MessagesRequest, ContentBlock, Role};
use async_trait::async_trait;

use crate::utils::error::{AppError, AppResult};
use super::super::model::{ChatRequest, ChatResponse};

pub struct AnthropicProvider {
    client: Client,
    default_model: String,
}

impl AnthropicProvider {
    pub fn new(api_key: String, default_model: String) -> Self {
        let client = Client::new(api_key);

        Self {
            client,
            default_model,
        }
    }
}

#[async_trait]
impl super::AiProvider for AnthropicProvider {
    async fn chat(&self, request: &ChatRequest) -> AppResult<ChatResponse> {
        let model = request.model.as_ref()
            .unwrap_or(&self.default_model)
            .clone();

        let mut messages_request = MessagesRequest::new(
            model.clone(),
            vec![ContentBlock::Text {
                text: request.message.clone(),
            }],
        );

        // Set system prompt if provided
        if let Some(system_prompt) = &request.system_prompt {
            messages_request = messages_request.with_system(system_prompt.clone());
        }

        // Set temperature if provided
        if let Some(temp) = request.temperature {
            messages_request = messages_request.with_temperature(temp);
        }

        // Set max tokens if provided
        if let Some(max_tokens) = request.max_tokens {
            messages_request = messages_request.with_max_tokens(max_tokens as usize);
        } else {
            // Anthropic requires max_tokens, so set a default
            messages_request = messages_request.with_max_tokens(2048);
        }

        let response = self
            .client
            .messages(messages_request)
            .await
            .map_err(|e| AppError::ExternalService(format!("Anthropic API error: {}", e)))?;

        // Extract text from response
        let content = response
            .content
            .iter()
            .filter_map(|block| match block {
                ContentBlock::Text { text } => Some(text.clone()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("\n");

        if content.is_empty() {
            return Err(AppError::ExternalService("No response from Anthropic".to_string()));
        }

        // Calculate total tokens used
        let tokens_used = response.usage.map(|u| (u.input_tokens + u.output_tokens) as u32);

        Ok(ChatResponse {
            response: content,
            provider: "anthropic".to_string(),
            model,
            tokens_used,
        })
    }

    async fn generate_embedding(&self, _text: &str, _model: Option<String>) -> AppResult<Vec<f32>> {
        // Anthropic doesn't provide embedding API as of now
        Err(AppError::ExternalService(
            "Anthropic does not support embeddings".to_string(),
        ))
    }

    fn provider_name(&self) -> &str {
        "anthropic"
    }
}
