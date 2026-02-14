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
model_service = ModelService(model_path=settings.model_path)
context_service = ContextService(model_service=model_service)


@asynccontextmanager
async def lifespan(app: FastAPI):
    logger.info("Starting rho-bot server …")
    await init_db()
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
    return {"status": "ok"}
