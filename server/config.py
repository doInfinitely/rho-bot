from pydantic_settings import BaseSettings


class Settings(BaseSettings):
    app_name: str = "rho-bot"
    debug: bool = False

    # Auth
    secret_key: str = "CHANGE-ME-in-production-use-openssl-rand-hex-32"
    algorithm: str = "HS256"
    access_token_expire_minutes: int = 60 * 24  # 24 hours

    # Database
    database_url: str = "postgresql+asyncpg://postgres:postgres@localhost:5432/rhobot"

    # Model
    model_path: str = ""

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
