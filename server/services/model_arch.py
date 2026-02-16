"""
ActionPolicy — GPT-2-based action prediction model.

Architecture (derived from checkpoint ``action_ckpt_0080.pt``):

    encoder_proj  : Linear(512→768) → GELU → Linear(768→768)
    gpt2          : GPT-2 Small (12 layers, 768 hidden, 12 heads)
    action_head
      ├ mouse_head: Linear(768→768) → GELU → Linear(768→4096)   # 64×64 grid
      └ key_head  : Linear(768→768) → GELU → Linear(768→83)     # 83 key classes
    value_head    : Linear(768→768) → GELU → Linear(768→1)

The model expects 512-dim vision features (from an external encoder such as
CLIP or a ResNet) and outputs mouse-position logits over a discretized 64×64
grid plus key-press logits over 83 key classes.
"""

from __future__ import annotations

import torch
import torch.nn as nn
from transformers import GPT2Config, GPT2Model

# ---------------------------------------------------------------------------
# Constants matching the checkpoint
# ---------------------------------------------------------------------------

ENCODER_DIM = 512
HIDDEN_DIM = 768
MOUSE_GRID = 64  # 64×64 = 4096 positions
NUM_MOUSE_CLASSES = MOUSE_GRID * MOUSE_GRID  # 4096
NUM_KEY_CLASSES = 83


# ---------------------------------------------------------------------------
# Sub-modules
# ---------------------------------------------------------------------------

class ActionHead(nn.Module):
    def __init__(self, hidden_dim: int = HIDDEN_DIM) -> None:
        super().__init__()
        self.mouse_head = nn.Sequential(
            nn.Linear(hidden_dim, hidden_dim),
            nn.GELU(),
            nn.Linear(hidden_dim, NUM_MOUSE_CLASSES),
        )
        self.key_head = nn.Sequential(
            nn.Linear(hidden_dim, hidden_dim),
            nn.GELU(),
            nn.Linear(hidden_dim, NUM_KEY_CLASSES),
        )

    def forward(self, hidden: torch.Tensor):
        return {
            "mouse_logits": self.mouse_head(hidden),  # (B, 4096)
            "key_logits": self.key_head(hidden),        # (B, 83)
        }


class ValueHead(nn.Module):
    def __init__(self, hidden_dim: int = HIDDEN_DIM) -> None:
        super().__init__()
        self.net = nn.Sequential(
            nn.Linear(hidden_dim, hidden_dim),
            nn.GELU(),
            nn.Linear(hidden_dim, 1),
        )

    def forward(self, hidden: torch.Tensor) -> torch.Tensor:
        return self.net(hidden)  # (B, 1)


# ---------------------------------------------------------------------------
# Full policy
# ---------------------------------------------------------------------------

class ActionPolicy(nn.Module):
    """GPT-2 backbone with projection, action heads, and value head."""

    def __init__(self) -> None:
        super().__init__()

        # Project external 512-dim vision features into GPT-2's hidden space
        self.encoder_proj = nn.Sequential(
            nn.Linear(ENCODER_DIM, HIDDEN_DIM),
            nn.GELU(),
            nn.Linear(HIDDEN_DIM, HIDDEN_DIM),
        )

        # GPT-2 Small backbone
        config = GPT2Config.from_pretrained("gpt2")
        self.gpt2 = GPT2Model(config)

        # Prediction heads
        self.action_head = ActionHead(HIDDEN_DIM)
        self.value_head = ValueHead(HIDDEN_DIM)

    def forward(
        self,
        encoder_features: torch.Tensor,
        attention_mask: torch.Tensor | None = None,
    ) -> dict[str, torch.Tensor]:
        """
        Parameters
        ----------
        encoder_features : (B, seq_len, 512)
            Vision features from an external encoder (e.g. CLIP ViT patches).
        attention_mask : (B, seq_len), optional
            1 for real tokens, 0 for padding.

        Returns
        -------
        dict with keys:
            mouse_logits  : (B, 4096)
            key_logits    : (B, 83)
            value         : (B, 1)
        """
        # Project encoder features into GPT-2 embedding space
        inputs_embeds = self.encoder_proj(encoder_features)  # (B, seq, 768)

        # Run through GPT-2
        gpt_out = self.gpt2(
            inputs_embeds=inputs_embeds,
            attention_mask=attention_mask,
        )
        hidden = gpt_out.last_hidden_state  # (B, seq, 768)

        # Use the last token's representation for prediction
        last_hidden = hidden[:, -1, :]  # (B, 768)

        # Predict actions and value
        action_out = self.action_head(last_hidden)
        value = self.value_head(last_hidden)

        return {
            "mouse_logits": action_out["mouse_logits"],
            "key_logits": action_out["key_logits"],
            "value": value,
        }


def load_policy(state_dict: dict[str, torch.Tensor], device: torch.device) -> ActionPolicy:
    """Instantiate an ActionPolicy and load a state_dict into it."""
    model = ActionPolicy()
    model.load_state_dict(state_dict, strict=True)
    model.to(device)
    model.eval()
    return model
