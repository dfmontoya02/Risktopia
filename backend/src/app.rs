use crate::state::{AppState, GameRoom, PlayerHandle};
use crate::transport::websocket::{
    ChatMessagePayload, ChatSendPayload, ClientEnvelope, ClientHelloPayload, GameStartPayload,
    QueueUpdatePayload, ServerEnvelope,
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
        id: client_id.clone(),
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
                    let mut queue = state.queue.lock().await;
                    let already_in_queue = queue.iter().any(|p| p.id == client_id);
                    if !already_in_queue {
                        queue.push(PlayerHandle {
                            id: client_id.clone(),
                            sender: tx.clone(),
                        });
                        let waiting = queue.len() as u32;
                        let queue_update = ServerEnvelope {
                            type_: "queue_update".to_string(),
                            version: 1,
                            payload: QueueUpdatePayload {
                                players_waiting: waiting,
                            },
                        };
                        let update_json = serde_json::to_string(&queue_update).unwrap();
                        for p in queue.iter() {
                            p.sender.send(Message::Text(update_json.clone())).ok();
                        }
                        if queue.len() >= 4 {
                            let players: Vec<_> = queue.drain(0..4).collect();
                            let remaining = queue.len() as u32;
                            let update_json_after = serde_json::to_string(&ServerEnvelope {
                                type_: "queue_update".to_string(),
                                version: 1,
                                payload: QueueUpdatePayload {
                                    players_waiting: remaining,
                                },
                            })
                            .unwrap();
                            for p in queue.iter() {
                                p.sender.send(Message::Text(update_json_after.clone())).ok();
                            }
                            tokio::spawn(start_room(state.clone(), players));
                        }
                    }
                    println!("Queue: {:?}", queue);
                    // If already in queue, ignore
                } else if env.type_ == "queue_leave" {
                    let mut queue = state.queue.lock().await;
                    let prev_len = queue.len();
                    queue.retain(|p| p.id != client_id);
                    if queue.len() < prev_len {
                        let waiting = queue.len() as u32;
                        let queue_update = ServerEnvelope {
                            type_: "queue_update".to_string(),
                            version: 1,
                            payload: QueueUpdatePayload {
                                players_waiting: waiting,
                            },
                        };
                        let update_json = serde_json::to_string(&queue_update).unwrap();
                        for p in queue.iter() {
                            p.sender.send(Message::Text(update_json.clone())).ok();
                        }
                    }
                    println!("Queue: {:?}", queue);
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
                                let rooms = state.rooms.lock().await;
                                if let Some(room) = rooms.get(game_id) {
                                    for p in room.players.iter() {
                                        p.sender.send(Message::Text(chat_json.clone())).ok();
                                    }
                                }
                            }
                        }
                    }
                } else {
                    println!(
                        "Unhandled message type from client {}: {}",
                        client_id, env.type_
                    );
                }
            } else {
                println!(
                    "Unhandled message from client {} (invalid envelope): {}",
                    client_id, text
                );
            }
        } else {
            println!("Unhandled message from client {} (non-text)", client_id);
        }
    }

    // Disconnect: remove from connected and queue
    {
        let mut connected = state.connected.lock().await;
        connected.retain(|p| p.id != client_id);
    }
    {
        let mut queue = state.queue.lock().await;
        queue.retain(|p| p.id != client_id);
    }
    println!("Client {} disconnected", client_id);
}

async fn start_room(state: AppState, players: Vec<PlayerHandle>) {
    let room_id = Uuid::new_v4().to_string();

    for (index, player) in players.iter().enumerate() {
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

    state
        .rooms
        .lock()
        .await
        .insert(room_id.clone(), GameRoom { players });

    println!("Started room {} with 4 players", room_id);

    // TODO:
    // - Initialize game state
    // - Run game loop
    // - Broadcast updates
}
