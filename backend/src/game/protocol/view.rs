use serde::Serialize;

use crate::game::card::{Card, CardType};
use crate::game::model::{GamePhase, PlayerId, TurnPhase};

#[derive(Clone, Debug, Serialize)]
pub struct GameView {
    pub you_player_id: u8,
    pub current_player_id: u8,
    pub turn_phase: TurnPhase,
    pub game_phase: GamePhase,
    pub territories: Vec<TerritoryView>,
    pub players_public: Vec<PlayerPublicView>,
    pub your_cards: Vec<CardView>,
    pub latest_combat_roll: Option<CombatRollView>,
    pub action_context: ActionContext,
}

#[derive(Clone, Debug, Serialize)]
pub struct TerritoryView {
    pub id: usize,
    pub continent_id: usize,
    pub owner_player_id: u8,
    pub troops: u32,
}

#[derive(Clone, Debug, Serialize)]
pub struct PlayerPublicView {
    pub player_id: u8,
    pub territories_owned: u32,
    pub card_count: usize,
    pub eliminated: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct CardView {
    pub card_id: u32,
    pub territory_id: usize,
    pub card_type: CardTypeView,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CardTypeView {
    Infantry,
    Cavalry,
    Artillery,
}

#[derive(Clone, Debug, Serialize)]
pub struct CombatRollView {
    pub from: usize,
    pub to: usize,
    pub attacker_rolls: Vec<u8>,
    pub defender_rolls: Vec<u8>,
    pub attacker_losses: u8,
    pub defender_losses: u8,
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ActionContext {
    Setup {},
    Reinforcement { troops_remaining: u32 },
    Attack { captured_this_turn: bool },
    Fortify { used_fortify: bool },
    GameOver { winner_player_id: u8 },
}

impl From<CardType> for CardTypeView {
    fn from(value: CardType) -> Self {
        match value {
            CardType::Infantry => CardTypeView::Infantry,
            CardType::Cavalry => CardTypeView::Cavalry,
            CardType::Artillery => CardTypeView::Artillery,
        }
    }
}

impl From<Card> for CardView {
    fn from(value: Card) -> Self {
        Self {
            card_id: value.id,
            territory_id: value.territory as usize,
            card_type: value.card_type.into(),
        }
    }
}

pub fn player_id_to_u8(id: PlayerId) -> u8 {
    id.0
}
