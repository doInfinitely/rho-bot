import os
import re

from pydantic import model_validator
from pydantic_settings import BaseSettings


def _normalize_database_url(raw: str) -> str:
    """Normalize a database URL for asyncpg.

    - Converts ``postgres://`` or ``postgresql://`` to ``postgresql+asyncpg://``
    - Fills in empty port with 5432
    """
    if not raw:
        return "postgresql+asyncpg://postgres:postgres@localhost:5432/rhobot"

    # Normalize driver prefix for asyncpg
    raw = re.sub(r"^postgres(ql)?(\+asyncpg)?://", "postgresql+asyncpg://", raw)

    # Fix empty port (e.g. "…@host:/db" → "…@host:5432/db")
    raw = re.sub(r"@([^/:]+):(/)", "@\\1:5432/", raw)

    return raw


class Settings(BaseSettings):
    app_name: str = "rho-bot"
    debug: bool = False

    # Auth
    secret_key: str = "CHANGE-ME-in-production-use-openssl-rand-hex-32"
    algorithm: str = "HS256"
    access_token_expire_minutes: int = 60 * 24 * 30  # 30 days

    # Database
    database_url: str = ""

    # Model — remote inference (Railway → Modal)
    model_inference_url: str = ""  # Modal endpoint URL; empty = stub noop mode

    # Model — local inference (self-hosted / dev with GPU)
    model_path: str = ""
    model_device: str = ""  # "" = auto-detect (CUDA > MPS > CPU), or e.g. "cpu", "cuda:0"
    model_encryption_key: str = ""  # hex-encoded 32-byte AES-256 key; empty = unencrypted (dev)

    # S3 / Cloudflare R2 (screenshot storage)
    s3_bucket: str = ""
    s3_endpoint_url: str = ""  # R2 endpoint
    s3_region: str = "auto"

    # Stripe
    stripe_secret_key: str = ""
    stripe_webhook_secret: str = ""
    stripe_pro_price_id: str = ""
    stripe_team_price_id: str = ""
    frontend_url: str = "http://localhost:3000"

    # ElevenLabs (voice proxy)
    elevenlabs_api_key: str = ""

    @model_validator(mode="after")
    def _fix_database_url(self) -> "Settings":
        """Normalize the database URL after pydantic-settings loads env vars.

        Also picks up Railway's built-in DATABASE_URL as a fallback.
        """
        raw = self.database_url or os.environ.get("DATABASE_URL", "")
        self.database_url = _normalize_database_url(raw)
        return self

    class Config:
        env_file = ".env"
        env_prefix = "RHOBOT_"


settings = Settings()
