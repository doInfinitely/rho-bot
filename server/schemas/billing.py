from __future__ import annotations

from pydantic import BaseModel, Field


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


class CheckoutRequest(BaseModel):
    amount: int = Field(ge=0, description="Monthly amount in whole dollars")


class CheckoutResponse(BaseModel):
    url: str


class PortalResponse(BaseModel):
    url: str
