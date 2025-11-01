use axum::extract::ws::{Message, WebSocket};
use futures::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tracing::{error, info, warn};
use uuid::Uuid;

use super::connections::ConnectionManager;
use super::model::{Connection, WebSocketMessage};

pub async fn handle_socket(socket: WebSocket, manager: ConnectionManager, user_id: Option<String>) {
    let connection_id = Uuid::new_v4().to_string();
    info!("New WebSocket connection: {}", connection_id);

    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

    // Create connection
    let connection = Connection {
        id: connection_id.clone(),
        user_id: user_id.clone(),
        rooms: vec![],
    };

    // Register connection
    manager.add_connection(connection, tx).await;

    // Spawn task to handle outgoing messages
    let mut send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    // Handle incoming messages
    let manager_clone = manager.clone();
    let connection_id_clone = connection_id.clone();

    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Err(e) = process_message(msg, &manager_clone, &connection_id_clone).await {
                error!("Error processing message: {}", e);
            }
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = (&mut send_task) => {
            recv_task.abort();
        }
        _ = (&mut recv_task) => {
            send_task.abort();
        }
    }

    // Clean up connection
    manager.remove_connection(&connection_id).await;
    info!("WebSocket connection closed: {}", connection_id);
}

async fn process_message(
    msg: Message,
    manager: &ConnectionManager,
    connection_id: &str,
) -> Result<(), String> {
    match msg {
        Message::Text(text) => {
            // Parse JSON message
            let ws_message: WebSocketMessage = serde_json::from_str(&text)
                .map_err(|e| format!("Invalid message format: {}", e))?;

            handle_ws_message(ws_message, manager, connection_id).await?;
        }
        Message::Binary(_) => {
            warn!("Binary messages not supported");
        }
        Message::Ping(_) => {
            // Axum handles pong automatically
        }
        Message::Pong(_) => {
            // Received pong
        }
        Message::Close(_) => {
            info!("Received close message");
        }
    }

    Ok(())
}

async fn handle_ws_message(
    message: WebSocketMessage,
    manager: &ConnectionManager,
    connection_id: &str,
) -> Result<(), String> {
    match message {
        WebSocketMessage::Ping => {
            // Send pong
            if let Some((_, tx)) = manager.get_connection(connection_id).await {
                let pong = WebSocketMessage::Pong;
                let json = serde_json::to_string(&pong).unwrap();
                let _ = tx.send(Message::Text(json));
            }
        }
        WebSocketMessage::Text { content } => {
            info!("Received text: {}", content);
            // Echo back or handle as needed
            if let Some((_, tx)) = manager.get_connection(connection_id).await {
                let response = WebSocketMessage::Text {
                    content: format!("Echo: {}", content),
                };
                let json = serde_json::to_string(&response).unwrap();
                let _ = tx.send(Message::Text(json));
            }
        }
        WebSocketMessage::Join { room } => {
            manager.add_to_room(connection_id, room.clone()).await;
            info!("Connection {} joined room {}", connection_id, room);

            // Notify room
            let notification = WebSocketMessage::Text {
                content: format!("User joined room: {}", room),
            };
            let json = serde_json::to_string(&notification).unwrap();
            manager.broadcast_to_room(&room, Message::Text(json)).await;
        }
        WebSocketMessage::Leave { room } => {
            manager.remove_from_room(connection_id, &room).await;
            info!("Connection {} left room {}", connection_id, room);
        }
        WebSocketMessage::Broadcast { room, content } => {
            let broadcast_msg = WebSocketMessage::Text { content };
            let json = serde_json::to_string(&broadcast_msg).unwrap();
            manager.broadcast_to_room(&room, Message::Text(json)).await;
        }
        WebSocketMessage::Error { message } => {
            error!("Error message: {}", message);
        }
        _ => {}
    }

    Ok(())
}
