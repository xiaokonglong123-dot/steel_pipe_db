"""Tests for the ASGI ``RequestBodySizeLimitMiddleware`` (Fix #3).

``src.config.MAX_UPLOAD_SIZE`` was previously dead config; the limit now
lives at the ASGI layer so oversized bodies are rejected with HTTP 413
*before* any router sees them — no ``UploadFile.read()`` spooling, no OOM.

We hit ``POST /auth/login`` (any endpoint that accepts a body works) with a
payload just over ``MAX_UPLOAD_SIZE`` and assert 413.
"""

from __future__ import annotations

import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

TEST_DB_PATH = "data/test_upload_limit.db"
os.environ["DATABASE_URL"] = f"sqlite+aiosqlite:///{TEST_DB_PATH}"
os.environ.setdefault("JWT_SECRET_KEY", "test-secret-upload-limit")

import asyncio

import pytest
from httpx import ASGITransport, AsyncClient
from sqlalchemy import create_engine as create_sync_engine
from sqlalchemy.ext.asyncio import create_async_engine
from sqlalchemy.orm import sessionmaker as sync_sessionmaker

from src import database as db
from src.config import settings
from src.main import app
from src.models import Base

SYNC_DB_URL = f"sqlite:///{TEST_DB_PATH}"
ASYNC_DB_URL = f"sqlite+aiosqlite:///{TEST_DB_PATH}"

sync_engine = create_sync_engine(SYNC_DB_URL, echo=False)
SyncSession = sync_sessionmaker(bind=sync_engine)
test_engine = create_async_engine(ASYNC_DB_URL, echo=False)


@pytest.fixture(autouse=True)
def setup_db():
    from sqlalchemy.ext.asyncio import async_sessionmaker, AsyncSession

    db.engine = test_engine
    db.async_session_factory = async_sessionmaker(
        test_engine, class_=AsyncSession, expire_on_commit=False,
    )

    Base.metadata.create_all(bind=sync_engine)
    yield
    Base.metadata.drop_all(bind=sync_engine)
    sync_engine.dispose()
    asyncio.run(test_engine.dispose())


def test_oversized_body_rejected_with_413():
    """Body just above MAX_UPLOAD_SIZE → 413 from the middleware itself."""
    max_bytes = settings.MAX_UPLOAD_SIZE
    oversized = b"x" * (max_bytes + 1)

    async def _run():
        transport = ASGITransport(app=app)
        async with AsyncClient(transport=transport, base_url="http://test") as client:
            # /auth/login is irrelevant — the middleware short-circuits before
            # the router is even matched, so any endpoint that accepts a body
            # demonstrates the cap.
            r = await client.post(
                "/auth/login",
                content=oversized,
                headers={"content-type": "text/plain"},
            )
            assert r.status_code == 413

    asyncio.run(_run())


def test_within_limit_passes_middleware():
    """Body at exactly MAX_UPLOAD_SIZE is not rejected by the middleware.

    We don't care whether /auth/login then 401/422s — we only assert the
    middleware did NOT short-circuit (any status other than 413).
    """
    at_limit = b"x" * settings.MAX_UPLOAD_SIZE

    async def _run():
        transport = ASGITransport(app=app)
        async with AsyncClient(transport=transport, base_url="http://test") as client:
            r = await client.post(
                "/auth/login",
                content=at_limit,
                headers={"content-type": "text/plain"},
            )
            assert r.status_code != 413

    asyncio.run(_run())
