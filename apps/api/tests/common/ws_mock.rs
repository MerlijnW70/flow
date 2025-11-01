// WebSocket Mock Infrastructure for Testing
// Provides mock WebSocket client and message handling

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// WebSocket message types
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsMessage {
    Ping,
    Pong,
    Text { content: String },
    Join { room: String, user_id: String },
    Leave { room: String, user_id: String },
    Broadcast { room: String, content: String, from_user: String },
    Error { message: String },
    Connected { connection_id: String },
    Disconnected { connection_id: String },
}

/// Mock WebSocket connection
#[derive(Clone, Debug)]
pub struct MockWsConnection {
    pub id: String,
    pub user_id: Option<String>,
    pub rooms: Vec<String>,
    pub is_connected: bool,
    pub messages_sent: Vec<WsMessage>,
    pub messages_received: Vec<WsMessage>,
}

impl MockWsConnection {
    pub fn new(user_id: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            user_id,
            rooms: Vec::new(),
            is_connected: true,
            messages_sent: Vec::new(),
            messages_received: Vec::new(),
        }
    }

    pub fn send(&mut self, message: WsMessage) {
        self.messages_sent.push(message);
    }

    pub fn receive(&mut self, message: WsMessage) {
        self.messages_received.push(message);
    }

    pub fn join_room(&mut self, room: &str) {
        if !self.rooms.contains(&room.to_string()) {
            self.rooms.push(room.to_string());
        }
    }

    pub fn leave_room(&mut self, room: &str) {
        self.rooms.retain(|r| r != room);
    }

    pub fn disconnect(&mut self) {
        self.is_connected = false;
    }

    pub fn is_in_room(&self, room: &str) -> bool {
        self.rooms.contains(&room.to_string())
    }
}

/// Mock WebSocket server for testing
#[derive(Clone)]
pub struct MockWsServer {
    connections: Arc<Mutex<HashMap<String, MockWsConnection>>>,
    rooms: Arc<Mutex<HashMap<String, Vec<String>>>>, // room -> connection_ids
    message_history: Arc<Mutex<Vec<(String, WsMessage)>>>, // (connection_id, message)
}

impl MockWsServer {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(Mutex::new(HashMap::new())),
            rooms: Arc::new(Mutex::new(HashMap::new())),
            message_history: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Connect a new client
    pub fn connect(&self, user_id: Option<String>) -> String {
        let connection = MockWsConnection::new(user_id);
        let connection_id = connection.id.clone();

        self.connections
            .lock()
            .unwrap()
            .insert(connection_id.clone(), connection);

        // Send connected message
        self.send_to_connection(
            &connection_id,
            WsMessage::Connected {
                connection_id: connection_id.clone(),
            },
        );

        connection_id
    }

    /// Disconnect a client
    pub fn disconnect(&self, connection_id: &str) -> Result<(), String> {
        let mut connections = self.connections.lock().unwrap();
        let connection = connections
            .get_mut(connection_id)
            .ok_or("Connection not found")?;

        // Leave all rooms
        let rooms_to_leave = connection.rooms.clone();
        drop(connections);

        for room in rooms_to_leave {
            self.leave_room(connection_id, &room).ok();
        }

        // Mark as disconnected
        let mut connections = self.connections.lock().unwrap();
        if let Some(conn) = connections.get_mut(connection_id) {
            conn.disconnect();
        }

        Ok(())
    }

    /// Send message from connection
    pub fn send_message(&self, connection_id: &str, message: WsMessage) -> Result<(), String> {
        // Record in history
        self.message_history
            .lock()
            .unwrap()
            .push((connection_id.to_string(), message.clone()));

        // Update connection
        let mut connections = self.connections.lock().unwrap();
        let connection = connections
            .get_mut(connection_id)
            .ok_or("Connection not found")?;

        if !connection.is_connected {
            return Err("Connection is closed".to_string());
        }

        connection.send(message.clone());

        // Handle special messages
        match message {
            WsMessage::Join { ref room, .. } => {
                drop(connections);
                self.join_room(connection_id, room)?;
            }
            WsMessage::Leave { ref room, .. } => {
                drop(connections);
                self.leave_room(connection_id, room)?;
            }
            WsMessage::Broadcast {
                ref room,
                ref content,
                ref from_user,
            } => {
                drop(connections);
                self.broadcast_to_room(room, content, from_user)?;
            }
            WsMessage::Ping => {
                drop(connections);
                self.send_to_connection(connection_id, WsMessage::Pong);
            }
            _ => {}
        }

        Ok(())
    }

    /// Send message to specific connection
    pub fn send_to_connection(&self, connection_id: &str, message: WsMessage) {
        if let Some(connection) = self.connections.lock().unwrap().get_mut(connection_id) {
            connection.receive(message);
        }
    }

    /// Join a room
    pub fn join_room(&self, connection_id: &str, room: &str) -> Result<(), String> {
        // Add connection to room
        self.rooms
            .lock()
            .unwrap()
            .entry(room.to_string())
            .or_insert_with(Vec::new)
            .push(connection_id.to_string());

        // Update connection
        let mut connections = self.connections.lock().unwrap();
        let connection = connections
            .get_mut(connection_id)
            .ok_or("Connection not found")?;

        connection.join_room(room);
        Ok(())
    }

