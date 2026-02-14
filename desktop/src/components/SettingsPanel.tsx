import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

interface Settings {
  server_url: string;
  auth_token: string;
  capture_interval_ms: number;
}

const defaults: Settings = {
  server_url: "ws://localhost:8000/ws/agent",
  auth_token: "",
  capture_interval_ms: 500,
};

export default function SettingsPanel() {
  const [settings, setSettings] = useState<Settings>(defaults);
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    invoke<Settings>("get_settings")
      .then((s) => setSettings(s))
      .catch(() => {});
  }, []);

  const handleSave = async () => {
    try {
      await invoke("save_settings", { settings });
      setSaved(true);
      setTimeout(() => setSaved(false), 2000);
    } catch (e) {
      console.error("Failed to save settings:", e);
    }
  };

  return (
    <div className="space-y-4 max-w-md">
      <div>
        <label className="block text-xs text-neutral-400 mb-1">
          Server URL
        </label>
        <input
          type="text"
          value={settings.server_url}
          onChange={(e) =>
            setSettings((s) => ({ ...s, server_url: e.target.value }))
          }
          className="w-full px-3 py-2 text-sm bg-neutral-900 border border-neutral-800 rounded-lg focus:outline-none focus:border-rho-500"
        />
      </div>

      <div>
        <label className="block text-xs text-neutral-400 mb-1">
          Auth Token (JWT)
        </label>
        <input
          type="password"
          value={settings.auth_token}
          onChange={(e) =>
            setSettings((s) => ({ ...s, auth_token: e.target.value }))
          }
          className="w-full px-3 py-2 text-sm bg-neutral-900 border border-neutral-800 rounded-lg focus:outline-none focus:border-rho-500"
        />
      </div>

      <div>
        <label className="block text-xs text-neutral-400 mb-1">
          Capture Interval (ms)
        </label>
        <input
          type="number"
          min={100}
          max={5000}
          step={100}
          value={settings.capture_interval_ms}
          onChange={(e) =>
            setSettings((s) => ({
              ...s,
              capture_interval_ms: parseInt(e.target.value) || 500,
            }))
          }
          className="w-full px-3 py-2 text-sm bg-neutral-900 border border-neutral-800 rounded-lg focus:outline-none focus:border-rho-500"
        />
      </div>

      <button
        onClick={handleSave}
        className="px-4 py-2 text-sm font-medium rounded-lg bg-rho-600 hover:bg-rho-700 transition-colors"
      >
        {saved ? "Saved!" : "Save Settings"}
      </button>
    </div>
  );
}
