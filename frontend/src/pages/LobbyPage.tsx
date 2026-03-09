import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { ConnectionBadge } from "../components/ConnectionBadge";
import { ChatPanel } from "../components/ChatPanel";
import { useWs } from "../net/WebSocketProvider";

export function LobbyPage() {
  const nav = useNavigate();
  const { send, subscribe } = useWs();
  const [waiting, setWaiting] = useState(0);

  useEffect(() => {
    return subscribe((message) => {
      if (message.type === "queue_update") {
        setWaiting((message as any).payload.players_waiting ?? 0);
        return;
      }
      if (message.type === "game_start") {
        const gameId = (message as any).payload.game_id as string;
        nav(`/game/${gameId}`);
        return;
      }
      if (message.type === "state_update") {
        const gameId = (message as any).payload?.state?.game_id as string | undefined;
        if (gameId) {
          nav(`/game/${gameId}`);
        }
      }
    });
  }, [nav, subscribe]);

  return (
    <div className="min-h-full bg-zinc-950 text-white">
      <div className="mx-auto max-w-5xl p-6">
        <div className="flex items-center justify-between">
          <div>
            <div className="text-2xl font-semibold">Lobby</div>
            <div className="text-white/60">Waiting for players…</div>
          </div>
          <ConnectionBadge />
        </div>

        <div className="mt-6 grid gap-6 md:grid-cols-3">
          <div className="md:col-span-1 rounded-2xl border border-white/10 bg-white/5 p-5">
            <div className="text-sm text-white/70">Players waiting</div>
            <div className="mt-2 text-4xl font-semibold">{waiting}</div>

            <div className="mt-4 flex gap-3">
              <button
                className="rounded-xl bg-white/10 px-4 py-2 text-sm hover:bg-white/15"
                onClick={() => send("queue_join", {})}
              >
                Re-join queue
              </button>
              <button
                className="rounded-xl bg-white/5 px-4 py-2 text-sm hover:bg-white/10"
                onClick={() => {
                  send("queue_leave", {});
                  nav("/");
                }}
              >
                Leave
              </button>
            </div>
          </div>

          <div className="md:col-span-2 h-[520px]">
            <ChatPanel scope="lobby" />
          </div>
        </div>
      </div>
    </div>
  );
}