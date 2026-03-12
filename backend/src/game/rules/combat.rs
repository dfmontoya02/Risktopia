use crate::game::model::{GameCore, PlayerId, TerritoryId};
use crate::game::protocol::{GameError, GameEvent};
use rand::Rng;

use super::ownership::adjust_ownership;

#[derive(Debug, Clone)]
pub struct PendingCaptureMoveResult {
    pub from: TerritoryId,
    pub to: TerritoryId,
    pub min_troops: u32,
}


#[derive(Debug, Default)]
pub struct AttackResolution {
    pub events: Vec<GameEvent>,
    pub captured: bool,
    pub roll: CombatRoll,
    pub pending_capture: Option<PendingCaptureMoveResult>,
}

#[derive(Debug, Default, Clone)]
pub struct CombatRoll {
    pub from: TerritoryId,
    pub to: TerritoryId,
    pub attacker_rolls: Vec<u8>,
    pub defender_rolls: Vec<u8>,
    pub attacker_losses: u8,
    pub defender_losses: u8,
}

pub fn resolve_attack(
    core: &mut GameCore,
    player: PlayerId,
    from: TerritoryId,
    to: TerritoryId,
    attacker_dice: u32,
) -> Result<AttackResolution, GameError> {
    if from >= core.territories.len() || to >= core.territories.len() {
        return Err(GameError::InvalidTerritory);
    }
    if from == to {
        return Err(GameError::InvalidAction);
    }
    println!(
        "resolve_attack debug: from={} to={} backend_adjacency={:?}",
        from,
        to,
        core.map.adjacency[from]
    );
    
    if !core.map.adjacency[from].contains(&to) {
        return Err(GameError::NotAdjacent);
    }    
    if core.territories[from].owner != player {
        return Err(GameError::InvalidOwner);
    }
    if core.territories[to].owner == player {
        return Err(GameError::InvalidAction);
    }
    if core.territories[from].troops <= 1 {
        return Err(GameError::NotEnoughTroops);
    }

    if !(1..=3).contains(&attacker_dice) {
        return Err(GameError::InvalidAction);
    }

    let max_attack_dice = 3_u32.min(core.territories[from].troops - 1);
    if attacker_dice > max_attack_dice {
        return Err(GameError::InvalidAction);
    }

    let defend_dice = core.territories[to].troops.min(2);

    let mut rng = rand::rng();
    let mut attacker_rolls: Vec<u8> =
        (0..attacker_dice).map(|_| rng.random_range(1..=6)).collect();
    let mut defender_rolls: Vec<u8> =
        (0..defend_dice).map(|_| rng.random_range(1..=6)).collect();

    attacker_rolls.sort_unstable_by(|a, b| b.cmp(a));
    defender_rolls.sort_unstable_by(|a, b| b.cmp(a));

    let mut attacker_losses = 0u8;
    let mut defender_losses = 0u8;

    for (attack_roll, defend_roll) in attacker_rolls.iter().zip(defender_rolls.iter()) {
        if attack_roll > defend_roll {
            defender_losses += 1;
        } else {
            attacker_losses += 1;
        }
    }

    core.territories[from].troops -= attacker_losses as u32;
    core.territories[to].troops = core.territories[to]
        .troops
        .saturating_sub(defender_losses as u32);

    let mut events = vec![GameEvent::AttackResolved {
        from,
        to,
        attacker_rolls: attacker_rolls.clone(),
        defender_rolls: defender_rolls.clone(),
        attacker_losses,
        defender_losses,
    }];

    let mut captured = false;
    let mut pending_capture = None;

    if core.territories[to].troops == 0 {
        captured = true;
        let defender_id = core.territories[to].owner;

        // Transfer ownership immediately, but do not move troops yet.
        core.territories[to].owner = player;
        core.territories[to].troops = 0;

        adjust_ownership(core, player, defender_id);

        events.push(GameEvent::TerritoryCaptured {
            territory: to,
            new_owner: player,
        });

        if let Some(defender) = core.players.iter().find(|p| p.id == defender_id)
            && defender.territories_owned == 0
        {
            events.push(GameEvent::PlayerEliminated {
                player: defender_id,
            });
        }

        pending_capture = Some(PendingCaptureMoveResult {
            from,
            to,
            min_troops: attacker_dice,
        });
    }

    Ok(AttackResolution {
        events,
        captured,
        roll: CombatRoll {
            from,
            to,
            attacker_rolls,
            defender_rolls,
            attacker_losses,
            defender_losses,
        },
        pending_capture,
    })
}

#[cfg(test)]
mod tests {
    use crate::game::model::{Continent, GameMap, Player, TerritoryState};

    use super::*;

    fn test_core() -> GameCore {
        GameCore {
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
        }
    }

    #[test]
    fn resolve_attack_emits_combat_roll_event() {
        let mut core = test_core();
        let resolution = resolve_attack(&mut core, PlayerId(0), 0, 1, 3).unwrap();        
        assert!(
            resolution
                .events
                .iter()
                .any(|event| matches!(event, GameEvent::AttackResolved { .. }))
        );
        assert_eq!(resolution.roll.attacker_rolls.len(), 3);
        assert_eq!(resolution.roll.defender_rolls.len(), 1);
        assert!(
            resolution
                .roll
                .attacker_rolls
                .iter()
                .all(|roll| (1..=6).contains(roll))
        );
        assert!(
            resolution
                .roll
                .defender_rolls
                .iter()
                .all(|roll| (1..=6).contains(roll))
        );
    }

    #[test]
    fn resolve_attack_rejects_too_many_dice() {
        let mut core = test_core();

        let result = resolve_attack(&mut core, PlayerId(0), 0, 1, 4);

        assert!(result.is_err());
    }

    #[test]
    fn resolve_attack_rejects_more_dice_than_available_troops_allow() {
        let mut core = test_core();
        core.territories[0].troops = 2; // can only attack with 1 die

        let result = resolve_attack(&mut core, PlayerId(0), 0, 1, 2);

        assert!(result.is_err());
    }
}
