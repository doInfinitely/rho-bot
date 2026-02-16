"use client";

import { useEffect, useState } from "react";
import Link from "next/link";
import { History, Bot, ChevronRight } from "lucide-react";
import { getSessions } from "@/lib/api";
import type { SessionSummary } from "@/lib/api";

export default function SessionsPage() {
  const [sessions, setSessions] = useState<SessionSummary[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    getSessions(50)
      .then(setSessions)
      .catch(console.error)
      .finally(() => setLoading(false));
  }, []);

  return (
    <div>
      <div className="mb-8">
        <h1 className="text-2xl font-bold tracking-tight flex items-center gap-3">
          <History className="w-6 h-6 text-rho-400" />
          Sessions
        </h1>
        <p className="mt-1 text-sm text-neutral-500">
          View all agent and recording sessions.
        </p>
      </div>

      {loading ? (
        <div className="flex items-center justify-center h-64">
          <div className="w-6 h-6 border-2 border-rho-500 border-t-transparent rounded-full animate-spin" />
        </div>
      ) : sessions.length === 0 ? (
        <div className="p-12 rounded-xl border border-neutral-800 bg-neutral-900/50 text-center">
          <Bot className="w-10 h-10 text-neutral-600 mx-auto mb-4" />
          <h3 className="text-lg font-semibold mb-2">No sessions yet</h3>
          <p className="text-sm text-neutral-500 mb-6 max-w-sm mx-auto">
            Install the desktop agent and connect it to start your first
            session.
          </p>
          <Link
            href="/download"
            className="inline-flex items-center gap-2 px-5 py-2.5 text-sm font-medium rounded-lg bg-rho-600 hover:bg-rho-700 transition-colors"
          >
            Download Agent
          </Link>
        </div>
      ) : (
        <div className="border border-neutral-800 rounded-xl overflow-hidden">
          <table className="w-full text-sm">
            <thead>
              <tr className="bg-neutral-900/80">
                <th className="text-left px-4 py-3 text-xs font-medium text-neutral-500 uppercase tracking-wider">
                  Session ID
                </th>
                <th className="text-left px-4 py-3 text-xs font-medium text-neutral-500 uppercase tracking-wider">
                  Started
                </th>
                <th className="text-left px-4 py-3 text-xs font-medium text-neutral-500 uppercase tracking-wider">
                  Ended
                </th>
                <th className="text-left px-4 py-3 text-xs font-medium text-neutral-500 uppercase tracking-wider">
                  Actions
                </th>
                <th className="text-left px-4 py-3 text-xs font-medium text-neutral-500 uppercase tracking-wider">
                  Status
                </th>
                <th className="px-4 py-3" />
              </tr>
            </thead>
            <tbody>
              {sessions.map((s) => (
                <tr
                  key={s.session_id}
                  className="border-t border-neutral-800/50 hover:bg-neutral-900/40 transition-colors"
                >
                  <td className="px-4 py-3 font-mono text-xs text-neutral-400">
                    {s.session_id.slice(0, 8)}&hellip;
                  </td>
                  <td className="px-4 py-3 text-neutral-400">
                    {new Date(s.started_at * 1000).toLocaleString()}
                  </td>
                  <td className="px-4 py-3 text-neutral-400">
                    {s.ended_at
                      ? new Date(s.ended_at * 1000).toLocaleString()
                      : "—"}
                  </td>
                  <td className="px-4 py-3 text-neutral-300 font-medium">
                    {s.action_count}
                  </td>
                  <td className="px-4 py-3">
                    <span
                      className={`inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium ${
                        s.ended_at === null
                          ? "bg-green-400/10 text-green-400"
                          : "bg-neutral-800 text-neutral-400"
                      }`}
                    >
                      {s.ended_at === null ? "Active" : "Ended"}
                    </span>
                  </td>
                  <td className="px-4 py-3 text-right">
                    <Link
                      href={`/dashboard/sessions/${s.session_id}`}
                      className="text-neutral-500 hover:text-rho-400 transition-colors"
                    >
                      <ChevronRight className="w-4 h-4" />
                    </Link>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
