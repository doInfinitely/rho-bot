"""
Modal GPU inference service for rho-bot.

Loads a MacroDose HierarchicalModel checkpoint from a Modal Volume on cold
start and serves action predictions via an HTTPS endpoint that the
Railway-hosted FastAPI server calls.

The model takes a screenshot + accessibility tree and returns a click
coordinate or keypress action.

Deploy::

    modal deploy inference/app.py

Volume setup::

    modal volume put rho-bot-models <checkpoint>.pt model.pt
"""

from __future__ import annotations

import base64
import io
import logging
import os
import sys
import traceback
import uuid

import modal

logger = logging.getLogger(__name__)

# ---------------------------------------------------------------------------
# Modal resources
# ---------------------------------------------------------------------------

app = modal.App("rho-bot-inference")

model_volume = modal.Volume.from_name("rho-bot-models", create_if_missing=True)

# Path to the macrodose source tree (sibling directory)
MACRODOSE_DIR = os.path.join(os.path.dirname(os.path.dirname(__file__)), "..", "macrodose")
MACRODOSE_DIR = os.path.normpath(
    os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "macrodose")
)

inference_image = (
    modal.Image.debian_slim(python_version="3.12")
    .pip_install(
        "fastapi[standard]",
        "torch>=2.0.0",
        "torchvision>=0.15.0",
        "Pillow>=10.0.0",
        "pydantic>=2.0.0",
        "transformers>=4.40.0",
        "numpy>=1.24",
    )
    # Mount rho-bot server schemas for ActionPayload / ContextPayload
    .add_local_dir("server", remote_path="/app/server")
    # Mount macrodose model code for HierarchicalModel
    .add_local_dir(os.path.join(MACRODOSE_DIR, "model"), remote_path="/app/model")
    .add_local_dir(os.path.join(MACRODOSE_DIR, "inference"), remote_path="/app/inference")
    .add_local_dir(os.path.join(MACRODOSE_DIR, "training"), remote_path="/app/training")
    .add_local_dir(os.path.join(MACRODOSE_DIR, "planner"), remote_path="/app/planner")
    .add_local_dir(os.path.join(MACRODOSE_DIR, "logger"), remote_path="/app/logger")
)


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def _find_model_file() -> str | None:
    """Search common paths for the model weights file."""
    candidates = [
        "/models/model.pt",
        "/models/models/model.pt",
    ]
    for path in candidates:
        if os.path.exists(path):
            return path
    return None


# ---------------------------------------------------------------------------
# Key mapping: macrodose key index → rho-bot key name
# ---------------------------------------------------------------------------

# Macrodose KEYBOARD_KEYS ordering (from model/action_head.py)
# Letters a-z (0-25), digits 0-9 (26-35), modifiers (36-45),
# whitespace/editing (46-50), navigation (51-58), F-keys (59-70),
# punctuation (71-82)

def _macrodose_key_to_rhobot(key_name: str) -> str | None:
    """Map a macrodose key name to a rho-bot postprocessing key name."""
    mapping = {
        "enter": "return", "space": "space", "tab": "tab",
        "backspace": "delete", "escape": "escape",
        "up": "up", "down": "down", "left": "left", "right": "right",
        "home": "home", "end": "end", "page_up": "pageup", "page_down": "pagedown",
        "shift": "shift", "ctrl": "ctrl", "alt": "alt", "cmd": "cmd",
        "caps_lock": "capslock",
    }
    # Single letters and digits pass through
    if len(key_name) == 1 and (key_name.isalpha() or key_name.isdigit()):
        return key_name
    # F-keys
    if key_name.startswith("f") and key_name[1:].isdigit():
        return key_name
    return mapping.get(key_name)


# ---------------------------------------------------------------------------
# Inference class
# ---------------------------------------------------------------------------

MOUSE_GRID = 64


