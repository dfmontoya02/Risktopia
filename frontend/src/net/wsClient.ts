import type { Envelope, ServerToClient } from "../types/messages";
import { uuid } from "../util/ids";

type ConnectionState = "connecting" | "connected" | "reconnecting" | "disconnected";

type Handler = (msg: ServerToClient) => void;

export class WsClient {
  private ws: WebSocket | null = null;
  private handlers = new Set<Handler>();
  private sendQueue: string[] = [];
  private reconnectAttempt = 0;
  private closedByUser = false;
  private url: string;


  public state: ConnectionState = "disconnected";
  public clientId: string | null = null;

  constructor(url: string) {
    this.url = url;
  }

  onMessage(handler: Handler) {
    this.handlers.add(handler);
    return () => this.handlers.delete(handler);
  }

  connect() {
    this.closedByUser = false;
    this.setState("connecting");

    try {
      this.ws = new WebSocket(this.url);
    } catch {
      this.scheduleReconnect();
      return;
    }

    this.ws.onopen = () => {
      this.reconnectAttempt = 0;
      this.setState("connected");
      this.flushQueue();
    };

    this.ws.onmessage = (ev) => {
        let parsed: ServerToClient;
        try {
          parsed = JSON.parse(ev.data) as ServerToClient;
        } catch {
          return;
        }
      
        if (parsed.type === "server_hello" && typeof (parsed as any).payload?.client_id === "string") {
          this.clientId = (parsed as any).payload.client_id;
        }
      
        for (const h of this.handlers) h(parsed);
      };

    this.ws.onclose = () => {
      this.ws = null;
      if (this.closedByUser) {
        this.setState("disconnected");
        return;
      }
      this.setState("reconnecting");
      this.scheduleReconnect();
    };

    this.ws.onerror = () => {
      // let onclose handle reconnection
    };
  }

  close() {
    this.closedByUser = true;
    this.ws?.close();
    this.ws = null;
    this.setState("disconnected");
  }

  send<T>(type: string, payload: T) {
    const env: Envelope<T> = {
      type,
      version: 1,
      request_id: uuid(),
      payload,
    };
    const str = JSON.stringify(env);

    if (this.ws && this.state === "connected" && this.ws.readyState === WebSocket.OPEN) {
      this.ws.send(str);
    } else {
      this.sendQueue.push(str);
    }
  }

  private flushQueue() {
    if (!this.ws || this.ws.readyState !== WebSocket.OPEN) return;
    while (this.sendQueue.length) {
      const msg = this.sendQueue.shift();
      if (msg) this.ws.send(msg);
    }
  }

  private scheduleReconnect() {
    const attempt = this.reconnectAttempt++;
    const base = Math.min(8000, 500 * Math.pow(2, attempt)); // 500, 1000, 2000...
    const jitter = Math.floor(Math.random() * 250);
    const delay = base + jitter;

    setTimeout(() => {
      if (!this.closedByUser) this.connect();
    }, delay);
  }

  private setState(s: ConnectionState) {
    this.state = s;
    // consumers read via provider state; provider polls/sets state
  }
}