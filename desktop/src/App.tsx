import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import StatusPanel from "./components/StatusPanel";
import SettingsPanel from "./components/SettingsPanel";
import ActivityLog from "./components/ActivityLog";

export type AgentState = "disconnected" | "connected" | "running" | "paused";

export interface ActionEntry {
  id: string;
  type: string;
  timestamp: number;
  confidence: number;
}

export default function App() {
  const [tab, setTab] = useState<"status" | "activity" | "settings">("status");
  const [agentState, setAgentState] = useState<AgentState>("disconnected");
  const [actions, setActions] = useState<ActionEntry[]>([]);

  // Poll agent state from the Rust backend
  useEffect(() => {
    const interval = setInterval(async () => {
      try {
        const state = await invoke<string>("get_agent_state");
        setAgentState(state as AgentState);
      } catch {
        // Tauri command not available yet
      }
      try {
        const log = await invoke<ActionEntry[]>("get_recent_actions");
        setActions(log);
      } catch {
        // Tauri command not available yet
      }
    }, 1000);
    return () => clearInterval(interval);
  }, []);

  const tabs = [
    { id: "status" as const, label: "Status" },
    { id: "activity" as const, label: "Activity" },
    { id: "settings" as const, label: "Settings" },
  ];

  return (
    <div className="min-h-screen bg-neutral-950 text-neutral-100 flex flex-col">
      {/* Titlebar / tab row */}
      <div className="titlebar-drag flex items-center gap-1 px-3 pt-3 pb-1">
        <span className="text-sm font-semibold tracking-tight text-rho-400 mr-3">
          rho-bot
        </span>
        {tabs.map((t) => (
          <button
            key={t.id}
            onClick={() => setTab(t.id)}
            className={`px-3 py-1 text-xs rounded-md transition-colors ${
              tab === t.id
                ? "bg-neutral-800 text-neutral-100"
                : "text-neutral-500 hover:text-neutral-300"
            }`}
          >
            {t.label}
          </button>
        ))}
      </div>

      {/* Content */}
      <div className="flex-1 p-4 overflow-auto">
        {tab === "status" && <StatusPanel state={agentState} />}
        {tab === "activity" && <ActivityLog actions={actions} />}
        {tab === "settings" && <SettingsPanel />}
      </div>
    </div>
  );
}
