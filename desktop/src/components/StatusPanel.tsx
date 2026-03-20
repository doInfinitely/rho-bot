import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { AgentState } from "../App";

const stateColors: Record<AgentState, string> = {
  disconnected: "bg-red-500",
  connecting: "bg-yellow-400",
  connected: "bg-yellow-400",
  reconnecting: "bg-yellow-400",
  running: "bg-emerald-400",
  recording: "bg-blue-400",
  paused: "bg-neutral-400",
  quota_exceeded: "bg-amber-500",
};

const stateLabels: Record<AgentState, string> = {
  disconnected: "Disconnected",
  connecting: "Connecting…",
  connected: "Connected (idle)",
  reconnecting: "Reconnecting…",
  running: "Running",
  recording: "Recording",
  paused: "Paused",
  quota_exceeded: "Usage Limit Reached",
};

interface Permissions {
  screen_recording: boolean;
  accessibility: boolean;
}

interface Props {
  state: AgentState;
  error?: string;
}

export default function StatusPanel({ state, error }: Props) {
  const [perms, setPerms] = useState<Permissions | null>(null);

  useEffect(() => {
    const check = () => {
      invoke<Permissions>("check_permissions")
        .then(setPerms)
        .catch(() => {});
    };
    check();
    const interval = setInterval(check, 5000);
    return () => clearInterval(interval);
  }, []);

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

  const agentBusy =
    state === "running" ||
    state === "connecting" ||
    state === "reconnecting";
  const missingPerms =
    perms && (!perms.screen_recording || !perms.accessibility);

  return (
    <div className="space-y-6">
      {/* Permissions warning */}
      {missingPerms && (
        <div className="rounded-lg border border-amber-500/30 bg-amber-500/10 px-4 py-3 text-sm space-y-2">
          <p className="font-medium text-amber-200">Permissions needed</p>
          <p className="text-xs text-amber-300/80">
            Open <strong>System Settings &gt; Privacy &amp; Security</strong>{" "}
            and enable:
          </p>
          <ul className="text-xs text-amber-300/80 list-disc list-inside space-y-1">
            {!perms.screen_recording && (
              <li>
                <strong>Screen Recording</strong> — required to capture your
                screen
              </li>
            )}
            {!perms.accessibility && (
              <li>
                <strong>Accessibility</strong> — required to read UI elements
                and execute actions
              </li>
            )}
          </ul>
          <p className="text-[10px] text-amber-400/60">
            You may need to restart the app after granting permissions.
          </p>
        </div>
      )}

      {/* Status indicator */}
      <div className="flex items-center gap-3">
        <div
          className={`w-3 h-3 rounded-full ${stateColors[state]} ${
            state === "connecting" || state === "reconnecting"
              ? "animate-pulse"
              : ""
          }`}
        />
        <span className="text-sm font-medium">{stateLabels[state]}</span>
      </div>

      {/* Recording indicator */}
      {(state === "recording" || state === "disconnected") && (
        <div className="rounded-lg border border-blue-500/20 bg-blue-500/5 px-4 py-3 text-xs text-blue-300/80">
          {state === "recording"
            ? "Recording your actions in the background."
            : "Recording will start automatically when connected."}
        </div>
      )}

      {/* Error banner */}
      {error && state !== "running" && (
        <div className="rounded-lg border border-red-500/30 bg-red-500/10 px-4 py-3 text-sm space-y-1">
          <p className="font-medium text-red-300">Error</p>
          <p className="text-xs text-red-300/80 font-mono break-all">
            {error}
          </p>
        </div>
      )}

      {/* Controls — agent only */}
      <div className="flex gap-2">
        <button
          onClick={handleStart}
          disabled={agentBusy}
          className="px-4 py-2 text-sm font-medium rounded-lg bg-rho-600 hover:bg-rho-700 disabled:opacity-40 disabled:cursor-not-allowed transition-colors"
        >
          Start Agent
        </button>
        <button
          onClick={handleStop}
          disabled={state !== "running" && state !== "connecting" && state !== "reconnecting"}
          className="px-4 py-2 text-sm font-medium rounded-lg bg-neutral-800 hover:bg-neutral-700 disabled:opacity-40 disabled:cursor-not-allowed transition-colors"
        >
          Stop Agent
        </button>
      </div>

      {/* Quota exceeded banner */}
      {state === "quota_exceeded" && (
        <div className="rounded-lg border border-amber-500/30 bg-amber-500/10 px-4 py-3 text-sm text-amber-200 space-y-1">
          <p className="font-medium">Usage limit reached</p>
          <p className="text-xs text-amber-300/80">
            You&apos;ve used all the tasks included in your current plan.
            Upgrade or wait for the next billing cycle.
          </p>
        </div>
      )}

      {/* Info */}
      <div className="text-xs text-neutral-500 space-y-1">
        <p>
          rho-bot continuously records your screen and actions in the background.
          Start the agent to have it execute tasks for you.
        </p>
        <p>
          Requires Accessibility and Screen Recording permissions in System
          Settings &gt; Privacy &amp; Security.
        </p>
      </div>
    </div>
  );
}
