import { invoke } from "@tauri-apps/api/core";
import type { AgentState } from "../App";

const stateColors: Record<AgentState, string> = {
  disconnected: "bg-red-500",
  connected: "bg-yellow-400",
  running: "bg-emerald-400",
  paused: "bg-neutral-400",
};

const stateLabels: Record<AgentState, string> = {
  disconnected: "Disconnected",
  connected: "Connected (idle)",
  running: "Running",
  paused: "Paused",
};

export default function StatusPanel({ state }: { state: AgentState }) {
  const handleStart = async () => {
    try {
      await invoke("start_agent");
    } catch (e) {
      console.error("Failed to start agent:", e);
    }
  };

  const handleStop = async () => {
    try {
      await invoke("stop_agent");
    } catch (e) {
      console.error("Failed to stop agent:", e);
    }
  };

  return (
    <div className="space-y-6">
      {/* Status indicator */}
      <div className="flex items-center gap-3">
        <div className={`w-3 h-3 rounded-full ${stateColors[state]} animate-pulse`} />
        <span className="text-sm font-medium">{stateLabels[state]}</span>
      </div>

      {/* Controls */}
      <div className="flex gap-2">
        <button
          onClick={handleStart}
          disabled={state === "running"}
          className="px-4 py-2 text-sm font-medium rounded-lg bg-rho-600 hover:bg-rho-700 disabled:opacity-40 disabled:cursor-not-allowed transition-colors"
        >
          Start Agent
        </button>
        <button
          onClick={handleStop}
          disabled={state === "disconnected"}
          className="px-4 py-2 text-sm font-medium rounded-lg bg-neutral-800 hover:bg-neutral-700 disabled:opacity-40 disabled:cursor-not-allowed transition-colors"
        >
          Stop
        </button>
      </div>

      {/* Info */}
      <div className="text-xs text-neutral-500 space-y-1">
        <p>
          The agent captures your screen and accessibility tree, sends context
          to the rho-bot server, and executes predicted actions.
        </p>
        <p>
          Requires Accessibility and Screen Recording permissions in System
          Settings &gt; Privacy &amp; Security.
        </p>
      </div>
    </div>
  );
}
