"""WebSocket endpoint for real-time desktop-client <-> server communication."""

from __future__ import annotations

import logging

from fastapi import APIRouter, WebSocket, WebSocketDisconnect

from server.models.database import async_session
from server.schemas.action import ActionPayload
from server.schemas.context import ContextPayload
from server.schemas.training import TrainingPayload
from server.services.auth_service import decode_access_token
from server.services.billing_service import check_and_increment_quota

logger = logging.getLogger(__name__)

router = APIRouter(tags=["agent"])


@router.websocket("/ws/agent")
async def agent_ws(ws: WebSocket):
    """
    Protocol
    --------
    1. Client connects and sends a JSON ``{"token": "<jwt>"}`` as the first message.
    2. Server validates the token. On failure it closes with 4001.
    3. Client streams ``ContextPayload`` JSON objects.
    4. Server checks payment status & quota before processing.
    5. Server replies with ``ActionPayload`` JSON objects (``error`` field set on denial).
    """
    await ws.accept()

    # --- authenticate ---
    try:
        auth_msg = await ws.receive_json()
    except Exception:
        await ws.close(code=4001, reason="Expected auth JSON")
        return

    token = auth_msg.get("token", "")
    payload = decode_access_token(token)
    if payload is None:
        await ws.close(code=4001, reason="Invalid token")
        return

    user_id: str = payload.get("sub", "")
    logger.info("Agent WebSocket authenticated for user %s", user_id)

    # Import here to avoid circular imports at module level
    from server.main import context_service  # noqa: WPS433

    # --- main loop ---
    try:
        while True:
            data = await ws.receive_json()

            try:
                context = ContextPayload(**data)

                async with async_session() as db:
                    # --- quota gate ---
                    allowed, reason = await check_and_increment_quota(db, user_id)
                    if not allowed:
                        logger.info(
                            "Quota denied for user %s: %s", user_id, reason
                        )
                        denied = ActionPayload(error=reason)
                        await ws.send_json(denied.model_dump())
                        continue

                    await context_service.ensure_session(context.session_id, user_id, db)
                    action = await context_service.process_context(context, user_id, db)

                await ws.send_json(action.model_dump())
            except Exception as inner_exc:
                # Send the error back to the client instead of killing the connection.
                logger.exception(
                    "Error processing context for user %s: %s", user_id, inner_exc
                )
                error_action = ActionPayload(
                    error=f"Server error: {type(inner_exc).__name__}: {inner_exc}"
                )
                try:
                    await ws.send_json(error_action.model_dump())
                except Exception:
                    # Can't even send the error — bail out
                    break

    except WebSocketDisconnect:
        logger.info("Agent WebSocket disconnected (user %s)", user_id)
    except Exception:
        logger.exception("Agent WebSocket error (user %s)", user_id)
        try:
            await ws.close(code=1011)
        except Exception:
            pass


@router.websocket("/ws/record")
async def record_ws(ws: WebSocket):
    """
    Passive-recording endpoint: receives context/action training pairs.

    Protocol
    --------
    1. Client connects and sends ``{"token": "<jwt>"}`` as the first message.
    2. Server validates the token.  On failure it closes with 4001.
    3. Client streams ``TrainingPayload`` JSON objects (context + user actions).
    4. Server persists each pair and replies with ``{"status": "ok"}``.
    """
    await ws.accept()

    # --- authenticate ---
    try:
        auth_msg = await ws.receive_json()
    except Exception:
        await ws.close(code=4001, reason="Expected auth JSON")
        return

    token = auth_msg.get("token", "")
    payload = decode_access_token(token)
    if payload is None:
        await ws.close(code=4001, reason="Invalid token")
        return

    user_id: str = payload.get("sub", "")
    logger.info("Recording WebSocket authenticated for user %s", user_id)

    # Import here to avoid circular imports at module level
    from server.main import context_service  # noqa: WPS433

    # --- main loop ---
    try:
        while True:
            data = await ws.receive_json()

            try:
                training = TrainingPayload(**data)

                async with async_session() as db:
                    await context_service.ensure_session(
                        training.context.session_id, user_id, db
                    )
                    await context_service.store_training_pair(training, user_id, db)

                await ws.send_json({"status": "ok"})
            except Exception as inner_exc:
                logger.exception(
                    "Error processing training data for user %s: %s",
                    user_id,
                    inner_exc,
                )
                try:
                    await ws.send_json({
                        "status": "error",
                        "detail": f"{type(inner_exc).__name__}: {inner_exc}",
                    })
                except Exception:
                    break

    except WebSocketDisconnect:
        logger.info("Recording WebSocket disconnected (user %s)", user_id)
    except Exception:
        logger.exception("Recording WebSocket error (user %s)", user_id)
        try:
            await ws.close(code=1011)
        except Exception:
            pass
