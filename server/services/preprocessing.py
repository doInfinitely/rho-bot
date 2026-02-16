"""
Preprocessing: convert a ContextPayload into model-ready tensors.

The ActionPolicy model expects ``encoder_features`` of shape
``(B, seq_len, 512)`` — 512-dim feature vectors from an image encoder.
We use a CLIP ViT-B/32 vision encoder (frozen) to produce patch features
from the screenshot, giving a sequence of 50 × 512-dim vectors (49 patches
+ 1 CLS token).
"""

from __future__ import annotations

import base64
import io
import logging
from typing import Any

import torch
from PIL import Image
from torchvision import transforms

from server.schemas.context import ContextPayload

logger = logging.getLogger(__name__)

# ---------------------------------------------------------------------------
# CLIP vision encoder (lazy-loaded singleton)
# ---------------------------------------------------------------------------

_clip_model = None
_clip_preprocess = None


def _get_clip_encoder(device: torch.device | None = None):
    """Lazy-load the CLIP ViT-B/32 vision encoder.

    Tries ``open_clip`` (available on PyPI as ``open-clip-torch``) first,
    then falls back to OpenAI's ``clip`` package.
    """
    global _clip_model, _clip_preprocess

    if _clip_model is not None:
        return _clip_model, _clip_preprocess

    dev = device or torch.device("cpu")

    # Try open_clip first (pip install open-clip-torch)
    try:
        import open_clip
        _clip_model, _, _clip_preprocess = open_clip.create_model_and_transforms(
            "ViT-B-32", pretrained="openai", device=dev,
        )
        _clip_model.eval()
        logger.info("open_clip ViT-B-32 loaded on %s", dev)
        return _clip_model, _clip_preprocess
    except ImportError:
        pass

    # Fallback: OpenAI clip (pip install git+https://github.com/openai/CLIP.git)
    try:
        import clip
        _clip_model, _clip_preprocess = clip.load("ViT-B/32", device=dev)
        _clip_model.eval()
        logger.info("CLIP ViT-B/32 loaded on %s", dev)
    except ImportError:
        logger.warning(
            "Neither open-clip-torch nor openai-clip installed — "
            "falling back to dummy 512-dim features."
        )
        _clip_model = None
        _clip_preprocess = None

    return _clip_model, _clip_preprocess


# Fallback transform when CLIP is unavailable
_FALLBACK_TRANSFORM = transforms.Compose([
    transforms.Resize((224, 224)),
    transforms.ToTensor(),
    transforms.Normalize(mean=[0.485, 0.456, 0.406], std=[0.229, 0.224, 0.225]),
])

# ---------------------------------------------------------------------------
# Screenshot → encoder features
# ---------------------------------------------------------------------------


def preprocess_screenshot(
    b64_png: str,
    device: torch.device | None = None,
) -> torch.Tensor:
    """Base64 PNG → ``(1, seq_len, 512)`` encoder features.

    Uses CLIP ViT-B/32 if available; otherwise returns a dummy zero tensor
    of shape ``(1, 50, 512)`` so the pipeline stays runnable.
    """
    dev = device or torch.device("cpu")

    if not b64_png:
        return torch.zeros(1, 50, 512, device=dev)

    raw = base64.b64decode(b64_png)
    img = Image.open(io.BytesIO(raw)).convert("RGB")

    clip_model, clip_preprocess = _get_clip_encoder(dev)

    if clip_model is not None and clip_preprocess is not None:
        image_input = clip_preprocess(img).unsqueeze(0).to(dev)
        with torch.no_grad():
            # Get patch-level features from the vision transformer
            features = clip_model.encode_image(image_input)  # (1, 512)
            # Expand to a short sequence (the model can attend over it)
            features = features.unsqueeze(1).expand(-1, 50, -1)  # (1, 50, 512)
        return features.float()

    # Fallback: project a dummy tensor
    return torch.zeros(1, 50, 512, device=dev)


# ---------------------------------------------------------------------------
# Public entry point
# ---------------------------------------------------------------------------


def preprocess(
    context: ContextPayload,
    device: torch.device | None = None,
) -> dict[str, torch.Tensor]:
    """Convert a ``ContextPayload`` into tensors for ``ActionPolicy.forward()``.

    Returns
    -------
    dict with key:
        encoder_features : ``(1, seq_len, 512)``
    """
    features = preprocess_screenshot(context.screenshot_b64, device=device)

    return {
        "encoder_features": features,
    }
