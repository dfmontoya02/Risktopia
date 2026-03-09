use crate::game::PlayerAction;
use crate::state::AppState;
use crate::transport::websocket::{QueueUpdatePayload, ServerEnvelope};
use axum::extract::ws::Message;

use super::commands::RoomCommand;
use super::manager;
use super::types::PlayerHandle;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SubmitActionError {
    RoomNotFound,
    NotInRoom,
    SendFailed,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum RefreshStateError {
    RoomNotFound,
    NotInRoom,
    SendFailed,
}

pub async fn join_queue(state: AppState, player: PlayerHandle) {
    manager::add_player_to_queue(state, player).await;
}

pub async fn leave_queue(state: AppState, client_id: &str) {
    let mut queue = state.queue.lock().await;
    let prev_len = queue.len();
    queue.retain(|p| p.client_id != client_id);
    if queue.len() < prev_len {
        let waiting = queue.len() as u32;
        let queue_update = ServerEnvelope {
            type_: "queue_update".to_string(),
            version: 1,
            payload: QueueUpdatePayload {
                players_waiting: waiting,
            },
        };
        if let Ok(update_json) = serde_json::to_string(&queue_update) {
            for player in queue.iter() {
                player.sender.send(Message::Text(update_json.clone())).ok();
            }
        }
    }
}

pub async fn submit_action(
    state: AppState,
    client_id: &str,
    game_id: &str,
    action: PlayerAction,
) -> Result<(), SubmitActionError> {
    let (tx, player_id) = resolve_room_membership(state, client_id, game_id)
        .await
        .map_err(|err| match err {
            RefreshStateError::RoomNotFound => SubmitActionError::RoomNotFound,
            RefreshStateError::NotInRoom => SubmitActionError::NotInRoom,
            RefreshStateError::SendFailed => SubmitActionError::SendFailed,
        })?;

    tx.send(RoomCommand::PlayerAction { player_id, action })
        .map_err(|_| SubmitActionError::SendFailed)
}

pub async fn request_state_refresh(
    state: AppState,
    client_id: &str,
    game_id: &str,
) -> Result<(), RefreshStateError> {
    let (tx, player_id) = resolve_room_membership(state, client_id, game_id).await?;

    tx.send(RoomCommand::RefreshState { player_id })
        .map_err(|_| RefreshStateError::SendFailed)
}

pub async fn handle_disconnect(state: AppState, client_id: &str) {
    {
        let mut connected = state.connected.lock().await;
        connected.retain(|p| p.client_id != client_id);
    }
    leave_queue(state.clone(), client_id).await;

    let disconnect = {
        let rooms = state.rooms.lock().await;
        let mut out = None;
        for room in rooms.values() {
            if let Some(player) = room.players.iter().find(|p| p.client_id == client_id) {
                out = Some((room.tx.clone(), player.player_id));
                break;
            }
        }
        out
    };

    if let Some((tx, player_id)) = disconnect {
        tx.send(RoomCommand::Disconnect { player_id }).ok();
    }
}

pub async fn can_send_to_room(state: AppState, client_id: &str, game_id: &str) -> bool {
    let rooms = state.rooms.lock().await;
    let Some(room) = rooms.get(game_id) else {
        return false;
    };
    room.players
        .iter()
        .any(|player| player.client_id == client_id)
}

async fn resolve_room_membership(
    state: AppState,
    client_id: &str,
    game_id: &str,
) -> Result<
    (
        tokio::sync::mpsc::UnboundedSender<RoomCommand>,
        crate::game::PlayerId,
    ),
    RefreshStateError,
> {
    let rooms = state.rooms.lock().await;
    let Some(room) = rooms.get(game_id) else {
        return Err(RefreshStateError::RoomNotFound);
    };
    let Some(player) = room.players.iter().find(|p| p.client_id == client_id) else {
        return Err(RefreshStateError::NotInRoom);
    };
    Ok((room.tx.clone(), player.player_id))
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use axum::extract::ws::Message;
    use serde_json::Value;
    use tokio::sync::mpsc::unbounded_channel;
    use tokio::time::timeout;

    use crate::game::PlayerId;
    use crate::rooms::commands::RoomCommand;
    use crate::rooms::types::{GameRoom, PlayerHandle};
    use crate::state::AppState;

    use super::{handle_disconnect, submit_action};

    fn make_player(
        client_id: &str,
        player_id: u8,
    ) -> (PlayerHandle, tokio::sync::mpsc::UnboundedReceiver<Message>) {
        let (tx, rx) = unbounded_channel::<Message>();
        (
            PlayerHandle {
                client_id: client_id.to_string(),
                player_id: PlayerId(player_id),
                sender: tx,
            },
            rx,
        )
    }

    #[tokio::test]
    async fn disconnect_broadcasts_queue_update_to_remaining_queue() {
        let state = AppState::new();
        let (p1, _rx1) = make_player("leaving", 0);
        let (p2, mut rx2) = make_player("staying", 1);

        {
            let mut queue = state.queue.lock().await;
            queue.push(p1.clone());
            queue.push(p2.clone());
        }
        {
            let mut connected = state.connected.lock().await;
            connected.push(p1);
            connected.push(p2);
        }

        handle_disconnect(state, "leaving").await;

        let msg = timeout(Duration::from_secs(2), rx2.recv())
            .await
            .expect("timed out waiting for queue update")
            .expect("queue update channel closed unexpectedly");
        let Message::Text(text) = msg else {
            panic!("expected text message");
        };
        let envelope: Value = serde_json::from_str(&text).expect("valid json");
        assert_eq!(envelope["type"], "queue_update");
        assert_eq!(envelope["payload"]["players_waiting"], 1);
    }

    #[tokio::test]
    async fn submit_action_returns_not_in_room_error() {
        let state = AppState::new();
        let (player, _rx) = make_player("in-room", 0);
        let (room_tx, mut _room_rx) = unbounded_channel::<RoomCommand>();
        state.rooms.lock().await.insert(
            "room-a".to_string(),
            GameRoom {
                players: vec![player],
                tx: room_tx,
            },
        );

        let result = submit_action(
            state,
            "outsider",
            "room-a",
            crate::game::PlayerAction::EndTurn,
        )
        .await;
        assert!(matches!(result, Err(super::SubmitActionError::NotInRoom)));
    }
}
