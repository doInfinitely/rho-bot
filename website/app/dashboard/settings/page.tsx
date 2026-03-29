"use client";

import { useState } from "react";

export default function SettingsPage() {
  const [apiUrl, setApiUrl] = useState(
    process.env.NEXT_PUBLIC_API_URL || "http://localhost:8000"
  );

  return (
    <div className="space-y-8 max-w-lg">
      <h1 className="text-2xl font-bold">Settings</h1>

      <div className="space-y-4">
        <div>
          <label className="block text-xs text-neutral-400 mb-1">
            Server API URL
          </label>
          <input
            type="text"
            value={apiUrl}
            onChange={(e) => setApiUrl(e.target.value)}
            className="w-full px-4 py-2.5 text-sm bg-neutral-900 border border-neutral-800 rounded-lg focus:outline-none focus:border-rho-500 transition-colors"
          />
          <p className="text-xs text-neutral-500 mt-1">
            The base URL of the rho-bot server your desktop client connects to.
          </p>
        </div>

        <div>
          <h3 className="text-sm font-medium mb-2">Desktop Client</h3>
          <p className="text-xs text-neutral-500 mb-3">
            Download the desktop client for macOS to start capturing context
            and receiving action predictions.
          </p>
          <a
            href="/api/releases/download?arch=arm64"
            className="inline-block px-4 py-2 text-sm font-medium rounded-lg bg-rho-600 hover:bg-rho-700 transition-colors"
          >
            Download for Apple Silicon
          </a>
          <p className="text-xs text-neutral-500 mt-2">
            Latest macOS release from GitHub. If you have an Intel Mac, use{" "}
            <a
              href="/api/releases/download?arch=x64"
              className="text-rho-400 hover:text-rho-300 transition-colors"
            >
              the Intel build
            </a>
            .
          </p>
        </div>

        <div>
          <h3 className="text-sm font-medium mb-2">Danger Zone</h3>
          <button className="px-4 py-2 text-sm font-medium rounded-lg border border-red-800 text-red-400 hover:bg-red-900/20 transition-colors">
            Delete Account
          </button>
        </div>
      </div>
    </div>
  );
}
