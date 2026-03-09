use crate::game::{PlayerAction, PlayerId};

/// Command sent into a room's channel from app/room services.
#[derive(Debug, Clone)]
pub enum RoomCommand {
    PlayerAction {
        player_id: PlayerId,
        action: PlayerAction,
    },
    RefreshState {
        player_id: PlayerId,
    },
    Disconnect {
        player_id: PlayerId,
    },
}
