use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use axum::extract::ws::Message;

use super::model::Connection;

pub type Tx = mpsc::UnboundedSender<Message>;
pub type ConnectionMap = Arc<RwLock<HashMap<String, (Connection, Tx)>>>;

#[derive(Clone)]
pub struct ConnectionManager {
    connections: ConnectionMap,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn add_connection(&self, connection: Connection, tx: Tx) {
        let mut connections = self.connections.write().await;
        connections.insert(connection.id.clone(), (connection, tx));
    }

    pub async fn remove_connection(&self, connection_id: &str) {
        let mut connections = self.connections.write().await;
        connections.remove(connection_id);
    }

    pub async fn get_connection(&self, connection_id: &str) -> Option<(Connection, Tx)> {
        let connections = self.connections.read().await;
        connections.get(connection_id).cloned()
    }

    pub async fn broadcast_to_room(&self, room: &str, message: Message) {
        let connections = self.connections.read().await;

        for (connection, tx) in connections.values() {
            if connection.rooms.contains(&room.to_string()) {
                let _ = tx.send(message.clone());
            }
        }
    }

    pub async fn send_to_user(&self, user_id: &str, message: Message) {
        let connections = self.connections.read().await;

        for (connection, tx) in connections.values() {
            if let Some(uid) = &connection.user_id {
                if uid == user_id {
                    let _ = tx.send(message.clone());
                }
            }
        }
    }

    pub async fn add_to_room(&self, connection_id: &str, room: String) {
        let mut connections = self.connections.write().await;

        if let Some((connection, tx)) = connections.get_mut(connection_id) {
            if !connection.rooms.contains(&room) {
                connection.rooms.push(room);
            }
        }
    }

    pub async fn remove_from_room(&self, connection_id: &str, room: &str) {
        let mut connections = self.connections.write().await;

        if let Some((connection, tx)) = connections.get_mut(connection_id) {
            connection.rooms.retain(|r| r != room);
        }
    }

    pub async fn connection_count(&self) -> usize {
        let connections = self.connections.read().await;
        connections.len()
    }

    pub async fn room_member_count(&self, room: &str) -> usize {
        let connections = self.connections.read().await;
        connections
            .values()
            .filter(|(conn, _)| conn.rooms.contains(&room.to_string()))
            .count()
    }
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}
