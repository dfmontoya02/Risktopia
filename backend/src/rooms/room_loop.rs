// Main game execution loop. Handle commands from the app and broadcast updates to players.
use axum::extract::ws::Message;
use std::collections::HashSet;
use tokio::sync::mpsc::UnboundedReceiver;

use super::commands::RoomCommand;
use super::manager;
use super::types::PlayerHandle;
use crate::game::card::new_standard_deck;
use crate::game::model::DEFAULT_TERRITORY_COUNT;
use crate::game::rules::{calculate_reinforcements, end_turn};
use crate::game::{
    ActionContext, CombatRollView, GameCore, GameError, GameEvent, GameMap, GamePhase, GameState,
    GameView, Player, PlayerId, PlayerPublicView, ReinforcementPhase, SetupPhase, TerritoryState,
    TerritoryView, player_id_to_u8,
};
use crate::state::AppState;
use crate::transport::websocket::{
    ActionErrorPayload, ErrorCode, GameEventPayload, GameViewPayload, ProtocolErrorPayload,
    ServerEnvelope, StateSnapshot, StateUpdatePayload, TerritorySnapshot,
};

/// Run the room loop: receive commands, handle disconnects, remove room when all players leave.
pub async fn run(
    state: AppState,
    room_id: String,
    players: Vec<PlayerHandle>,
    mut rx: UnboundedReceiver<RoomCommand>,
) {
    let mut connected: HashSet<PlayerId> = players.iter().map(|p| p.player_id).collect();
    let (mut core, mut game_state) = initialize_game_runtime(&players);
    let mut latest_combat_roll: Option<CombatRollView> = None;
    let mut state_version: u64 = 1;
    broadcast_state_update(&players, &room_id, &core, &game_state, state_version);
    broadcast_game_view(
        &players,
        &room_id,
        &core,
        &game_state,
        state_version,
        latest_combat_roll.clone(),
    );

    while let Some(cmd) = rx.recv().await {
        match cmd {
            RoomCommand::Disconnect { player_id } => {
                connected.remove(&player_id);
                handle_player_disconnect(&mut core, player_id);
                if should_advance_turn_for_disconnect(&game_state, player_id) {
                    if connected.is_empty() {
                        manager::remove_room(state, &room_id).await;
                        return;
                    }
                    let next_player = end_turn(&core, player_id);
                    game_state = GameState::Reinforcement(ReinforcementPhase {
                        player: next_player,
                        troops_remaining: calculate_reinforcements(&core, next_player),
                    });
                    state_version += 1;
                    broadcast_events(
                        &players,
                        &room_id,
                        &[GameEvent::PhaseChanged {
                            current_player: next_player,
                            phase: game_state.turn_phase(),
                        }],
                    );
                    broadcast_state_update(&players, &room_id, &core, &game_state, state_version);
                    broadcast_game_view(
                        &players,
                        &room_id,
                        &core,
                        &game_state,
                        state_version,
                        latest_combat_roll.clone(),
                    );
                }
                if connected.is_empty() {
                    manager::remove_room(state, &room_id).await;
                    return;
                }
            }
            RoomCommand::PlayerAction { player_id, action } => {
                if connected.contains(&player_id) {
                    match game_state
                        .clone()
                        .apply_action(&mut core, player_id, action)
                    {
                        Ok((next_state, events)) => {
                            game_state = next_state;
                            if let Some(roll) = latest_combat_from_events(&events) {
                                latest_combat_roll = Some(roll);
                            }
                            state_version += 1;
                            broadcast_events(&players, &room_id, &events);
                            broadcast_state_update(
                                &players,
                                &room_id,
                                &core,
                                &game_state,
                                state_version,
                            );
                            broadcast_game_view(
                                &players,
                                &room_id,
                                &core,
                                &game_state,
                                state_version,
                                latest_combat_roll.clone(),
                            );
                            if matches!(game_state, GameState::GameOver(_)) {
                                manager::remove_room(state, &room_id).await;
                                return;
                            }
                        }
                        Err(err) => {
                            send_action_error(&players, player_id, &room_id, err);
                        }
                    }
                } else {
                    send_protocol_error(
                        &players,
                        player_id,
                        ErrorCode::UnauthorizedRoomAccess,
                        "Player is not connected to room",
                    );
                }
            }
            RoomCommand::RefreshState { player_id } => {
                let Some(player) = players.iter().find(|p| p.player_id == player_id) else {
                    continue;
                };
                send_state_update_to_player(player, &room_id, &core, &game_state, state_version);
                send_game_view_to_player(
                    player,
                    &room_id,
                    &core,
                    &game_state,
                    state_version,
                    latest_combat_roll.clone(),
                );
            }
        }
    }

    // Channel closed (e.g. all senders dropped) — clean up room
    manager::remove_room(state, &room_id).await;
}

