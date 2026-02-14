"use client";

import { useEffect, useState } from "react";
import { Bot, Zap, Clock, Activity } from "lucide-react";
import { getMe, getAgentStatus, getSessions } from "@/lib/api";
import type { AgentStatus, SessionSummary } from "@/lib/api";

export default function DashboardPage() {
  const [email, setEmail] = useState("");
  const [agent, setAgent] = useState<AgentStatus | null>(null);
  const [sessions, setSessions] = useState<SessionSummary[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    async function load() {
      try {
        const [me, status, sess] = await Promise.all([
          getMe(),
          getAgentStatus(),
          getSessions(5),
        ]);
        setEmail(me.email);
        setAgent(status);
        setSessions(sess);
      } catch {
        // Auth error will be caught by layout redirect
      } finally {
        setLoading(false);
      }
    }
    load();
  }, []);

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="w-6 h-6 border-2 border-rho-500 border-t-transparent rounded-full animate-spin" />
      </div>
    );
  }

  const stats = [
    {
      label: "Agent Status",
      value: agent?.is_online ? "Online" : "Offline",
      icon: Bot,
      color: agent?.is_online ? "text-green-400" : "text-neutral-500",
    },
    {
      label: "Total Actions",
      value: agent?.total_actions?.toLocaleString() ?? "0",
      icon: Zap,
      color: "text-rho-400",
    },
    {
      label: "Sessions",
      value: sessions.length.toString(),
      icon: Activity,
      color: "text-rho-400",
    },
    {
      label: "Last Active",
      value: agent?.last_seen
        ? new Date(agent.last_seen * 1000).toLocaleDateString()
        : "Never",
      icon: Clock,
      color: "text-neutral-400",
    },
  ];

  return (
    <div>
      <div className="mb-8">
        <h1 className="text-2xl font-bold tracking-tight">Dashboard</h1>
        <p className="mt-1 text-sm text-neutral-500">
          Welcome back, {email}
        </p>
      </div>

      {/* Stats grid */}
      <div className="grid grid-cols-2 lg:grid-cols-4 gap-4 mb-8">
        {stats.map(({ label, value, icon: Icon, color }) => (
          <div
            key={label}
            className="p-5 rounded-xl border border-neutral-800 bg-neutral-900/50"
          >
            <div className="flex items-center gap-3 mb-3">
              <div className="w-8 h-8 rounded-lg bg-neutral-800 flex items-center justify-center">
                <Icon className={`w-4 h-4 ${color}`} />
              </div>
            </div>
            <p className="text-2xl font-bold tracking-tight">{value}</p>
            <p className="text-xs text-neutral-500 mt-1">{label}</p>
          </div>
        ))}
      </div>

      {/* Recent sessions */}
      <div>
        <h2 className="text-lg font-semibold mb-4">Recent Sessions</h2>
        {sessions.length === 0 ? (
          <div className="p-8 rounded-xl border border-neutral-800 bg-neutral-900/50 text-center">
            <Bot className="w-8 h-8 text-neutral-600 mx-auto mb-3" />
            <p className="text-sm text-neutral-500">
              No sessions yet. Install the desktop agent to get started.
            </p>
          </div>
        ) : (
          <div className="border border-neutral-800 rounded-xl overflow-hidden">
            <table className="w-full text-sm">
              <thead>
                <tr className="bg-neutral-900/80">
                  <th className="text-left px-4 py-3 text-xs font-medium text-neutral-500 uppercase tracking-wider">
                    Session
                  </th>
                  <th className="text-left px-4 py-3 text-xs font-medium text-neutral-500 uppercase tracking-wider">
                    Started
                  </th>
                  <th className="text-left px-4 py-3 text-xs font-medium text-neutral-500 uppercase tracking-wider">
                    Actions
                  </th>
                  <th className="text-left px-4 py-3 text-xs font-medium text-neutral-500 uppercase tracking-wider">
                    Status
                  </th>
                </tr>
              </thead>
              <tbody>
                {sessions.map((s) => (
                  <tr
                    key={s.session_id}
                    className="border-t border-neutral-800/50"
                  >
                    <td className="px-4 py-3 font-mono text-xs text-neutral-400">
                      {s.session_id.slice(0, 8)}…
                    </td>
                    <td className="px-4 py-3 text-neutral-400">
                      {new Date(s.started_at * 1000).toLocaleString()}
                    </td>
                    <td className="px-4 py-3 text-neutral-300">
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
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>
    </div>
  );
}
