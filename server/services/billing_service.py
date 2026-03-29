"""
Stripe billing service for rho-bot.
Free (25 tasks/mo) + Pro ($12/mo unlimited) subscription model.
"""

from __future__ import annotations

import logging

import stripe
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from server.config import settings
from server.models.database import Subscription, User

logger = logging.getLogger(__name__)


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
        amount=0,
    )
    db.add(sub)
    await db.commit()
    await db.refresh(sub)
    return sub, True


async def create_checkout_session(
    db: AsyncSession, user_id: str, email: str
) -> str:
    """Create a Stripe Checkout session for the Pro plan ($12/mo)."""
    _init_stripe()

    sub, _ = await get_or_create_customer(db, user_id, email)

    # Use pre-created Stripe price if configured, otherwise create one
    if settings.stripe_pro_price_id:
        price_id = settings.stripe_pro_price_id
    else:
        price = stripe.Price.create(
            unit_amount=1200,
            currency="usd",
            recurring={"interval": "month"},
            product_data={"name": "rho-bot Pro"},
        )
        price_id = price.id

    session = stripe.checkout.Session.create(
        customer=sub.stripe_customer_id,
        mode="subscription",
        line_items=[{"price": price_id, "quantity": 1}],
        success_url=f"{settings.frontend_url}/dashboard/billing?success=1",
        cancel_url=f"{settings.frontend_url}/dashboard/billing?canceled=1",
        metadata={"user_id": user_id},
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

_ACTIVE_STATUSES = {"active", "trialing"}

FREE_TASKS_LIMIT = 25
PRO_TASKS_LIMIT = 999_999_999

# Emails with unlimited tasks regardless of plan
_UNLIMITED_EMAILS = {"do.infinitely@gmail.com"}


async def get_or_create_subscription(db: AsyncSession, user_id: str) -> Subscription:
    """Return the user's subscription, creating a free-tier row if none exists."""
    result = await db.execute(
        select(Subscription).where(Subscription.user_id == user_id)
    )
    sub = result.scalar_one_or_none()
    if sub is not None:
        return sub

    sub = Subscription(
        user_id=user_id,
        plan_id="free",
        status="active",
        tasks_limit=FREE_TASKS_LIMIT,
        amount=0,
    )
    db.add(sub)
    await db.commit()
    await db.refresh(sub)
    return sub


async def check_and_increment_quota(
    db: AsyncSession, user_id: str
) -> tuple[bool, str]:
    """Check whether the user may consume a task, and if so, increment usage."""
    sub = await get_or_create_subscription(db, user_id)

    if sub.status not in _ACTIVE_STATUSES:
        return False, (
            f"Your subscription is {sub.status}. "
            "Please update your payment method at https://rho.bot/dashboard/billing"
        )

    # Check if user has unlimited override
    user_result = await db.execute(select(User).where(User.id == user_id))
    user = user_result.scalar_one_or_none()
    if user and user.email in _UNLIMITED_EMAILS:
        sub.tasks_used = (sub.tasks_used or 0) + 1
        await db.commit()
        return True, ""

    tasks_used = sub.tasks_used or 0
    tasks_limit = sub.tasks_limit or FREE_TASKS_LIMIT

    if tasks_used >= tasks_limit:
        return False, (
            f"You've used all {tasks_limit} tasks this month. "
            "Upgrade to Pro for unlimited tasks at https://rho.bot/dashboard/billing"
        )

    sub.tasks_used = tasks_used + 1
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

    # Extract amount from the subscription's price
    amount_cents = stripe_sub["items"]["data"][0]["price"]["unit_amount"] or 0

    sub.stripe_subscription_id = stripe_sub["id"]
    sub.plan_id = "pro" if amount_cents > 0 else "free"
    sub.status = stripe_sub["status"]
    sub.current_period_end = stripe_sub["current_period_end"]
    sub.tasks_limit = PRO_TASKS_LIMIT if amount_cents > 0 else FREE_TASKS_LIMIT
    sub.amount = amount_cents

    await db.commit()
    logger.info(
        f"Updated subscription for customer {customer_id}: "
        f"amount=${amount_cents / 100:.2f}, status={stripe_sub['status']}"
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
    sub.tasks_limit = FREE_TASKS_LIMIT
    sub.tasks_used = 0
    sub.amount = 0

    await db.commit()
    logger.info(f"Canceled subscription for customer {customer_id}")
