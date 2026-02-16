"""Billing endpoints: Stripe checkout, portal, webhooks, subscription info."""

from __future__ import annotations

import logging

import stripe
from fastapi import APIRouter, Depends, HTTPException, Request, status
from sqlalchemy.ext.asyncio import AsyncSession

from server.config import settings
from server.deps import get_current_user
from server.models.database import User, get_db
from server.schemas.billing import (
    CheckoutRequest,
    CheckoutResponse,
    PortalResponse,
    SubscriptionOut,
)
from server.services.billing_service import (
    create_billing_portal_session,
    create_checkout_session,
    get_subscription,
    handle_webhook_event,
)

logger = logging.getLogger(__name__)

router = APIRouter(prefix="/api/billing", tags=["billing"])

PLAN_NAMES = {"free": "Free", "pro": "Pro", "team": "Team"}


@router.get("/subscription", response_model=SubscriptionOut)
async def subscription_info(
    user: User = Depends(get_current_user),
    db: AsyncSession = Depends(get_db),
):
    """Return the authenticated user's current subscription."""
    sub = await get_subscription(db, user.id)
    if sub is None:
        # No subscription row yet — return the free defaults
        return SubscriptionOut(
            id="",
            plan_id="free",
            plan_name="Free (Unlimited)",
            status="active",
            tasks_used=0,
            tasks_limit=999_999_999,
        )

    return SubscriptionOut(
        id=sub.id,
        plan_id=sub.plan_id,
        plan_name=PLAN_NAMES.get(sub.plan_id, sub.plan_id),
        status=sub.status,
        current_period_end=sub.current_period_end,
        tasks_used=sub.tasks_used,
        tasks_limit=sub.tasks_limit,
    )


@router.post("/checkout", response_model=CheckoutResponse)
async def checkout(
    body: CheckoutRequest,
    user: User = Depends(get_current_user),
    db: AsyncSession = Depends(get_db),
):
    """Create a Stripe Checkout session for a plan upgrade."""
    if body.plan_id not in ("pro", "team"):
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail="Invalid plan. Choose 'pro' or 'team'.",
        )

    try:
        url = await create_checkout_session(db, user.id, user.email, body.plan_id)
    except Exception as e:
        logger.error(f"Checkout error: {e}")
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail="Failed to create checkout session",
        )

    return CheckoutResponse(url=url)


@router.post("/portal", response_model=PortalResponse)
async def billing_portal(
    user: User = Depends(get_current_user),
    db: AsyncSession = Depends(get_db),
):
    """Create a Stripe Billing Portal session."""
    try:
        url = await create_billing_portal_session(db, user.id)
    except ValueError as e:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST, detail=str(e)
        )
    except Exception as e:
        logger.error(f"Portal error: {e}")
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail="Failed to create billing portal session",
        )

    return PortalResponse(url=url)


@router.post("/webhook")
async def stripe_webhook(
    request: Request,
    db: AsyncSession = Depends(get_db),
):
    """Handle incoming Stripe webhook events."""
    payload = await request.body()
    sig_header = request.headers.get("stripe-signature", "")

    try:
        event = stripe.Webhook.construct_event(
            payload, sig_header, settings.stripe_webhook_secret
        )
    except stripe.error.SignatureVerificationError:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail="Invalid webhook signature",
        )
    except Exception as e:
        logger.error(f"Webhook parse error: {e}")
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail="Invalid webhook payload",
        )

    await handle_webhook_event(db, event)
    return {"received": True}
