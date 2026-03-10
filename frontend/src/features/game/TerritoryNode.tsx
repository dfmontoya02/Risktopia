import { CONTINENT_COLORS, PLAYER_COLORS, type MapTerritory } from "./mapData";

type Props = {
  territory: MapTerritory;
  ownerPlayerId: number;
  troops: number;
  selected: boolean;
  secondaryHighlight: boolean;
  onClick: (id: number) => void;
};

export function TerritoryNode(props: Props) {
  const continentColor =
    CONTINENT_COLORS[props.territory.continentId % CONTINENT_COLORS.length];

  const hasOwner = props.ownerPlayerId >= 0 && props.ownerPlayerId < 250;

  const ownerColor = hasOwner
    ? PLAYER_COLORS[props.ownerPlayerId % PLAYER_COLORS.length]
    : "#6b7280";

  const fillColor = hasOwner ? ownerColor : continentColor;

  return (
    <g
      onClick={() => props.onClick(props.territory.id)}
      className="cursor-pointer hover:brightness-110"
    >
      {props.territory.paths.map((d, i) => (
  <path
    key={i}
    d={d}
    fill={fillColor}
    stroke={props.selected ? "#fef08a" : "#111827"}
    strokeWidth={props.selected ? 1.5 : (props.secondaryHighlight ? 1 : 0.5) }
  />
))}

      {/* Troop marker */}
      <circle
        cx={props.territory.x}
        cy={props.territory.y}
        r={3}
        fill="#111827"
        stroke="#ffffff"
        strokeWidth={.2}
      />

      {/* Troop number */}
      <text
        x={props.territory.x}
        y={props.territory.y + 2}
        textAnchor="middle"
        fontSize="5"
        fill="#ffffff"
        style={{ pointerEvents: "none", userSelect: "none" }}
      >
        {props.troops}
      </text>


      {props.territory.labelSvg && (
  <g
    dangerouslySetInnerHTML={{ __html: props.territory.labelSvg }}
    style={{ pointerEvents: "none" }}
  />
)}
    </g>
  );
}