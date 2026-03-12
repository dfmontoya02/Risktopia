use serde::{Deserialize, Serialize};

use crate::game::card::CardId;
use crate::game::model::TerritoryId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlayerAction {
    PlaceTroops {
        territory: TerritoryId,
        count: u32,
    },
    Attack {
        from: TerritoryId,
        to: TerritoryId,
        dice: u8,
    },
    MoveCapturedTroops {
        count: u32,
    },
    EndAttack,
    Fortify {
        from: TerritoryId,
        to: TerritoryId,
        count: u32,
    },
    PlayCards {
        cards: [CardId; 3],
    },
    EndTurn,
}