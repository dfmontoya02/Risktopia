import type { Envelope, ServerToClient } from "../types/messages";

type Handler = (msg: ServerToClient) => void;
type ConnectionState = "connecting" | "connected" | "reconnecting" | "disconnected";

export class MockWsClient {
  private handlers = new Set<Handler>();
  private timer: number | null = null;

  public state: ConnectionState = "disconnected";
  public clientId: string | null = null;

  private waiting = 0;
  private inQueue = false;

  constructor(_url: string) {
  }

  onMessage(handler: Handler) {
    this.handlers.add(handler);
    return () => this.handlers.delete(handler);
  }

  connect() {
    this.state = "connecting";
    setTimeout(() => {
      this.state = "connected";
      this.clientId = `mock-${Math.random().toString(16).slice(2, 8)}`;
      this.emit({
        type: "server_hello",
        version: 1,
        payload: { client_id: this.clientId },
      });
    }, 200);
  }

  close() {
    if (this.timer) window.clearInterval(this.timer);
    this.timer = null;
    this.state = "disconnected";
  }

  send<T>(type: string, payload: T) {
    // Basic simulation
    if (type === "queue_join") {
      this.inQueue = true;
      this.startQueueSim();
    }

    if (type === "queue_leave") {
      this.inQueue = false;
      this.waiting = 0;
      if (this.timer) window.clearInterval(this.timer);
      this.timer = null;
      this.emit({ type: "queue_update", version: 1, payload: { players_waiting: this.waiting } });
    }

    if (type === "chat_send") {
      const p = payload as any;
      this.emit({
        type: "chat_message",
        version: 1,
        payload: {
          scope: p.scope ?? "lobby",
          game_id: p.game_id,
          from: this.clientId ?? "you",
          message: p.message ?? "",
          timestamp_ms: Date.now(),
        },
      });
    }

    if (type === "game_action") {
      const p = payload as any;
      if (p?.game_id) {
        this.emit({
          type: "game_view",
          version: 1,
          payload: this.mockGameView(p.game_id, 2),
        });
      }
    }

    if (type === "state_refresh") {
      const p = payload as any;
      if (p?.game_id) {
        this.emit({
          type: "state_update",
          version: 1,
          payload: {
            state_version: 2,
            state: {
              game_id: p.game_id,
              current_player: 0,
              turn_phase: "Setup",
              game_phase: "InProgress",
              territories: Array.from({ length: 42 }, () => ({ owner: 255, troops: 0 })),
            },
          },
        });
        this.emit({
          type: "game_view",
          version: 1,
          payload: this.mockGameView(p.game_id, 2),
        });
      }
    }
  }

  private startQueueSim() {
    if (this.timer) return;
    this.waiting = 0;
    this.emit({ type: "queue_update", version: 1, payload: { players_waiting: this.waiting } });

    this.timer = window.setInterval(() => {
      if (!this.inQueue) return;

      if (this.waiting < 4) {
        this.waiting++;
        this.emit({ type: "queue_update", version: 1, payload: { players_waiting: this.waiting } });
      }

      if (this.waiting >= 4) {
        const gameId = `game-${Math.random().toString(16).slice(2, 8)}`;
        this.emit({
          type: "game_start",
          version: 1,
          payload: { game_id: gameId, player_index: 0 },
        });

        // also push an initial state update
        this.emit({
          type: "state_update",
          version: 1,
          payload: {
            state_version: 1,
            state: {
              game_id: gameId,
              current_player: 0,
              turn_phase: "Setup",
              game_phase: "InProgress",
              territories: Array.from({ length: 42 }, () => ({ owner: 255, troops: 0 })),
            },
          },
        });
        this.emit({
          type: "game_view",
          version: 1,
          payload: this.mockGameView(gameId, 1),
        });

        // stop sim
        if (this.timer) window.clearInterval(this.timer);
        this.timer = null;
      }
    }, 900);
  }

  private emit<T>(env: Envelope<T>) {
    const msg = env as unknown as ServerToClient;
    for (const h of this.handlers) h(msg);
  }

  private mockGameView(gameId: string, stateVersion: number) {
    return {
      game_id: gameId,
      state_version: stateVersion,
      view: {
        you_player_id: 0,
        current_player_id: 0,
        turn_phase: "Setup",
        game_phase: "InProgress",
        territories: Array.from({ length: 42 }, (_, id) => ({
          id,
          continent_id: Math.floor(id / 7),
          owner_player_id: id % 4,
          troops: (id % 5) + 1,
        })),
        players_public: Array.from({ length: 4 }, (_, id) => ({
          player_id: id,
          territories_owned: 10,
          card_count: 2,
          eliminated: false,
        })),
        your_cards: [],
        latest_combat_roll: null,
        action_context: { kind: "setup" },
      },
    };
  }
}