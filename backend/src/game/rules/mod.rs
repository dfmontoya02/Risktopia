pub mod combat;
pub mod ownership;
pub mod reinforcement;
pub mod turn_order;
pub mod validation;

pub use combat::resolve_attack;
pub use reinforcement::calculate_reinforcements;
pub use turn_order::end_turn;
