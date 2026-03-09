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
};

export function ActionBar(props: Props) {
  return (
    <div className="mt-4 flex items-center justify-between gap-3 rounded-xl border border-white/10 bg-black/20 p-3">
      <div className="text-sm text-white/80">
        <span className="font-semibold">Phase:</span> {props.phase}{" "}
        <span className="ml-3 font-semibold">Selection:</span> {props.selectionLabel}
        {!props.yourTurn ? <span className="ml-3 text-amber-300">Not your turn</span> : null}
      </div>
      <div className="flex gap-2">
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
  );
}
