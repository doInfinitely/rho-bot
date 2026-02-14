"use client";

import { useEffect, useState } from "react";
import { useParams } from "next/navigation";
import Link from "next/link";
import { getSessionActions, type ActionLogEntry } from "@/lib/api";

export default function SessionDetailPage() {
  const { id } = useParams<{ id: string }>();
  const [actions, setActions] = useState<ActionLogEntry[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    if (!id) return;
    getSessionActions(id)
      .then(setActions)
      .catch(console.error)
      .finally(() => setLoading(false));
  }, [id]);

  return (
    <div className="space-y-6">
      <div className="flex items-center gap-3">
        <Link
          href="/dashboard"
          className="text-sm text-neutral-500 hover:text-neutral-300"
        >
          &larr; Back
        </Link>
        <h1 className="text-2xl font-bold">
          Session{" "}
          <span className="font-mono text-rho-400">
            {id?.toString().slice(0, 8)}
          </span>
        </h1>
      </div>

      {loading ? (
        <p className="text-sm text-neutral-500">Loading actions...</p>
      ) : actions.length === 0 ? (
        <p className="text-sm text-neutral-500">No actions in this session.</p>
      ) : (
        <div className="border border-neutral-800 rounded-2xl overflow-hidden">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-neutral-800 text-left text-xs text-neutral-500">
                <th className="px-4 py-3">Time</th>
                <th className="px-4 py-3">Action</th>
                <th className="px-4 py-3">Confidence</th>
                <th className="px-4 py-3">Success</th>
              </tr>
            </thead>
            <tbody>
              {actions.map((a) => (
                <tr
                  key={a.action_id}
                  className="border-b border-neutral-800/50 hover:bg-neutral-900/60"
                >
                  <td className="px-4 py-3 font-mono text-xs text-neutral-400">
                    {new Date(a.timestamp * 1000).toLocaleTimeString()}
                  </td>
                  <td className="px-4 py-3">
                    <span className="px-2 py-0.5 rounded-md bg-neutral-800 text-rho-400 font-mono text-xs">
                      {a.action_type}
                    </span>
                  </td>
                  <td className="px-4 py-3 font-mono text-xs">
                    {(a.confidence * 100).toFixed(0)}%
                  </td>
                  <td className="px-4 py-3">
                    <span
                      className={`inline-block w-2 h-2 rounded-full ${
                        a.success ? "bg-emerald-400" : "bg-red-400"
                      }`}
                    />
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
