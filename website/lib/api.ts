/**
 * API client for the rho-bot server.
 */

const API_BASE = process.env.NEXT_PUBLIC_API_URL || "http://localhost:8000";

async function request<T>(
  path: string,
  options: RequestInit = {}
): Promise<T> {
  const token =
    typeof window !== "undefined" ? localStorage.getItem("rho_token") : null;

  const res = await fetch(`${API_BASE}${path}`, {
    ...options,
    headers: {
      "Content-Type": "application/json",
      ...(token ? { Authorization: `Bearer ${token}` } : {}),
      ...options.headers,
    },
  });

  if (!res.ok) {
    const body = await res.json().catch(() => ({}));
    throw new Error(body.detail || `Request failed: ${res.status}`);
  }

  return res.json();
}

// ---- Auth ----

export interface TokenResponse {
  access_token: string;
  token_type: string;
}

export async function signup(
  email: string,
  password: string
): Promise<TokenResponse> {
  return request<TokenResponse>("/auth/signup", {
    method: "POST",
    body: JSON.stringify({ email, password }),
  });
}

export async function login(
  email: string,
  password: string
): Promise<TokenResponse> {
  return request<TokenResponse>("/auth/login", {
    method: "POST",
    body: JSON.stringify({ email, password }),
  });
}

// ---- Dashboard ----

export interface AgentStatus {
  session_id: string | null;
  is_online: boolean;
  last_seen: number | null;
  total_actions: number;
}

export interface SessionSummary {
  session_id: string;
  started_at: number;
  ended_at: number | null;
  action_count: number;
}

export interface ActionLogEntry {
  action_id: string;
  session_id: string;
  timestamp: number;
  action_type: string;
  confidence: number;
  success: boolean;
}

export async function getMe() {
  return request<{ id: string; email: string }>("/api/me");
}

export async function getAgentStatus() {
  return request<AgentStatus>("/api/agent/status");
}

export async function getSessions(limit = 20, offset = 0) {
  return request<SessionSummary[]>(
    `/api/sessions?limit=${limit}&offset=${offset}`
  );
}

export async function getSessionActions(sessionId: string) {
  return request<ActionLogEntry[]>(`/api/sessions/${sessionId}/actions`);
}

// ---- Billing / Subscriptions ----

export interface Subscription {
  id: string;
  plan_id: string;
  plan_name: string;
  status: "active" | "trialing" | "past_due" | "canceled" | "incomplete";
  current_period_end: number;
  tasks_used: number;
  tasks_limit: number;
}

export async function getSubscription(): Promise<Subscription> {
  return request<Subscription>("/api/billing/subscription");
}

export async function createCheckoutSession(
  planId: string
): Promise<{ url: string }> {
  return request<{ url: string }>("/api/billing/checkout", {
    method: "POST",
    body: JSON.stringify({ plan_id: planId }),
  });
}

export async function createBillingPortalSession(): Promise<{ url: string }> {
  return request<{ url: string }>("/api/billing/portal", {
    method: "POST",
  });
}
