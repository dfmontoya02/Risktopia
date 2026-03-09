pub mod board;
pub mod core;
pub mod ids;
pub mod phase_state;
pub mod player;

pub use board::{Continent, GameMap, TerritoryState};
pub use core::GameCore;
pub use ids::{PlayerId, TerritoryId};
pub use phase_state::{
    AttackPhase, FortifyPhase, GameOverState, GamePhase, GameState, ReinforcementPhase, SetupPhase,
    TurnPhase,
};
pub use player::Player;
