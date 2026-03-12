export type Envelope<T = unknown> = {
  type: string;
  version: number;
  request_id?: string;
  payload: T;
};

export type ClientHelloPayload = { client_id: string };

export type QueueUpdatePayload = {
  players_waiting: number;
};

export type GameStartPayload = {
  game_id: string;
  player_index: number;
};

export type TurnPhase = "Setup" | "Reinforcement" | "Attack" | "Fortify" | "GameOver";
export type GamePhase = "InProgress" | "Finished";

export type StateUpdatePayload = {
  state_version: number;
  state: {
    game_id: string;
    current_player: number;
    turn_phase: TurnPhase;
    game_phase: GamePhase;
    territories: { owner: number; troops: number }[];
  };
};

export type CardTypeView = "infantry" | "cavalry" | "artillery";

export type TerritoryView = {
  id: number;
  continent_id: number;
  owner_player_id: number;
  troops: number;
};

export type PlayerPublicView = {
  player_id: number;
  territories_owned: number;
  card_count: number;
  eliminated: boolean;
};

export type CardView = {
  card_id: number;
  territory_id: number;
  card_type: CardTypeView;
};

export type CombatRollView = {
  from: number;
  to: number;
  attacker_rolls: number[];
  defender_rolls: number[];
  attacker_losses: number;
  defender_losses: number;
};

export type ActionContext =
  | { kind: "setup" }
  | { kind: "reinforcement"; troops_remaining: number }
  | { kind: "attack"; captured_this_turn: boolean }
  | { kind: "capture_move"; from: number; to: number; min_troops: number; max_troops: number }
  | { kind: "fortify"; used_fortify: boolean }
  | { kind: "game_over"; winner_player_id: number };

export type GameView = {
  you_player_id: number;
  current_player_id: number;
  turn_phase: TurnPhase;
  game_phase: GamePhase;
  territories: TerritoryView[];
  players_public: PlayerPublicView[];
  your_cards: CardView[];
  latest_combat_roll: CombatRollView | null;
  action_context: ActionContext;
};

export type GameViewPayload = {
  game_id: string;
  state_version: number;
  view: GameView;
};

export type GameEventPayload = {
  game_id: string;
  event: unknown;
};

export type ActionErrorPayload = {
  game_id: string;
  code:
    | "INVALID_ENVELOPE"
    | "UNKNOWN_MESSAGE_TYPE"
    | "INVALID_PAYLOAD"
    | "UNAUTHORIZED_ROOM_ACCESS"
    | "ROOM_NOT_FOUND"
    | "GAME_ACTION_REJECTED";
  message: string;
};

export type ProtocolErrorPayload = {
  code:
    | "INVALID_ENVELOPE"
    | "UNKNOWN_MESSAGE_TYPE"
    | "INVALID_PAYLOAD"
    | "UNAUTHORIZED_ROOM_ACCESS"
    | "ROOM_NOT_FOUND"
    | "GAME_ACTION_REJECTED";
  message: string;
};

export type ChatMessagePayload = {
  scope: "lobby" | "game";
  game_id?: string;
  from: string;
  message: string;
  timestamp_ms: number;
};

export type ServerToClient =
  | Envelope<ClientHelloPayload>
  | Envelope<QueueUpdatePayload>
  | Envelope<GameStartPayload>
  | Envelope<StateUpdatePayload>
  | Envelope<GameViewPayload>
  | Envelope<ChatMessagePayload>
  | Envelope<GameEventPayload>
  | Envelope<ActionErrorPayload>
  | Envelope<ProtocolErrorPayload>;

export type PlayerAction =
  | { PlaceTroops: { territory: number; count: number } }
  | { Attack: { from: number; to: number; dice: number } }
  | { MoveCapturedTroops: { count: number } }
  | { EndAttack: null }
  | { Fortify: { from: number; to: number; count: number } }
  | { PlayCards: { cards: [number, number, number] } }
  | { EndTurn: null };
  
export type GameActionPayload = {
  game_id: string;
  action: PlayerAction;
};

export type StateRefreshPayload = {
  game_id: string;
};

export type ClientToServer =
  | Envelope<{}>
  | Envelope<{ scope: "lobby" | "game"; game_id?: string; message: string }>
  | Envelope<GameActionPayload>
  | Envelope<StateRefreshPayload>;