use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WebSocketMessage {
    Ping,
    Pong,
    Text { content: String },
    Join { room: String },
    Leave { room: String },
    Broadcast { room: String, content: String },
    Error { message: String },
}

#[derive(Debug, Clone)]
pub struct Connection {
    pub id: String,
    pub user_id: Option<String>,
    pub rooms: Vec<String>,
}
