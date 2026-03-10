import type { TerritoryView } from "../../types/messages";
import { ADJACENCY, MAP_EDGES, MAP_TERRITORIES } from "./mapData";
import { TerritoryNode } from "./TerritoryNode";

type Props = {
  territories: TerritoryView[];
  selectedFrom: number | null;
  selectedTo: number | null;
  onTerritoryClick: (id: number) => void;
};

export function MapBoardSvg(props: Props) {
  const territoryById = new Map<number, TerritoryView>(
    props.territories.map((territory) => [territory.id, territory]),
  );

  return (
    <svg viewBox="0 0 210 297" className="h-[520px] w-full rounded-xl border border-white/10 bg-black/20">
      {MAP_EDGES.map(([from, to]) => {
        const a = MAP_TERRITORIES[from];
        const b = MAP_TERRITORIES[to];
        return (
          <line
            key={`edge-${from}-${to}`}
            x1={a.x}
            y1={a.y}
            x2={b.x}
            y2={b.y}
            stroke="#4b5563"
            strokeWidth={1.5}
            opacity={0.8}
          />
        );
      })}

      {MAP_TERRITORIES.map((territory) => {
        const state = territoryById.get(territory.id);
        const selected = territory.id === props.selectedFrom || territory.id === props.selectedTo;
        return (
          <TerritoryNode
            key={territory.id}
            territory={territory}
            ownerPlayerId={state?.owner_player_id ?? 255}
            troops={state?.troops ?? 0}
            selected={selected}
            secondaryHighlight={(props.selectedFrom != null && props.selectedTo == null) && ((ADJACENCY.get(props.selectedFrom)?.has(territory.id)) ?? false)}
            onClick={props.onTerritoryClick}
          />
        );
      })}
    </svg>
  );
}
