import type { ActionEntry } from "../App";

export default function ActivityLog({ actions }: { actions: ActionEntry[] }) {
  if (actions.length === 0) {
    return (
      <p className="text-sm text-neutral-500">No actions recorded yet.</p>
    );
  }

  return (
    <div className="space-y-1">
      {actions.map((a) =>
        a.error ? (
          <div
            key={a.id}
            className="px-3 py-2 rounded-lg bg-amber-500/10 border border-amber-500/20 text-xs text-amber-300"
          >
            {a.error}
          </div>
        ) : (
          <div
            key={a.id}
            className="flex items-center justify-between px-3 py-2 rounded-lg bg-neutral-900 text-xs"
          >
            <div className="flex items-center gap-2">
              <span className="font-mono text-rho-400">{a.type}</span>
              <span className="text-neutral-500">
                {new Date(a.timestamp * 1000).toLocaleTimeString()}
              </span>
            </div>
            <span className="text-neutral-500">
              {(a.confidence * 100).toFixed(0)}%
            </span>
          </div>
        )
      )}
    </div>
  );
}
