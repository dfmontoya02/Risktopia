use crate::game::model::{
    AttackPhase, FortifyPhase, GameCore, GameState, PlayerId, TurnPhase,
};
use crate::game::model::phase_state::PendingCaptureMove;
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

        // If a capture move is pending, only allow completing that move.
        if let Some(pending) = self.pending_capture.clone() {
            return match action {
                PlayerAction::MoveCapturedTroops { count } => {
                    if count < pending.min_troops {
                        return Err(GameError::InvalidAction);
                    }

                    let available_to_move = core.territories[pending.from].troops.saturating_sub(1);
                    if count > available_to_move {
                        return Err(GameError::NotEnoughTroops);
                    }

                    core.territories[pending.from].troops -= count;
                    core.territories[pending.to].troops += count;

                    self.pending_capture = None;

                    Ok((
                        GameState::Attack(self),
                        vec![GameEvent::TroopsMovedAfterCapture {
                            from: pending.from,
                            to: pending.to,
                            count,
                        }],
                    ))
                }
                _ => Err(GameError::InvalidAction),
            };
        }

        match action {
            PlayerAction::Attack { from, to, dice } => {
                let resolution = resolve_attack(core, player, from, to, dice.into())?;

                if resolution.captured {
                    self.captured_this_turn = true;
                }

                if let Some(pending) = resolution.pending_capture {
                    self.pending_capture = Some(PendingCaptureMove {
                        from: pending.from,
                        to: pending.to,
                        min_troops: pending.min_troops,
                    });
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