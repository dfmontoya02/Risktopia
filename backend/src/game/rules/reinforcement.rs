use crate::game::model::{GameCore, PlayerId};

pub fn calculate_reinforcements(core: &GameCore, player: PlayerId) -> u32 {
    let owned = core
        .territories
        .iter()
        .filter(|t| t.owner == player)
        .count() as u32;
    let base = (owned / 3).max(3);
    let continent_bonus = core
        .map
        .continents
        .iter()
        .filter(|continent| {
            continent
                .territories
                .iter()
                .all(|&territory| core.territories[territory].owner == player)
        })
        .map(|continent| continent.bonus)
        .sum::<u32>();
    base + continent_bonus
}
