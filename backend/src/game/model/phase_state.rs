use serde::{Deserialize, Serialize};

use crate::game::protocol::{GameError, GameEvent, PlayerAction};

use super::core::GameCore;
use super::ids::PlayerId;

#[derive(Clone, Debug)]
pub enum GameState {
    Setup(SetupPhase),
    Reinforcement(ReinforcementPhase),
    Attack(AttackPhase),
    Fortify(FortifyPhase),
    GameOver(GameOverState),
}

#[derive(Clone, Debug)]
pub struct SetupPhase {
    pub player: PlayerId,
}

#[derive(Clone, Debug)]
pub struct ReinforcementPhase {
    pub player: PlayerId,
    pub troops_remaining: u32,
}

use crate::game::model::TerritoryId;

#[derive(Clone, Debug)]
pub struct PendingCaptureMove {
    pub from: TerritoryId,
    pub to: TerritoryId,
    pub min_troops: u32,
}

#[derive(Clone, Debug)]
pub struct AttackPhase {
    pub player: PlayerId,
    pub captured_this_turn: bool,
    pub pending_capture: Option<PendingCaptureMove>,
}

#[derive(Clone, Debug)]
pub struct FortifyPhase {
    pub player: PlayerId,
    pub used_fortify: bool,
    pub conquered_this_turn: bool,
}

#[derive(Clone, Debug)]
pub struct GameOverState {
    pub winner: PlayerId,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum TurnPhase {
    Setup,
    Reinforcement,
    Attack,
    Fortify,
    GameOver,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum GamePhase {
    InProgress,
    Finished,
}

impl GameState {
    pub fn apply_action(
        self,
        core: &mut GameCore,
        player: PlayerId,
        action: PlayerAction,
    ) -> Result<(Self, Vec<GameEvent>), GameError> {
        match self {
            GameState::Reinforcement(phase) => phase.apply(core, player, action),
            GameState::Attack(phase) => phase.apply(core, player, action),
            GameState::Fortify(phase) => phase.apply(core, player, action),
            GameState::Setup(phase) => phase.apply(core, player, action),
            GameState::GameOver(_) => Err(GameError::GameFinished),
        }
    }

    pub fn turn_phase(&self) -> TurnPhase {
        match self {
            GameState::Setup(_) => TurnPhase::Setup,
            GameState::Reinforcement(_) => TurnPhase::Reinforcement,
            GameState::Attack(_) => TurnPhase::Attack,
            GameState::Fortify(_) => TurnPhase::Fortify,
            GameState::GameOver(_) => TurnPhase::GameOver,
        }
    }
}
