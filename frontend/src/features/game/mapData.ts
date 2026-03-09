export type MapTerritory = {
  id: number;
  name: string;
  continentId: number;
  x: number;
  y: number;
};

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

const TERRITORY_NAMES: string[] = [
  "Alaska",
  "Northwest Territory",
  "Greenland",
  "Alberta",
  "Ontario",
  "Quebec",
  "Western US",
  "Eastern US",
  "Central America",
  "Venezuela",
  "Peru",
  "Brazil",
  "Argentina",
  "Iceland",
  "Scandinavia",
  "Ukraine",
  "Great Britain",
  "Northern Europe",
  "Western Europe",
  "Southern Europe",
  "North Africa",
  "Egypt",
  "East Africa",
  "Congo",
  "South Africa",
  "Madagascar",
  "Ural",
  "Siberia",
  "Yakutsk",
  "Kamchatka",
  "Irkutsk",
  "Mongolia",
  "Japan",
  "Afghanistan",
  "Middle East",
  "India",
  "Siam",
  "China",
  "Indonesia",
  "New Guinea",
  "Western Australia",
  "Eastern Australia",
];

function continentForId(id: number): number {
  if (id <= 8) return 0;
  if (id <= 12) return 1;
  if (id <= 19) return 2;
  if (id <= 25) return 3;
  if (id <= 37) return 4;
  return 5;
}

const CONTINENT_CENTERS = [
  { x: 170, y: 140 },
  { x: 240, y: 360 },
  { x: 430, y: 150 },
  { x: 450, y: 360 },
  { x: 700, y: 170 },
  { x: 690, y: 360 },
];

const CONTINENT_COUNTS = [9, 4, 7, 6, 12, 4];

export const MAP_TERRITORIES: MapTerritory[] = TERRITORY_NAMES.map((name, id) => {
  const continentId = continentForId(id);
  const startOffset = CONTINENT_COUNTS.slice(0, continentId).reduce((a, b) => a + b, 0);
  const localIndex = id - startOffset;
  const localCount = CONTINENT_COUNTS[continentId];
  const angle = (Math.PI * 2 * localIndex) / localCount;
  const radius = 60 + (localIndex % 2) * 12;
  const center = CONTINENT_CENTERS[continentId];
  return {
    id,
    name,
    continentId,
    x: center.x + Math.cos(angle) * radius,
    y: center.y + Math.sin(angle) * radius,
  };
});

// Keep this in sync with backend's current simplified ring adjacency.
export const MAP_EDGES: Array<[number, number]> = Array.from({ length: MAP_TERRITORIES.length }, (_, id) => {
  const to = (id + 1) % MAP_TERRITORIES.length;
  return [id, to];
});

export const ADJACENCY = new Map<number, Set<number>>();
for (const [from, to] of MAP_EDGES) {
  if (!ADJACENCY.has(from)) ADJACENCY.set(from, new Set());
  if (!ADJACENCY.has(to)) ADJACENCY.set(to, new Set());
  ADJACENCY.get(from)!.add(to);
  ADJACENCY.get(to)!.add(from);
}