fn initialize_game_runtime(players: &[PlayerHandle]) -> (GameCore, GameState) {
    let territory_count = DEFAULT_TERRITORY_COUNT;
    let ids: Vec<PlayerId> = players.iter().map(|p| p.player_id).collect();
    let mut territories = Vec::with_capacity(territory_count);
    let mut owned = vec![0u32; ids.len()];

    for territory in 0..territory_count {
        let owner_idx = territory % ids.len().max(1);
        if !ids.is_empty() {
            owned[owner_idx] += 1;
            territories.push(TerritoryState {
                owner: ids[owner_idx],
                troops: 1,
            });
        }
    }

    let game_players: Vec<Player> = ids
        .iter()
        .enumerate()
        .map(|(index, id)| Player {
            id: *id,
            cards: vec![],
            reserve_troops: 10,
            territories_owned: owned[index],
            eliminated: false,
        })
        .collect();

    let core = GameCore {
        players: game_players,
        territories,
        deck: new_standard_deck(&(0..territory_count as u32).collect::<Vec<u32>>()),
        discard: vec![],
        map: GameMap::default_static(),
        sets_turned_in: 0,
    };

    let first_player = ids.first().copied().unwrap_or(PlayerId(0));
    (
        core,
        GameState::Setup(SetupPhase {
            player: first_player,
        }),
    )
}

fn broadcast_events(players: &[PlayerHandle], room_id: &str, events: &[GameEvent]) {
    for event in events {
        let json = serde_json::to_string(&ServerEnvelope {
            type_: "game_event".to_string(),
            version: 1,
            payload: GameEventPayload {
                game_id: room_id.to_string(),
                event: event.clone(),
            },
        });
        if let Ok(payload) = json {
            for player in players {
                player.sender.send(Message::Text(payload.clone())).ok();
            }
        }
    }
}

fn broadcast_state_update(
    players: &[PlayerHandle],
    room_id: &str,
    core: &GameCore,
    state: &GameState,
    state_version: u64,
) {
    for player in players {
        send_state_update_to_player(player, room_id, core, state, state_version);
    }
}

fn send_state_update_to_player(
    player: &PlayerHandle,
    room_id: &str,
    core: &GameCore,
    state: &GameState,
    state_version: u64,
) {
    let current_player = match state {
        GameState::Setup(phase) => phase.player,
        GameState::Reinforcement(phase) => phase.player,
        GameState::Attack(phase) => phase.player,
        GameState::Fortify(phase) => phase.player,
        GameState::GameOver(phase) => phase.winner,
    };
    let game_phase = if matches!(state, GameState::GameOver(_)) {
        GamePhase::Finished
    } else {
        GamePhase::InProgress
    };
    let territories = core
        .territories
        .iter()
        .map(|territory| TerritorySnapshot {
            owner: territory.owner.0,
            troops: territory.troops,
        })
        .collect::<Vec<_>>();

    let json = serde_json::to_string(&ServerEnvelope {
        type_: "state_update".to_string(),
        version: 1,
        payload: StateUpdatePayload {
            state_version,
            state: StateSnapshot {
                game_id: room_id.to_string(),
                current_player: current_player.0,
                turn_phase: state.turn_phase(),
                game_phase,
                territories,
            },
        },
    });
    if let Ok(payload) = json {
        player.sender.send(Message::Text(payload)).ok();
    }
}

