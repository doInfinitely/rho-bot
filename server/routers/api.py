"""REST endpoints consumed by the website dashboard."""

from __future__ import annotations

from fastapi import APIRouter, Depends
from sqlalchemy import func, select
from sqlalchemy.ext.asyncio import AsyncSession

from server.deps import get_current_user
from server.models.database import ActionLog, Session, User, get_db
from server.schemas.auth import UserOut
from server.schemas.dashboard import ActionLogEntry, AgentStatus, SessionSummary

router = APIRouter(prefix="/api", tags=["dashboard"])


@router.get("/me", response_model=UserOut)
async def get_me(user: User = Depends(get_current_user)):
    return UserOut(id=user.id, email=user.email)


@router.get("/agent/status", response_model=AgentStatus)
async def agent_status(
    user: User = Depends(get_current_user),
    db: AsyncSession = Depends(get_db),
):
    """Return the status of the user's most recent agent session."""
    result = await db.execute(
        select(Session)
        .where(Session.user_id == user.id)
        .order_by(Session.started_at.desc())
        .limit(1)
    )
    session = result.scalar_one_or_none()
    if session is None:
        return AgentStatus()

    return AgentStatus(
        session_id=session.id,
        is_online=session.ended_at is None,
        last_seen=session.started_at,
        total_actions=session.action_count or 0,
    )


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
