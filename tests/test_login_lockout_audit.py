"""Login lockout audit tests (D10).

Covers:
* 10th failed login triggers exactly one login_lockout audit row (user_id NULL)
* 9 failures write nothing
* locked attempts (429) do not add new audit rows
* IP-tier 429s write nothing (username tier never reached)
* audit hash chain stays verifiable after a lockout event
* successful login clears failure records without writing audit

Rate limiter is a module-level singleton — the autouse fx fixture resets
ip_hits / user_fail so tests never contaminate each other. 10-failure tests
use distinct X-Forwarded-For IPs: the IP tier (<=5/min) would 429 the 6th
same-IP attempt before the username tier ever locks.
"""

from __future__ import annotations

import asyncio
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

TEST_DB_PATH = "data/test_login_lockout_audit.db"
os.environ["DATABASE_URL"] = f"sqlite+aiosqlite:///{TEST_DB_PATH}"
os.environ.setdefault("JWT_SECRET_KEY", "test-secret-login-lockout")

from rust_core import auth as rust_auth

rust_auth.init(os.environ["JWT_SECRET_KEY"])

from httpx import ASGITransport, AsyncClient
import pytest
from sqlalchemy import create_engine as create_sync_engine
from sqlalchemy.ext.asyncio import AsyncSession, async_sessionmaker, create_async_engine
from sqlalchemy.orm import sessionmaker as sync_sessionmaker

from src import database as db
from src.main import app
from src.models import AuditLog, Base, User
from src.security import get_current_user, login_limiter, require_csrf


SYNC_DB_URL = f"sqlite:///{TEST_DB_PATH}"
ASYNC_DB_URL = f"sqlite+aiosqlite:///{TEST_DB_PATH}"
sync_engine = create_sync_engine(SYNC_DB_URL, echo=False)
SyncSession = sync_sessionmaker(bind=sync_engine)


def _seed():
    Base.metadata.create_all(bind=sync_engine)
    with SyncSession() as s:
        for row in s.query(AuditLog).all():
            s.delete(row)
        for row in s.query(User).all():
            s.delete(row)
        s.commit()
        s.add(User(
            id=1, username="admin", role="admin", display_name="Admin",
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


def _allow_csrf():
    async def _dep():
        return None
    return _dep


def _stub_user(user_id: int):
    async def _dep():
        with SyncSession() as s:
            return s.get(User, user_id)
    return _dep


def _audit_rows() -> list[AuditLog]:
    with SyncSession() as s:
        return s.query(AuditLog).order_by(AuditLog.id).all()


def _audit_count() -> int:
    with SyncSession() as s:
        return s.query(AuditLog).count()


@pytest.fixture
def client():
    transport = ASGITransport(app=app)
    return AsyncClient(transport=transport, base_url="http://test")


@pytest.fixture(autouse=True)
def fx():
    _reset_db()
    _seed()
    login_limiter.ip_hits.clear()
    login_limiter.user_fail.clear()
    db.engine = create_async_engine(ASYNC_DB_URL, echo=False)
    db.async_session_factory = async_sessionmaker(
        db.engine, class_=AsyncSession, expire_on_commit=False
    )
    yield
    app.dependency_overrides.clear()
    Base.metadata.drop_all(bind=sync_engine)
    sync_engine.dispose()
    login_limiter.ip_hits.clear()
    login_limiter.user_fail.clear()


async def _post(client, url: str, data: dict, xff: str):
    return await client.post(
        url,
        data={**data, "csrf_token": "test-csrf"},
        headers={"X-Forwarded-For": xff},
        follow_redirects=False,
    )


async def _fail_n_times(client, n: int, start_ip: int = 1) -> str:
    """Send n failed logins (wrong password) from distinct XFF IPs.

    Returns the IP used for the n-th (last) attempt.
    """
    last_ip = ""
    for i in range(start_ip, start_ip + n):
        last_ip = f"203.0.113.{i}"
        r = await _post(
            client, "/login", {"username": "admin", "password": "WrongPass!"}, last_ip
        )
        assert r.status_code == 401, r.text[:200]
    return last_ip


def test_10th_failure_triggers_lockout_audit(client):
    app.dependency_overrides[require_csrf] = _allow_csrf()

    async def run():
        async with client as c:
            last_ip = await _fail_n_times(c, 10)
            return last_ip

    last_ip = asyncio.run(run())
    rows = _audit_rows()
    assert len(rows) == 1, f"expected exactly 1 audit row, got {len(rows)}"
    row = rows[0]
    assert row.action == "login_lockout"
    assert row.user_id is None
    assert "username=admin" in row.detail
    assert f"ip={last_ip}" in row.detail
    assert "failures=10" in row.detail
    assert "locked_for=300s" in row.detail


def test_9_failures_no_audit(client):
    app.dependency_overrides[require_csrf] = _allow_csrf()

    async def run():
        async with client as c:
            await _fail_n_times(c, 9)

    asyncio.run(run())
    assert _audit_count() == 0


def test_locked_attempt_no_new_audit(client):
    app.dependency_overrides[require_csrf] = _allow_csrf()

    async def run():
        async with client as c:
            await _fail_n_times(c, 10)
            r = await _post(
                c, "/login", {"username": "admin", "password": "WrongPass!"}, "203.0.113.99"
            )
            return r

    r = asyncio.run(run())
    assert r.status_code == 429, r.text[:200]
    assert _audit_count() == 1


def test_ip_429_no_audit(client):
    app.dependency_overrides[require_csrf] = _allow_csrf()

    async def run():
        async with client as c:
            for i in range(6):
                r = await _post(
                    c, "/login", {"username": "admin", "password": "WrongPass!"}, "203.0.113.50"
                )
                if i < 5:
                    assert r.status_code == 401, r.text[:200]
            return r

    r = asyncio.run(run())
    assert r.status_code == 429, r.text[:200]
    assert _audit_count() == 0


def test_chain_verifies_after_lockout(client):
    app.dependency_overrides[get_current_user] = _stub_user(1)
    app.dependency_overrides[require_csrf] = _allow_csrf()

    async def run():
        async with client as c:
            await _fail_n_times(c, 10)
            v = await _post(c, "/audit/verify", {}, "203.0.113.88")
            return v

    v = asyncio.run(run())
    assert v.status_code == 200, v.text[:400]
    assert "哈希链完整" in v.text


def test_success_clears_failures_no_audit(client):
    app.dependency_overrides[require_csrf] = _allow_csrf()

    async def run():
        async with client as c:
            await _fail_n_times(c, 3)
            r = await _post(
                c, "/login", {"username": "admin", "password": "Pa$$w0rd"}, "203.0.113.77"
            )
            return r

    r = asyncio.run(run())
    assert r.status_code == 303, r.text[:200]
    assert _audit_count() == 0
