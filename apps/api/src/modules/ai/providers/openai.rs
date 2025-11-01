use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs,
        CreateEmbeddingRequestArgs,
    },
    Client,
};
use async_trait::async_trait;

use crate::utils::error::{AppError, AppResult};
use super::super::model::{ChatRequest, ChatResponse};

pub struct OpenAIProvider {
    client: Client<OpenAIConfig>,
    default_model: String,
}

impl OpenAIProvider {
    pub fn new(api_key: String, default_model: String) -> Self {
        let config = OpenAIConfig::new().with_api_key(api_key);
        let client = Client::with_config(config);

        Self {
            client,
            default_model,
        }
    }
}

#[async_trait]
impl super::AiProvider for OpenAIProvider {
    async fn chat(&self, request: &ChatRequest) -> AppResult<ChatResponse> {
        let model = request.model.as_ref()
            .unwrap_or(&self.default_model)
            .clone();

        let mut messages: Vec<ChatCompletionRequestMessage> = vec![];

        // Add system prompt if provided
        if let Some(system_prompt) = &request.system_prompt {
            messages.push(
                ChatCompletionRequestSystemMessageArgs::default()
                    .content(system_prompt)
                    .build()
                    .map_err(|e| AppError::ExternalService(e.to_string()))?
                    .into(),
            );
        }

        // Add user message
        messages.push(
            ChatCompletionRequestUserMessageArgs::default()
                .content(&request.message)
                .build()
                .map_err(|e| AppError::ExternalService(e.to_string()))?
                .into(),
        );

        let mut req_builder = CreateChatCompletionRequestArgs::default();
        req_builder.model(&model).messages(messages);

        if let Some(temp) = request.temperature {
            req_builder.temperature(temp);
        }

        if let Some(max_tokens) = request.max_tokens {
            req_builder.max_tokens(max_tokens);
        }

        let chat_request = req_builder
            .build()
            .map_err(|e| AppError::ExternalService(e.to_string()))?;

        let response = self
            .client
            .chat()
            .create(chat_request)
            .await
            .map_err(|e| AppError::ExternalService(format!("OpenAI API error: {}", e)))?;

        let content = response
            .choices
            .first()
            .and_then(|c| c.message.content.clone())
            .ok_or_else(|| AppError::ExternalService("No response from OpenAI".to_string()))?;

        let tokens_used = response.usage.map(|u| u.total_tokens);

        Ok(ChatResponse {
            response: content,
            provider: "openai".to_string(),
            model,
            tokens_used,
        })
    }

    async fn generate_embedding(&self, text: &str, model: Option<String>) -> AppResult<Vec<f32>> {
        let model = model.unwrap_or_else(|| "text-embedding-3-small".to_string());

        let request = CreateEmbeddingRequestArgs::default()
            .model(&model)
            .input(text)
            .build()
            .map_err(|e| AppError::ExternalService(e.to_string()))?;

        let response = self
            .client
            .embeddings()
            .create(request)
            .await
            .map_err(|e| AppError::ExternalService(format!("OpenAI API error: {}", e)))?;

        let embedding = response
            .data
            .first()
            .map(|d| d.embedding.clone())
            .ok_or_else(|| AppError::ExternalService("No embedding returned".to_string()))?;

        Ok(embedding)
    }

    fn provider_name(&self) -> &str {
        "openai"
    }
}
