"""
Model inference service.

Supports two modes:

1. **Remote** (production on Railway): POSTs ``ContextPayload`` JSON to a
   Modal GPU endpoint and receives ``ActionPayload`` JSON back.  Activated
   when ``model_inference_url`` is set.

2. **Local** (self-hosted / dev with GPU): loads an (optionally encrypted)
   PyTorch checkpoint in-process.  Activated when ``model_path`` is set.

When neither is configured the service returns ``noop`` actions so dev
environments work without any model.
"""

from __future__ import annotations

import logging
import uuid

import httpx

from server.schemas.action import ActionPayload, ActionType
from server.schemas.context import ContextPayload

logger = logging.getLogger(__name__)

_INFERENCE_TIMEOUT = 30.0  # seconds


class ModelService:
    """Unified inference interface — remote, local, or stub."""

    def __init__(
        self,
        *,
        inference_url: str = "",
        model_path: str = "",
        device_override: str = "",
        encryption_key: str = "",
    ) -> None:
        self._inference_url = inference_url.rstrip("/") if inference_url else ""
        self._local_model = None

        if self._inference_url:
            logger.info("ModelService: remote mode → %s", self._inference_url)
        elif model_path:
            self._init_local(model_path, device_override, encryption_key)
        else:
            logger.info("ModelService: stub mode (no model configured)")

    # ------------------------------------------------------------------
    # Local-mode helpers (only used when model_path is set)
    # ------------------------------------------------------------------

    def _init_local(self, path: str, device_override: str, encryption_key: str) -> None:
        """Load a PyTorch model in-process for local inference."""
        import asyncio
        import io
        from functools import partial

        import torch

        from server.services.preprocessing import preprocess
        from server.services.postprocessing import postprocess

        # Detect device
        if device_override:
            device = torch.device(device_override)
        elif torch.cuda.is_available():
            device = torch.device("cuda")
        elif hasattr(torch.backends, "mps") and torch.backends.mps.is_available():
            device = torch.device("mps")
        else:
            device = torch.device("cpu")

        logger.info("Loading model from %s onto %s …", path, device)

        if encryption_key:
            from server.services.model_security import decrypt_file

            raw_bytes = decrypt_file(path, encryption_key)
            buf = io.BytesIO(raw_bytes)
            state = torch.load(buf, map_location=device, weights_only=True)
        else:
            state = torch.load(path, map_location=device, weights_only=True)

        if isinstance(state, torch.nn.Module):
            model = state
        elif isinstance(state, dict):
            logger.error(
                "Loaded a state_dict but no model architecture is registered yet. "
                "Wrap your nn.Module subclass and load state_dict into it here."
            )
            return
        else:
            logger.error("Unexpected checkpoint type: %s", type(state))
            return

        model.to(device)
        model.eval()
        logger.info("Model loaded and set to eval mode on %s", device)

        # Stash everything needed for local inference
        self._local_model = model
        self._device = device
        self._preprocess = preprocess
        self._postprocess = postprocess
        self._torch = torch

    # ------------------------------------------------------------------
    # Public API
    # ------------------------------------------------------------------

    async def predict_action(self, context: ContextPayload) -> ActionPayload:
        """Predict the next action — delegates to remote, local, or stub."""

        # 1. Remote mode
        if self._inference_url:
            return await self._predict_remote(context)

        # 2. Local mode
        if self._local_model is not None:
            return await self._predict_local(context)

        # 3. Stub
        return ActionPayload(
            action_id=str(uuid.uuid4()),
            type=ActionType.noop,
            confidence=0.0,
        )

    # ------------------------------------------------------------------
    # Remote inference (Railway → Modal)
    # ------------------------------------------------------------------

    async def _predict_remote(self, context: ContextPayload) -> ActionPayload:
        """POST the context to the Modal endpoint and parse the response."""
        try:
            async with httpx.AsyncClient(timeout=_INFERENCE_TIMEOUT) as client:
                resp = await client.post(
                    self._inference_url,
                    json=context.model_dump(),
                )
                resp.raise_for_status()
                return ActionPayload(**resp.json())
        except httpx.HTTPStatusError as exc:
            logger.error("Inference endpoint returned %s: %s", exc.response.status_code, exc.response.text)
        except Exception as exc:
            logger.error("Remote inference failed: %s", exc)

        # On failure, return a safe noop so the agent doesn't crash.
        return ActionPayload(
            action_id=str(uuid.uuid4()),
            type=ActionType.noop,
            confidence=0.0,
        )

    # ------------------------------------------------------------------
    # Local inference (in-process PyTorch)
    # ------------------------------------------------------------------

    async def _predict_local(self, context: ContextPayload) -> ActionPayload:
        """Run preprocessing → forward → postprocessing in a thread pool."""
        import asyncio
        from functools import partial

        loop = asyncio.get_running_loop()
        return await loop.run_in_executor(None, partial(self._run_local, context))

    def _run_local(self, context: ContextPayload) -> ActionPayload:
        """Synchronous local inference — runs inside an executor thread."""
        tensors = self._preprocess(context)
        tensors = {k: v.to(self._device) for k, v in tensors.items()}

        with self._torch.no_grad():
            output = self._local_model(**tensors)

        return self._postprocess(output, context.window_bounds)
