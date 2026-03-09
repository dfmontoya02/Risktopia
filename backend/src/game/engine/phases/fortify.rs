use crate::game::model::{
    FortifyPhase, GameCore, GameOverState, GameState, PlayerId, ReinforcementPhase, TurnPhase,
};
use crate::game::protocol::{GameError, GameEvent, PlayerAction};
use crate::game::rules::validation::{remaining_players, validate_owned_territory};
use crate::game::rules::{calculate_reinforcements, end_turn};
use rand::seq::SliceRandom;

impl FortifyPhase {
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
            PlayerAction::Fortify { from, to, count } => {
                if self.used_fortify {
                    return Err(GameError::AlreadyFortified);
                }
                if count == 0 {
                    return Err(GameError::InvalidFortify);
                }
                validate_owned_territory(core, player, from)?;
                validate_owned_territory(core, player, to)?;
                if from == to {
                    return Err(GameError::InvalidFortify);
                }
                if !core.map.adjacency[from].contains(&to) {
                    return Err(GameError::NotAdjacent);
                }
                if core.territories[from].troops <= count {
                    return Err(GameError::NotEnoughTroops);
                }

                core.territories[from].troops -= count;
                core.territories[to].troops += count;
                self.used_fortify = true;

                Ok((
                    GameState::Fortify(self),
                    vec![GameEvent::Fortified { from, to, count }],
                ))
            }
            PlayerAction::EndTurn => {
                if self.conquered_this_turn {
                    award_territory_card(core, player);
                }
                let next_player = end_turn(core, player);
                if remaining_players(core) == 1 {
                    return Ok((
                        GameState::GameOver(GameOverState {
                            winner: next_player,
                        }),
                        vec![GameEvent::PhaseChanged {
                            current_player: next_player,
                            phase: TurnPhase::GameOver,
                        }],
                    ));
                }
                let troops_remaining = calculate_reinforcements(core, next_player);
                Ok((
                    GameState::Reinforcement(ReinforcementPhase {
                        player: next_player,
                        troops_remaining,
                    }),
                    vec![
                        GameEvent::TurnEnded { next_player },
                        GameEvent::PhaseChanged {
                            current_player: next_player,
                            phase: TurnPhase::Reinforcement,
                        },
                    ],
                ))
            }
            _ => Err(GameError::InvalidAction),
        }
    }
}

fn award_territory_card(core: &mut GameCore, player: PlayerId) {
    if core.deck.is_empty() && !core.discard.is_empty() {
        core.deck.append(&mut core.discard);
        core.deck.shuffle(&mut rand::rng());
    }
    let Some(card) = core.deck.pop() else {
        return;
    };
    if let Some(player_state) = core.players.iter_mut().find(|p| p.id == player) {
        player_state.cards.push(card);
    }
}

#[cfg(test)]
mod tests {
    use crate::game::card::{Card, CardType};
    use crate::game::model::{
        Continent, FortifyPhase, GameCore, GameMap, Player, PlayerId, TerritoryState,
    };
    use crate::game::protocol::{GameError, PlayerAction};

    #[test]
    fn fortify_rejects_non_adjacent_move() {
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
        core.map.adjacency[0].clear();
        let phase = FortifyPhase {
            player: PlayerId(0),
            used_fortify: false,
            conquered_this_turn: false,
        };

        let result = phase.apply(
            &mut core,
            PlayerId(0),
            PlayerAction::Fortify {
                from: 0,
                to: 1,
                count: 1,
            },
        );

        assert_eq!(result.unwrap_err(), GameError::NotAdjacent);
    }

    #[test]
    fn end_turn_awards_card_after_conquest() {
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
            deck: vec![Card {
                card_type: CardType::Infantry,
                territory: 0,
                id: 7,
            }],
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

        let phase = FortifyPhase {
            player: PlayerId(0),
            used_fortify: false,
            conquered_this_turn: true,
        };

        let (_state, _events) = phase
            .apply(&mut core, PlayerId(0), PlayerAction::EndTurn)
            .expect("end turn should succeed");

        assert_eq!(core.players[0].cards.len(), 1);
    }
}
