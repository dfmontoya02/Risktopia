use axum::extract::ws::Message;
use std::fmt;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use tokio::sync::mpsc;

pub struct PlayerHandle {
    pub id: String,
    pub sender: mpsc::UnboundedSender<Message>,
}
impl fmt::Debug for PlayerHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PlayerHandle")
            .field("id", &self.id)
            .field("sender", &"-")
            .finish()
    }
}

#[derive(Default)]
pub struct GameRoom {
    pub players: Vec<PlayerHandle>,
}

#[derive(Clone)]
pub struct AppState {
    // All connected WebSocket clients (for lobby chat broadcast)
    pub connected: Arc<Mutex<Vec<PlayerHandle>>>,
    pub queue: Arc<Mutex<Vec<PlayerHandle>>>,
    pub rooms: Arc<Mutex<HashMap<String, GameRoom>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            connected: Arc::new(Mutex::new(Vec::new())),
            queue: Arc::new(Mutex::new(Vec::new())),
            rooms: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}
