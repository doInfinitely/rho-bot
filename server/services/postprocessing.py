"""
Postprocessing: convert raw model output tensors into an ActionPayload.

The model is expected to return a dict with these keys:

- ``action_type_logits``:  ``(1, num_action_types)`` — unnormalised scores
- ``coordinates_raw``:     ``(1, 2)`` — predicted [x, y] in normalised [0, 1]
- ``text_tokens``:         ``(1, max_text_len)`` — integer token ids (0 = pad)
- ``key_logits``:          ``(1, num_keys)`` — unnormalised scores
- ``modifier_logits``:     ``(1, num_modifiers)`` — independent logits per modifier

If a key is missing the corresponding field is left as ``None``.
"""

from __future__ import annotations

import logging
import uuid

import torch
import torch.nn.functional as F

from server.schemas.action import ActionPayload, ActionType
from server.schemas.context import WindowBounds

logger = logging.getLogger(__name__)

# ── Vocabularies ──────────────────────────────────────────────────────────

# Must match the order used in the model's output head.
ACTION_TYPES: list[ActionType] = [
    ActionType.click,
    ActionType.type,
    ActionType.scroll,
    ActionType.keypress,
    ActionType.hotkey,
    ActionType.wait,
    ActionType.noop,
]

KEY_NAMES: list[str] = [
    "return", "tab", "space", "delete", "escape",
    "left", "right", "down", "up",
    "a", "b", "c", "d", "e", "f", "g", "h", "i", "j",
    "k", "l", "m", "n", "o", "p", "q", "r", "s", "t",
    "u", "v", "w", "x", "y", "z",
]

MODIFIER_NAMES: list[str] = ["cmd", "shift", "alt", "ctrl"]

# Simple id-to-char map for text decoding (ASCII printable).
# A production model would use a real tokenizer (SentencePiece, BPE, etc.).
_ID_TO_CHAR: dict[int, str] = {i: chr(i) for i in range(32, 127)}


# ── Helpers ───────────────────────────────────────────────────────────────

def _decode_text_tokens(token_ids: torch.Tensor) -> str:
    """Decode a 1-D tensor of integer token IDs to a string.

    IDs of 0 (padding) are skipped.  Unknown IDs are replaced with '?'.
    """
    chars: list[str] = []
    for tid in token_ids.squeeze().tolist():
        tid = int(tid)
        if tid == 0:
            continue
        chars.append(_ID_TO_CHAR.get(tid, "?"))
    return "".join(chars)


# ── Public entry point ────────────────────────────────────────────────────

def postprocess(
    model_output: dict[str, torch.Tensor],
    window_bounds: WindowBounds | None = None,
) -> ActionPayload:
    """Convert model output tensors into a fully-formed ``ActionPayload``.

    Parameters
    ----------
    model_output:
        Dict of tensors as returned by the model's ``forward()``.
    window_bounds:
        If provided, used to scale normalised coordinates to pixel values.
        Falls back to 1920x1080 defaults.
    """
    wb = window_bounds or WindowBounds()

    # -- Action type ----------------------------------------------------------
    action_type = ActionType.noop
    confidence = 0.0

    if "action_type_logits" in model_output:
        logits = model_output["action_type_logits"].squeeze(0)  # (num_types,)
        probs = F.softmax(logits, dim=-1)
        idx = int(torch.argmax(probs).item())
        if idx < len(ACTION_TYPES):
            action_type = ACTION_TYPES[idx]
            confidence = float(probs[idx].item())

    # -- Coordinates ----------------------------------------------------------
    coordinates: list[float] | None = None

    if "coordinates_raw" in model_output:
        raw = model_output["coordinates_raw"].squeeze(0)  # (2,)
        # Model outputs normalised [0, 1]; scale to pixel space.
        px_x = float(raw[0].item()) * wb.width + wb.x
        px_y = float(raw[1].item()) * wb.height + wb.y
        coordinates = [round(px_x, 1), round(px_y, 1)]

    # -- Text -----------------------------------------------------------------
    text: str | None = None

    if "text_tokens" in model_output:
        decoded = _decode_text_tokens(model_output["text_tokens"])
        if decoded:
            text = decoded

    # -- Key ------------------------------------------------------------------
    key: str | None = None

    if "key_logits" in model_output:
        logits = model_output["key_logits"].squeeze(0)
        idx = int(torch.argmax(logits).item())
        if idx < len(KEY_NAMES):
            key = KEY_NAMES[idx]

    # -- Modifiers ------------------------------------------------------------
    modifiers: list[str] = []

    if "modifier_logits" in model_output:
        logits = model_output["modifier_logits"].squeeze(0)
        preds = torch.sigmoid(logits) > 0.5
        for i, active in enumerate(preds.tolist()):
            if active and i < len(MODIFIER_NAMES):
                modifiers.append(MODIFIER_NAMES[i])

    # -- Assemble -------------------------------------------------------------
    # Only populate fields relevant to the predicted action type.
    payload_kwargs: dict = {
        "action_id": str(uuid.uuid4()),
        "type": action_type,
        "confidence": round(confidence, 4),
        "modifiers": modifiers,
    }

    if action_type in (ActionType.click, ActionType.scroll):
        payload_kwargs["coordinates"] = coordinates
    if action_type == ActionType.type:
        payload_kwargs["text"] = text
    if action_type in (ActionType.keypress, ActionType.hotkey):
        payload_kwargs["key"] = key

    return ActionPayload(**payload_kwargs)
