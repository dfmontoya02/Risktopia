type ActionContext =
  | { kind: "setup" }
  | { kind: "reinforcement"; troops_remaining: number }
  | { kind: "attack"; captured_this_turn: boolean }
  | { kind: "capture_move"; from: number; to: number; min_troops: number; max_troops: number }
  | { kind: "fortify"; used_fortify: boolean }
  | { kind: "game_over"; winner_player_id: number }
  | null;

type Props = {
  phase: string;
  yourTurn: boolean;
  selectionLabel: string;
  onClear: () => void;
  onRefresh: () => void;
  onEndTurn: () => void;
  endTurnLabel: string;
  endTurnDisabled: boolean;
  onSubmit: () => void;
  submitDisabled: boolean;

  attackDice: number;
  setAttackDice: (value: number) => void;

  reinforceCount: number;
  setReinforceCount: (value: number) => void;

  fortifyCount: number;
  setFortifyCount: (value: number) => void;

  captureMoveCount: number;
  setCaptureMoveCount: (value: number) => void;

  maxAttackDice: number;
  maxReinforceCount: number;
  maxFortifyCount: number;

  actionContext: ActionContext;
};

export function ActionBar(props: Props) {
  return (
    <div className="mt-4 rounded-xl border border-white/10 bg-black/20 p-3">
      <div className="flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between">
        <div className="text-sm text-white/80">
          <span className="font-semibold">Phase:</span> {props.phase}
          <span className="ml-3 font-semibold">Selection:</span> {props.selectionLabel}
          {!props.yourTurn ? <span className="ml-3 text-amber-300">Not your turn</span> : null}
        </div>

        <div className="flex flex-wrap items-end gap-3">
        {props.actionContext?.kind === "capture_move" ? (() => {
            const captureContext = props.actionContext;
            return (
              <div className="flex flex-col gap-1 rounded-lg border border-amber-400/20 bg-amber-400/10 px-3 py-2 text-sm text-amber-100">
                <div className="font-semibold">Move troops into captured territory</div>
                <div>
                  From {captureContext.from} → {captureContext.to}
                </div>
                <div>
                  Min: {captureContext.min_troops} • Max: {captureContext.max_troops}
                </div>
                <label className="mt-1 text-sm text-white/90">
                  Troops
                  <input
                    type="number"
                    min={captureContext.min_troops}
                    max={captureContext.max_troops}
                    value={props.captureMoveCount}
                    onChange={(e) =>
                      props.setCaptureMoveCount(
                        Math.max(
                          captureContext.min_troops,
                          Math.min(Number(e.target.value), captureContext.max_troops),
                        ),
                      )
                    }
                    className="ml-2 w-20 rounded-lg border border-white/10 bg-zinc-900 px-2 py-1 text-white"
                  />
                </label>
              </div>
            );
          })() : null}
          
          {props.phase === "Reinforcement" && props.actionContext?.kind === "reinforcement" ? (
            <label className="text-sm text-white/80">
              Troops
              <input
                type="number"
                min={1}
                max={props.maxReinforceCount}
                value={props.reinforceCount}
                onChange={(e) =>
                  props.setReinforceCount(
                    Math.max(1, Math.min(Number(e.target.value), props.maxReinforceCount)),
                  )
                }
                className="ml-2 w-20 rounded-lg border border-white/10 bg-zinc-900 px-2 py-1 text-white"
              />
            </label>
          ) : null}

          {props.phase === "Attack" && props.actionContext?.kind === "attack" ? (
            <label className="text-sm text-white/80">
              Dice
              <select
                value={Math.min(props.attackDice, props.maxAttackDice)}
                onChange={(e) =>
                  props.setAttackDice(
                    Math.max(1, Math.min(Number(e.target.value), props.maxAttackDice)),
                  )
                }
                className="ml-2 rounded-lg border border-white/10 bg-zinc-900 px-2 py-1 text-white"
              >
                {Array.from({ length: props.maxAttackDice }, (_, i) => i + 1).map((value) => (
                  <option key={value} value={value}>
                    {value}
                  </option>
                ))}
              </select>
            </label>
          ) : null}

          {props.phase === "Fortify" ? (
            <label className="text-sm text-white/80">
              Troops
              <input
                type="number"
                min={1}
                max={props.maxFortifyCount}
                value={props.fortifyCount}
                onChange={(e) =>
                  props.setFortifyCount(
                    Math.max(1, Math.min(Number(e.target.value), props.maxFortifyCount)),
                  )
                }
                className="ml-2 w-20 rounded-lg border border-white/10 bg-zinc-900 px-2 py-1 text-white"
              />
            </label>
          ) : null}

          <button
            className="rounded-lg bg-white/10 px-3 py-2 text-sm hover:bg-white/15"
            onClick={props.onClear}
          >
            Clear
          </button>

          <button
            className="rounded-lg bg-sky-600 px-3 py-2 text-sm font-semibold text-white hover:bg-sky-500"
            onClick={props.onRefresh}
          >
            Refresh state
          </button>

          <button
            className="rounded-lg bg-violet-600 px-3 py-2 text-sm font-semibold text-white disabled:cursor-not-allowed disabled:opacity-50"
            disabled={props.endTurnDisabled}
            onClick={props.onEndTurn}
          >
            {props.endTurnLabel}
          </button>

          <button
            className="rounded-lg bg-emerald-600 px-4 py-2 text-sm font-semibold text-white disabled:cursor-not-allowed disabled:opacity-50"
            disabled={props.submitDisabled}
            onClick={props.onSubmit}
          >
            Submit
          </button>
        </div>
      </div>
    </div>
  );
}