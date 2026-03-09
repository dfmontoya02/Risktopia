//! Risk-style territory cards: types, card struct, set turn-in progression, and standard deck.

/// Card type (Infantry, Cavalry, Artillery). Used for set validation and bonus troops.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardType {
    Infantry,
    Cavalry,
    Artillery,
}

/// Unique identifier for a card instance (e.g. index in the initial deck).
pub type CardId = u32;

/// A single territory card: type, which territory it depicts, and unique id.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Card {
    pub card_type: CardType,
    /// Index of the territory pictured on this card (into GameMap / territories).
    pub territory: u32,
    pub id: CardId,
}

/// Number of troops granted for turning in the n-th set of three cards (0-based index).
/// Classic Risk progression: 4, 6, 8, 10, 12, 15, 20, 25, 30, then +5 per set (35, 40, 45, ...).
pub fn troops_for_set_index(set_index: u32) -> u32 {
    const FIRST: [u32; 10] = [4, 6, 8, 10, 12, 15, 20, 25, 30, 35];
    let i = set_index as usize;
    if i < FIRST.len() {
        FIRST[i]
    } else {
        35 + (i - FIRST.len() + 1) as u32 * 5
    }
}

/// Standard Risk deck: one card per territory, 14 Infantry, 14 Cavalry, 14 Artillery.
/// `territory_indices` must have length 42 for classic Risk (or any length for custom maps).
/// Cards are assigned types in order: first third Infantry, second third Cavalry, last third Artillery.
pub fn new_standard_deck(territory_indices: &[u32]) -> Vec<Card> {
    let n = territory_indices.len();
    let third = n / 3;
    territory_indices
        .iter()
        .enumerate()
        .map(|(i, &territory)| {
            let card_type = if i < third {
                CardType::Infantry
            } else if i < 2 * third {
                CardType::Cavalry
            } else {
                CardType::Artillery
            };
            Card {
                card_type,
                territory,
                id: i as CardId,
            }
        })
        .collect()
}

/// Check if three cards form a valid set: either all same type or one of each type.
pub fn is_valid_set(cards: &[Card; 3]) -> bool {
    let a = cards[0].card_type;
    let b = cards[1].card_type;
    let c = cards[2].card_type;
    (a == b && b == c) || (a != b && b != c && a != c)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progression_first_ten() {
        assert_eq!(troops_for_set_index(0), 4);
        assert_eq!(troops_for_set_index(1), 6);
        assert_eq!(troops_for_set_index(9), 35);
    }

    #[test]
    fn progression_after_ten() {
        assert_eq!(troops_for_set_index(10), 40);
        assert_eq!(troops_for_set_index(11), 45);
    }

    #[test]
    fn standard_deck_42() {
        let territories: Vec<u32> = (0..42).collect();
        let deck = new_standard_deck(&territories);
        assert_eq!(deck.len(), 42);
        // 14 Infantry (0..14), 14 Cavalry (14..28), 14 Artillery (28..42)
        let inf: usize = deck
            .iter()
            .filter(|c| c.card_type == CardType::Infantry)
            .count();
        let cav: usize = deck
            .iter()
            .filter(|c| c.card_type == CardType::Cavalry)
            .count();
        let art: usize = deck
            .iter()
            .filter(|c| c.card_type == CardType::Artillery)
            .count();
        assert_eq!(inf, 14);
        assert_eq!(cav, 14);
        assert_eq!(art, 14);
    }

    #[test]
    fn valid_set_same_type() {
        let cards = [
            Card {
                card_type: CardType::Infantry,
                territory: 0,
                id: 0,
            },
            Card {
                card_type: CardType::Infantry,
                territory: 1,
                id: 1,
            },
            Card {
                card_type: CardType::Infantry,
                territory: 2,
                id: 2,
            },
        ];
        assert!(is_valid_set(&cards));
    }

    #[test]
    fn valid_set_mixed() {
        let cards = [
            Card {
                card_type: CardType::Infantry,
                territory: 0,
                id: 0,
            },
            Card {
                card_type: CardType::Cavalry,
                territory: 1,
                id: 1,
            },
            Card {
                card_type: CardType::Artillery,
                territory: 2,
                id: 2,
            },
        ];
        assert!(is_valid_set(&cards));
    }

    #[test]
    fn invalid_set_two_and_one() {
        let cards = [
            Card {
                card_type: CardType::Infantry,
                territory: 0,
                id: 0,
            },
            Card {
                card_type: CardType::Infantry,
                territory: 1,
                id: 1,
            },
            Card {
                card_type: CardType::Cavalry,
                territory: 2,
                id: 2,
            },
        ];
        assert!(!is_valid_set(&cards));
    }
}
