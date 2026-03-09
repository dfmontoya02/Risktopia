use crate::game::card::Card;

use super::ids::PlayerId;

#[derive(Clone, Debug)]
pub struct Player {
    pub id: PlayerId,
    pub cards: Vec<Card>,
    pub reserve_troops: u32,
    pub territories_owned: u32,
    pub eliminated: bool,
}
