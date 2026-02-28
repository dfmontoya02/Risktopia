import { useEffect, useState } from "react";
import { useParams } from "react-router-dom";
import { ConnectionBadge } from "../components/ConnectionBadge";
import { ChatPanel } from "../components/ChatPanel";
import { useWs } from "../net/WebSocketProvider";

export function GamePage() {
  const { gameId } = useParams();
  const { lastMessage } = useWs();
  const [stateVersion, setStateVersion] = useState<number>(0);

  useEffect(() => {
    if (!lastMessage) return;
    if (lastMessage.type !== "state_update") return;
    const v = (lastMessage as any).payload.state_version ?? 0;
    setStateVersion(v);
  }, [lastMessage]);

  return (
    <div className="min-h-full bg-zinc-950 text-white">
      <div className="mx-auto max-w-6xl p-6">
        <div className="flex items-center justify-between">
          <div>
            <div className="text-2xl font-semibold">Game</div>
            <div className="text-white/60">
              Game ID: <span className="text-white/80">{gameId}</span> • State v{stateVersion}
            </div>
          </div>
          <ConnectionBadge />
        </div>

        <div className="mt-6 grid gap-6 lg:grid-cols-12">
          {/* Left: status */}
          <div className="lg:col-span-3 rounded-2xl border border-white/10 bg-white/5 p-5">
            <div className="font-semibold">Status</div>
            <div className="mt-2 text-sm text-white/60">
              Placeholder for player list, turn, phase, actions.
            </div>

            <div className="mt-4 rounded-xl border border-white/10 bg-black/20 p-4 text-sm text-white/70">
              Coming next:
              <ul className="mt-2 list-disc space-y-1 pl-5">
                <li>Render territories + troop counts</li>
                <li>Click territory selection</li>
                <li>Send intents over WS</li>
              </ul>
            </div>
          </div>

          {/* Center: board */}
          <div className="lg:col-span-6 rounded-2xl border border-white/10 bg-white/5 p-5">
            <div className="font-semibold">Board</div>
            <div className="mt-3 flex h-[520px] items-center justify-center rounded-xl border border-white/10 bg-black/20 text-white/50">
              2D board placeholder (Canvas / PixiJS later)
            </div>
          </div>

          {/* Right: chat */}
          <div className="lg:col-span-3 h-[600px]">
            <ChatPanel scope="game" gameId={gameId} />
          </div>
        </div>
      </div>
    </div>
  );
}