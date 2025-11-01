use axum::{
    extract::{ws::WebSocketUpgrade, Query, State},
    response::Response,
    routing::get,
    Router,
};
use serde::Deserialize;
use std::sync::Arc;

use super::connections::ConnectionManager;
use super::handler::handle_socket;

#[derive(Clone)]
struct WebSocketState {
    manager: Arc<ConnectionManager>,
}

#[derive(Deserialize)]
struct WebSocketQuery {
    user_id: Option<String>,
}

pub fn routes() -> Router {
    let manager = Arc::new(ConnectionManager::new());
    let state = WebSocketState { manager };

    Router::new()
        .route("/ws", get(websocket_handler))
        .with_state(state)
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<WebSocketState>,
    Query(query): Query<WebSocketQuery>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, (*state.manager).clone(), query.user_id))
}
