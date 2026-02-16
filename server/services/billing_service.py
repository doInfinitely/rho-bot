"""
Stripe billing service for rho-bot.
Handles checkout sessions, webhooks, and subscription management.
"""

from __future__ import annotations

import logging

import stripe
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from server.config import settings
from server.models.database import Subscription

logger = logging.getLogger(__name__)

# ---- Plan config ----

PLAN_CONFIG = {
    "pro": {
        "name": "Pro",
        "price_id": settings.stripe_pro_price_id,
        "tasks_limit": 500,
    },
    "team": {
        "name": "Team",
        "price_id": settings.stripe_team_price_id,
        "tasks_limit": 2000,
    },
}


def _init_stripe():
    stripe.api_key = settings.stripe_secret_key


# ---- Helpers ----


async def get_or_create_customer(
    db: AsyncSession, user_id: str, email: str
) -> tuple[Subscription, bool]:
    """Return the user's subscription row, creating one (with a Stripe customer) if needed."""
    _init_stripe()

    result = await db.execute(
        select(Subscription).where(Subscription.user_id == user_id)
    )
    sub = result.scalar_one_or_none()

    if sub is not None:
        return sub, False

    # Create a Stripe customer
    customer = stripe.Customer.create(email=email, metadata={"user_id": user_id})

    sub = Subscription(
        user_id=user_id,
        stripe_customer_id=customer.id,
        plan_id="free",
        status="active",
        tasks_limit=FREE_TASKS_LIMIT,
    )
    db.add(sub)
    await db.commit()
    await db.refresh(sub)
    return sub, True


async def create_checkout_session(
    db: AsyncSession, user_id: str, email: str, plan_id: str
) -> str:
    """Create a Stripe Checkout session and return its URL."""
    _init_stripe()

    plan = PLAN_CONFIG.get(plan_id)
    if plan is None:
        raise ValueError(f"Unknown plan: {plan_id}")

    sub, _ = await get_or_create_customer(db, user_id, email)

    session = stripe.checkout.Session.create(
        customer=sub.stripe_customer_id,
        mode="subscription",
        line_items=[{"price": plan["price_id"], "quantity": 1}],
        subscription_data={"trial_period_days": 14},
        success_url=f"{settings.frontend_url}/dashboard/billing?success=1",
        cancel_url=f"{settings.frontend_url}/dashboard/billing?canceled=1",
        metadata={"user_id": user_id, "plan_id": plan_id},
    )

    return session.url


async def create_billing_portal_session(
    db: AsyncSession, user_id: str
) -> str:
    """Create a Stripe Billing Portal session and return its URL."""
    _init_stripe()

    result = await db.execute(
        select(Subscription).where(Subscription.user_id == user_id)
    )
    sub = result.scalar_one_or_none()
    if sub is None:
        raise ValueError("No subscription found")

    session = stripe.billing_portal.Session.create(
        customer=sub.stripe_customer_id,
        return_url=f"{settings.frontend_url}/dashboard/billing",
    )

    return session.url


async def get_subscription(db: AsyncSession, user_id: str) -> Subscription | None:
    """Return the user's subscription, or None."""
    result = await db.execute(
        select(Subscription).where(Subscription.user_id == user_id)
    )
    return result.scalar_one_or_none()


# ---- Quota enforcement ----

# Subscription statuses that permit usage
_ACTIVE_STATUSES = {"active", "trialing"}

FREE_TASKS_LIMIT = 999_999_999  # uncapped for now


async def get_or_create_subscription(db: AsyncSession, user_id: str) -> Subscription:
    """Return the user's subscription, creating a free-tier row if none exists."""
    result = await db.execute(
        select(Subscription).where(Subscription.user_id == user_id)
    )
    sub = result.scalar_one_or_none()
    if sub is not None:
        return sub

    # First time this user touches billing — bootstrap a free-tier row.
    # No Stripe customer is created yet; that happens when they upgrade.
    sub = Subscription(
        user_id=user_id,
        plan_id="free",
        status="active",
        tasks_limit=FREE_TASKS_LIMIT,
    )
    db.add(sub)
    await db.commit()
    await db.refresh(sub)
    return sub


