use crate::game::model::{
    GameCore, GameState, PlayerId, ReinforcementPhase, SetupPhase, TurnPhase,
};
use crate::game::protocol::{GameError, GameEvent, PlayerAction};
use crate::game::rules::validation::{get_player, get_player_mut, validate_owned_territory};
use crate::game::rules::{calculate_reinforcements, end_turn};

impl SetupPhase {
    pub fn apply(
        self,
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
                validate_owned_territory(core, player, territory)?;
                let player_state = get_player_mut(core, player)?;
                if player_state.reserve_troops < count {
                    return Err(GameError::NotEnoughTroops);
                }
                player_state.reserve_troops -= count;
                core.territories[territory].troops += count;

                let mut events = vec![GameEvent::TroopsPlaced { territory, count }];
                if core.players.iter().all(|p| p.reserve_troops == 0) {
                    let next_player = player;
                    let troops_remaining = calculate_reinforcements(core, next_player);
                    events.push(GameEvent::PhaseChanged {
                        current_player: next_player,
                        phase: TurnPhase::Reinforcement,
                    });
                    return Ok((
                        GameState::Reinforcement(ReinforcementPhase {
                            player: next_player,
                            troops_remaining,
                        }),
                        events,
                    ));
                }

                if get_player(core, player)?.reserve_troops == 0 {
                    let next_player = end_turn(core, player);
                    events.push(GameEvent::PhaseChanged {
                        current_player: next_player,
                        phase: TurnPhase::Setup,
                    });
                    return Ok((
                        GameState::Setup(SetupPhase {
                            player: next_player,
                        }),
                        events,
                    ));
                }

                Ok((GameState::Setup(self), events))
            }
            _ => Err(GameError::InvalidAction),
        }
    }
}
