use super::ids::{PlayerId, TerritoryId};

pub const DEFAULT_TERRITORY_COUNT: usize = 42;

// Canonical territory order shared with frontend mapData.ts:
//
// 0  Alaska
// 1  Northwest Territory
// 2  Greenland
// 3  Alberta
// 4  Ontario
// 5  Eastern Canada
// 6  Western United States
// 7  Eastern United States
// 8  Central America
// 9  Venezuela
// 10 Peru
// 11 Brazil
// 12 Argentina
// 13 North Africa
// 14 Egypt
// 15 East Africa
// 16 Central Africa
// 17 South Africa
// 18 Madagascar
// 19 Iceland
// 20 Scandinavia
// 21 Russia
// 22 Great Britain
// 23 Northern Europe
// 24 Western Europe
// 25 Southern Europe
// 26 Ural
// 27 Siberia
// 28 Yakutsk
// 29 Irkutsk
// 30 Kamchatka
// 31 Afghanistan
// 32 Middle East
// 33 India
// 34 Mongolia
// 35 China
// 36 Japan
// 37 Southeast Asia
// 38 Indonesia
// 39 New Guinea
// 40 Western Australia
// 41 Eastern Australia

/// Undirected edge list for the built-in Risk map.
/// Each entry is a connection between two adjacent territories.
pub const DEFAULT_EDGES: [[TerritoryId; 2]; 83] = [
    // North America
    [0, 1], // Alaska to Northwest Territory
    [0, 3], // Alaska to Alberta
    [1, 2], // Northwest Territory to Greenland
    [1, 3], // Northwest Territory to Alberta
    [1, 4], // Northwest Territory to Ontario
    [2, 4], // Greenland to Ontario
    [2, 5], // Greenland to Eastern Canada
    [3, 4], // Alberta to Ontario
    [3, 6], // Alberta to Western US
    [4, 5], // Ontario to Eastern Canada
    [4, 6], // Ontario to Western US
    [4, 7], // Ontario to Eastern US
    [5, 7], // Eastern Canada to Eastern US
    [6, 7], // Western US to Eastern US
    [6, 8], // Western US to Central America
    [7, 8], // Eastern US to Central America

    // South America
    [9, 10],  // Venezuela to Peru
    [9, 11],  // Venezuela to Brazil
    [10, 11], // Peru to Brazil
    [10, 12], // Peru to Argentina
    [11, 12], // Brazil to Argentina

    // Africa
    [13, 14], // North Africa to Egypt
    [13, 15], // North Africa to East Africa
    [13, 16], // North Africa to Central Africa
    [14, 15], // Egypt to East Africa
    [15, 16], // East Africa to Central Africa
    [15, 17], // East Africa to South Africa
    [15, 18], // East Africa to Madagascar
    [16, 17], // Central Africa to South Africa
    [17, 18], // South Africa to Madagascar

    // Europe
    [19, 22], // Iceland to Great Britain
    [19, 20], // Iceland to Scandinavia
    [20, 22], // Scandinavia to Great Britain
    [20, 23], // Scandinavia to Northern Europe
    [20, 21], // Scandinavia to Russia
    [22, 23], // Great Britain to Northern Europe
    [22, 24], // Great Britain to Western Europe
    [23, 24], // Northern Europe to Western Europe
    [23, 25], // Northern Europe to Southern Europe
    [23, 21], // Northern Europe to Russia
    [24, 25], // Western Europe to Southern Europe
    [25, 21], // Southern Europe to Russia

    // Asia
    [26, 27], // Ural to Siberia
    [26, 35], // Ural to China
    [26, 31], // Ural to Afghanistan
    [27, 28], // Siberia to Yakutsk
    [27, 29], // Siberia to Irkutsk
    [27, 34], // Siberia to Mongolia
    [27, 35], // Siberia to China
    [28, 29], // Yakutsk to Irkutsk
    [28, 30], // Yakutsk to Kamchatka
    [29, 30], // Irkutsk to Kamchatka
    [29, 34], // Irkutsk to Mongolia
    [30, 34], // Kamchatka to Mongolia
    [30, 36], // Kamchatka to Japan
    [31, 32], // Afghanistan to Middle East
    [31, 35], // Afghanistan to China
    [31, 33], // Afghanistan to India
    [32, 33], // Middle East to India
    [33, 35], // India to China
    [33, 37], // India to Southeast Asia
    [34, 35], // Mongolia to China
    [34, 36], // Mongolia to Japan
    [35, 37], // China to Southeast Asia

    // Australia
    [38, 39], // Indonesia to New Guinea
    [38, 40], // Indonesia to Western Australia
    [39, 40], // New Guinea to Western Australia
    [39, 41], // New Guinea to Eastern Australia
    [40, 41], // Western Australia to Eastern Australia

    // Inter-continent connections
    [0, 30],  // Alaska to Kamchatka
    [2, 19],  // Greenland to Iceland
    [8, 9],   // Central America to Venezuela
    [11, 13], // Brazil to North Africa
    [13, 24], // North Africa to Western Europe
    [13, 25], // North Africa to Southern Europe
    [14, 25], // Egypt to Southern Europe
    [14, 32], // Egypt to Middle East
    [15, 32], // East Africa to Middle East
    [25, 32], // Southern Europe to Middle East
    [21, 32], // Russia to Middle East
    [21, 26], // Russia to Ural
    [21, 31], // Russia to Afghanistan
    [37, 38], // Southeast Asia to Indonesia
];

// Default continents are described as static specs (slice of territories + bonus)
// and converted into owned `Continent` values at runtime.
pub const DEFAULT_CONTINENT_SPECS: [(&[TerritoryId], u32); 6] = [
    // North America
    (&[0, 1, 2, 3, 4, 5, 6, 7, 8], 5),
    // South America
    (&[9, 10, 11, 12], 2),
    // Africa
    (&[13, 14, 15, 16, 17, 18], 3),
    // Europe
    (&[19, 20, 21, 22, 23, 24, 25], 5),
    // Asia
    (&[26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37], 7),
    // Australia
    (&[38, 39, 40, 41], 2),
];

#[derive(Clone, Debug)]
pub struct GameMap {
    pub adjacency: Vec<Vec<TerritoryId>>,
    pub continents: Vec<Continent>,
}

#[derive(Clone, Debug)]
pub struct Continent {
    pub territories: Vec<TerritoryId>,
    pub bonus: u32,
}

impl GameMap {
    pub fn default_static() -> Self {
        let mut adjacency = vec![Vec::<TerritoryId>::new(); DEFAULT_TERRITORY_COUNT];

        for [a, b] in DEFAULT_EDGES {
            adjacency[a].push(b);
            adjacency[b].push(a);
        }

        // Keep adjacency deterministic and duplicate-free.
        for neighbors in &mut adjacency {
            neighbors.sort_unstable();
            neighbors.dedup();
        }

        let continents = DEFAULT_CONTINENT_SPECS
            .iter()
            .map(|(territories, bonus)| Continent {
                territories: territories.to_vec(),
                bonus: *bonus,
            })
            .collect();

        Self {
            adjacency,
            continents,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct TerritoryState {
    pub owner: PlayerId,
    pub troops: u32,
}