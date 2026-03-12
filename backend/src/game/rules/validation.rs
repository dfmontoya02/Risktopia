use crate::game::model::{GameCore, Player, PlayerId, TerritoryId};
use crate::game::protocol::GameError;
use std::collections::VecDeque;


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

/*
Uses BFS/DFS:
    1. start at from
    2. only traverse adjacent territories owned by player
    3. if to is reachable, it’s valid
*/
pub fn validate_owned_path(
    core: &GameCore,
    player: PlayerId,
    from: usize,
    to: usize,
) -> Result<(), GameError> {
    if from == to {
        return Err(GameError::InvalidFortify);
    }

    validate_owned_territory(core, player, from)?;
    validate_owned_territory(core, player, to)?;

    let mut visited = vec![false; core.territories.len()];
    let mut queue = VecDeque::new();

    visited[from] = true;
    queue.push_back(from);

    while let Some(curr) = queue.pop_front() {
        if curr == to {
            return Ok(());
        }

        for &neighbor in &core.map.adjacency[curr] {
            if !visited[neighbor] && core.territories[neighbor].owner == player {
                visited[neighbor] = true;
                queue.push_back(neighbor);
            }
        }
    }

    Err(GameError::NotAdjacent)
}