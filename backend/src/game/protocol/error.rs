#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum GameError {
    GameFinished,
    NotYourTurn,
    InvalidAction,
    InvalidTerritory,
    NotEnoughTroops,
    InvalidOwner,
    NotAdjacent,
    InvalidDice,
    AlreadyFortified,
    InvalidFortify,
}
