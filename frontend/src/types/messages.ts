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
  
  export type StateUpdatePayload = {
    state_version: number;
    state: unknown; // later: define your real GameState type
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
    | Envelope<ChatMessagePayload>;
  
  export type ClientToServer =
    | Envelope<{}>
    | Envelope<{ scope: "lobby" | "game"; game_id?: string; message: string }>;