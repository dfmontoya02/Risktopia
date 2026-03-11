import { useCallback, useEffect, useRef, useState } from "react";
import { useParams } from "react-router-dom";
import { ConnectionBadge } from "../components/ConnectionBadge";
import { ChatPanel } from "../components/ChatPanel";
import { useWs } from "../net/WebSocketProvider";
import type { GameView } from "../types/messages";
import { ActionBar } from "../features/game/ActionBar";
import { ADJACENCY, PLAYER_COLORS } from "../features/game/mapData";
import { MapBoardSvg } from "../features/game/MapBoardSvg";

export function GamePage() {
  const { gameId } = useParams();
  const { subscribe, send, state } = useWs();
  const [stateVersion, setStateVersion] = useState<number>(0);
  const [view, setView] = useState<GameView | null>(null);
  const [selectedFrom, setSelectedFrom] = useState<number | null>(null);
  const [selectedTo, setSelectedTo] = useState<number | null>(null);
  const lastConnectionState = useRef(state);

  useEffect(() => {
    return subscribe((message) => {
      if (message.type === "game_view") {
        const incomingGameId = (message as any).payload?.game_id as string | undefined;
        if (incomingGameId && incomingGameId !== gameId) return;
        setStateVersion((message as any).payload.state_version ?? 0);
        setView((message as any).payload.view as GameView);
        setSelectedFrom(null);
        setSelectedTo(null);
      } else if (message.type === "state_update") {
        const incomingGameId = (message as any).payload?.state?.game_id as string | undefined;
        if (incomingGameId && incomingGameId !== gameId) return;
        const v = (message as any).payload.state_version ?? 0;
        setStateVersion(v);
      }
    });
  }, [gameId, subscribe]);

  const requestFreshState = useCallback(() => {
    if (!gameId) return;
    send("state_refresh", { game_id: gameId });
  }, [gameId, send]);

  useEffect(() => {
    requestFreshState();
  }, [requestFreshState]);

  useEffect(() => {
    const wasConnected = lastConnectionState.current === "connected";
    const nowConnected = state === "connected";
    if (nowConnected && !wasConnected) {
      requestFreshState();
    }
    lastConnectionState.current = state;
  }, [requestFreshState, state]);

  function getTerritory(id: number) {
    return view?.territories.find((territory) => territory.id === id);
  }

  function onTerritoryClick(id: number) {
    if (!view) return;
    if (view.you_player_id !== view.current_player_id) return;
    const territory = getTerritory(id);
    if (!territory) return;

    const you = view.you_player_id;
    const phase = view.turn_phase;
    if (phase === "Setup" || phase === "Reinforcement") {
      if (territory.owner_player_id !== you && territory.troops > 0) return;
      setSelectedFrom(id);
      setSelectedTo(null);
      return;
    }

    if (phase === "Attack") {
      if (selectedFrom == null) {
        if (territory.owner_player_id !== you || territory.troops <= 1) return;
        setSelectedFrom(id);
        setSelectedTo(null);
        return;
      }

      if (id === selectedFrom) {
        setSelectedFrom(null);
        setSelectedTo(null);
        return;
      }

      const adjacent = ADJACENCY.get(selectedFrom);
      if (!adjacent?.has(id)) return;
      if (territory.owner_player_id === you) return;
      setSelectedTo(id);
      return;
    }

    if (phase === "Fortify") {
      if (selectedFrom == null) {
        if (territory.owner_player_id !== you || territory.troops <= 1) return;
        setSelectedFrom(id);
        setSelectedTo(null);
        return;
      }
      if (id === selectedFrom) {
        setSelectedFrom(null);
        setSelectedTo(null);
        return;
      }
      const adjacent = ADJACENCY.get(selectedFrom);
      if (!adjacent?.has(id)) return;
      if (territory.owner_player_id !== you) return;
      setSelectedTo(id);
    }
  }

  function submitAction() {
    if (!view || !gameId) return;
    if (view.you_player_id !== view.current_player_id) return;

    if (view.turn_phase === "Setup" || view.turn_phase === "Reinforcement") {
      if (selectedFrom == null) return;
      send("game_action", {
        game_id: gameId,
        action: { PlaceTroops: { territory: selectedFrom, count: 1 } },
      });
      return;
    }

    if (view.turn_phase === "Attack") {
      if (selectedFrom == null || selectedTo == null) return;
      const from = getTerritory(selectedFrom);
      if (!from || from.troops <= 1) return;
      const maxDice = Math.min(3, from.troops - 1);
      send("game_action", {
        game_id: gameId,
        action: { Attack: { from: selectedFrom, to: selectedTo, dice: maxDice } },
      });
      return;
    }

    if (view.turn_phase === "Fortify") {
      if (selectedFrom == null || selectedTo == null) return;
      const from = getTerritory(selectedFrom);
      if (!from || from.troops <= 1) return;
      send("game_action", {
        game_id: gameId,
        action: {
          Fortify: {
            from: selectedFrom,
            to: selectedTo,
            count: from.troops - 1,
          },
        },
      });
    }
  }

  function endTurnAction() {
    if (!view || !gameId) return;
    if (view.you_player_id !== view.current_player_id) return;

    if (view.turn_phase === "Attack") {
      send("game_action", { game_id: gameId, action: { EndAttack: null } });
      return;
    }

    if (view.turn_phase === "Fortify") {
      send("game_action", { game_id: gameId, action: { EndTurn: null } });
    }
  }

  const yourTurn = !!view && view.you_player_id === view.current_player_id;
  const submitDisabled =
    !view ||
    !yourTurn ||
    ((view.turn_phase === "Setup" || view.turn_phase === "Reinforcement") && selectedFrom == null) ||
    ((view.turn_phase === "Attack" || view.turn_phase === "Fortify") &&
      (selectedFrom == null || selectedTo == null));
  const canEndTurn = !!view && yourTurn && (view.turn_phase === "Attack" || view.turn_phase === "Fortify");
  const endTurnLabel = view?.turn_phase === "Attack" ? "End attack" : "End turn";

  const selectionLabel =
    selectedFrom == null
      ? "none"
      : selectedTo == null
        ? `${selectedFrom}`
        : `${selectedFrom} -> ${selectedTo}`;

  function playerColor(playerId: number): string {
    if (playerId < 0 || playerId >= 250) return "#6b7280";
    return PLAYER_COLORS[playerId % PLAYER_COLORS.length];
  }

  useEffect(() => {
    function isTypingTarget(target: EventTarget | null): boolean {
      if (!(target instanceof HTMLElement)) return false;
      const tag = target.tagName;
      return (
        tag === "INPUT" ||
        tag === "TEXTAREA" ||
        tag === "SELECT" ||
        target.isContentEditable
      );
    }

    function onKeyDown(event: KeyboardEvent) {
      if (event.key !== "Enter" || event.repeat) return;
      if (event.altKey || event.ctrlKey || event.metaKey || event.shiftKey) return;
      if (isTypingTarget(event.target)) return;
      if (submitDisabled) return;
      event.preventDefault();
      submitAction();
    }

    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [submitAction, submitDisabled]);

  return (
    <div className="min-h-full bg-zinc-950 text-white">
      <div className="mx-auto p-6">
        <div className="flex items-center justify-between">
          <div>
            <div className="text-2xl font-semibold">Game</div>
            <div className="text-white/60">
              Game ID: <span className="text-white/80">{gameId}</span> • State v{stateVersion}
            </div>
            <div className="mt-1 text-sm text-white/70">
              You are:{" "}
              <span className="font-semibold text-white">
                {view ? `P${view.you_player_id}` : "syncing..."}
              </span>
            </div>
          </div>
          <ConnectionBadge />
        </div>

        <div className="mt-6 grid gap-6 lg:grid-cols-12">
          {/* Left: status */}
          <div className="lg:col-span-3 rounded-2xl border border-white/10 bg-white/5 p-5">
            <div className="font-semibold">Status</div>
            <div className="mt-2 text-sm text-white/70">
              Turn: {view ? `${view.current_player_id}` : "-"} • Phase: {view?.turn_phase ?? "-"}
            </div>
            {!view ? (
              <div className="mt-2 text-sm text-amber-300">Waiting for state sync...</div>
            ) : null}
            <div className="mt-4 rounded-xl border border-white/10 bg-black/20 p-4 text-sm text-white/70">
              <div className="font-semibold text-white">Your cards ({view?.your_cards.length ?? 0})</div>
              <div className="mt-2 max-h-40 overflow-auto text-xs text-white/70">
                {(view?.your_cards ?? []).map((card) => (
                  <div key={card.card_id}>
                    #{card.card_id} {card.card_type} (T{card.territory_id})
                  </div>
                ))}
              </div>
              <div className="mt-3 font-semibold text-white">Player card counts</div>
              <div className="mt-1 text-xs text-white/70">
                {(view?.players_public ?? []).map((player) => (
                  <div key={player.player_id} className="flex items-center gap-2">
                    <span
                      className="inline-block h-2.5 w-2.5 rounded-full border border-white/20"
                      style={{ backgroundColor: playerColor(player.player_id) }}
                    />
                    P{player.player_id}: {player.card_count}
                  </div>
                ))}
              </div>
              {view?.latest_combat_roll ? (
                <div className="mt-3 text-xs text-amber-200">
                  Latest roll {view.latest_combat_roll.attacker_rolls.join(" ")} {"->"}{" "}
                  {view.latest_combat_roll.defender_rolls.join(" ")}
                </div>
              ) : null}
            </div>
          </div>

          {/* Center: board */}
          <div className="lg:col-span-6 rounded-2xl border border-white/10 bg-white/5 p-5">
            <div className="font-semibold">Board</div>
            <div className="mt-3">
              <MapBoardSvg
                territories={view?.territories ?? []}
                selectedFrom={selectedFrom}
                selectedTo={selectedTo}
                onTerritoryClick={onTerritoryClick}
              />
              <ActionBar
                phase={view?.turn_phase ?? "-"}
                yourTurn={yourTurn}
                selectionLabel={selectionLabel}
                submitDisabled={submitDisabled}
                onClear={() => {
                  setSelectedFrom(null);
                  setSelectedTo(null);
                }}
                onRefresh={requestFreshState}
                onEndTurn={endTurnAction}
                endTurnLabel={endTurnLabel}
                endTurnDisabled={!canEndTurn}
                onSubmit={submitAction}
              />
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