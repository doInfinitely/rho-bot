from __future__ import annotations

from typing import Any

from pydantic import BaseModel, Field


class WindowBounds(BaseModel):
    x: float = 0.0
    y: float = 0.0
    width: float = 1920.0
    height: float = 1080.0


class InputEvent(BaseModel):
    type: str  # "click", "keypress", "scroll", "drag"
    x: float | None = None
    y: float | None = None
    key: str | None = None
    modifiers: list[str] = Field(default_factory=list)
    timestamp: float = 0.0


class ContextPayload(BaseModel):
    """Everything the desktop client captures in one tick and sends to the server."""

    session_id: str
    timestamp: float
    screenshot_b64: str = ""  # base64-encoded PNG
    accessibility_tree: dict[str, Any] = Field(default_factory=dict)
    recent_events: list[InputEvent] = Field(default_factory=list)
    active_app: str = ""
    window_bounds: WindowBounds = Field(default_factory=WindowBounds)
