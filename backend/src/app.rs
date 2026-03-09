use crate::game::PlayerId;
use crate::rooms::service::{
    RefreshStateError, SubmitActionError, can_send_to_room, handle_disconnect, join_queue,
    leave_queue, request_state_refresh, submit_action,
};
use crate::rooms::types::PlayerHandle;
use crate::state::AppState;
use crate::transport::websocket::{
    ChatMessagePayload, ChatSendPayload, ClientEnvelope, ClientHelloPayload, ErrorCode,
    GameActionPayload, ProtocolErrorPayload, ServerEnvelope, StateRefreshPayload,
};
use axum::{Router, routing::get};
use axum::{
    extract::{
        State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
};
use futures::{SinkExt, StreamExt};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

pub fn build_router() -> Router {
    Router::new()
        .route("/", get(|| async { "Game server running" }))
        .route("/ws", get(ws_handler))
        .with_state(AppState::new())
}

pub async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let client_id = Uuid::new_v4().to_string();

    let (mut ws_sender, mut ws_receiver) = socket.split();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Message>();

    // Forward outgoing channel to websocket
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    // Send server_hello so frontend can set clientId from payload.client_id
    let hello = ServerEnvelope {
        type_: "server_hello".to_string(),
        version: 1,
        payload: ClientHelloPayload {
            client_id: client_id.clone(),
        },
    };
    let json = serde_json::to_string(&hello).unwrap();
    tx.send(Message::Text(json)).ok();

    let handle = PlayerHandle {
        client_id: client_id.clone(),
        player_id: PlayerId(0),
        sender: tx.clone(),
    };
    {
        let mut connected = state.connected.lock().await;
        connected.push(handle);
    }

    // Read loop (handle client messages)
    while let Some(Ok(msg)) = ws_receiver.next().await {
        if let Message::Close(_) = msg {
            break;
        }
        if let Message::Text(text) = msg {
            if let Ok(env) = serde_json::from_str::<ClientEnvelope>(&text) {
                if env.type_ == "queue_join" {
                    join_queue(
                        state.clone(),
                        PlayerHandle {
                            client_id: client_id.clone(),
                            player_id: PlayerId(0),
                            sender: tx.clone(),
                        },
                    )
                    .await;
                } else if env.type_ == "queue_leave" {
                    leave_queue(state.clone(), &client_id).await;
                } else if env.type_ == "game_action" {
                    handle_game_action_message(state.clone(), &client_id, env.payload, tx.clone())
                        .await;
                } else if env.type_ == "state_refresh" {
                    handle_state_refresh_message(
                        state.clone(),
                        &client_id,
                        env.payload,
                        tx.clone(),
                    )
                    .await;
                } else if env.type_ == "chat_send" {
                    if let Ok(payload) =
                        serde_json::from_value::<ChatSendPayload>(env.payload.clone())
                    {
                        let scope_ok = payload.scope == "lobby" || payload.scope == "game";
                        let game_ok = payload.scope != "game" || payload.game_id.is_some();
                        let msg_ok = !payload.message.trim().is_empty();
                        if scope_ok && game_ok && msg_ok {
                            let timestamp_ms = SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_millis() as u64;
                            let chat_msg = ServerEnvelope {
                                type_: "chat_message".to_string(),
                                version: 1,
                                payload: ChatMessagePayload {
                                    scope: payload.scope.clone(),
                                    game_id: payload.game_id.clone(),
                                    from: client_id.clone(),
                                    message: payload.message.trim().to_string(),
                                    timestamp_ms,
                                },
                            };
                            let chat_json = serde_json::to_string(&chat_msg).unwrap();
                            if payload.scope == "lobby" {
                                let connected = state.connected.lock().await;
                                for p in connected.iter() {
                                    p.sender.send(Message::Text(chat_json.clone())).ok();
                                }
                            } else if let Some(ref game_id) = payload.game_id {
                                if can_send_to_room(state.clone(), &client_id, game_id).await {
                                    let rooms = state.rooms.lock().await;
                                    if let Some(room) = rooms.get(game_id) {
                                        for p in room.players.iter() {
                                            p.sender.send(Message::Text(chat_json.clone())).ok();
                                        }
                                    }
                                } else {
                                    send_protocol_error(
                                        &tx,
                                        ErrorCode::UnauthorizedRoomAccess,
                                        "Client is not a member of this game room",
                                    );
                                }
                            }
                        }
                    }
                } else {
                    send_protocol_error(
                        &tx,
                        ErrorCode::UnknownMessageType,
                        &format!(
                            "Unhandled message type from client {}: {}",
                            client_id, env.type_
                        ),
                    );
                }
            } else {
                send_protocol_error(
                    &tx,
                    ErrorCode::InvalidEnvelope,
                    &format!(
                        "Unhandled message from client {} (invalid envelope)",
                        client_id
                    ),
                );
            }
        } else {
            send_protocol_error(
                &tx,
                ErrorCode::InvalidPayload,
                &format!("Unhandled non-text message from client {}", client_id),
            );
        }
    }

    // Disconnect: remove from connected and queue; notify room if this client was in a game
    handle_disconnect(state.clone(), &client_id).await;
    println!("Client {} disconnected", client_id);
}

