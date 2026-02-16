import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";

interface AuthResult {
  email: string;
  token: string;
}

export default function LoginScreen({
  onLogin,
}: {
  onLogin: (email: string) => void;
}) {
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [isSignup, setIsSignup] = useState(false);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError(null);

    try {
      const result = await invoke<AuthResult>(isSignup ? "signup" : "login", {
        creds: { email, password },
      });
      onLogin(result.email);
    } catch (err: unknown) {
      const msg =
        err instanceof Error ? err.message : typeof err === "string" ? err : "Unknown error";
      setError(msg);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="min-h-screen bg-neutral-950 text-neutral-100 flex items-center justify-center p-6">
      <div className="w-full max-w-xs space-y-8">
        {/* Branding */}
        <div className="text-center">
          <h1 className="text-2xl font-bold tracking-tight">
            <span className="text-rho-400">rho</span>-bot
          </h1>
          <p className="mt-1 text-xs text-neutral-500">
            AI Desktop Agent
          </p>
        </div>

        {/* Form */}
        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <label className="block text-xs text-neutral-400 mb-1">
              Email
            </label>
            <input
              type="email"
              required
              autoFocus
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              className="w-full px-3 py-2 text-sm bg-neutral-900 border border-neutral-800 rounded-lg focus:outline-none focus:border-rho-500"
              placeholder="you@example.com"
            />
          </div>

          <div>
            <label className="block text-xs text-neutral-400 mb-1">
              Password
            </label>
            <input
              type="password"
              required
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              className="w-full px-3 py-2 text-sm bg-neutral-900 border border-neutral-800 rounded-lg focus:outline-none focus:border-rho-500"
              placeholder="••••••••"
            />
          </div>

          {error && (
            <div className="rounded-lg border border-red-500/30 bg-red-500/10 px-3 py-2 text-xs text-red-300">
              {error}
            </div>
          )}

          <button
            type="submit"
            disabled={loading || !email || !password}
            className="w-full px-4 py-2.5 text-sm font-medium rounded-lg bg-rho-600 hover:bg-rho-700 disabled:opacity-40 disabled:cursor-not-allowed transition-colors"
          >
            {loading ? "..." : isSignup ? "Create Account" : "Log In"}
          </button>
        </form>

        {/* Toggle */}
        <div className="text-center">
          <button
            type="button"
            onClick={() => {
              setIsSignup(!isSignup);
              setError(null);
            }}
            className="text-xs text-neutral-500 hover:text-neutral-300 transition-colors"
          >
            {isSignup
              ? "Already have an account? Log in"
              : "Don't have an account? Sign up"}
          </button>
        </div>
      </div>
    </div>
  );
}
