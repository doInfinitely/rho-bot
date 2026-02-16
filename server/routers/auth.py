import logging

from fastapi import APIRouter, Depends, HTTPException, status
from fastapi.responses import JSONResponse
from sqlalchemy import select, text
from sqlalchemy.ext.asyncio import AsyncSession

from server.models.database import User, get_db
from server.schemas.auth import Token, UserCreate, UserLogin, UserOut
from server.services.auth_service import create_access_token, hash_password, verify_password

logger = logging.getLogger(__name__)

router = APIRouter(prefix="/auth", tags=["auth"])


@router.post("/signup", response_model=Token, status_code=status.HTTP_201_CREATED)
async def signup(body: UserCreate, db: AsyncSession = Depends(get_db)):
    try:
        result = await db.execute(select(User).where(User.email == body.email))
        if result.scalar_one_or_none() is not None:
            raise HTTPException(status_code=409, detail="Email already registered")

        user = User(email=body.email, hashed_password=hash_password(body.password))
        db.add(user)
        await db.commit()
        await db.refresh(user)

        token = create_access_token({"sub": user.id, "email": user.email})
        return Token(access_token=token)
    except HTTPException:
        raise
    except Exception as exc:
        logger.exception("Signup failed")
        return JSONResponse(
            status_code=503,
            content={"detail": f"Database error: {type(exc).__name__}: {exc}"},
        )


@router.post("/login", response_model=Token)
async def login(body: UserLogin, db: AsyncSession = Depends(get_db)):
    try:
        result = await db.execute(select(User).where(User.email == body.email))
        user = result.scalar_one_or_none()
        if user is None or not verify_password(body.password, user.hashed_password):
            raise HTTPException(status_code=401, detail="Invalid credentials")

        token = create_access_token({"sub": user.id, "email": user.email})
        return Token(access_token=token)
    except HTTPException:
        raise
    except Exception as exc:
        logger.exception("Login failed")
        return JSONResponse(
            status_code=503,
            content={"detail": f"Database error: {type(exc).__name__}: {exc}"},
        )
