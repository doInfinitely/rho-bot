from __future__ import annotations

from pydantic import BaseModel


class AgentStatus(BaseModel):
    session_id: str | None = None
    is_online: bool = False
    last_seen: float | None = None
    total_actions: int = 0


class SessionSummary(BaseModel):
    session_id: str
    started_at: float
    ended_at: float | None = None
    action_count: int = 0


class ActionLogEntry(BaseModel):
    action_id: str
    session_id: str
    timestamp: float
    action_type: str
    confidence: float
    success: bool = True
