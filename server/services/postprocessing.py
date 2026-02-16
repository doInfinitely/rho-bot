"""
Postprocessing: convert ActionPolicy output tensors into an ActionPayload.

The model returns::

    {
        "mouse_logits": (B, 4096)  — 64×64 discretized screen grid
        "key_logits":   (B, 83)    — 83 key classes
        "value":        (B, 1)     — state value (unused at inference)
    }

We pick the most-likely action (click vs keypress) by comparing the top
confidence from each head, then populate the ActionPayload accordingly.
"""

from __future__ import annotations

import logging
import uuid

import torch
import torch.nn.functional as F

from server.schemas.action import ActionPayload, ActionType
from server.schemas.context import WindowBounds

logger = logging.getLogger(__name__)

# ---------------------------------------------------------------------------
# Grid and key-class constants (must match model_arch.py / training)
# ---------------------------------------------------------------------------

MOUSE_GRID = 64  # screen discretised to 64×64

# 83 key classes — indices must match the training vocabulary.
# This is the most common ordering; adjust if your training used a different one.
KEY_NAMES: list[str] = [
    # Special keys (0–12)
    "noop", "return", "tab", "space", "delete", "escape",
    "left", "right", "down", "up", "home", "end", "pageup",
    # Punctuation / symbols (13–24)
    "pagedown", "minus", "equal", "bracketleft", "bracketright",
    "backslash", "semicolon", "quote", "grave", "comma", "period", "slash",
    # Letters (25–50)
    "a", "b", "c", "d", "e", "f", "g", "h", "i", "j",
    "k", "l", "m", "n", "o", "p", "q", "r", "s", "t",
    "u", "v", "w", "x", "y", "z",
    # Digits (51–60)
    "0", "1", "2", "3", "4", "5", "6", "7", "8", "9",
    # F-keys (61–72)
    "f1", "f2", "f3", "f4", "f5", "f6", "f7", "f8", "f9", "f10", "f11", "f12",
    # Modifiers used as standalone keys (73–76)
    "shift", "ctrl", "alt", "cmd",
    # Extra (77–82)
    "insert", "printscreen", "scrolllock", "pause", "numlock", "capslock",
]

MODIFIER_NAMES: list[str] = ["cmd", "shift", "alt", "ctrl"]

# Index of the "noop" key in KEY_NAMES (used to detect no-key-action)
_NOOP_KEY_IDX = 0


# ---------------------------------------------------------------------------
# Public entry point
# ---------------------------------------------------------------------------

def postprocess(
    model_output: dict[str, torch.Tensor],
    window_bounds: WindowBounds | None = None,
) -> ActionPayload:
    """Convert model output tensors into a fully-formed ``ActionPayload``."""
    wb = window_bounds or WindowBounds()

    mouse_logits = model_output["mouse_logits"].squeeze(0)  # (4096,)
    key_logits = model_output["key_logits"].squeeze(0)      # (83,)

    # Probabilities for each head
    mouse_probs = F.softmax(mouse_logits, dim=-1)
    key_probs = F.softmax(key_logits, dim=-1)

    best_mouse_idx = int(torch.argmax(mouse_probs).item())
    best_mouse_conf = float(mouse_probs[best_mouse_idx].item())

    best_key_idx = int(torch.argmax(key_probs).item())
    best_key_conf = float(key_probs[best_key_idx].item())

    # Decide: click vs keypress (pick whichever head is more confident).
    # If the key head picks "noop" treat it as a click regardless.
    key_is_noop = best_key_idx == _NOOP_KEY_IDX
    prefer_click = key_is_noop or best_mouse_conf >= best_key_conf

    if prefer_click:
        # Decode grid index → pixel coordinates
        grid_y = best_mouse_idx // MOUSE_GRID
        grid_x = best_mouse_idx % MOUSE_GRID
        px_x = (grid_x + 0.5) / MOUSE_GRID * wb.width + wb.x
        px_y = (grid_y + 0.5) / MOUSE_GRID * wb.height + wb.y

        return ActionPayload(
            action_id=str(uuid.uuid4()),
            type=ActionType.click,
            coordinates=[round(px_x, 1), round(px_y, 1)],
            confidence=round(best_mouse_conf, 4),
        )
    else:
        key_name = KEY_NAMES[best_key_idx] if best_key_idx < len(KEY_NAMES) else "unknown"

        return ActionPayload(
            action_id=str(uuid.uuid4()),
            type=ActionType.keypress,
            key=key_name,
            confidence=round(best_key_conf, 4),
        )
