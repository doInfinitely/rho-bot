"""
Context ingestion & persistence.

Receives context bundles from the desktop client, stores them for
training-data collection, and forwards them to the model service.
"""

from __future__ import annotations

import json
import logging
import time

from sqlalchemy.ext.asyncio import AsyncSession

from server.models.database import ActionLog, ContextLog, Session, TrainingPair
from server.schemas.action import ActionPayload
from server.schemas.context import ContextPayload
from server.schemas.training import TrainingPayload
from server.services.model_service import ModelService
from server.services.screenshot_service import upload_screenshot

logger = logging.getLogger(__name__)


class ContextService:
    def __init__(self, model_service: ModelService) -> None:
        self.model = model_service

    async def ensure_session(self, session_id: str, user_id: str, db: AsyncSession) -> None:
        """Create a session row if one doesn't already exist."""
        existing = await db.get(Session, session_id)
        if existing is None:
            db.add(Session(id=session_id, user_id=user_id, started_at=time.time()))
            await db.commit()

    async def process_context(
        self,
        context: ContextPayload,
        user_id: str,
        db: AsyncSession,
    ) -> ActionPayload:
        """Ingest a context bundle, persist it, run inference, persist & return the action."""

        # 1. Persist context
        ctx_log = ContextLog(
            session_id=context.session_id,
            timestamp=context.timestamp,
            active_app=context.active_app,
            accessibility_tree_json=json.dumps(context.accessibility_tree),
            screenshot_path="",
        )
        db.add(ctx_log)
        await db.flush()  # populate ctx_log.id

        # 1b. Upload screenshot to R2
        screenshot_key = await upload_screenshot(
            context.screenshot_b64,
            f"screenshots/{context.session_id}/{ctx_log.id}.png",
        )
        if screenshot_key:
            ctx_log.screenshot_path = screenshot_key

        # 2. Predict
        action = await self.model.predict_action(context)

        # 3. Persist action
        action_log = ActionLog(
            session_id=context.session_id,
            timestamp=time.time(),
            action_type=action.type.value,
            action_payload_json=action.model_dump_json(),
            confidence=action.confidence,
        )
        db.add(action_log)

        # 4. Auto-record training pair from agent predictions
        training_pair = TrainingPair(
            session_id=context.session_id,
            user_id=user_id,
            timestamp=context.timestamp,
            active_app=context.active_app,
            accessibility_tree_json=json.dumps(context.accessibility_tree),
            screenshot_path=screenshot_key,
            user_actions_json=action.model_dump_json(),
            source="agent",
        )
        db.add(training_pair)

        # 5. Bump session action count
        session = await db.get(Session, context.session_id)
        if session is not None:
            session.action_count = (session.action_count or 0) + 1

        await db.commit()
        return action

    async def store_training_pair(
        self,
        payload: TrainingPayload,
        user_id: str,
        db: AsyncSession,
    ) -> None:
        """Persist a context/action pair captured during passive recording."""

        pair = TrainingPair(
            session_id=payload.context.session_id,
            user_id=user_id,
            timestamp=payload.context.timestamp,
            active_app=payload.context.active_app,
            accessibility_tree_json=json.dumps(payload.context.accessibility_tree),
            screenshot_path="",
            user_actions_json=json.dumps(
                [a.model_dump() for a in payload.user_actions]
            ),
            source="recording",
        )
        db.add(pair)
        await db.flush()  # populate pair.id

        # Upload screenshot to R2
        screenshot_key = await upload_screenshot(
            payload.context.screenshot_b64,
            f"training/{payload.context.session_id}/{pair.id}.png",
        )
        if screenshot_key:
            pair.screenshot_path = screenshot_key

        await db.commit()
