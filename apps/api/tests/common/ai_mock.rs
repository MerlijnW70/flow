// AI Provider Mock Infrastructure for Testing
// Provides mock OpenAI, Anthropic, and local model responses

use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Mock AI response
#[derive(Clone, Debug)]
pub struct MockAiResponse {
    pub content: String,
    pub model: String,
    pub tokens_used: u32,
    pub finish_reason: String,
}

/// Mock streaming chunk
#[derive(Clone, Debug)]
pub struct MockStreamChunk {
    pub delta: String,
    pub finish: bool,
}

/// Mock embedding response
#[derive(Clone, Debug)]
pub struct MockEmbedding {
    pub vector: Vec<f32>,
    pub dimensions: usize,
}

/// Mock AI provider
#[derive(Clone)]
pub struct MockAiProvider {
    responses: Arc<Mutex<HashMap<String, MockAiResponse>>>,
    stream_chunks: Arc<Mutex<Vec<MockStreamChunk>>>,
    embeddings: Arc<Mutex<HashMap<String, MockEmbedding>>>,
    request_count: Arc<Mutex<u32>>,
    error_mode: Arc<Mutex<Option<String>>>,
}

impl MockAiProvider {
    /// Create a new mock AI provider
    pub fn new() -> Self {
        Self {
            responses: Arc::new(Mutex::new(HashMap::new())),
            stream_chunks: Arc::new(Mutex::new(Vec::new())),
            embeddings: Arc::new(Mutex::new(HashMap::new())),
            request_count: Arc::new(Mutex::new(0)),
            error_mode: Arc::new(Mutex::new(None)),
        }
    }

    /// Set a mock response for a prompt
    pub fn set_response(&self, prompt: &str, response: MockAiResponse) {
        self.responses
            .lock()
            .unwrap()
            .insert(prompt.to_string(), response);
    }

    /// Set mock streaming chunks
    pub fn set_stream_chunks(&self, chunks: Vec<MockStreamChunk>) {
        *self.stream_chunks.lock().unwrap() = chunks;
    }

    /// Set mock embedding for text
    pub fn set_embedding(&self, text: &str, embedding: MockEmbedding) {
        self.embeddings
            .lock()
            .unwrap()
            .insert(text.to_string(), embedding);
    }

    /// Enable error mode (simulate API failures)
    pub fn set_error_mode(&self, error: Option<String>) {
        *self.error_mode.lock().unwrap() = error;
    }

    /// Get chat completion
    pub fn chat(&self, prompt: &str, model: &str) -> Result<MockAiResponse, String> {
        self.increment_request_count();

        // Check error mode
        if let Some(error) = self.error_mode.lock().unwrap().as_ref() {
            return Err(error.clone());
        }

        // Get response or return default
        let responses = self.responses.lock().unwrap();
        Ok(responses
            .get(prompt)
            .cloned()
            .unwrap_or_else(|| MockAiResponse {
                content: format!("Mock response for: {}", prompt),
                model: model.to_string(),
                tokens_used: 50,
                finish_reason: "stop".to_string(),
            }))
    }

    /// Get streaming response
    pub fn stream(&self, _prompt: &str) -> Result<Vec<MockStreamChunk>, String> {
        self.increment_request_count();

        // Check error mode
        if let Some(error) = self.error_mode.lock().unwrap().as_ref() {
            return Err(error.clone());
        }

        Ok(self.stream_chunks.lock().unwrap().clone())
    }

    /// Generate embeddings
    pub fn embed(&self, text: &str) -> Result<MockEmbedding, String> {
        self.increment_request_count();

        // Check error mode
        if let Some(error) = self.error_mode.lock().unwrap().as_ref() {
            return Err(error.clone());
        }

        // Get embedding or return default
        let embeddings = self.embeddings.lock().unwrap();
        Ok(embeddings.get(text).cloned().unwrap_or_else(|| {
            MockEmbedding {
                vector: vec![0.1; 1536], // OpenAI ada-002 dimensions
                dimensions: 1536,
            }
        }))
    }

    /// Get request count
    pub fn request_count(&self) -> u32 {
        *self.request_count.lock().unwrap()
    }

    /// Reset request count
    pub fn reset_count(&self) {
        *self.request_count.lock().unwrap() = 0;
    }

    /// Clear all mocks
    pub fn clear(&self) {
        self.responses.lock().unwrap().clear();
        self.stream_chunks.lock().unwrap().clear();
        self.embeddings.lock().unwrap().clear();
        self.reset_count();
        *self.error_mode.lock().unwrap() = None;
    }

    fn increment_request_count(&self) {
        *self.request_count.lock().unwrap() += 1;
    }
}

impl Default for MockAiProvider {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a mock OpenAI chat response JSON
pub fn mock_openai_response(content: &str, model: &str) -> Value {
    json!({
        "id": "chatcmpl-123",
        "object": "chat.completion",
        "created": 1677652288,
        "model": model,
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": content
            },
            "finish_reason": "stop"
        }],
        "usage": {
            "prompt_tokens": 9,
            "completion_tokens": 12,
            "total_tokens": 21
        }
    })
}

/// Create a mock Anthropic response JSON
pub fn mock_anthropic_response(content: &str, model: &str) -> Value {
    json!({
        "id": "msg_123",
        "type": "message",
        "role": "assistant",
        "content": [{
            "type": "text",
            "text": content
        }],
        "model": model,
        "stop_reason": "end_turn",
        "usage": {
            "input_tokens": 10,
            "output_tokens": 15
        }
    })
}

