import React, { createContext, useContext, useEffect, useMemo, useRef, useState } from "react";
import type { ServerToClient } from "../types/messages";
import { WsClient } from "./wsClient";
import { MockWsClient } from "./mockWsClient";

type WsContextValue = {
  state: "connecting" | "connected" | "reconnecting" | "disconnected";
  clientId: string | null;
  send: <T,>(type: string, payload: T) => void;
  lastMessage: ServerToClient | null;
};

const WsContext = createContext<WsContextValue | null>(null);

export function useWs() {
  const ctx = useContext(WsContext);
  if (!ctx) throw new Error("useWs must be used inside WebSocketProvider");
  return ctx;
}

export function WebSocketProvider({ children }: { children: React.ReactNode }) {
  const wsUrl = import.meta.env.VITE_WS_URL as string;
  const useMock = (import.meta.env.VITE_MOCK_WS as string) === "true";

  const clientRef = useRef<WsClient | MockWsClient | null>(null);

  const [state, setState] = useState<WsContextValue["state"]>("disconnected");
  const [clientId, setClientId] = useState<string | null>(null);
  const [lastMessage, setLastMessage] = useState<ServerToClient | null>(null);

  useEffect(() => {
    const client = useMock ? new MockWsClient(wsUrl) : new WsClient(wsUrl);
    clientRef.current = client;

    // Connect
    client.connect();

    // Poll connection state (keeps client class simple)
    const poll = window.setInterval(() => {
      setState(client.state);
      setClientId(client.clientId);
    }, 150);

    const off = client.onMessage((msg) => {
      setLastMessage(msg);
    });

    return () => {
      off();
      window.clearInterval(poll);
      client.close();
      clientRef.current = null;
    };
  }, [wsUrl, useMock]);

  const send = useMemo(() => {
    return <T,>(type: string, payload: T) => {
      clientRef.current?.send(type, payload);
    };
  }, []);

  const value: WsContextValue = { state, clientId, send, lastMessage };

  return <WsContext.Provider value={value}>{children}</WsContext.Provider>;
}