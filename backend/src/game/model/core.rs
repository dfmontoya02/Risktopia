use crate::game::card::Card;

use super::board::{GameMap, TerritoryState};
use super::player::Player;

#[derive(Clone, Debug)]
pub struct GameCore {
    pub players: Vec<Player>,
    pub territories: Vec<TerritoryState>,
    pub deck: Vec<Card>,
    pub discard: Vec<Card>,
    pub map: GameMap,
    pub sets_turned_in: u32,
}
