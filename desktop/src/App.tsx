import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import LoginScreen from "./components/LoginScreen";
import StatusPanel from "./components/StatusPanel";
import SettingsPanel from "./components/SettingsPanel";
import ActivityLog from "./components/ActivityLog";

export type AgentState = "disconnected" | "connected" | "running" | "recording" | "paused" | "quota_exceeded";

export interface ActionEntry {
  id: string;
  type: string;
  timestamp: number;
  confidence: number;
  error?: string | null;
}

interface AuthState {
  logged_in: boolean;
  email: string;
  server_url: string;
}

export default function App() {
  const [authChecked, setAuthChecked] = useState(false);
  const [loggedIn, setLoggedIn] = useState(false);
  const [userEmail, setUserEmail] = useState("");

  const [tab, setTab] = useState<"status" | "activity" | "settings">("status");
  const [agentState, setAgentState] = useState<AgentState>("disconnected");
  const [actions, setActions] = useState<ActionEntry[]>([]);

  // Check auth on mount
  useEffect(() => {
    invoke<AuthState>("get_auth_state")
      .then((s) => {
        setLoggedIn(s.logged_in);
        setUserEmail(s.email);
      })
      .catch(() => {})
      .finally(() => setAuthChecked(true));
  }, []);

  // Poll agent state when logged in
  useEffect(() => {
    if (!loggedIn) return;

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
  }, [loggedIn]);

  const handleLogout = async () => {
    try {
      await invoke("logout");
    } catch {
      // ignore
    }
    setLoggedIn(false);
    setUserEmail("");
    setAgentState("disconnected");
    setActions([]);
    setTab("status");
  };

  // Don't flash anything while checking auth
  if (!authChecked) {
    return (
      <div className="min-h-screen bg-neutral-950 flex items-center justify-center">
        <div className="w-5 h-5 border-2 border-rho-400 border-t-transparent rounded-full animate-spin" />
      </div>
    );
  }

  // Not logged in -> show login
  if (!loggedIn) {
    return (
      <LoginScreen
        onLogin={(email) => {
          setLoggedIn(true);
          setUserEmail(email);
        }}
      />
    );
  }

  // Logged in -> main app
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

        {/* User indicator + logout */}
        <div className="ml-auto flex items-center gap-2">
          <span className="text-[10px] text-neutral-500 truncate max-w-[120px]">
            {userEmail}
          </span>
          <button
            onClick={handleLogout}
            className="text-[10px] text-neutral-600 hover:text-red-400 transition-colors"
          >
            Sign out
          </button>
        </div>
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
