use super::ids::{PlayerId, TerritoryId};

#[derive(Clone, Debug)]
pub struct GameMap {
    pub adjacency: Vec<Vec<TerritoryId>>,
    pub continents: Vec<Continent>,
}

#[derive(Clone, Debug)]
pub struct Continent {
    pub territories: Vec<TerritoryId>,
    pub bonus: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct TerritoryState {
    pub owner: PlayerId,
    pub troops: u32,
}