@app.cls(
    image=inference_image,
    gpu="T4",
    volumes={"/models": model_volume},
    scaledown_window=300,
    timeout=60,
)
class Inference:
    """Loads the MacroDose model once per container, serves predictions."""

    @modal.enter()
    def load_model(self):
        import torch

        sys.path.insert(0, "/app")

        for dirpath, _, filenames in os.walk("/models"):
            for fn in filenames:
                logger.info("Volume file: %s", os.path.join(dirpath, fn))

        model_path = _find_model_file()

        if model_path is None:
            logger.warning("No model weights found in /models — stub mode")
            self.executor = None
            return

        self.device = torch.device("cuda" if torch.cuda.is_available() else "cpu")
        logger.info("Loading HierarchicalModel from %s onto %s", model_path, self.device)

        from inference.hierarchical_executor import HierarchicalExecutor

        self.executor = HierarchicalExecutor(
            checkpoint_path=model_path,
            embed_dim=512,
            device=str(self.device),
        )
        logger.info("HierarchicalModel loaded on %s", self.device)

    @modal.fastapi_endpoint(method="POST")
    def predict(self, payload: dict):
        """Accept a ContextPayload dict, return an ActionPayload dict."""
        import torch
        import torch.nn.functional as F
        from PIL import Image

        if "/app" not in sys.path:
            sys.path.insert(0, "/app")

        from server.schemas.action import ActionPayload, ActionType
        from server.schemas.context import ContextPayload

        try:
            if payload.get("window_bounds") is None:
                payload.pop("window_bounds", None)

            context = ContextPayload(**payload)

            if self.executor is None:
                return ActionPayload(
                    action_id=str(uuid.uuid4()),
                    type=ActionType.noop,
                    confidence=0.0,
                ).model_dump()

            # Decode screenshot
            if not context.screenshot_b64:
                return ActionPayload(
                    action_id=str(uuid.uuid4()),
                    type=ActionType.noop,
                    confidence=0.0,
                ).model_dump()

            raw = base64.b64decode(context.screenshot_b64)
            screenshot = Image.open(io.BytesIO(raw)).convert("RGB")

            wb = context.window_bounds
            screen_size = (int(wb.width), int(wb.height))

            # Convert recent_events to history format
            history_events = None
            if context.recent_events:
                history_events = []
                for evt in context.recent_events:
                    h = {
                        "event_type": evt.type,
                        "timestamp": evt.timestamp,
                        "mouse_x": evt.x,
                        "mouse_y": evt.y,
                        "key_state": {},
                    }
                    if evt.key:
                        h["key_state"] = {evt.key: True}
                    history_events.append(h)

            # Run hierarchical inference
            result = self.executor.execute(
                screenshot=screenshot,
                accessibility_tree=context.accessibility_tree or None,
                screen_size=screen_size,
                history_events=history_events,
            )

            # Convert to ActionPayload
            mx, my = result.mouse_xy
            mouse_conf = result.levels[-1].confidence if result.levels else 0.0

            # Check if any key has high probability
            best_key_name = None
            best_key_conf = 0.0
            for key_name, prob in result.key_probs.items():
                rho_name = _macrodose_key_to_rhobot(key_name)
                if rho_name and prob > best_key_conf:
                    best_key_conf = prob
                    best_key_name = rho_name

            # Decide click vs keypress
            if best_key_name and best_key_conf > mouse_conf and best_key_conf > 0.5:
                return ActionPayload(
                    action_id=str(uuid.uuid4()),
                    type=ActionType.keypress,
                    key=best_key_name,
                    confidence=round(best_key_conf, 4),
                ).model_dump()
            else:
                # Offset coordinates by window bounds
                px_x = mx + wb.x
                px_y = my + wb.y

                return ActionPayload(
                    action_id=str(uuid.uuid4()),
                    type=ActionType.click,
                    coordinates=[round(px_x, 1), round(px_y, 1)],
                    confidence=round(mouse_conf, 4),
                ).model_dump()

        except Exception:
            logger.error("predict failed:\n%s", traceback.format_exc())
            return ActionPayload(
                action_id=str(uuid.uuid4()),
                type=ActionType.noop,
                confidence=0.0,
                error=traceback.format_exc(),
            ).model_dump()
