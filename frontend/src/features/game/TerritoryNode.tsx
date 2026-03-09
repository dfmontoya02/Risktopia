import { CONTINENT_COLORS, PLAYER_COLORS, type MapTerritory } from "./mapData";

type Props = {
  territory: MapTerritory;
  ownerPlayerId: number;
  troops: number;
  selected: boolean;
  onClick: (id: number) => void;
};

export function TerritoryNode(props: Props) {
  const continentColor = CONTINENT_COLORS[props.territory.continentId % CONTINENT_COLORS.length];
  const hasOwner = props.ownerPlayerId >= 0 && props.ownerPlayerId < 250;
  const ownerColor = hasOwner
    ? PLAYER_COLORS[props.ownerPlayerId % PLAYER_COLORS.length]
    : "#6b7280";

  return (
    <g onClick={() => props.onClick(props.territory.id)} className="cursor-pointer">
      <circle
        cx={props.territory.x}
        cy={props.territory.y}
        r={20}
        fill={continentColor}
        stroke={props.selected ? "#fef08a" : "#111827"}
        strokeWidth={props.selected ? 4 : 2}
      />
      <circle
        cx={props.territory.x}
        cy={props.territory.y}
        r={11}
        fill={ownerColor}
        stroke="#111827"
        strokeWidth={1.5}
      />
      <text
        x={props.territory.x}
        y={props.territory.y + 4}
        textAnchor="middle"
        fontSize="10"
        fill="#ffffff"
        style={{ pointerEvents: "none", userSelect: "none" }}
      >
        {props.troops}
      </text>
      <text
        x={props.territory.x}
        y={props.territory.y + 32}
        textAnchor="middle"
        fontSize="10"
        fill="#e5e7eb"
        style={{ pointerEvents: "none", userSelect: "none" }}
      >
        {props.territory.id}: {props.territory.name}
      </text>
    </g>
  );
}