async fn handle_game_action_message(
    state: AppState,
    client_id: &str,
    payload: serde_json::Value,
    sender: tokio::sync::mpsc::UnboundedSender<Message>,
) {
    if let Ok(action_payload) = serde_json::from_value::<GameActionPayload>(payload) {
        if let Err(err) = submit_action(
            state,
            client_id,
            &action_payload.game_id,
            action_payload.action,
        )
        .await
        {
            let (code, message) = match err {
                SubmitActionError::RoomNotFound => (
                    ErrorCode::RoomNotFound,
                    "Cannot submit action: room was not found",
                ),
                SubmitActionError::NotInRoom => (
                    ErrorCode::UnauthorizedRoomAccess,
                    "Cannot submit action: client is not in room",
                ),
                SubmitActionError::SendFailed => (
                    ErrorCode::GameActionRejected,
                    "Cannot submit action: room command channel unavailable",
                ),
            };
            send_protocol_error(&sender, code, message);
        }
    } else {
        send_protocol_error(
            &sender,
            ErrorCode::InvalidPayload,
            "game_action payload is invalid",
        );
    }
}

async fn handle_state_refresh_message(
    state: AppState,
    client_id: &str,
    payload: serde_json::Value,
    sender: tokio::sync::mpsc::UnboundedSender<Message>,
) {
    if let Ok(refresh_payload) = serde_json::from_value::<StateRefreshPayload>(payload) {
        if let Err(err) = request_state_refresh(state, client_id, &refresh_payload.game_id).await {
            let (code, message) = match err {
                RefreshStateError::RoomNotFound => (
                    ErrorCode::RoomNotFound,
                    "Cannot refresh state: room was not found",
                ),
                RefreshStateError::NotInRoom => (
                    ErrorCode::UnauthorizedRoomAccess,
                    "Cannot refresh state: client is not in room",
                ),
                RefreshStateError::SendFailed => (
                    ErrorCode::GameActionRejected,
                    "Cannot refresh state: room command channel unavailable",
                ),
            };
            send_protocol_error(&sender, code, message);
        }
    } else {
        send_protocol_error(
            &sender,
            ErrorCode::InvalidPayload,
            "state_refresh payload is invalid",
        );
    }
}

fn send_protocol_error(
    sender: &tokio::sync::mpsc::UnboundedSender<Message>,
    code: ErrorCode,
    message: &str,
) {
    if let Ok(json) = serde_json::to_string(&ServerEnvelope {
        type_: "protocol_error".to_string(),
        version: 1,
        payload: ProtocolErrorPayload {
            code,
            message: message.to_string(),
        },
    }) {
        sender.send(Message::Text(json)).ok();
    }
}

#[cfg(test)]
mod tests {
    use axum::extract::ws::Message;
    use serde_json::Value;
    use tokio::sync::mpsc::unbounded_channel;

    use crate::game::{PlayerAction, PlayerId};
    use crate::rooms::commands::RoomCommand;
    use crate::rooms::types::{GameRoom, PlayerHandle};
    use crate::state::AppState;

    use super::handle_game_action_message;

    #[tokio::test]
    async fn routes_game_action_to_room_command_channel() {
        let state = AppState::new();
        let (player_tx, mut _player_rx) = unbounded_channel::<Message>();
        let (room_tx, mut room_rx) = unbounded_channel::<RoomCommand>();

        let client_id = "client-1".to_string();
        let game_id = "game-1".to_string();
        let player = PlayerHandle {
            client_id: client_id.clone(),
            player_id: PlayerId(2),
            sender: player_tx,
        };

        state.rooms.lock().await.insert(
            game_id.clone(),
            GameRoom {
                players: vec![player],
                tx: room_tx,
            },
        );

        let payload = serde_json::json!({
            "game_id": game_id,
            "action": {
                "PlaceTroops": { "territory": 3, "count": 2 }
            }
        });
        let (tx, _rx) = unbounded_channel::<Message>();
        handle_game_action_message(state, &client_id, payload, tx).await;

        let cmd = room_rx.recv().await.expect("expected routed room command");
        match cmd {
            RoomCommand::PlayerAction { player_id, action } => {
                assert_eq!(player_id, PlayerId(2));
                assert!(matches!(
                    action,
                    PlayerAction::PlaceTroops {
                        territory: 3,
                        count: 2
                    }
                ));
            }
            RoomCommand::RefreshState { .. } => panic!("expected PlayerAction command"),
            RoomCommand::Disconnect { .. } => panic!("expected PlayerAction command"),
        }
    }

    #[tokio::test]
    async fn returns_protocol_error_on_invalid_game_action_payload() {
        let state = AppState::new();
        let (tx, mut rx) = unbounded_channel::<Message>();
        handle_game_action_message(state, "client-x", serde_json::json!({"bad":"payload"}), tx)
            .await;

        let Some(Message::Text(text)) = rx.recv().await else {
            panic!("expected protocol_error message");
        };
        let envelope: Value = serde_json::from_str(&text).expect("valid json");
        assert_eq!(envelope["type"], "protocol_error");
        assert_eq!(envelope["payload"]["code"], "INVALID_PAYLOAD");
    }

    #[tokio::test]
    async fn returns_protocol_error_when_room_not_found() {
        let state = AppState::new();
        let (tx, mut rx) = unbounded_channel::<Message>();
        let payload = serde_json::json!({
            "game_id":"missing-room",
            "action":{"EndTurn":null}
        });
        handle_game_action_message(state, "client-x", payload, tx).await;

        let Some(Message::Text(text)) = rx.recv().await else {
            panic!("expected protocol_error message");
        };
        let envelope: Value = serde_json::from_str(&text).expect("valid json");
        assert_eq!(envelope["type"], "protocol_error");
        assert_eq!(envelope["payload"]["code"], "ROOM_NOT_FOUND");
    }
}
