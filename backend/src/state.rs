use crate::rooms::types::{GameRoom, PlayerHandle};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

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
