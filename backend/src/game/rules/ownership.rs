use crate::game::model::{GameCore, PlayerId};

pub fn adjust_ownership(core: &mut GameCore, attacker: PlayerId, defender: PlayerId) {
    let Some(attacker_idx) = core.players.iter().position(|p| p.id == attacker) else {
        return;
    };
    let Some(defender_idx) = core.players.iter().position(|p| p.id == defender) else {
        return;
    };

    core.players[attacker_idx].territories_owned += 1;

    core.players[defender_idx].territories_owned =
        core.players[defender_idx].territories_owned.saturating_sub(1);

    if core.players[defender_idx].territories_owned == 0 {
        core.players[defender_idx].eliminated = true;

        let transferred_cards = std::mem::take(&mut core.players[defender_idx].cards);
        core.players[attacker_idx].cards.extend(transferred_cards);
    }
}

#[cfg(test)]
mod tests {
    use crate::game::card::{Card, CardType};
    use crate::game::model::{GameCore, Player, PlayerId, TerritoryState, GameMap, Continent};

    use super::adjust_ownership;

    #[test]
    fn adjust_ownership_transfers_cards_on_elimination() {
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
                    cards: vec![
                        Card {
                            card_type: CardType::Infantry,
                            territory: 0,
                            id: 10,
                        },
                        Card {
                            card_type: CardType::Cavalry,
                            territory: 1,
                            id: 11,
                        },
                    ],
                    reserve_troops: 0,
                    territories_owned: 1,
                    eliminated: false,
                },
            ],
            territories: vec![
                TerritoryState {
                    owner: PlayerId(0),
                    troops: 3,
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

        adjust_ownership(&mut core, PlayerId(0), PlayerId(1));

        assert_eq!(core.players[0].territories_owned, 2);
        assert_eq!(core.players[1].territories_owned, 0);
        assert!(core.players[1].eliminated);
        assert_eq!(core.players[0].cards.len(), 2);
        assert!(core.players[1].cards.is_empty());
    }
}