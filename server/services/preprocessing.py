"""
Preprocessing: convert a ContextPayload into model-ready tensors.

The functions here are intentionally decoupled from any specific model
architecture so the same preprocessing can be reused across experiments.
"""

from __future__ import annotations

import base64
import io
import json
import logging
from typing import Any

import torch
from PIL import Image
from torchvision import transforms

from server.schemas.context import ContextPayload

logger = logging.getLogger(__name__)

# ── Screenshot ────────────────────────────────────────────────────────────

# Standard ImageNet-style normalisation; adjust if your model uses different stats.
_SCREENSHOT_TRANSFORM = transforms.Compose([
    transforms.Resize((224, 224)),
    transforms.ToTensor(),  # HWC uint8 [0,255] -> CHW float [0,1]
    transforms.Normalize(mean=[0.485, 0.456, 0.406], std=[0.229, 0.224, 0.225]),
])


def preprocess_screenshot(b64_png: str) -> torch.Tensor:
    """Base64-encoded PNG -> normalised float tensor ``(1, 3, 224, 224)``."""
    if not b64_png:
        return torch.zeros(1, 3, 224, 224)

    raw = base64.b64decode(b64_png)
    img = Image.open(io.BytesIO(raw)).convert("RGB")
    tensor = _SCREENSHOT_TRANSFORM(img)  # (3, 224, 224)
    return tensor.unsqueeze(0)  # (1, 3, 224, 224)


# ── Accessibility tree ────────────────────────────────────────────────────

# Simple vocabulary built on the fly; a production model would use a
# pre-built tokenizer (e.g. SentencePiece).  This gives a workable
# integer-encoded sequence for prototyping.

_MAX_TREE_TOKENS = 256


def _flatten_tree(node: dict[str, Any], parts: list[str], depth: int = 0) -> None:
    """Recursively flatten an accessibility tree node to text tokens."""
    role = node.get("role", "")
    name = node.get("name", "")
    value = node.get("value", "")

    token = f"{role}:{name}"
    if value:
        token += f"={value}"
    parts.append(token)

    for child in node.get("children", []):
        _flatten_tree(child, parts, depth + 1)


def preprocess_accessibility_tree(
    tree: dict[str, Any],
    vocab: dict[str, int] | None = None,
) -> torch.LongTensor:
    """Accessibility-tree dict -> padded integer tensor ``(1, MAX_TREE_TOKENS)``.

    If *vocab* is ``None`` a hash-based encoding is used (good enough for
    prototyping; swap for a learned tokenizer later).
    """
    parts: list[str] = []
    if tree:
        _flatten_tree(tree, parts)

    if vocab is not None:
        ids = [vocab.get(t, 0) for t in parts]
    else:
        # Deterministic hash encoding into a 30k vocab range
        ids = [(hash(t) % 30000) + 1 for t in parts]

    # Pad / truncate
    ids = ids[:_MAX_TREE_TOKENS]
    ids += [0] * (_MAX_TREE_TOKENS - len(ids))
    return torch.tensor([ids], dtype=torch.long)  # (1, 256)


# ── Recent events ─────────────────────────────────────────────────────────

_EVENT_TYPES = ["click", "keypress", "scroll", "drag"]
_MODIFIER_NAMES = ["cmd", "shift", "alt", "ctrl"]
_MAX_EVENTS = 16
# Feature dim per event: one-hot type (4) + x (1) + y (1) + key hash (1) + modifiers (4) = 11
_EVENT_FEATURE_DIM = len(_EVENT_TYPES) + 2 + 1 + len(_MODIFIER_NAMES)


def preprocess_events(events: list[dict[str, Any]]) -> torch.Tensor:
    """List of InputEvent dicts -> float tensor ``(1, MAX_EVENTS, 11)``."""
    out = torch.zeros(1, _MAX_EVENTS, _EVENT_FEATURE_DIM)

    for i, evt in enumerate(events[:_MAX_EVENTS]):
        etype = evt.get("type", "")
        # One-hot event type
        if etype in _EVENT_TYPES:
            out[0, i, _EVENT_TYPES.index(etype)] = 1.0

        # Normalised coordinates (assume 1920x1080 default)
        x = evt.get("x") or 0.0
        y = evt.get("y") or 0.0
        out[0, i, len(_EVENT_TYPES)] = x / 1920.0
        out[0, i, len(_EVENT_TYPES) + 1] = y / 1080.0

        # Key hash (normalised to [0, 1])
        key = evt.get("key", "") or ""
        out[0, i, len(_EVENT_TYPES) + 2] = (hash(key) % 1000) / 1000.0 if key else 0.0

        # Modifier flags
        modifiers = [m.lower() for m in evt.get("modifiers", [])]
        for j, mod in enumerate(_MODIFIER_NAMES):
            if mod in modifiers:
                out[0, i, len(_EVENT_TYPES) + 3 + j] = 1.0

    return out  # (1, 16, 11)


# ── Public entry point ────────────────────────────────────────────────────

def preprocess(context: ContextPayload) -> dict[str, torch.Tensor]:
    """Convert a full ``ContextPayload`` into a dict of named tensors.

    The dict keys match what the model's ``forward()`` expects::

        {
            "screenshot": Tensor (1, 3, 224, 224),
            "accessibility_tree": LongTensor (1, 256),
            "events": Tensor (1, 16, 11),
        }

    Call ``.to(device)`` on each value before passing to the model.
    """
    return {
        "screenshot": preprocess_screenshot(context.screenshot_b64),
        "accessibility_tree": preprocess_accessibility_tree(context.accessibility_tree),
        "events": preprocess_events(
            [e.model_dump() for e in context.recent_events]
        ),
    }
