import uuid

from sqlalchemy import Boolean, Column, Float, Integer, String, Text
from sqlalchemy.ext.asyncio import AsyncSession, create_async_engine
from sqlalchemy.orm import DeclarativeBase, sessionmaker

from server.config import settings

engine = create_async_engine(
    settings.database_url,
    echo=settings.debug,
    connect_args={"timeout": 5},  # 5s connect timeout (asyncpg default is 60s)
)
async_session = sessionmaker(engine, class_=AsyncSession, expire_on_commit=False)


class Base(DeclarativeBase):
    pass


class User(Base):
    __tablename__ = "users"

    id = Column(String, primary_key=True, default=lambda: str(uuid.uuid4()))
    email = Column(String, unique=True, nullable=False, index=True)
    hashed_password = Column(String, nullable=False)


class Session(Base):
    __tablename__ = "sessions"

    id = Column(String, primary_key=True, default=lambda: str(uuid.uuid4()))
    user_id = Column(String, nullable=False, index=True)
    started_at = Column(Float, nullable=False)
    ended_at = Column(Float, nullable=True)
    action_count = Column(Integer, default=0)
    goal = Column(Text, default="")


class ContextLog(Base):
    __tablename__ = "context_logs"

    id = Column(String, primary_key=True, default=lambda: str(uuid.uuid4()))
    session_id = Column(String, nullable=False, index=True)
    timestamp = Column(Float, nullable=False)
    active_app = Column(String, default="")
    accessibility_tree_json = Column(Text, default="{}")
    # screenshot stored on disk, path referenced here
    screenshot_path = Column(String, default="")


class ActionLog(Base):
    __tablename__ = "action_logs"

    id = Column(String, primary_key=True, default=lambda: str(uuid.uuid4()))
    session_id = Column(String, nullable=False, index=True)
    timestamp = Column(Float, nullable=False)
    action_type = Column(String, nullable=False)
    action_payload_json = Column(Text, default="{}")
    confidence = Column(Float, default=0.0)
    success = Column(Boolean, default=True)


class TrainingPair(Base):
    """A context/action pair recorded while the agent was inactive, for later training."""

    __tablename__ = "training_pairs"

    id = Column(String, primary_key=True, default=lambda: str(uuid.uuid4()))
    session_id = Column(String, nullable=False, index=True)
    user_id = Column(String, nullable=False, index=True)
    timestamp = Column(Float, nullable=False)
    active_app = Column(String, default="")
    accessibility_tree_json = Column(Text, default="{}")
    screenshot_path = Column(String, default="")
    user_actions_json = Column(Text, default="[]")


class Subscription(Base):
    __tablename__ = "subscriptions"

    id = Column(String, primary_key=True, default=lambda: str(uuid.uuid4()))
    user_id = Column(String, nullable=False, unique=True, index=True)
    stripe_customer_id = Column(String, nullable=True, unique=True)
    stripe_subscription_id = Column(String, nullable=True, unique=True)
    plan_id = Column(String, nullable=False, default="free")  # free, pro, team
    status = Column(String, nullable=False, default="active")  # active, trialing, past_due, canceled, incomplete
    current_period_end = Column(Float, nullable=True)
    tasks_used = Column(Integer, default=0)
    tasks_limit = Column(Integer, default=50)


async def get_db() -> AsyncSession:
    async with async_session() as session:
        yield session


async def init_db():
    import asyncio
    import logging

    from sqlalchemy import text

    log = logging.getLogger(__name__)

    for attempt in range(3):
        try:
            async with engine.begin() as conn:
                await conn.run_sync(Base.metadata.create_all)
            break
        except Exception as e:
            if attempt < 2:
                wait = 2 ** attempt
                log.warning("DB init attempt %d failed (%s), retrying in %ds…", attempt + 1, e, wait)
                await asyncio.sleep(wait)
            else:
                log.error("DB init failed after %d attempts: %s", attempt + 1, e)
                raise

    # ---- lightweight migrations (add columns to existing tables) ----
    migrations = [
        "ALTER TABLE sessions ADD COLUMN IF NOT EXISTS goal TEXT DEFAULT ''",
    ]

    try:
        async with engine.begin() as conn:
            for stmt in migrations:
                await conn.execute(text(stmt))
        log.info("Migrations applied successfully")
    except Exception as e:
        log.warning("Migration step failed (non-fatal): %s", e)
