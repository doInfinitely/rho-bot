"""REST endpoints consumed by the website dashboard and iOS app."""

from __future__ import annotations

import time

from fastapi import APIRouter, Depends, HTTPException
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from server.deps import get_current_user
from server.models.database import ActionLog, Session, User, get_db
from server.schemas.auth import UserOut
from server.schemas.dashboard import (
    ActionLogEntry,
    AgentStatus,
    GoalRequest,
    GoalResponse,
    SessionSummary,
)

router = APIRouter(prefix="/api", tags=["dashboard"])


# ---- helpers ----

async def _latest_session(user_id: str, db: AsyncSession) -> Session | None:
    result = await db.execute(
        select(Session)
        .where(Session.user_id == user_id)
        .order_by(Session.started_at.desc())
        .limit(1)
    )
    return result.scalar_one_or_none()


async def _active_session(user_id: str, db: AsyncSession) -> Session | None:
    """Return the most recent session only if it is still active (not ended)."""
    session = await _latest_session(user_id, db)
    if session is not None and session.ended_at is None:
        return session
    return None


# ---- user ----

@router.get("/me", response_model=UserOut)
async def get_me(user: User = Depends(get_current_user)):
    return UserOut(id=user.id, email=user.email)


# ---- agent status ----

@router.get("/agent/status", response_model=AgentStatus)
async def agent_status(
    user: User = Depends(get_current_user),
    db: AsyncSession = Depends(get_db),
):
    """Return the status of the user's most recent agent session."""
    session = await _latest_session(user.id, db)
    if session is None:
        return AgentStatus()

    return AgentStatus(
        session_id=session.id,
        is_online=session.ended_at is None,
        last_seen=session.started_at,
        total_actions=session.action_count or 0,
        goal=session.goal or "",
    )


# ---- goal management ----

@router.get("/agent/goal", response_model=GoalResponse)
async def get_goal(
    user: User = Depends(get_current_user),
    db: AsyncSession = Depends(get_db),
):
    """Return the current goal for the user's active session."""
    session = await _latest_session(user.id, db)
    return GoalResponse(
        goal=session.goal or "" if session else "",
        session_id=session.id if session else None,
    )


@router.post("/agent/goal", response_model=GoalResponse)
async def set_goal(
    body: GoalRequest,
    user: User = Depends(get_current_user),
    db: AsyncSession = Depends(get_db),
):
    """Set the goal for the user's active session (creates one if needed)."""
    session = await _active_session(user.id, db)
    if session is None:
        session = Session(
            user_id=user.id,
            started_at=time.time(),
            goal=body.goal,
        )
        db.add(session)
    else:
        session.goal = body.goal
    await db.commit()
    await db.refresh(session)
    return GoalResponse(goal=session.goal or "", session_id=session.id)


# ---- agent start / stop ----

@router.post("/agent/start", response_model=AgentStatus)
async def start_agent(
    user: User = Depends(get_current_user),
    db: AsyncSession = Depends(get_db),
):
    """Create a new agent session (or return the existing active one)."""
    session = await _active_session(user.id, db)
    if session is not None:
        return AgentStatus(
            session_id=session.id,
            is_online=True,
            last_seen=session.started_at,
            total_actions=session.action_count or 0,
            goal=session.goal or "",
        )

    session = Session(user_id=user.id, started_at=time.time())
    db.add(session)
    await db.commit()
    await db.refresh(session)
    return AgentStatus(
        session_id=session.id,
        is_online=True,
        last_seen=session.started_at,
        total_actions=0,
        goal=session.goal or "",
    )


@router.post("/agent/stop", response_model=AgentStatus)
async def stop_agent(
    user: User = Depends(get_current_user),
    db: AsyncSession = Depends(get_db),
):
    """End the user's active session."""
    session = await _active_session(user.id, db)
    if session is None:
        raise HTTPException(status_code=404, detail="No active session")

    session.ended_at = time.time()
    await db.commit()
    return AgentStatus(
        session_id=session.id,
        is_online=False,
        last_seen=session.started_at,
        total_actions=session.action_count or 0,
        goal=session.goal or "",
    )


# ---- sessions ----

@router.get("/sessions", response_model=list[SessionSummary])
async def list_sessions(
    user: User = Depends(get_current_user),
    db: AsyncSession = Depends(get_db),
    limit: int = 20,
    offset: int = 0,
):
    result = await db.execute(
        select(Session)
        .where(Session.user_id == user.id)
        .order_by(Session.started_at.desc())
        .offset(offset)
        .limit(limit)
    )
    sessions = result.scalars().all()
    return [
        SessionSummary(
            session_id=s.id,
            started_at=s.started_at,
            ended_at=s.ended_at,
            action_count=s.action_count or 0,
            goal=s.goal or "",
        )
        for s in sessions
    ]


@router.get("/sessions/{session_id}/actions", response_model=list[ActionLogEntry])
async def session_actions(
    session_id: str,
    user: User = Depends(get_current_user),
    db: AsyncSession = Depends(get_db),
):
    # Verify the session belongs to the user
    session = await db.get(Session, session_id)
    if session is None or session.user_id != user.id:
        return []

    result = await db.execute(
        select(ActionLog)
        .where(ActionLog.session_id == session_id)
        .order_by(ActionLog.timestamp.asc())
    )
    logs = result.scalars().all()
    return [
        ActionLogEntry(
            action_id=a.id,
            session_id=a.session_id,
            timestamp=a.timestamp,
            action_type=a.action_type,
            confidence=a.confidence,
            success=a.success,
        )
        for a in logs
    ]
