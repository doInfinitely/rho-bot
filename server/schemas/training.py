"""Schema for passive-recording training data (context + observed user actions)."""

from __future__ import annotations

from pydantic import BaseModel, Field

from server.schemas.context import ContextPayload, InputEvent


class TrainingPayload(BaseModel):
    """A context/action pair captured while the agent is inactive.

    The *context* is the screen state (screenshot, accessibility tree, etc.)
    at the moment of capture.  *user_actions* are the input events the user
    performed in the interval that followed, serving as ground-truth labels
    for behavioural-cloning training.
    """

    context: ContextPayload
    user_actions: list[InputEvent] = Field(default_factory=list)
