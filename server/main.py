"""
rho-bot server
==============
FastAPI application: WebSocket for the desktop agent, REST for the website dashboard.
"""

from contextlib import asynccontextmanager
import logging

from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware

from server.config import settings
from server.models.database import init_db
from server.routers import api, auth, billing, ws
from server.services.context_service import ContextService
from server.services.model_service import ModelService

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

# ---- singletons initialised at startup ----
model_service = ModelService(
    inference_url=settings.model_inference_url,
    model_path=settings.model_path,
    device_override=settings.model_device,
    encryption_key=settings.model_encryption_key,
)
context_service = ContextService(model_service=model_service)


@asynccontextmanager
async def lifespan(app: FastAPI):
    logger.info("Starting rho-bot server …")
    try:
        await init_db()
        logger.info("Database initialised successfully")
    except Exception as exc:
        logger.error("Database init failed (server will start without DB): %s", exc)
    yield
    logger.info("Shutting down rho-bot server …")


app = FastAPI(
    title="rho-bot",
    description="Hierarchical goal-induction agent server",
    version="0.1.0",
    lifespan=lifespan,
)

# CORS – allow the website and desktop client origins
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],  # tighten in production
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

app.include_router(auth.router)
app.include_router(ws.router)
app.include_router(api.router)
app.include_router(billing.router)


@app.get("/health")
async def health():
    return {
        "status": "ok",
        "database_url_set": settings.database_url != "postgresql+asyncpg://postgres:postgres@localhost:5432/rhobot",
        "inference_url_set": bool(settings.model_inference_url),
    }
