use axum::response::sse::{Event, KeepAlive, Sse};
use futures::stream::{self, Stream};
use std::{convert::Infallible, time::Duration};

use super::model::StreamChunk;

/// Create a Server-Sent Events (SSE) stream for AI responses
pub fn create_sse_stream(
    chunks: Vec<String>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = stream::iter(chunks)
        .enumerate()
        .map(|(i, chunk)| {
            let is_last = i == chunks.len() - 1;
            let stream_chunk = StreamChunk {
                content: chunk,
                done: is_last,
            };

            let json = serde_json::to_string(&stream_chunk).unwrap_or_default();

            Ok(Event::default().data(json))
        });

    Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(1)))
}

/// Simulate streaming response by chunking text
/// In production, you'd actually stream from the AI provider
pub fn chunk_response(text: String, chunk_size: usize) -> Vec<String> {
    text.chars()
        .collect::<Vec<_>>()
        .chunks(chunk_size)
        .map(|chunk| chunk.iter().collect::<String>())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_response() {
        let text = "Hello, world! This is a test.".to_string();
        let chunks = chunk_response(text, 10);

        assert!(!chunks.is_empty());
        assert_eq!(chunks.join(""), "Hello, world! This is a test.");
    }
}
