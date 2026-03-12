use crate::game::model::{AttackPhase, FortifyPhase, GameCore, GameState, PlayerId, TurnPhase};
use crate::game::protocol::{GameError, GameEvent, PlayerAction};
use crate::game::rules::resolve_attack;

impl AttackPhase {
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
            PlayerAction::Attack { from, to, dice } => {
                let resolution = resolve_attack(core, player, from, to, dice.into())?;                                               
                if resolution.captured {
                    self.captured_this_turn = true;
                }
                Ok((GameState::Attack(self), resolution.events))
            }
            PlayerAction::EndAttack => Ok((
                GameState::Fortify(FortifyPhase {
                    player,
                    used_fortify: false,
                    conquered_this_turn: self.captured_this_turn,
                }),
                vec![GameEvent::PhaseChanged {
                    current_player: player,
                    phase: TurnPhase::Fortify,
                }],
            )),
            _ => Err(GameError::InvalidAction),
        }
    }
}
