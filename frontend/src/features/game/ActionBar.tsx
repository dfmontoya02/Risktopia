type ActionContext =
  | { kind: "setup" }
  | { kind: "reinforcement"; troops_remaining: number }
  | { kind: "attack"; captured_this_turn: boolean }
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
          {props.phase === "Reinforcement" && props.actionContext?.kind === "reinforcement" ? (
            <label className="text-sm text-white/80">
              Troops
              <input
                type="number"
                min={1}
                max={props.actionContext.troops_remaining}
                value={props.reinforceCount}
                onChange={(e) => props.setReinforceCount(Number(e.target.value))}
                className="ml-2 w-20 rounded-lg border border-white/10 bg-zinc-900 px-2 py-1 text-white"
              />
            </label>
          ) : null}

          {props.phase === "Attack" ? (
            <label className="text-sm text-white/80">
              Dice
              <select
                value={props.attackDice}
                onChange={(e) => props.setAttackDice(Number(e.target.value))}
                className="ml-2 rounded-lg border border-white/10 bg-zinc-900 px-2 py-1 text-white"
              >
                <option value={1}>1</option>
                <option value={2}>2</option>
                <option value={3}>3</option>
              </select>
            </label>
          ) : null}

          {props.phase === "Fortify" ? (
            <label className="text-sm text-white/80">
              Troops
              <input
                type="number"
                min={1}
                value={props.fortifyCount}
                onChange={(e) => props.setFortifyCount(Number(e.target.value))}
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