async def check_and_increment_quota(
    db: AsyncSession, user_id: str
) -> tuple[bool, str]:
    """Check whether the user may consume a task, and if so, increment usage.

    Returns
    -------
    (allowed, reason)
        *allowed* is ``True`` when the action should proceed.
        When ``False``, *reason* contains a human-readable explanation.
    """
    sub = await get_or_create_subscription(db, user_id)

    # 1. Subscription must be in good standing
    if sub.status not in _ACTIVE_STATUSES:
        return False, (
            f"Your subscription is {sub.status}. "
            "Please update your payment method at https://rho.bot/dashboard/billing"
        )

    # 2. Must be under the task quota (free plan is uncapped for now)
    if sub.plan_id != "free" and sub.tasks_used >= sub.tasks_limit:
        return False, (
            f"You've used all {sub.tasks_limit} tasks included in your "
            f"{sub.plan_id.title()} plan this period. "
            "Upgrade your plan or wait for the next billing cycle."
        )

    # All good — bump usage
    sub.tasks_used = (sub.tasks_used or 0) + 1
    await db.commit()
    return True, ""


# ---- Webhook handling ----


async def handle_webhook_event(db: AsyncSession, event: dict) -> None:
    """Process a Stripe webhook event."""
    event_type = event["type"]
    data = event["data"]["object"]

    logger.info(f"Processing Stripe event: {event_type}")

    if event_type in (
        "customer.subscription.created",
        "customer.subscription.updated",
    ):
        await _upsert_subscription(db, data)

    elif event_type == "customer.subscription.deleted":
        await _cancel_subscription(db, data)

    elif event_type == "invoice.payment_failed":
        customer_id = data.get("customer")
        if customer_id:
            result = await db.execute(
                select(Subscription).where(
                    Subscription.stripe_customer_id == customer_id
                )
            )
            sub = result.scalar_one_or_none()
            if sub:
                sub.status = "past_due"
                await db.commit()


async def _upsert_subscription(db: AsyncSession, stripe_sub: dict) -> None:
    """Update local subscription from Stripe subscription object."""
    customer_id = stripe_sub["customer"]

    result = await db.execute(
        select(Subscription).where(
            Subscription.stripe_customer_id == customer_id
        )
    )
    sub = result.scalar_one_or_none()
    if sub is None:
        logger.warning(f"No local subscription for customer {customer_id}")
        return

    # Determine plan from the price ID
    price_id = stripe_sub["items"]["data"][0]["price"]["id"]
    plan_id = "free"
    plan_name = "Free"
    tasks_limit = 50

    for pid, config in PLAN_CONFIG.items():
        if config["price_id"] == price_id:
            plan_id = pid
            plan_name = config["name"]
            tasks_limit = config["tasks_limit"]
            break

    sub.stripe_subscription_id = stripe_sub["id"]
    sub.plan_id = plan_id
    sub.status = stripe_sub["status"]
    sub.current_period_end = stripe_sub["current_period_end"]
    sub.tasks_limit = tasks_limit

    await db.commit()
    logger.info(
        f"Updated subscription for customer {customer_id}: "
        f"plan={plan_id}, status={stripe_sub['status']}"
    )


async def _cancel_subscription(db: AsyncSession, stripe_sub: dict) -> None:
    """Handle subscription cancellation."""
    customer_id = stripe_sub["customer"]

    result = await db.execute(
        select(Subscription).where(
            Subscription.stripe_customer_id == customer_id
        )
    )
    sub = result.scalar_one_or_none()
    if sub is None:
        return

    sub.plan_id = "free"
    sub.status = "canceled"
    sub.stripe_subscription_id = None
    sub.tasks_limit = 50
    sub.tasks_used = 0

    await db.commit()
    logger.info(f"Canceled subscription for customer {customer_id}")
