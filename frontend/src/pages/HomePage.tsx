import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { ConnectionBadge } from "../components/ConnectionBadge";
import { useWs } from "../net/WebSocketProvider";

export function HomePage() {
  const nav = useNavigate();
  const { state, send, lastMessage } = useWs();
  const [waiting, setWaiting] = useState<number>(0);

  useEffect(() => {
    if (!lastMessage) return;
    if (lastMessage.type === "queue_update") {
      setWaiting((lastMessage as any).payload.players_waiting ?? 0);
    }
    if (lastMessage.type === "game_start") {
      const gameId = (lastMessage as any).payload.game_id as string;
      nav(`/game/${gameId}`);
    }
  }, [lastMessage, nav]);

  return (
    <div className="min-h-full bg-gradient-to-b from-zinc-950 to-zinc-900 text-white">
      <div className="mx-auto max-w-5xl p-6">
        <div className="flex items-center justify-between">
          <div>
            <div className="text-2xl font-semibold">Risk Online (MVP)</div>
            <div className="text-white/60">Frontend scaffold — matchmaking, routing, chat hooks</div>
          </div>
          <ConnectionBadge />
        </div>

        <div className="mt-8 grid gap-6 md:grid-cols-2">
          <div className="rounded-2xl border border-white/10 bg-white/5 p-5">
            <div className="text-lg font-semibold">Matchmaking</div>
            <div className="mt-1 text-sm text-white/60">
              Click join. When 4 players are queued, the server starts a game.
            </div>

            <div className="mt-4 flex items-center justify-between rounded-xl border border-white/10 bg-black/20 px-4 py-3">
              <div className="text-sm text-white/80">Players waiting</div>
              <div className="text-xl font-semibold">{waiting}</div>
            </div>

            <div className="mt-4 flex gap-3">
              <button
                className="rounded-xl bg-white/10 px-4 py-2 text-sm hover:bg-white/15 disabled:opacity-50"
                disabled={state !== "connected"}
                onClick={() => {
                  send("queue_join", {});
                  nav("/lobby");
                }}
              >
                Join queue
              </button>

              <button
                className="rounded-xl bg-white/5 px-4 py-2 text-sm hover:bg-white/10"
                onClick={() => nav("/lobby")}
              >
                Go to lobby
              </button>
            </div>

            <div className="mt-3 text-xs text-white/40">
              Tip: set <code className="rounded bg-black/30 px-1">VITE_MOCK_WS=true</code> to
              simulate a backend.
            </div>
          </div>

          <div className="rounded-2xl border border-white/10 bg-white/5 p-5">
            <div className="text-lg font-semibold">Next up</div>
            <ul className="mt-3 list-disc space-y-2 pl-5 text-sm text-white/70">
              <li>Replace mock matchmaking with real Rust events</li>
              <li>Game shell layout (board center, chat right)</li>
              <li>Territory selection UI + sending intents</li>
              <li>Render server state (owners/troops)</li>
            </ul>
          </div>
        </div>
      </div>
    </div>
  );
}