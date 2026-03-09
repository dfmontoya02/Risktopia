use axum::extract::ws::Message;
use std::fmt;
use tokio::sync::mpsc;

use crate::game::PlayerId;

use super::commands::RoomCommand;

#[derive(Clone)]
pub struct PlayerHandle {
    pub client_id: String,
    pub player_id: PlayerId,
    pub sender: mpsc::UnboundedSender<Message>,
}

impl fmt::Debug for PlayerHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PlayerHandle")
            .field("client_id", &self.client_id)
            .field("player_id", &self.player_id)
            .field("sender", &"-")
            .finish()
    }
}

pub struct GameRoom {
    pub players: Vec<PlayerHandle>,
    pub tx: mpsc::UnboundedSender<RoomCommand>,
}
