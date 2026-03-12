use crate::game::model::{
    AttackPhase, GameCore, GameState, PlayerId, ReinforcementPhase, TurnPhase,
};
use crate::game::protocol::{GameError, GameEvent, PlayerAction};
use crate::game::rules::validation::validate_owned_territory;

impl ReinforcementPhase {
    pub fn apply(
        mut self,
        core: &mut GameCore,
        player: PlayerId,
        action: PlayerAction,
    ) -> Result<(GameState, Vec<GameEvent>), GameError> {
        if player != self.player {
            return Err(GameError::NotYourTurn);
        }

        match action {
            PlayerAction::PlaceTroops { territory, count } => {
                if count == 0 {
                    return Err(GameError::InvalidAction);
                }
                if count > self.troops_remaining {
                    return Err(GameError::NotEnoughTroops);
                }
                validate_owned_territory(core, player, territory)?;
                core.territories[territory].troops += count;
                self.troops_remaining -= count;

                let mut events = vec![GameEvent::TroopsPlaced { territory, count }];
                if self.troops_remaining == 0 {
                    events.push(GameEvent::PhaseChanged {
                        current_player: player,
                        phase: TurnPhase::Attack,
                    });
                    return Ok((
                        GameState::Attack(AttackPhase {
                            player,
                            captured_this_turn: false,
                            pending_capture: None,
                        }),
                        events,
                    ));
                }
                Ok((GameState::Reinforcement(self), events))
            }
            PlayerAction::PlayCards { .. } => Ok((GameState::Reinforcement(self), vec![])),
            _ => Err(GameError::InvalidAction),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::game::model::{
        Continent, GameCore, GameMap, GameState, Player, PlayerId, ReinforcementPhase,
        TerritoryState, TurnPhase,
    };
    use crate::game::protocol::{GameEvent, PlayerAction};

    #[test]
    fn reinforcement_transitions_to_attack() {
        let mut core = GameCore {
            players: vec![
                Player {
                    id: PlayerId(0),
                    cards: vec![],
                    reserve_troops: 0,
                    territories_owned: 2,
                    eliminated: false,
                },
                Player {
                    id: PlayerId(1),
                    cards: vec![],
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
                    owner: PlayerId(0),
                    troops: 1,
                },
                TerritoryState {
                    owner: PlayerId(1),
                    troops: 2,
                },
            ],
            deck: vec![],
            discard: vec![],
            map: GameMap {
                adjacency: vec![vec![1, 2], vec![0], vec![0]],
                continents: vec![Continent {
                    territories: vec![0, 1, 2],
                    bonus: 0,
                }],
            },
            sets_turned_in: 0,
        };
        let phase = ReinforcementPhase {
            player: PlayerId(0),
            troops_remaining: 3,
        };

        let (state, events) = phase
            .apply(
                &mut core,
                PlayerId(0),
                PlayerAction::PlaceTroops {
                    territory: 0,
                    count: 3,
                },
            )
            .unwrap();

        assert!(matches!(state, GameState::Attack(_)));
        assert!(events.iter().any(|event| matches!(
            event,
            GameEvent::PhaseChanged {
                phase: TurnPhase::Attack,
                ..
            }
        )));
    }
}
