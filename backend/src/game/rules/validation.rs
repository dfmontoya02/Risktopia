use crate::game::model::{GameCore, Player, PlayerId, TerritoryId};
use crate::game::protocol::GameError;

pub fn validate_owned_territory(
    core: &GameCore,
    player: PlayerId,
    territory: TerritoryId,
) -> Result<(), GameError> {
    if territory >= core.territories.len() {
        return Err(GameError::InvalidTerritory);
    }
    if core.territories[territory].owner != player {
        return Err(GameError::InvalidOwner);
    }
    Ok(())
}

pub fn get_player(core: &GameCore, player: PlayerId) -> Result<&Player, GameError> {
    core.players
        .iter()
        .find(|p| p.id == player)
        .ok_or(GameError::InvalidOwner)
}

pub fn get_player_mut(core: &mut GameCore, player: PlayerId) -> Result<&mut Player, GameError> {
    core.players
        .iter_mut()
        .find(|p| p.id == player)
        .ok_or(GameError::InvalidOwner)
}

pub fn remaining_players(core: &GameCore) -> usize {
    core.players
        .iter()
        .filter(|player| !player.eliminated && player.territories_owned > 0)
        .count()
}
