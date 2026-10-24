"""App factory, lifespan, exception handlers — single-worker FastAPI entry point."""

import os
from contextlib import asynccontextmanager

import anyio
from fastapi import FastAPI
from fastapi.responses import JSONResponse
from fastapi.staticfiles import StaticFiles

from src.config import settings
from src.database import engine, async_session_factory
from src.models import Base, Location, User
from src.modules import register_all_routers
from src.modules.warehouse.services.audit_service import (
    ComplianceError,
    ConflictError,
    ForbiddenError,
)
from rust_core import auth as rust_auth
from sqlalchemy import select
from starlette.types import ASGIApp, Receive, Scope, Send

# Shared Jinja2 templates environment.
# Constructed by ``src.templates_env`` so routers can import the singleton
# without going through this module (which would create a circular import
# because ``register_all_routers`` is called at import time below).
# Re-exported here for backwards compatibility with external callers/tests
# that still write ``from src.main import templates``.
from src.templates_env import templates  # noqa: E402,F401


# ---------------------------------------------------------------------------
# ASGI middleware: reject oversized request bodies before the app reads them.
# Starlette has no built-in request-size middleware; this raw-ASGI wrapper
# counts the real inbound bytes and, on overflow, emits HTTP 413 once and
# aborts body streaming so ``UploadFile.read()`` never spools an oversized
# body to memory (prevents OOM; previously ``MAX_UPLOAD_SIZE`` was dead
# config). Exactly one response reaches the client.
# ---------------------------------------------------------------------------


def _declared_content_length(scope: Scope) -> int | None:
    """Return the declared Content-Length as an int, or None if absent/invalid."""
    for name, value in scope.get("headers", []):
        if name == b"content-length":
            try:
                return int(value)
            except ValueError:
                return None
    return None


class RequestBodySizeLimitMiddleware:
    """ASGI middleware enforcing a maximum inbound body size (HTTP 413).

    Two layers:

    * Fast path — a declared ``Content-Length`` over the cap is rejected
      without reading the body (cheap, covers well-behaved clients).
    * Streaming guard — every ``http.request`` chunk is counted against the
      actual bytes received. On overflow the middleware emits a 413 once and
      then (a) answers further ``receive()`` calls with ``http.disconnect``
      so the app aborts body reading, and (b) swallows every message the app
      tries to send afterwards. Exactly one response reaches the client —
      no double-response, and a forged/understated ``Content-Length`` cannot
      smuggle an oversized body past the limit.
    """

    def __init__(self, app: ASGIApp, max_upload_size: int) -> None:
        self.app = app
        self.max_upload_size = int(max_upload_size)

    async def __call__(self, scope: Scope, receive: Receive, send: Send) -> None:
        if scope["type"] != "http":
            await self.app(scope, receive, send)
            return

        content_length = _declared_content_length(scope)
        if content_length is not None and content_length > self.max_upload_size:
            await _send_413(send, self.max_upload_size)
            return

        bytes_received = 0
        limit_exceeded = False

        async def wrapped_receive():
            nonlocal bytes_received, limit_exceeded
            if limit_exceeded:
                return {"type": "http.disconnect"}
            message = await receive()
            if message.get("type") == "http.request":
                bytes_received += len(message.get("body", b""))
                if bytes_received > self.max_upload_size:
                    limit_exceeded = True
                    await _send_413(send, self.max_upload_size)
                    return {"type": "http.disconnect"}
            return message

        async def wrapped_send(message) -> None:
            if limit_exceeded:
                # Our 413 is already committed — drop any late app response.
                return
            await send(message)

        await self.app(scope, wrapped_receive, wrapped_send)


async def _send_413(send: Send, max_bytes: int) -> None:
    body = f"request body exceeds {max_bytes} bytes".encode()
    await send(
        {
            "type": "http.response.start",
            "status": 413,
            "headers": [
                (b"content-type", b"text/plain; charset=utf-8"),
                (b"content-length", str(len(body)).encode()),
                (b"connection", b"close"),
            ],
        }
    )
    await send({"type": "http.response.body", "body": body})


# ---------------------------------------------------------------------------
# Lifespan
# ---------------------------------------------------------------------------
@asynccontextmanager
async def lifespan(app: FastAPI):
    # -- Fail-fast: JWT_SECRET_KEY must be set --------------------------------
    if not settings.JWT_SECRET_KEY:
        raise RuntimeError(
            "JWT_SECRET_KEY must be set (fail-fast at startup). "
            "Set via .env or environment variable."
        )

    # -- Init Rust auth module ------------------------------------------------
    rust_auth.init(settings.JWT_SECRET_KEY)

    # -- Seed DEFAULT location idempotently -----------------------------------
    async with async_session_factory() as session:
        existing = await session.get(Location, "DEFAULT")
        if existing is None:
            session.add(
                Location(
                    code="DEFAULT",
                    name="默认库位",
                    used_count=0,
                    is_active=True,
                )
            )
            await session.commit()

    # -- Upsert admin user from env vars (optional) ---------------------------
    admin_username = os.environ.get("ADMIN_USERNAME")
    admin_password = os.environ.get("ADMIN_PASSWORD")
    if admin_username and admin_password:
        async with async_session_factory() as session:
            result = await session.execute(
                select(User).where(User.username == admin_username)
            )
            user = result.scalar_one_or_none()
            # argon2 hashing is heavy — keep it off the event loop (see D5).
            password_hash = await anyio.to_thread.run_sync(
                rust_auth.hash_password, admin_password
            )
            if user is None:
                session.add(
                    User(
                        username=admin_username,
                        password_hash=password_hash,
                        display_name=admin_username,
                        role="admin",
                        active=True,
                    )
                )
            else:
                user.password_hash = password_hash
            await session.commit()

    yield

    # -- Cleanup: dispose engine ---------------------------------------------
    await engine.dispose()


# ---------------------------------------------------------------------------
# App factory
# ---------------------------------------------------------------------------
app = FastAPI(
    lifespan=lifespan,
    title="Ayanami_Rei",
    version="0.8.0",
)

app.add_middleware(
    RequestBodySizeLimitMiddleware,
    max_upload_size=settings.MAX_UPLOAD_SIZE,
)

# -- Register routers -------------------------------------------------------
register_all_routers(app)

# -- Exception handlers -----------------------------------------------------


@app.exception_handler(ConflictError)
async def conflict_handler(request, exc):
    return JSONResponse(status_code=409, content={"detail": str(exc)})


@app.exception_handler(ForbiddenError)
async def forbidden_handler(request, exc):
    return JSONResponse(status_code=403, content={"detail": str(exc)})


@app.exception_handler(ComplianceError)
async def compliance_handler(request, exc):
    return JSONResponse(status_code=422, content={"detail": str(exc)})
