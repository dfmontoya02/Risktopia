use crate::game::model::{GameCore, PlayerId};

pub fn end_turn(core: &GameCore, player: PlayerId) -> PlayerId {
    if core.players.is_empty() {
        return player;
    }
    let current_index = core
        .players
        .iter()
        .position(|p| p.id == player)
        .unwrap_or(0);
    let len = core.players.len();
    for step in 1..=len {
        let idx = (current_index + step) % len;
        let candidate = &core.players[idx];
        if !candidate.eliminated && candidate.territories_owned > 0 {
            return candidate.id;
        }
    }
    player
}

#[cfg(test)]
mod tests {
    use crate::game::model::{Continent, GameMap, Player, TerritoryState};

    use super::*;

    #[test]
    fn end_turn_skips_eliminated_players() {
        let mut core = GameCore {
            players: vec![
                Player {
                    id: PlayerId(0),
                    cards: vec![],
                    reserve_troops: 0,
                    territories_owned: 1,
                    eliminated: false,
                },
                Player {
                    id: PlayerId(1),
                    cards: vec![],
                    reserve_troops: 0,
                    territories_owned: 1,
                    eliminated: false,
                },
            ],
            territories: vec![
                TerritoryState {
                    owner: PlayerId(0),
                    troops: 5,
                },
                TerritoryState {
                    owner: PlayerId(1),
                    troops: 1,
                },
            ],
            deck: vec![],
            discard: vec![],
            map: GameMap {
                adjacency: vec![vec![1], vec![0]],
                continents: vec![Continent {
                    territories: vec![0, 1],
                    bonus: 0,
                }],
            },
            sets_turned_in: 0,
        };

        core.players[1].eliminated = true;
        core.players[1].territories_owned = 0;
        assert_eq!(end_turn(&core, PlayerId(0)), PlayerId(0));
    }
}
