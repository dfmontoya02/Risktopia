// Room Manager
// Responsible for:
// - matchmaking
// - creating rooms
// - tracking active rooms
use axum::extract::ws::Message;
use uuid::Uuid;

use super::commands::RoomCommand;
use super::types::{GameRoom, PlayerHandle};
use crate::game::PlayerId;
use crate::state::AppState;
use crate::transport::websocket::{GameStartPayload, QueueUpdatePayload, ServerEnvelope};

/// Add a player to the matchmaking queue. Broadcasts queue_update to all in queue.
/// If queue size reaches 4, starts a new room with those 4 players and removes them from the queue.
pub async fn add_player_to_queue(state: AppState, player: PlayerHandle) {
    let mut queue = state.queue.lock().await;
    let already_in_queue = queue.iter().any(|p| p.client_id == player.client_id);
    if already_in_queue {
        return;
    }
    queue.push(player);
    let waiting = queue.len() as u32;
    let update_json = serde_json::to_string(&ServerEnvelope {
        type_: "queue_update".to_string(),
        version: 1,
        payload: QueueUpdatePayload {
            players_waiting: waiting,
        },
    })
    .unwrap();
    for p in queue.iter() {
        p.sender.send(Message::Text(update_json.clone())).ok();
    }
    if queue.len() >= 4 {
        let players: Vec<PlayerHandle> = queue.drain(0..4).collect();
        let remaining = queue.len() as u32;
        let update_after = serde_json::to_string(&ServerEnvelope {
            type_: "queue_update".to_string(),
            version: 1,
            payload: QueueUpdatePayload {
                players_waiting: remaining,
            },
        })
        .unwrap();
        for p in queue.iter() {
            p.sender.send(Message::Text(update_after.clone())).ok();
        }
        drop(queue);
        tokio::spawn(async move { start_room(state, players).await });
    }
}

/// Create a new room with the given players, assign player indices, notify clients, and spawn the room loop.
pub async fn start_room(state: AppState, players: Vec<PlayerHandle>) {
    let room_id = Uuid::new_v4().to_string();
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<RoomCommand>();

    let players_with_id: Vec<PlayerHandle> = players
        .into_iter()
        .enumerate()
        .map(|(index, mut p)| {
            p.player_id = PlayerId(index as u8);
            p
        })
        .collect();

    for (index, player) in players_with_id.iter().enumerate() {
        let msg = ServerEnvelope {
            type_: "game_start".to_string(),
            version: 1,
            payload: GameStartPayload {
                game_id: room_id.clone(),
                player_index: index as u32,
            },
        };
        let json = serde_json::to_string(&msg).unwrap();
        player.sender.send(Message::Text(json)).ok();
    }

    state.rooms.lock().await.insert(
        room_id.clone(),
        GameRoom {
            players: players_with_id.clone(),
            tx: tx.clone(),
        },
    );

    tokio::spawn(crate::rooms::room_loop::run(
        state,
        room_id.clone(),
        players_with_id,
        rx,
    ));
    println!("Started room {}", room_id);
}

/// Remove a room from tracking. Called when the room is finished (e.g. all players disconnected).
pub async fn remove_room(state: AppState, room_id: &str) {
    state.rooms.lock().await.remove(room_id);
    println!("Removed room {}", room_id);
}
