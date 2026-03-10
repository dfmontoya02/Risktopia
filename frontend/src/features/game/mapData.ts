import riskMapSvg from "./riskMap.svg?raw";

export const CONTINENT_COLORS: string[] = [
  "#3b82f6",
  "#22c55e",
  "#f59e0b",
  "#ef4444",
  "#a855f7",
  "#14b8a6",
];

export const PLAYER_COLORS: string[] = [
  "#f87171",
  "#60a5fa",
  "#34d399",
  "#fbbf24",
  "#c084fc",
  "#fb7185",
];

function continentForId(id: number): number {
  if (id <= 8) return 0; // North America
  if (id <= 12) return 1; // South America
  if (id <= 18) return 2; // Africa
  if (id <= 25) return 3; // Europe
  if (id <= 37) return 4; // Asia
  return 5; // Australia
}

// // Must stay in sync with backend TERRITORY_NAMES.
// const TERRITORY_NAMES: string[] = [
//   // North America: Continent 0
//   "Alaska", // 0
//   "Northwest Territory", // 1
//   "Greenland", // 2
//   "Alberta", // 3
//   "Ontario", // 4
//   "Eastern Canada", // 5
//   "Western US", // 6
//   "Eastern US", // 7
//   "Central America", // 8
//   // South America: Continent 1
//   "Venezuela", // 9
//   "Peru", // 10
//   "Brazil", // 11
//   "Argentina", // 12
//   // Africa: Continent 2
//   "North Africa", // 13
//   "Egypt", // 14
//   "East Africa", // 15
//   "Central Africa", // 16
//   "South Africa", // 17
//   "Madagascar", // 18
//   // Europe: Continent 3
//   "Iceland", // 19
//   "Scandinavia", // 20
//   "Russia", // 21
//   "Great Britain", // 22
//   "Northern Europe", // 23
//   "Western Europe", // 24
//   "Southern Europe", // 25
//   // Asia: Continent 4
//   "Ural", // 26
//   "Siberia", // 27
//   "Yakutsk", // 28
//   "Irkutsk", // 29
//   "Kamchatka", // 30
//   "Afghanistan", // 31
//   "Middle East", // 32
//   "India", // 33
//   "Mongolia", // 34
//   "China", // 35
//   "Japan", // 36
//   "Southeast Asia", // 37
//   // Australia: Continent 5
//   "Indonesia", // 38
//   "New Guinea", // 39
//   "Western Australia", // 40
//   "Eastern Australia", // 41
// ];

// Stable string IDs for territories, prefixed by continent.
// NA_ = North America, SA_ = South America, AF_ = Africa,
// EU_ = Europe, AS_ = Asia, AU_ = Australia.
export const TERRITORY_IDS: string[] = [
  // North America: 0–8
  "NA_Alaska", // 0
  "NA_NorthwestTerritory", // 1
  "NA_Greenland", // 2
  "NA_Alberta", // 3
  "NA_Ontario", // 4
  "NA_EasternCanada", // 5
  "NA_WesternUnitedStates", // 6
  "NA_EasternUnitedStates", // 7
  "NA_CentralAmerica", // 8
  // South America: 9–12
  "SA_Venezuela", // 9
  "SA_Peru", // 10
  "SA_Brazil", // 11
  "SA_Argentina", // 12
  // Africa: 13–18
  "AF_NorthAfrica", // 13
  "AF_Egypt", // 14
  "AF_EastAfrica", // 15
  "AF_CentralAfrica", // 16
  "AF_SouthAfrica", // 17
  "AF_Madagascar", // 18
  // Europe: 19–25
  "EU_Iceland", // 19
  "EU_Scandinavia", // 20
  "EU_Russia", // 21
  "EU_GreatBritain", // 22
  "EU_NorthernEurope", // 23
  "EU_WesternEurope", // 24
  "EU_SouthernEurope", // 25
  // Asia: 26–37
  "AS_Ural", // 26
  "AS_Siberia", // 27
  "AS_Yakutsk", // 28
  "AS_Irkutsk", // 29
  "AS_Kamchatka", // 30
  "AS_Afghanistan", // 31
  "AS_MiddleEast", // 32
  "AS_India", // 33
  "AS_Mongolia", // 34
  "AS_China", // 35
  "AS_Japan", // 36
  "AS_SoutheastAsia", // 37
  // Australia: 38–41
  "AU_Indonesia", // 38
  "AU_NewGuinea", // 39
  "AU_WesternAustralia", // 40
  "AU_EasternAustralia", // 41
];

// Keep this in sync with backend DEFAULT_ADJACENCY.
export const MAP_EDGES: Array<[number, number]> = [
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
  [9, 10], // Venezuela to Peru
  [9, 11], // Venezuela to Brazil
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
  // Connect continents
  [0, 30], // Alaska to Kamchatka
  [2, 19], // Greenland to Iceland
  [8, 9], // Central America to Venezuela
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

export const ADJACENCY = new Map<number, Set<number>>();
for (const [from, to] of MAP_EDGES) {
  if (!ADJACENCY.has(from)) ADJACENCY.set(from, new Set());
  if (!ADJACENCY.has(to)) ADJACENCY.set(to, new Set());
  ADJACENCY.get(from)!.add(to);
  ADJACENCY.get(to)!.add(from);
}

export type MapTerritory = {
  id: number;
  name: string;
  continentId: number;
  paths: string[];
  x: number;
  y: number;
  labelSvg?: string;
};
function loadTerritoriesFromSvg(): MapTerritory[] {
  const parser = new DOMParser();
  const doc = parser.parseFromString(riskMapSvg, "image/svg+xml");

  const svg = doc.documentElement as unknown as SVGSVGElement;

  svg.style.position = "absolute";
  svg.style.visibility = "hidden";
  document.body.appendChild(svg);

  const territories: MapTerritory[] = TERRITORY_IDS.map((territoryId, id) => {
    const group = svg.getElementById(territoryId) as SVGGElement;

    if (!group) {
      throw new Error(`Missing SVG group for ${territoryId}`);
    }

    const paths = Array.from(group.querySelectorAll("path"));

    if (paths.length === 0) {
      throw new Error(`No paths inside ${territoryId}`);
    }

    const dValues = paths.map(p => p.getAttribute("d")!);

    const bbox = group.getBBox();

    const x = bbox.x + bbox.width / 2;
    const y = bbox.y + bbox.height / 2;

    const label = group.querySelector("text");

    const labelSvg = label ? label.outerHTML : undefined;

    return {
      id,
      name: label?.textContent ?? territoryId,
      continentId: continentForId(id),
      paths: dValues,
      x,
      y,
      labelSvg,
    };
  });

  svg.remove();

  return territories;
}

export const MAP_TERRITORIES: MapTerritory[] = loadTerritoriesFromSvg();

