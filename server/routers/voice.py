"""ElevenLabs TTS / STT / Voices proxy — auth-gated."""

import logging

import httpx
from fastapi import APIRouter, Depends, File, Form, HTTPException, UploadFile
from fastapi.responses import StreamingResponse
from pydantic import BaseModel

from server.config import settings
from server.deps import get_current_user
from server.models.database import User

logger = logging.getLogger(__name__)

ELEVENLABS_BASE = "https://api.elevenlabs.io/v1"

router = APIRouter(prefix="/api/voice", tags=["voice"])


class TTSRequest(BaseModel):
    """Request for text-to-speech."""

    text: str
    voice_id: str = "JBFqnCBsd6RMkjVDRZzb"  # default: George
    model_id: str = "eleven_turbo_v2_5"


@router.post("/tts")
async def tts(request: TTSRequest, user: User = Depends(get_current_user)):
    """Proxy TTS request to ElevenLabs. Returns audio/mpeg stream."""
    if not settings.elevenlabs_api_key:
        raise HTTPException(status_code=500, detail="ELEVENLABS_API_KEY not configured")

    url = f"{ELEVENLABS_BASE}/text-to-speech/{request.voice_id}"
    headers = {"xi-api-key": settings.elevenlabs_api_key, "Content-Type": "application/json"}
    body = {
        "text": request.text,
        "model_id": request.model_id,
        "voice_settings": {"stability": 0.5, "similarity_boost": 0.75},
    }

    async with httpx.AsyncClient(timeout=30) as client:
        resp = await client.post(url, json=body, headers=headers)
        if resp.status_code != 200:
            logger.error("ElevenLabs TTS error %d: %s", resp.status_code, resp.text[:200])
            raise HTTPException(status_code=resp.status_code, detail=resp.text[:200])

        return StreamingResponse(
            iter([resp.content]),
            media_type="audio/mpeg",
            headers={"Content-Disposition": 'inline; filename="speech.mp3"'},
        )


@router.post("/stt")
async def stt(
    user: User = Depends(get_current_user),
    file: UploadFile = File(...),
    model_id: str = Form("scribe_v1"),
):
    """Proxy STT request to ElevenLabs. Returns transcribed text."""
    if not settings.elevenlabs_api_key:
        raise HTTPException(status_code=500, detail="ELEVENLABS_API_KEY not configured")

    url = f"{ELEVENLABS_BASE}/speech-to-text"
    headers = {"xi-api-key": settings.elevenlabs_api_key}
    audio_bytes = await file.read()

    async with httpx.AsyncClient(timeout=60) as client:
        resp = await client.post(
            url,
            headers=headers,
            files={"file": (file.filename or "audio.m4a", audio_bytes, file.content_type or "audio/m4a")},
            data={"model_id": model_id},
        )
        if resp.status_code != 200:
            logger.error("ElevenLabs STT error %d: %s", resp.status_code, resp.text[:200])
            raise HTTPException(status_code=resp.status_code, detail=resp.text[:200])

        return resp.json()


@router.get("/voices")
async def list_voices(user: User = Depends(get_current_user)):
    """Proxy voice list from ElevenLabs."""
    if not settings.elevenlabs_api_key:
        raise HTTPException(status_code=500, detail="ELEVENLABS_API_KEY not configured")

    url = f"{ELEVENLABS_BASE}/voices"
    headers = {"xi-api-key": settings.elevenlabs_api_key}

    async with httpx.AsyncClient(timeout=15) as client:
        resp = await client.get(url, headers=headers)
        if resp.status_code != 200:
            raise HTTPException(status_code=resp.status_code, detail=resp.text[:200])

        data = resp.json()
        voices = [
            {"voice_id": v["voice_id"], "name": v["name"], "category": v.get("category", "premade")}
            for v in data.get("voices", [])
        ]
        return {"voices": voices}
