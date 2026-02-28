import { useWs } from "../net/WebSocketProvider";

export function ConnectionBadge() {
  const { state, clientId } = useWs();

  const label =
    state === "connected"
      ? "Connected"
      : state === "connecting"
      ? "Connecting"
      : state === "reconnecting"
      ? "Reconnecting"
      : "Disconnected";

  const dotClass =
    state === "connected"
      ? "bg-green-500"
      : state === "connecting" || state === "reconnecting"
      ? "bg-yellow-500"
      : "bg-red-500";

  return (
    <div className="flex items-center gap-2 rounded-xl border border-white/10 bg-white/5 px-3 py-1 text-sm">
      <span className={`h-2 w-2 rounded-full ${dotClass}`} />
      <span className="text-white/90">{label}</span>
      <span className="text-white/40">•</span>
      <span className="text-white/60">{clientId ?? "no-id"}</span>
    </div>
  );
}