from __future__ import annotations

from pydantic import BaseModel


class SubscriptionOut(BaseModel):
    id: str
    plan_id: str
    plan_name: str
    status: str
    current_period_end: float | None = None
    tasks_used: int = 0
    tasks_limit: int = 0
    amount: int = 0  # monthly amount in cents

    class Config:
        from_attributes = True


class CheckoutResponse(BaseModel):
    url: str


class PortalResponse(BaseModel):
    url: str
