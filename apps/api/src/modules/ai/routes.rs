use axum::{
    routing::post,
    Router, Json, extract::State,
};
use std::sync::Arc;
use validator::Validate;

use crate::config::AiConfig;
use crate::utils::{
    error::AppResult,
    response::ApiResponse,
    validation::validate_struct,
};

use super::model::{ChatRequest, EmbeddingRequest};
use super::service::AiService;
use super::streaming::{chunk_response, create_sse_stream};

#[derive(Clone)]
struct AiState {
    service: Arc<AiService>,
}

pub fn routes(config: AiConfig) -> Router {
    let service = Arc::new(AiService::new(config));
    let state = AiState { service };

    Router::new()
        .route("/ai/chat", post(chat))
        .route("/ai/chat/stream", post(chat_stream))
        .route("/ai/embeddings", post(generate_embedding))
        .with_state(state)
}

async fn chat(
    State(state): State<AiState>,
    Json(request): Json<ChatRequest>,
) -> AppResult<impl axum::response::IntoResponse> {
    validate_struct(&request)?;

    let response = state.service.chat(request).await?;

    Ok(ApiResponse::success(response))
}

async fn chat_stream(
    State(state): State<AiState>,
    Json(mut request): Json<ChatRequest>,
) -> AppResult<impl axum::response::IntoResponse> {
    validate_struct(&request)?;

    // Force non-streaming for the actual API call
    request.stream = false;

    // Get the full response
    let response = state.service.chat(request).await?;

    // Chunk the response for streaming (in production, you'd stream from the provider)
    let chunks = chunk_response(response.response, 20);

    // Create SSE stream
    Ok(create_sse_stream(chunks))
}

async fn generate_embedding(
    State(state): State<AiState>,
    Json(request): Json<EmbeddingRequest>,
) -> AppResult<impl axum::response::IntoResponse> {
    validate_struct(&request)?;

    let response = state.service.generate_embedding(request).await?;

    Ok(ApiResponse::success(response))
}