/// Create mock streaming chunks
pub fn mock_stream_chunks(text: &str) -> Vec<MockStreamChunk> {
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut chunks = Vec::new();

    for (i, word) in words.iter().enumerate() {
        let delta = if i == 0 {
            word.to_string()
        } else {
            format!(" {}", word)
        };

        chunks.push(MockStreamChunk {
            delta,
            finish: false,
        });
    }

    // Add final chunk
    chunks.push(MockStreamChunk {
        delta: String::new(),
        finish: true,
    });

    chunks
}

/// Create a mock embedding vector
pub fn mock_embedding_vector(dimensions: usize) -> Vec<f32> {
    (0..dimensions)
        .map(|i| (i as f32 / dimensions as f32) * 0.1)
        .collect()
}

/// Create mock OpenAI embedding response
pub fn mock_openai_embedding_response(text: &str, dimensions: usize) -> Value {
    json!({
        "object": "list",
        "data": [{
            "object": "embedding",
            "embedding": mock_embedding_vector(dimensions),
            "index": 0
        }],
        "model": "text-embedding-ada-002",
        "usage": {
            "prompt_tokens": text.split_whitespace().count(),
            "total_tokens": text.split_whitespace().count()
        }
    })
}

/// Simulate rate limit error
pub fn mock_rate_limit_error() -> Value {
    json!({
        "error": {
            "message": "Rate limit exceeded. Please try again later.",
            "type": "rate_limit_error",
            "code": "rate_limit_exceeded"
        }
    })
}

/// Simulate invalid API key error
pub fn mock_invalid_key_error() -> Value {
    json!({
        "error": {
            "message": "Invalid API key provided",
            "type": "invalid_request_error",
            "code": "invalid_api_key"
        }
    })
}

/// Simulate content filter error
pub fn mock_content_filter_error() -> Value {
    json!({
        "error": {
            "message": "Your request was rejected due to content policy",
            "type": "content_policy_violation",
            "code": "content_filter"
        }
    })
}

/// Simulate model not found error
pub fn mock_model_not_found_error() -> Value {
    json!({
        "error": {
            "message": "The model 'gpt-99' does not exist",
            "type": "invalid_request_error",
            "code": "model_not_found"
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_provider_chat() {
        let provider = MockAiProvider::new();

        let response = MockAiResponse {
            content: "Hello, World!".to_string(),
            model: "gpt-4".to_string(),
            tokens_used: 10,
            finish_reason: "stop".to_string(),
        };

        provider.set_response("test prompt", response.clone());

        let result = provider.chat("test prompt", "gpt-4").unwrap();
        assert_eq!(result.content, "Hello, World!");
        assert_eq!(result.model, "gpt-4");
        assert_eq!(provider.request_count(), 1);
    }

    #[test]
    fn test_mock_provider_default_response() {
        let provider = MockAiProvider::new();
        let result = provider.chat("unknown prompt", "gpt-4").unwrap();

        assert!(result.content.contains("Mock response for"));
        assert_eq!(provider.request_count(), 1);
    }

    #[test]
    fn test_mock_provider_streaming() {
        let provider = MockAiProvider::new();

        let chunks = vec![
            MockStreamChunk { delta: "Hello".to_string(), finish: false },
            MockStreamChunk { delta: " World".to_string(), finish: false },
            MockStreamChunk { delta: String::new(), finish: true },
        ];

        provider.set_stream_chunks(chunks);

        let result = provider.stream("test").unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].delta, "Hello");
        assert!(result[2].finish);
    }

    #[test]
    fn test_mock_provider_embeddings() {
        let provider = MockAiProvider::new();

        let embedding = MockEmbedding {
            vector: vec![0.1, 0.2, 0.3],
            dimensions: 3,
        };

        provider.set_embedding("test text", embedding);

        let result = provider.embed("test text").unwrap();
        assert_eq!(result.dimensions, 3);
        assert_eq!(result.vector.len(), 3);
    }

    #[test]
    fn test_mock_provider_error_mode() {
        let provider = MockAiProvider::new();
        provider.set_error_mode(Some("API error".to_string()));

        let result = provider.chat("test", "gpt-4");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "API error");
    }

    #[test]
    fn test_mock_stream_chunks_generation() {
        let chunks = mock_stream_chunks("Hello World Test");

        assert_eq!(chunks.len(), 4); // 3 words + finish chunk
        assert_eq!(chunks[0].delta, "Hello");
        assert!(!chunks[0].finish);
        assert!(chunks[3].finish);
    }

    #[test]
    fn test_mock_embedding_vector() {
        let vector = mock_embedding_vector(1536);
        assert_eq!(vector.len(), 1536);
    }

    #[test]
    fn test_mock_openai_response_format() {
        let response = mock_openai_response("Test content", "gpt-4");

        assert_eq!(response["object"], "chat.completion");
        assert_eq!(response["choices"][0]["message"]["content"], "Test content");
        assert_eq!(response["model"], "gpt-4");
    }

    #[test]
    fn test_mock_anthropic_response_format() {
        let response = mock_anthropic_response("Test content", "claude-3-opus");

        assert_eq!(response["type"], "message");
        assert_eq!(response["content"][0]["text"], "Test content");
        assert_eq!(response["model"], "claude-3-opus");
    }

    #[test]
    fn test_mock_provider_clear() {
        let provider = MockAiProvider::new();

        provider.set_response("test", MockAiResponse {
            content: "test".to_string(),
            model: "gpt-4".to_string(),
            tokens_used: 10,
            finish_reason: "stop".to_string(),
        });

        provider.chat("test", "gpt-4").unwrap();
        assert_eq!(provider.request_count(), 1);

        provider.clear();
        assert_eq!(provider.request_count(), 0);
    }
}
