"""Tests for the ASGI ``RequestBodySizeLimitMiddleware`` (Fix #3).

``src.config.MAX_UPLOAD_SIZE`` was previously dead config; the limit now
lives at the ASGI layer so oversized bodies are rejected with HTTP 413
*before* any router sees them — no ``UploadFile.read()`` spooling, no OOM.

We hit ``POST /auth/login`` (any endpoint that accepts a body works) with a
payload just over ``MAX_UPLOAD_SIZE`` and assert 413 — that covers the
declared ``Content-Length`` fast path. The rest of the file drives the raw
ASGI interface directly to cover the streaming guard (a forged/understated
``Content-Length`` must not smuggle an oversized body past the cap) and to
assert exactly one response ever reaches the client.
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
from src.main import RequestBodySizeLimitMiddleware, app
from src.models import Base

SYNC_DB_URL = f"sqlite:///{TEST_DB_PATH}"
ASYNC_DB_URL = f"sqlite+aiosqlite:///{TEST_DB_PATH}"

sync_engine = create_sync_engine(SYNC_DB_URL, echo=False)
SyncSession = sync_sessionmaker(bind=sync_engine)
test_engine = create_async_engine(ASYNC_DB_URL, echo=False)


@pytest.fixture(autouse=True)
def setup_db():
    from sqlalchemy.ext.asyncio import AsyncSession, async_sessionmaker

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


# ---------------------------------------------------------------------------
# Middleware unit tests (no app / DB / httpx) — drive the raw ASGI interface so
# the streaming guard is exercised with hand-built messages. A small explicit
# cap keeps the allocations trivial while testing the same code path.
# ---------------------------------------------------------------------------

_CAP = 1024


def _dummy_app(record: list) -> object:
    """An inner ASGI app that reads its whole body then replies 200."""

    async def inner_app(scope, receive, send):
        total = 0
        while True:
            message = await receive()
            if message["type"] == "http.disconnect":
                break
            total += len(message.get("body", b""))
            if not message.get("more_body"):
                break
        record.append(("inner_ran", total))
        await send({"type": "http.response.start", "status": 200, "headers": []})
        await send({"type": "http.response.body", "body": b"ok"})

    return inner_app


def test_forged_content_length_cannot_bypass_limit():
    """A client may declare a small Content-Length and still stream more.

    The declared value is only a fast path; the middleware must also count the
    real bytes. Assert we send exactly one response (413, never a late 200 —
    ASGI forbids a second ``http.response.start``).
    """
    chunks = [
        {"type": "http.request", "body": b"x" * 900, "more_body": True},
        {"type": "http.request", "body": b"y" * 900, "more_body": False},
    ]
    sent: list[dict] = []
    inner_calls: list = []

    async def receive():
        return chunks.pop(0)

    async def send(message):
        sent.append(message)

    scope = {"type": "http", "headers": [(b"content-length", b"10")]}
    middleware = RequestBodySizeLimitMiddleware(_dummy_app(inner_calls), _CAP)
    asyncio.run(middleware(scope, receive, send))

    starts = [m for m in sent if m["type"] == "http.response.start"]
    assert len(starts) == 1, f"exactly one response expected, got {starts}"
    assert starts[0]["status"] == 413
    # The 200 the inner app emitted after the overflow must have been dropped.
    assert not any(m.get("status") == 200 for m in sent)


def test_declared_oversized_content_length_short_circuits():
    """Declared Content-Length over the cap → 413 without reading the body."""
    sent: list[dict] = []
    inner_calls: list = []

    async def receive():
        raise AssertionError("body must not be read on the fast path")

    async def send(message):
        sent.append(message)

    scope = {"type": "http", "headers": [(b"content-length", str(_CAP + 1).encode())]}
    middleware = RequestBodySizeLimitMiddleware(_dummy_app(inner_calls), _CAP)
    asyncio.run(middleware(scope, receive, send))

    assert [m["type"] for m in sent] == ["http.response.start", "http.response.body"]
    assert sent[0]["status"] == 413
    assert inner_calls == []


def test_body_at_exactly_the_cap_is_allowed_through():
    """Boundary: bytes == cap passes; only exceeding it trips the guard."""
    chunks = [
        {"type": "http.request", "body": b"x" * _CAP, "more_body": True},
        {"type": "http.request", "body": b"", "more_body": False},
    ]
    sent: list[dict] = []
    inner_calls: list = []

    async def receive():
        return chunks.pop(0)

    async def send(message):
        sent.append(message)

    scope = {"type": "http", "headers": []}
    middleware = RequestBodySizeLimitMiddleware(_dummy_app(inner_calls), _CAP)
    asyncio.run(middleware(scope, receive, send))

    assert inner_calls == [("inner_ran", _CAP)]
    assert [m["type"] for m in sent] == ["http.response.start", "http.response.body"]
    assert sent[0]["status"] == 200