fn send_action_error(players: &[PlayerHandle], player_id: PlayerId, room_id: &str, err: GameError) {
    let Some(player) = players.iter().find(|p| p.player_id == player_id) else {
        return;
    };
    let json = serde_json::to_string(&ServerEnvelope {
        type_: "action_error".to_string(),
        version: 1,
        payload: ActionErrorPayload {
            game_id: room_id.to_string(),
            code: ErrorCode::GameActionRejected,
            message: format!("{err:?}"),
        },
    });
    if let Ok(payload) = json {
        player.sender.send(Message::Text(payload)).ok();
    }
}

fn broadcast_game_view(
    players: &[PlayerHandle],
    room_id: &str,
    core: &GameCore,
    state: &GameState,
    state_version: u64,
    latest_combat_roll: Option<CombatRollView>,
) {
    for player in players {
        send_game_view_to_player(
            player,
            room_id,
            core,
            state,
            state_version,
            latest_combat_roll.clone(),
        );
    }
}

fn send_game_view_to_player(
    player: &PlayerHandle,
    room_id: &str,
    core: &GameCore,
    state: &GameState,
    state_version: u64,
    latest_combat_roll: Option<CombatRollView>,
) {
    let view = build_game_view_for_player(core, state, player.player_id, latest_combat_roll);
    let json = serde_json::to_string(&ServerEnvelope {
        type_: "game_view".to_string(),
        version: 1,
        payload: GameViewPayload {
            game_id: room_id.to_string(),
            state_version,
            view,
        },
    });
    if let Ok(payload) = json {
        player.sender.send(Message::Text(payload)).ok();
    }
}

fn build_game_view_for_player(
    core: &GameCore,
    state: &GameState,
    viewer: PlayerId,
    latest_combat_roll: Option<CombatRollView>,
) -> GameView {
    let current_player = current_player_for_state(state).unwrap_or(viewer);
    let game_phase = if matches!(state, GameState::GameOver(_)) {
        GamePhase::Finished
    } else {
        GamePhase::InProgress
    };

    let territories = core
        .territories
        .iter()
        .enumerate()
        .map(|(id, territory)| TerritoryView {
            id,
            continent_id: continent_for_territory(&core.map, id),
            owner_player_id: player_id_to_u8(territory.owner),
            troops: territory.troops,
        })
        .collect();

    let players_public = core
        .players
        .iter()
        .map(|player| PlayerPublicView {
            player_id: player_id_to_u8(player.id),
            territories_owned: player.territories_owned,
            card_count: player.cards.len(),
            eliminated: player.eliminated,
        })
        .collect();

    let your_cards = core
        .players
        .iter()
        .find(|player| player.id == viewer)
        .map(|player| player.cards.iter().copied().map(Into::into).collect())
        .unwrap_or_default();

    let action_context = match state {
        GameState::Setup(_) => ActionContext::Setup {},
        GameState::Reinforcement(phase) => ActionContext::Reinforcement {
            troops_remaining: phase.troops_remaining,
        },
        GameState::Attack(phase) => ActionContext::Attack {
            captured_this_turn: phase.captured_this_turn,
        },
        GameState::Fortify(phase) => ActionContext::Fortify {
            used_fortify: phase.used_fortify,
        },
        GameState::GameOver(phase) => ActionContext::GameOver {
            winner_player_id: player_id_to_u8(phase.winner),
        },
    };

    GameView {
        you_player_id: player_id_to_u8(viewer),
        current_player_id: player_id_to_u8(current_player),
        turn_phase: state.turn_phase(),
        game_phase,
        territories,
        players_public,
        your_cards,
        latest_combat_roll,
        action_context,
    }
}

fn continent_for_territory(map: &GameMap, territory_id: usize) -> usize {
    map.continents
        .iter()
        .position(|continent| continent.territories.contains(&territory_id))
        .unwrap_or(0)
}

fn latest_combat_from_events(events: &[GameEvent]) -> Option<CombatRollView> {
    events.iter().rev().find_map(|event| match event {
        GameEvent::AttackResolved {
            from,
            to,
            attacker_rolls,
            defender_rolls,
            attacker_losses,
            defender_losses,
        } => Some(CombatRollView {
            from: *from,
            to: *to,
            attacker_rolls: attacker_rolls.clone(),
            defender_rolls: defender_rolls.clone(),
            attacker_losses: *attacker_losses,
            defender_losses: *defender_losses,
        }),
        _ => None,
    })
}

