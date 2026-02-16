"""
Modal GPU inference service for rho-bot.

Loads an (optionally encrypted) ActionPolicy checkpoint from a Modal
Volume on cold start, decrypts it into GPU memory, and serves predictions
via an HTTPS endpoint that the Railway-hosted FastAPI server calls.

Deploy::

    modal deploy inference/app.py

One-time volume setup::

    modal volume create rho-bot-models
    modal volume put rho-bot-models action_ckpt_0080.pt model.pt

Or with encryption::

    python -m server.services.model_security encrypt action_ckpt_0080.pt model.pt.enc --key $KEY
    modal volume put rho-bot-models model.pt.enc model.pt.enc
"""

from __future__ import annotations

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

# Persistent volume for model weights
model_volume = modal.Volume.from_name("rho-bot-models", create_if_missing=True)

# Secret that holds the AES-256 decryption key
model_secret = modal.Secret.from_name("rho-bot-model-key")

# Container image with all ML dependencies
inference_image = (
    modal.Image.debian_slim(python_version="3.12")
    .pip_install(
        "fastapi[standard]",
        "torch>=2.0.0",
        "torchvision>=0.15.0",
        "Pillow>=10.0.0",
        "cryptography>=42.0.0",
        "pydantic>=2.0.0",
        "transformers>=4.40.0",
        "open-clip-torch>=2.24.0",
    )
    # Mount the server package so we can import model_arch/preprocessing/postprocessing
    .add_local_dir("server", remote_path="/app/server")
)

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


def _find_model_file(encryption_key: str) -> str | None:
    """Search common paths for the model weights file.

    Handles the double-nesting that occurs when ``modal volume put``
    places files under a subdirectory (e.g. ``/models/models/...``).
    """
    enc_candidates = [
        "/models/model.pt.enc",
        "/models/models/model.pt.enc",
    ]
    plain_candidates = [
        "/models/model.pt",
        "/models/models/model.pt",
    ]

    candidates = enc_candidates if encryption_key else plain_candidates
    for path in candidates:
        if os.path.exists(path):
            return path
    return None


# ---------------------------------------------------------------------------
# Inference class
# ---------------------------------------------------------------------------


@app.cls(
    image=inference_image,
    gpu="A10G",
    volumes={"/models": model_volume},
    secrets=[model_secret],
    scaledown_window=300,  # keep warm for 5 min after last request
    timeout=60,
)
class Inference:
    """Loads the model once per container, then serves predictions."""

    @modal.enter()
    def load_model(self):
        """Called once when the container starts."""
        import torch

        sys.path.insert(0, "/app")

        encryption_key = os.environ.get("RHOBOT_MODEL_ENCRYPTION_KEY", "")

        # List what's actually in the volume for debugging
        for dirpath, dirnames, filenames in os.walk("/models"):
            for fn in filenames:
                logger.info("Volume file: %s", os.path.join(dirpath, fn))

        model_path = _find_model_file(encryption_key)

        if model_path is None:
            logger.warning("No model weights found in /models — running in stub mode")
            self.model = None
            self.device = torch.device("cuda" if torch.cuda.is_available() else "cpu")
            return

        self.device = torch.device("cuda" if torch.cuda.is_available() else "cpu")
        logger.info("Loading model from %s onto %s …", model_path, self.device)

        if encryption_key:
            from server.services.model_security import decrypt_file

            raw_bytes = decrypt_file(model_path, encryption_key)
            buf = io.BytesIO(raw_bytes)
            ckpt = torch.load(buf, map_location=self.device, weights_only=False)
        else:
            ckpt = torch.load(model_path, map_location=self.device, weights_only=False)

        # Extract the policy state_dict from the training checkpoint
        if isinstance(ckpt, dict) and "policy" in ckpt:
            state_dict = ckpt["policy"]
        elif isinstance(ckpt, dict):
            state_dict = ckpt
        else:
            logger.error("Unexpected checkpoint type: %s", type(ckpt))
            self.model = None
            return

        from server.services.model_arch import load_policy

        self.model = load_policy(state_dict, self.device)
        logger.info(
            "ActionPolicy loaded (epoch %s) on %s",
            ckpt.get("epoch", "?") if isinstance(ckpt, dict) else "?",
            self.device,
        )

    @modal.fastapi_endpoint(method="POST")
    def predict(self, payload: dict):
        """Accept a ContextPayload dict, return an ActionPayload dict."""
        import torch

        if "/app" not in sys.path:
            sys.path.insert(0, "/app")

        from server.schemas.action import ActionPayload, ActionType
        from server.schemas.context import ContextPayload
        from server.services.postprocessing import postprocess
        from server.services.preprocessing import preprocess

        try:
            # Default window_bounds if null/missing
            if payload.get("window_bounds") is None:
                payload.pop("window_bounds", None)

            context = ContextPayload(**payload)

            # Stub mode — no weights loaded
            if self.model is None:
                return ActionPayload(
                    action_id=str(uuid.uuid4()),
                    type=ActionType.noop,
                    confidence=0.0,
                ).model_dump()

            # Preprocess
            tensors = preprocess(context, device=self.device)
            tensors = {k: v.to(self.device) for k, v in tensors.items()}

            # Forward pass
            with torch.no_grad():
                output = self.model(**tensors)

            # Postprocess
            action = postprocess(output, context.window_bounds)
            return action.model_dump()

        except Exception:
            logger.error("predict failed:\n%s", traceback.format_exc())
            return ActionPayload(
                action_id=str(uuid.uuid4()),
                type=ActionType.noop,
                confidence=0.0,
                error=traceback.format_exc(),
            ).model_dump()
