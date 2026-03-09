use crate::game::{GameEvent, GamePhase, GameView, PlayerAction, TurnPhase};
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

#[derive(Debug, Deserialize)]
pub struct GameActionPayload {
    pub game_id: String,
    pub action: PlayerAction,
}

#[derive(Debug, Deserialize)]
pub struct StateRefreshPayload {
    pub game_id: String,
}

#[derive(Serialize)]
pub struct TerritorySnapshot {
    pub owner: u8,
    pub troops: u32,
}

#[derive(Serialize)]
pub struct StateSnapshot {
    pub game_id: String,
    pub current_player: u8,
    pub turn_phase: TurnPhase,
    pub game_phase: GamePhase,
    pub territories: Vec<TerritorySnapshot>,
}

#[derive(Serialize)]
pub struct StateUpdatePayload {
    pub state_version: u64,
    pub state: StateSnapshot,
}

#[derive(Serialize)]
pub struct GameEventPayload {
    pub game_id: String,
    pub event: GameEvent,
}

#[derive(Serialize)]
pub struct GameViewPayload {
    pub game_id: String,
    pub state_version: u64,
    pub view: GameView,
}

#[derive(Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    InvalidEnvelope,
    UnknownMessageType,
    InvalidPayload,
    UnauthorizedRoomAccess,
    RoomNotFound,
    GameActionRejected,
}

#[derive(Serialize)]
pub struct ActionErrorPayload {
    pub game_id: String,
    pub code: ErrorCode,
    pub message: String,
}

#[derive(Serialize)]
pub struct ProtocolErrorPayload {
    pub code: ErrorCode,
    pub message: String,
}
