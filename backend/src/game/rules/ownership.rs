use crate::game::model::{GameCore, PlayerId};

pub fn adjust_ownership(core: &mut GameCore, attacker: PlayerId, defender: PlayerId) {
    if let Some(player) = core.players.iter_mut().find(|p| p.id == attacker) {
        player.territories_owned += 1;
    }
    if let Some(player) = core.players.iter_mut().find(|p| p.id == defender) {
        player.territories_owned = player.territories_owned.saturating_sub(1);
        if player.territories_owned == 0 {
            player.eliminated = true;
        }
    }
}
