use serde::{Deserialize, Serialize};

// Envelope shape to interface with frontend
#[derive(Serialize)]
pub struct ServerEnvelope<T> {
    #[serde(rename = "type")]
    pub type_: String,
    pub version: u8,
    pub payload: T,
}

// Incoming client message envelope
#[derive(Debug, Deserialize)]
pub struct ClientEnvelope {
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(default)]
    pub payload: serde_json::Value,
}

#[derive(Serialize)]
pub struct ClientHelloPayload {
    pub client_id: String,
}

#[derive(Serialize)]
pub struct GameStartPayload {
    pub game_id: String,
    pub player_index: u32,
}

#[derive(Serialize)]
pub struct QueueUpdatePayload {
    pub players_waiting: u32,
}

// Server to client: single chat message
#[derive(Serialize)]
pub struct ChatMessagePayload {
    pub scope: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub game_id: Option<String>,
    pub from: String,
    pub message: String,
    pub timestamp_ms: u64,
}

// Client to server: chat_send payload
#[derive(Debug, Deserialize)]
pub struct ChatSendPayload {
    pub scope: String,
    pub game_id: Option<String>,
    pub message: String,
}
