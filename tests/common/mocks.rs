/// Mock external services for testing

#[cfg(feature = "ai")]
pub mod ai {
    use wiremock::{Mock, MockServer, ResponseTemplate};
    use wiremock::matchers::{method, path};

    /// Setup mock OpenAI server
    pub async fn setup_mock_openai() -> MockServer {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v1/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": "chatcmpl-123",
                "object": "chat.completion",
                "created": 1677652288,
                "model": "gpt-4",
                "choices": [{
                    "index": 0,
                    "message": {
                        "role": "assistant",
                        "content": "This is a mock response"
                    },
                    "finish_reason": "stop"
                }],
                "usage": {
                    "prompt_tokens": 10,
                    "completion_tokens": 10,
                    "total_tokens": 20
                }
            })))
            .mount(&mock_server)
            .await;

        mock_server
    }

    /// Setup mock Anthropic server
    pub async fn setup_mock_anthropic() -> MockServer {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v1/messages"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": "msg_123",
                "type": "message",
                "role": "assistant",
                "content": [{
                    "type": "text",
                    "text": "This is a mock response"
                }],
                "model": "claude-3-opus-20240229",
                "stop_reason": "end_turn",
                "usage": {
                    "input_tokens": 10,
                    "output_tokens": 10
                }
            })))
            .mount(&mock_server)
            .await;

        mock_server
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "ai")]
    use super::ai::*;

    #[tokio::test]
    #[cfg(feature = "ai")]
    async fn test_mock_openai_setup() {
        let server = setup_mock_openai().await;
        assert!(!server.address().to_string().is_empty());
    }

    #[tokio::test]
    #[cfg(feature = "ai")]
    async fn test_mock_anthropic_setup() {
        let server = setup_mock_anthropic().await;
        assert!(!server.address().to_string().is_empty());
    }
}
