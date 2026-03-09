import { useEffect, useMemo, useRef, useState } from "react";
import { useWs } from "../net/WebSocketProvider";

type ChatMsg = {
  from: string;
  message: string;
  timestamp_ms: number;
};

export function ChatPanel(props: { scope: "lobby" | "game"; gameId?: string }) {
  const { send, subscribe } = useWs();
  const [input, setInput] = useState("");
  const [messages, setMessages] = useState<ChatMsg[]>([]);
  const endRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    return subscribe((message) => {
      if (message.type !== "chat_message") return;

      const p: any = message.payload;
      if (p.scope !== props.scope) return;
      if (props.scope === "game" && p.game_id !== props.gameId) return;

      setMessages((m) => [...m, { from: p.from, message: p.message, timestamp_ms: p.timestamp_ms }]);
    });
  }, [props.scope, props.gameId, subscribe]);

  useEffect(() => {
    endRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages.length]);

  const title = useMemo(() => (props.scope === "lobby" ? "Lobby Chat" : "Game Chat"), [props.scope]);

  function onSend() {
    const msg = input.trim();
    if (!msg) return;

    send("chat_send", { scope: props.scope, game_id: props.gameId, message: msg });
    setInput("");
  }

  return (
    <div className="flex h-full flex-col rounded-2xl border border-white/10 bg-white/5">
      <div className="border-b border-white/10 px-4 py-3 text-sm text-white/80">{title}</div>

      <div className="flex-1 overflow-auto px-3 py-3">
        {messages.length === 0 ? (
          <div className="text-sm text-white/40">No messages yet.</div>
        ) : (
          <div className="space-y-2">
            {messages.map((m, idx) => (
              <div key={idx} className="text-sm">
                <span className="text-white/70">{m.from}: </span>
                <span className="text-white/90">{m.message}</span>
              </div>
            ))}
          </div>
        )}
        <div ref={endRef} />
      </div>

      <div className="flex gap-2 border-t border-white/10 p-3">
        <input
          className="w-full rounded-xl border border-white/10 bg-black/30 px-3 py-2 text-sm text-white outline-none placeholder:text-white/30"
          placeholder="Type a message…"
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter") onSend();
          }}
        />
        <button
          className="rounded-xl bg-white/10 px-4 py-2 text-sm text-white hover:bg-white/15"
          onClick={onSend}
        >
          Send
        </button>
      </div>
    </div>
  );
}