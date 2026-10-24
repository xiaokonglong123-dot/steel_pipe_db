"""Audit日志路由 e2e 测试.

PROJECT_RULES: /audit 需 authenticated (no role guard); /audit/verify 需
require_admin (AGENTS.md: audit verify 属 admin); GET 渲染
audit/audit_logs.html; /audit/verify POST 经 require_csrf.

覆盖:
* GET /audit  unauth → 401
* GET /audit  admin → 200
* GET /audit  viewer → 200
* GET /audit/verify on empty log → 200, chain ok (genesis only)
* GET /audit/verify viewer → 403 (admin-only)
* POST /audit/verify with CSRF stub → 200
* POST /audit/verify missing CSRF → 403
"""

from __future__ import annotations

import asyncio
import os
import sys
from typing import AsyncIterator

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

TEST_DB_PATH = "data/test_audit_route.db"
os.environ["DATABASE_URL"] = f"sqlite+aiosqlite:///{TEST_DB_PATH}"
os.environ.setdefault("JWT_SECRET_KEY", "test-secret-audit-route")

from httpx import ASGITransport, AsyncClient
import pytest
from sqlalchemy import create_engine as create_sync_engine
from sqlalchemy.ext.asyncio import AsyncSession, async_sessionmaker, create_async_engine
from sqlalchemy.orm import sessionmaker as sync_sessionmaker

from rust_core import auth as rust_auth
from src import database as db
from src.main import app
from src.models import AuditLog, Base, User
from src.security import get_current_user, require_csrf


SYNC_DB_URL = f"sqlite:///{TEST_DB_PATH}"
ASYNC_DB_URL = f"sqlite+aiosqlite:///{TEST_DB_PATH}"
sync_engine = create_sync_engine(SYNC_DB_URL, echo=False)
SyncSession = sync_sessionmaker(bind=sync_engine)
test_engine = create_async_engine(ASYNC_DB_URL, echo=False)


def _seed_users():
    Base.metadata.create_all(bind=sync_engine)
    with SyncSession() as s:
        for row in s.query(User).all():
            s.delete(row)
        s.commit()
        s.add(User(
            id=1, username="admin", role="admin", display_name="Admin",
            active=True, password_hash=rust_auth.hash_password("Pa$$w0rd"),
            token_version=0,
        ))
        s.add(User(
            id=2, username="viewer", role="viewer", display_name="Viewer",
            active=True, password_hash=rust_auth.hash_password("Pa$$w0rd"),
            token_version=0,
        ))
        s.commit()


def _reset_db():
    if os.path.exists(TEST_DB_PATH):
        os.remove(TEST_DB_PATH)
    for ext in ("-wal", "-shm"):
        p = TEST_DB_PATH + ext
        if os.path.exists(p):
            os.remove(p)


def _stub_user(user_id: int):
    async def _dep():
        with SyncSession() as s:
            return s.get(User, user_id)
    return _dep


def _allow_csrf():
    async def _dep():
        return None
    return _dep


@pytest.fixture
def client():
    transport = ASGITransport(app=app)
    return AsyncClient(transport=transport, base_url="http://test")


@pytest.fixture(autouse=True)
def fx():
    _reset_db()
    _seed_users()
    from sqlalchemy.ext.asyncio import AsyncSession, async_sessionmaker as mk
    db.engine = create_async_engine(ASYNC_DB_URL, echo=False)
    db.async_session_factory = mk(
        db.engine, class_=AsyncSession, expire_on_commit=False
    )
    yield
    app.dependency_overrides.clear()
    Base.metadata.drop_all(bind=sync_engine)
    sync_engine.dispose()
    asyncio.run(db.engine.dispose())


def test_get_audit_unauthenticated_returns_401(client):
    async def run():
        async with client as c:
            r = await c.get("/audit", follow_redirects=False)
        assert r.status_code == 401
    asyncio.run(run())


def test_get_audit_admin_returns_200(client):
    app.dependency_overrides[get_current_user] = _stub_user(1)
    async def run():
        async with client as c:
            r = await c.get("/audit", follow_redirects=False)
        assert r.status_code == 200
    asyncio.run(run())


def test_get_audit_viewer_returns_200(client):
    """Audit log is readable by any authenticated user — no role guard."""
    app.dependency_overrides[get_current_user] = _stub_user(2)
    async def run():
        async with client as c:
            r = await c.get("/audit", follow_redirects=False)
        assert r.status_code == 200
    asyncio.run(run())


def test_get_audit_verify_empty_chain_returns_200(client):
    """Empty audit table → chain is 'ok' (genesis only); verify returns 200."""
    app.dependency_overrides[get_current_user] = _stub_user(1)
    async def run():
        async with client as c:
            r = await c.get("/audit/verify", follow_redirects=False)
        assert r.status_code == 200
        # Template render — body must contain verify_result block.
        assert b"verify" in r.content.lower()
    asyncio.run(run())


def test_get_audit_verify_viewer_forbidden_403(client):
    """Chain verification is admin-only — a viewer is rejected, not served."""
    app.dependency_overrides[get_current_user] = _stub_user(2)
    async def run():
        async with client as c:
            r = await c.get("/audit/verify", follow_redirects=False)
        assert r.status_code == 403
    asyncio.run(run())


def test_get_audit_verify_unauthenticated_401(client):
    async def run():
        async with client as c:
            r = await c.get("/audit/verify", follow_redirects=False)
        assert r.status_code == 401
    asyncio.run(run())


def test_post_audit_verify_missing_csrf_rejected(client):
    """Real require_csrf dependency + no cookie → 403."""
    app.dependency_overrides[get_current_user] = _stub_user(1)
    async def run():
        async with client as c:
            r = await c.post(
                "/audit/verify",
                data={"csrf_token": "bad"},
                follow_redirects=False,
            )
        assert r.status_code == 403
    asyncio.run(run())


def test_post_audit_verify_with_csrf_passes(client):
    app.dependency_overrides[get_current_user] = _stub_user(1)
    app.dependency_overrides[require_csrf] = _allow_csrf()
    async def run():
        async with client as c:
            r = await c.post(
                "/audit/verify",
                data={"csrf_token": "stub"},
                follow_redirects=False,
            )
        assert r.status_code == 200
    asyncio.run(run())