fn send_protocol_error(
    players: &[PlayerHandle],
    player_id: PlayerId,
    code: ErrorCode,
    message: &str,
) {
    let Some(player) = players.iter().find(|p| p.player_id == player_id) else {
        return;
    };
    let json = serde_json::to_string(&ServerEnvelope {
        type_: "protocol_error".to_string(),
        version: 1,
        payload: ProtocolErrorPayload {
            code,
            message: message.to_string(),
        },
    });
    if let Ok(payload) = json {
        player.sender.send(Message::Text(payload)).ok();
    }
}

fn should_advance_turn_for_disconnect(state: &GameState, player_id: PlayerId) -> bool {
    current_player_for_state(state) == Some(player_id)
}

fn current_player_for_state(state: &GameState) -> Option<PlayerId> {
    match state {
        GameState::Setup(phase) => Some(phase.player),
        GameState::Reinforcement(phase) => Some(phase.player),
        GameState::Attack(phase) => Some(phase.player),
        GameState::Fortify(phase) => Some(phase.player),
        GameState::GameOver(_) => None,
    }
}

fn handle_player_disconnect(core: &mut GameCore, player_id: PlayerId) {
    for territory in &mut core.territories {
        if territory.owner == player_id {
            territory.troops = 1;
        }
    }
    if let Some(player) = core
        .players
        .iter_mut()
        .find(|player| player.id == player_id)
    {
        player.eliminated = true;
        player.territories_owned = 0;
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use axum::extract::ws::Message;
    use serde_json::Value;
    use tokio::sync::mpsc::unbounded_channel;
    use tokio::time::timeout;

    use crate::game::card::{Card, CardType};
    use crate::game::{GameState, Player, PlayerId, TerritoryState};
    use crate::rooms::commands::RoomCommand;
    use crate::rooms::types::PlayerHandle;
    use crate::state::AppState;

    use super::{build_game_view_for_player, latest_combat_from_events, run};

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

    async fn recv_text(rx: &mut tokio::sync::mpsc::UnboundedReceiver<Message>) -> String {
        let msg = timeout(Duration::from_secs(2), rx.recv())
            .await
            .expect("timed out waiting for websocket message")
            .expect("channel closed unexpectedly");
        match msg {
            Message::Text(text) => text.to_string(),
            _ => panic!("expected text websocket message"),
        }
    }

    #[tokio::test]
    async fn emits_initial_state_update_on_room_start() {
        let state = AppState::new();
        let (p1, mut rx1) = make_player("c1", 0);
        let (p2, mut _rx2) = make_player("c2", 1);
        let (tx, rx) = unbounded_channel::<RoomCommand>();

        let room_task = tokio::spawn(run(
            state.clone(),
            "room-init".to_string(),
            vec![p1, p2],
            rx,
        ));

        let text = recv_text(&mut rx1).await;
        let envelope: Value = serde_json::from_str(&text).expect("valid json");
        assert_eq!(envelope["type"], "state_update");
        assert_eq!(envelope["payload"]["state_version"], 1);

        tx.send(RoomCommand::Disconnect {
            player_id: PlayerId(0),
        })
        .ok();
        tx.send(RoomCommand::Disconnect {
            player_id: PlayerId(1),
        })
        .ok();
        room_task.await.expect("room loop task should complete");
    }

    #[tokio::test]
    async fn disconnect_of_current_player_advances_turn() {
        let state = AppState::new();
        let (p1, mut rx1) = make_player("c1", 0);
        let (p2, mut _rx2) = make_player("c2", 1);
        let (tx, rx) = unbounded_channel::<RoomCommand>();

        let room_task = tokio::spawn(run(
            state.clone(),
            "room-advance".to_string(),
            vec![p1, p2],
            rx,
        ));

        // Initial state update.
        let _ = recv_text(&mut rx1).await;
        tx.send(RoomCommand::Disconnect {
            player_id: PlayerId(0),
        })
        .ok();

        // First event after disconnect might be game_event; scan until next state_update.
        let mut found_update = false;
        for _ in 0..3 {
            let text = recv_text(&mut rx1).await;
            let envelope: Value = serde_json::from_str(&text).expect("valid json");
            if envelope["type"] == "state_update" {
                found_update = true;
                assert_eq!(envelope["payload"]["state"]["current_player"], 1);
                break;
            }
        }
        assert!(found_update, "expected post-disconnect state_update");

        tx.send(RoomCommand::Disconnect {
            player_id: PlayerId(1),
        })
        .ok();
        room_task.await.expect("room loop task should complete");
    }

    #[tokio::test]
    async fn invalid_game_action_returns_structured_action_error() {
        let state = AppState::new();
        let (p1, mut rx1) = make_player("c1", 0);
        let (p2, mut _rx2) = make_player("c2", 1);
        let (tx, rx) = unbounded_channel::<RoomCommand>();

        let room_task = tokio::spawn(run(
            state.clone(),
            "room-error".to_string(),
            vec![p1, p2],
            rx,
        ));

        // Consume initial state update.
        let _ = recv_text(&mut rx1).await;
        tx.send(RoomCommand::PlayerAction {
            player_id: PlayerId(0),
            action: crate::game::PlayerAction::EndTurn,
        })
        .ok();

        let mut found_error = false;
        for _ in 0..2 {
            let text = recv_text(&mut rx1).await;
            let envelope: Value = serde_json::from_str(&text).expect("valid json");
            if envelope["type"] == "action_error" {
                found_error = true;
                assert_eq!(envelope["payload"]["code"], "GAME_ACTION_REJECTED");
                break;
            }
        }
        assert!(found_error, "expected action_error envelope");

        tx.send(RoomCommand::Disconnect {
            player_id: PlayerId(0),
        })
        .ok();
        tx.send(RoomCommand::Disconnect {
            player_id: PlayerId(1),
        })
        .ok();
        room_task.await.expect("room loop task should complete");
    }

    #[test]
    fn per_player_view_only_includes_private_cards_for_viewer() {
        let core = crate::game::GameCore {
            players: vec![
                Player {
                    id: PlayerId(0),
                    cards: vec![Card {
                        card_type: CardType::Infantry,
                        territory: 0,
                        id: 10,
                    }],
                    reserve_troops: 0,
                    territories_owned: 1,
                    eliminated: false,
                },
                Player {
                    id: PlayerId(1),
                    cards: vec![Card {
                        card_type: CardType::Cavalry,
                        territory: 1,
                        id: 11,
                    }],
                    reserve_troops: 0,
                    territories_owned: 1,
                    eliminated: false,
                },
            ],
            territories: vec![
                TerritoryState {
                    owner: PlayerId(0),
                    troops: 3,
                },
                TerritoryState {
                    owner: PlayerId(1),
                    troops: 2,
                },
            ],
            deck: vec![],
            discard: vec![],
            map: crate::game::GameMap {
                adjacency: vec![vec![1], vec![0]],
                continents: vec![crate::game::Continent {
                    territories: vec![0, 1],
                    bonus: 0,
                }],
            },
            sets_turned_in: 0,
        };
        let state = GameState::Attack(crate::game::AttackPhase {
            player: PlayerId(0),
            captured_this_turn: false,
        });

        let view_for_p0 = build_game_view_for_player(&core, &state, PlayerId(0), None);
        assert_eq!(view_for_p0.your_cards.len(), 1);
        assert_eq!(view_for_p0.your_cards[0].card_id, 10);
        assert_eq!(view_for_p0.players_public.len(), 2);
        assert_eq!(view_for_p0.players_public[1].card_count, 1);
    }

    #[test]
    fn extracts_latest_combat_roll_from_attack_event() {
        let events = vec![crate::game::GameEvent::AttackResolved {
            from: 1,
            to: 2,
            attacker_rolls: vec![6, 5],
            defender_rolls: vec![4],
            attacker_losses: 0,
            defender_losses: 1,
        }];
        let roll = latest_combat_from_events(&events).expect("expected combat roll");
        assert_eq!(roll.from, 1);
        assert_eq!(roll.to, 2);
        assert_eq!(roll.attacker_rolls, vec![6, 5]);
        assert_eq!(roll.defender_rolls, vec![4]);
        assert_eq!(roll.attacker_losses, 0);
        assert_eq!(roll.defender_losses, 1);
    }
}
