"""
Modal GPU inference service for rho-bot.

Loads an encrypted PyTorch model from a Modal Volume on cold start,
decrypts it into GPU memory, and serves predictions via an HTTPS
endpoint that the Railway-hosted FastAPI server calls.

Deploy::

    modal deploy inference/app.py

One-time volume setup::

    modal volume create rho-bot-models
    modal volume put rho-bot-models model.pt.enc /models/model.pt.enc
"""

from __future__ import annotations

import io
import logging
import sys
import uuid

import modal

logger = logging.getLogger(__name__)

# ---------------------------------------------------------------------------
# Modal resources
# ---------------------------------------------------------------------------

app = modal.App("rho-bot-inference")

# Persistent volume for encrypted model weights
model_volume = modal.Volume.from_name("rho-bot-models", create_if_missing=True)

# Secret that holds the AES-256 decryption key
model_secret = modal.Secret.from_name("rho-bot-model-key")

# Container image with all ML dependencies
inference_image = (
    modal.Image.debian_slim(python_version="3.12")
    .pip_install(
        "torch>=2.0.0",
        "torchvision>=0.15.0",
        "Pillow>=10.0.0",
        "cryptography>=42.0.0",
        "pydantic>=2.0.0",
    )
    # Mount the server package so we can import preprocessing/postprocessing/schemas
    .add_local_dir("server", remote_path="/app/server")
)

# ---------------------------------------------------------------------------
# Inference class
# ---------------------------------------------------------------------------


@app.cls(
    image=inference_image,
    gpu="A10G",
    volumes={"/models": model_volume},
    secrets=[model_secret],
    container_idle_timeout=300,  # keep warm for 5 min after last request
    timeout=60,
)
class Inference:
    """Loads the model once per container, then serves predictions."""

    @modal.enter()
    def load_model(self):
        """Called once when the container starts."""
        import os

        import torch

        sys.path.insert(0, "/app")

        encryption_key = os.environ.get("RHOBOT_MODEL_ENCRYPTION_KEY", "")
        model_path = "/models/model.pt.enc" if encryption_key else "/models/model.pt"

        # Check if weights exist
        if not os.path.exists(model_path):
            logger.warning("No model weights found at %s — running in stub mode", model_path)
            self.model = None
            self.device = torch.device("cuda" if torch.cuda.is_available() else "cpu")
            return

        self.device = torch.device("cuda" if torch.cuda.is_available() else "cpu")
        logger.info("Loading model from %s onto %s …", model_path, self.device)

        if encryption_key:
            from server.services.model_security import decrypt_file

            raw_bytes = decrypt_file(model_path, encryption_key)
            buf = io.BytesIO(raw_bytes)
            state = torch.load(buf, map_location=self.device, weights_only=True)
        else:
            state = torch.load(model_path, map_location=self.device, weights_only=True)

        if isinstance(state, torch.nn.Module):
            self.model = state
        elif isinstance(state, dict):
            logger.error(
                "Loaded a state_dict but no model architecture is registered. "
                "Update inference/app.py to instantiate your nn.Module and "
                "call model.load_state_dict(state)."
            )
            self.model = None
            return
        else:
            logger.error("Unexpected checkpoint type: %s", type(state))
            self.model = None
            return

        self.model.to(self.device)
        self.model.eval()
        logger.info("Model ready on %s", self.device)

    @modal.fastapi_endpoint(method="POST")
    def predict(self, payload: dict):
        """Accept a ContextPayload dict, return an ActionPayload dict."""
        import torch

        sys.path.insert(0, "/app") if "/app" not in sys.path else None

        from server.schemas.action import ActionPayload, ActionType
        from server.schemas.context import ContextPayload
        from server.services.postprocessing import postprocess
        from server.services.preprocessing import preprocess

        context = ContextPayload(**payload)

        # Stub mode — no weights loaded
        if self.model is None:
            return ActionPayload(
                action_id=str(uuid.uuid4()),
                type=ActionType.noop,
                confidence=0.0,
            ).model_dump()

        # Preprocess
        tensors = preprocess(context)
        tensors = {k: v.to(self.device) for k, v in tensors.items()}

        # Forward pass
        with torch.no_grad():
            output = self.model(**tensors)

        # Postprocess
        action = postprocess(output, context.window_bounds)
        return action.model_dump()
