from __future__ import annotations

import uuid
from enum import Enum

from pydantic import BaseModel, Field


class ActionType(str, Enum):
    click = "click"
    type = "type"
    scroll = "scroll"
    keypress = "keypress"
    hotkey = "hotkey"
    wait = "wait"
    noop = "noop"


class ActionPayload(BaseModel):
    """An action the server tells the desktop client to execute."""

    action_id: str = Field(default_factory=lambda: str(uuid.uuid4()))
    type: ActionType = ActionType.noop
    coordinates: list[float] | None = None  # [x, y]
    text: str | None = None
    key: str | None = None
    modifiers: list[str] = Field(default_factory=list)
    confidence: float = 0.0
