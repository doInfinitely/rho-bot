import os
import re

from pydantic_settings import BaseSettings


def _resolve_database_url() -> str:
    """Pick up the database URL from env, handling Railway's formats.

    Railway provides ``DATABASE_URL`` (postgresql://…) automatically when a
    Postgres plugin is attached.  We also accept ``RHOBOT_DATABASE_URL`` for
    explicit overrides.  The URL is normalized to use the ``asyncpg`` driver.
    """
    raw = (
        os.environ.get("RHOBOT_DATABASE_URL")
        or os.environ.get("DATABASE_URL")
        or ""
    )

    if not raw:
        return "postgresql+asyncpg://postgres:postgres@localhost:5432/rhobot"

    # Normalize driver prefix for asyncpg
    raw = re.sub(r"^postgres(ql)?://", "postgresql+asyncpg://", raw)

    # Fix empty port (e.g. "…@host:/db" → "…@host:5432/db")
    raw = re.sub(r"@([^/:]+):(/)","@\\1:5432/", raw)

    return raw


class Settings(BaseSettings):
    app_name: str = "rho-bot"
    debug: bool = False

    # Auth
    secret_key: str = "CHANGE-ME-in-production-use-openssl-rand-hex-32"
    algorithm: str = "HS256"
    access_token_expire_minutes: int = 60 * 24  # 24 hours

    # Database (resolved via helper to handle Railway's DATABASE_URL)
    database_url: str = _resolve_database_url()

    # Model — remote inference (Railway → Modal)
    model_inference_url: str = ""  # Modal endpoint URL; empty = stub noop mode

    # Model — local inference (self-hosted / dev with GPU)
    model_path: str = ""
    model_device: str = ""  # "" = auto-detect (CUDA > MPS > CPU), or e.g. "cpu", "cuda:0"
    model_encryption_key: str = ""  # hex-encoded 32-byte AES-256 key; empty = unencrypted (dev)

    # Stripe
    stripe_secret_key: str = ""
    stripe_webhook_secret: str = ""
    stripe_pro_price_id: str = ""
    stripe_team_price_id: str = ""
    frontend_url: str = "http://localhost:3000"

    class Config:
        env_file = ".env"
        env_prefix = "RHOBOT_"


settings = Settings()
