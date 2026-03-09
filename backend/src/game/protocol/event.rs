use serde::Serialize;

use crate::game::model::{PlayerId, TerritoryId, TurnPhase};

#[derive(Clone, Debug, Serialize)]
pub enum GameEvent {
    TroopsPlaced {
        territory: TerritoryId,
        count: u32,
    },
    AttackResolved {
        from: TerritoryId,
        to: TerritoryId,
        attacker_rolls: Vec<u8>,
        defender_rolls: Vec<u8>,
        attacker_losses: u8,
        defender_losses: u8,
    },
    TerritoryCaptured {
        territory: TerritoryId,
        new_owner: PlayerId,
    },
    PlayerEliminated {
        player: PlayerId,
    },
    Fortified {
        from: TerritoryId,
        to: TerritoryId,
        count: u32,
    },
    TurnEnded {
        next_player: PlayerId,
    },
    PhaseChanged {
        current_player: PlayerId,
        phase: TurnPhase,
    },
}
