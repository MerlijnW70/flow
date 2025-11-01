use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct ChatRequest {
    #[validate(length(min = 1, message = "Message cannot be empty"))]
    pub message: String,

    #[serde(default)]
    pub provider: AiProvider,

    #[serde(default)]
    pub model: Option<String>,

    #[serde(default)]
    pub temperature: Option<f32>,

    #[serde(default)]
    pub max_tokens: Option<u32>,

    #[serde(default)]
    pub stream: bool,

    #[serde(default)]
    pub system_prompt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum AiProvider {
    #[default]
    Openai,
    Anthropic,
    Local,
}

#[derive(Debug, Serialize)]
pub struct ChatResponse {
    pub response: String,
    pub provider: String,
    pub model: String,
    pub tokens_used: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct StreamChunk {
    pub content: String,
    pub done: bool,
}

#[derive(Debug, Deserialize, Validate)]
pub struct EmbeddingRequest {
    #[validate(length(min = 1))]
    pub text: String,

    #[serde(default)]
    pub model: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct EmbeddingResponse {
    pub embedding: Vec<f32>,
    pub model: String,
    pub dimensions: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
}
