"""
Model inference service.

This is the clean interface between the server and your PyTorch/JAX models.
Replace the stub implementation with your actual model once trained.
"""

from __future__ import annotations

import logging
import uuid

from server.schemas.action import ActionPayload, ActionType
from server.schemas.context import ContextPayload

logger = logging.getLogger(__name__)


class ModelService:
    """Wraps the trained model(s) for action prediction.

    For now this is a stub that returns noop actions.
    Replace ``predict_action`` with real inference once your
    hierarchical goal induction models are ready.
    """

    def __init__(self, model_path: str = "") -> None:
        self.model_path = model_path
        self._model = None
        if model_path:
            self._load_model(model_path)

    # ------------------------------------------------------------------
    # Private helpers
    # ------------------------------------------------------------------

    def _load_model(self, path: str) -> None:
        """Load PyTorch/JAX checkpoint from *path*."""
        logger.info("Loading model from %s ...", path)
        # TODO: replace with actual model loading
        # e.g. self._model = torch.load(path)
        self._model = None

    # ------------------------------------------------------------------
    # Public API
    # ------------------------------------------------------------------

    async def predict_action(self, context: ContextPayload) -> ActionPayload:
        """Given a context bundle, predict the next action.

        Stub implementation: returns a ``noop`` with zero confidence.
        """
        if self._model is not None:
            # TODO: real inference
            #   preprocessed = preprocess(context)
            #   raw = self._model(preprocessed)
            #   return postprocess(raw)
            pass

        # ---- stub ----
        return ActionPayload(
            action_id=str(uuid.uuid4()),
            type=ActionType.noop,
            confidence=0.0,
        )
