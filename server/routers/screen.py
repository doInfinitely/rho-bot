"""WebSocket endpoint for relaying live desktop screen frames to iOS subscribers."""

from __future__ import annotations

import asyncio
import json
import logging
from collections import defaultdict

from fastapi import APIRouter, WebSocket, WebSocketDisconnect

from server.services.auth_service import decode_access_token

logger = logging.getLogger(__name__)

router = APIRouter(tags=["screen"])

# One desktop publisher per user
_screen_publishers: dict[str, WebSocket] = {}
# Multiple iOS subscribers per user
_screen_subscribers: dict[str, list[WebSocket]] = defaultdict(list)


async def _authenticate(ws: WebSocket) -> str | None:
    """Authenticate the first message as JWT. Returns user_id or None."""
    try:
        auth_msg = await ws.receive_json()
    except Exception:
        await ws.close(code=4001, reason="Expected auth JSON")
        return None

    token = auth_msg.get("token", "")
    payload = decode_access_token(token)
    if payload is None:
        await ws.close(code=4001, reason="Invalid token")
        return None

    return payload.get("sub", "")


@router.websocket("/ws/screen")
async def screen_ws(ws: WebSocket):
    """
    Unified screen-streaming endpoint for publishers (desktop) and subscribers (iOS).

    Protocol
    --------
    1. Client sends ``{"token": "<jwt>"}`` to authenticate.
    2. Client sends ``{"type": "register", "role": "publisher"|"subscriber"}``.
    3. Server relays frames from publisher to all subscribers for that user.
    """
    await ws.accept()

    user_id = await _authenticate(ws)
    if user_id is None:
        return

    # Wait for role registration
    try:
        reg_msg = await ws.receive_json()
    except Exception:
        await ws.close(code=4001, reason="Expected register message")
        return

    if reg_msg.get("type") != "register" or reg_msg.get("role") not in ("publisher", "subscriber"):
        await ws.close(code=4001, reason="Expected register with role publisher|subscriber")
        return

    role = reg_msg["role"]
    logger.info("Screen %s connected for user %s", role, user_id)

    if role == "publisher":
        await _handle_publisher(ws, user_id)
    else:
        await _handle_subscriber(ws, user_id)


async def _handle_publisher(ws: WebSocket, user_id: str):
    """Handle a desktop screen publisher connection."""
    # Kick existing publisher for this user
    old = _screen_publishers.pop(user_id, None)
    if old is not None:
        try:
            await old.close(code=4002, reason="Session replaced by new login")
        except Exception:
            pass
        logger.info("Kicked existing screen publisher for user %s", user_id)

    _screen_publishers[user_id] = ws

    # If subscribers are already waiting, tell publisher to start streaming
    subs = _screen_subscribers.get(user_id, [])
    if subs:
        try:
            await ws.send_json({"type": "start_stream", "interval_ms": 1000})
        except Exception:
            pass

    try:
        while True:
            # Receive raw text to avoid re-serializing base64 frames
            raw = await ws.receive_text()

            # Relay frame to all subscribers
            subs = _screen_subscribers.get(user_id, [])
            if subs:
                results = await asyncio.gather(
                    *(s.send_text(raw) for s in subs),
                    return_exceptions=True,
                )
                # Remove broken subscribers
                broken = [
                    subs[i] for i, r in enumerate(results) if isinstance(r, Exception)
                ]
                for b in broken:
                    try:
                        _screen_subscribers[user_id].remove(b)
                    except ValueError:
                        pass
                # If last subscriber dropped, tell publisher to stop
                if not _screen_subscribers.get(user_id):
                    try:
                        await ws.send_json({"type": "stop_stream"})
                    except Exception:
                        pass

    except WebSocketDisconnect:
        logger.info("Screen publisher disconnected (user %s)", user_id)
    except Exception:
        logger.exception("Screen publisher error (user %s)", user_id)
        try:
            await ws.close(code=1011)
        except Exception:
            pass
    finally:
        # Unregister publisher
        if _screen_publishers.get(user_id) is ws:
            del _screen_publishers[user_id]

        # Notify all subscribers that desktop went offline
        subs = _screen_subscribers.get(user_id, [])
        if subs:
            status_msg = json.dumps({"type": "status", "desktop_online": False})
            await asyncio.gather(
                *(s.send_text(status_msg) for s in subs),
                return_exceptions=True,
            )


async def _handle_subscriber(ws: WebSocket, user_id: str):
    """Handle an iOS screen subscriber connection."""
    _screen_subscribers[user_id].append(ws)

    # Tell subscriber whether desktop is online
    desktop_online = user_id in _screen_publishers
    try:
        await ws.send_json({"type": "status", "desktop_online": desktop_online})
    except Exception:
        pass

    # If this is the first subscriber and publisher exists, start streaming
    if len(_screen_subscribers[user_id]) == 1 and desktop_online:
        try:
            await _screen_publishers[user_id].send_json(
                {"type": "start_stream", "interval_ms": 1000}
            )
        except Exception:
            pass

    try:
        while True:
            # Subscribers can send control messages (e.g. set_interval)
            raw = await ws.receive_text()
            try:
                msg = json.loads(raw)
            except json.JSONDecodeError:
                continue

            msg_type = msg.get("type")
            if msg_type == "set_interval" and user_id in _screen_publishers:
                # Forward interval change to publisher
                try:
                    await _screen_publishers[user_id].send_text(raw)
                except Exception:
                    pass

    except WebSocketDisconnect:
        logger.info("Screen subscriber disconnected (user %s)", user_id)
    except Exception:
        logger.exception("Screen subscriber error (user %s)", user_id)
        try:
            await ws.close(code=1011)
        except Exception:
            pass
    finally:
        # Unregister subscriber
        try:
            _screen_subscribers[user_id].remove(ws)
        except ValueError:
            pass

        # If last subscriber left, tell publisher to stop streaming
        if not _screen_subscribers.get(user_id) and user_id in _screen_publishers:
            try:
                await _screen_publishers[user_id].send_json({"type": "stop_stream"})
            except Exception:
                pass
