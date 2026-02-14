from __future__ import annotations

from pydantic import BaseModel


class SubscriptionOut(BaseModel):
    id: str
    plan_id: str
    plan_name: str
    status: str
    current_period_end: float | None = None
    tasks_used: int = 0
    tasks_limit: int = 50

    class Config:
        from_attributes = True


class CheckoutRequest(BaseModel):
    plan_id: str  # "pro" or "team"


class CheckoutResponse(BaseModel):
    url: str


class PortalResponse(BaseModel):
    url: str