    /// Leave a room
    pub fn leave_room(&self, connection_id: &str, room: &str) -> Result<(), String> {
        // Remove from room
        if let Some(room_members) = self.rooms.lock().unwrap().get_mut(room) {
            room_members.retain(|id| id != connection_id);
        }

        // Update connection
        let mut connections = self.connections.lock().unwrap();
        let connection = connections
            .get_mut(connection_id)
            .ok_or("Connection not found")?;

        connection.leave_room(room);
        Ok(())
    }

    /// Broadcast message to room
    pub fn broadcast_to_room(
        &self,
        room: &str,
        content: &str,
        from_user: &str,
    ) -> Result<(), String> {
        let room_members = self
            .rooms
            .lock()
            .unwrap()
            .get(room)
            .cloned()
            .unwrap_or_default();

        for connection_id in room_members {
            self.send_to_connection(
                &connection_id,
                WsMessage::Broadcast {
                    room: room.to_string(),
                    content: content.to_string(),
                    from_user: from_user.to_string(),
                },
            );
        }

        Ok(())
    }

    /// Get connection
    pub fn get_connection(&self, connection_id: &str) -> Option<MockWsConnection> {
        self.connections
            .lock()
            .unwrap()
            .get(connection_id)
            .cloned()
    }

    /// Get room members
    pub fn get_room_members(&self, room: &str) -> Vec<String> {
        self.rooms
            .lock()
            .unwrap()
            .get(room)
            .cloned()
            .unwrap_or_default()
    }

    /// Get total connections
    pub fn connection_count(&self) -> usize {
        self.connections.lock().unwrap().len()
    }

    /// Get active connections (connected=true)
    pub fn active_connection_count(&self) -> usize {
        self.connections
            .lock()
            .unwrap()
            .values()
            .filter(|c| c.is_connected)
            .count()
    }

    /// Get message history
    pub fn message_history(&self) -> Vec<(String, WsMessage)> {
        self.message_history.lock().unwrap().clone()
    }

    /// Clear all connections and rooms
    pub fn clear(&self) {
        self.connections.lock().unwrap().clear();
        self.rooms.lock().unwrap().clear();
        self.message_history.lock().unwrap().clear();
    }
}

impl Default for MockWsServer {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper to create a JSON WebSocket message
pub fn create_ws_json_message(msg_type: &str, data: serde_json::Value) -> String {
    let mut obj = serde_json::Map::new();
    obj.insert("type".to_string(), serde_json::Value::String(msg_type.to_string()));

    for (key, value) in data.as_object().unwrap() {
        obj.insert(key.clone(), value.clone());
    }

    serde_json::to_string(&obj).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ws_connection_creation() {
        let conn = MockWsConnection::new(Some("user123".to_string()));
        assert!(conn.is_connected);
        assert_eq!(conn.user_id, Some("user123".to_string()));
        assert_eq!(conn.rooms.len(), 0);
    }

    #[test]
    fn test_ws_server_connect() {
        let server = MockWsServer::new();
        let conn_id = server.connect(Some("user1".to_string()));

        assert_eq!(server.connection_count(), 1);
        assert_eq!(server.active_connection_count(), 1);

        let conn = server.get_connection(&conn_id).unwrap();
        assert!(conn.is_connected);
    }

    #[test]
    fn test_ws_server_disconnect() {
        let server = MockWsServer::new();
        let conn_id = server.connect(Some("user1".to_string()));

        server.disconnect(&conn_id).unwrap();

        let conn = server.get_connection(&conn_id).unwrap();
        assert!(!conn.is_connected);
    }

    #[test]
    fn test_ws_join_and_leave_room() {
        let server = MockWsServer::new();
        let conn_id = server.connect(Some("user1".to_string()));

        server.join_room(&conn_id, "room1").unwrap();
        assert_eq!(server.get_room_members("room1").len(), 1);

        let conn = server.get_connection(&conn_id).unwrap();
        assert!(conn.is_in_room("room1"));

        server.leave_room(&conn_id, "room1").unwrap();
        assert_eq!(server.get_room_members("room1").len(), 0);
    }

    #[test]
    fn test_ws_broadcast_to_room() {
        let server = MockWsServer::new();

        let conn1 = server.connect(Some("user1".to_string()));
        let conn2 = server.connect(Some("user2".to_string()));
        let conn3 = server.connect(Some("user3".to_string()));

        server.join_room(&conn1, "room1").unwrap();
        server.join_room(&conn2, "room1").unwrap();
        // conn3 not in room1

        server.broadcast_to_room("room1", "Hello room!", "user1").unwrap();

        // Check messages received
        let c1 = server.get_connection(&conn1).unwrap();
        let c2 = server.get_connection(&conn2).unwrap();
        let c3 = server.get_connection(&conn3).unwrap();

        assert_eq!(c1.messages_received.len(), 2); // Connected + Broadcast
        assert_eq!(c2.messages_received.len(), 2);
        assert_eq!(c3.messages_received.len(), 1); // Only Connected
    }

    #[test]
    fn test_ws_ping_pong() {
        let server = MockWsServer::new();
        let conn_id = server.connect(None);

        server.send_message(&conn_id, WsMessage::Ping).unwrap();

        let conn = server.get_connection(&conn_id).unwrap();
        let last_received = conn.messages_received.last().unwrap();
        assert_eq!(*last_received, WsMessage::Pong);
    }

    #[test]
    fn test_ws_message_history() {
        let server = MockWsServer::new();
        let conn_id = server.connect(Some("user1".to_string()));

        server.send_message(&conn_id, WsMessage::Text {
            content: "Hello".to_string()
        }).unwrap();

        let history = server.message_history();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].0, conn_id);
    }
}
