"""Admin write-operation audit tests (D9: user + standards audit).

Covers:
* create_user / update_user / revoke_sessions / toggle_user -> audit rows
* create_standard / update_standard / delete_standard -> audit rows
* failure paths write zero audit rows (rollback)
* update_user password changes never leak the hash into detail
* mixed operations keep the audit hash chain verifiable

PROJECT_RULES: audit entries must live in the same BEGIN IMMEDIATE txn as
the operation they record; AuditLog.user_id is the actor.
"""

from __future__ import annotations

import asyncio
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

TEST_DB_PATH = "data/test_admin_audit.db"
os.environ["DATABASE_URL"] = f"sqlite+aiosqlite:///{TEST_DB_PATH}"
os.environ.setdefault("JWT_SECRET_KEY", "test-secret-admin-audit")

from httpx import ASGITransport, AsyncClient
import pytest
from sqlalchemy import create_engine as create_sync_engine
from sqlalchemy.ext.asyncio import AsyncSession, async_sessionmaker, create_async_engine
from sqlalchemy.orm import sessionmaker as sync_sessionmaker

from rust_core import auth as rust_auth
from src import database as db
from src.main import app
from src.models import AuditLog, Base, StandardSpec, User
from src.security import get_current_user, require_csrf
from src.modules.users.services.user_service import update_user

SYNC_DB_URL = f"sqlite:///{TEST_DB_PATH}"
ASYNC_DB_URL = f"sqlite+aiosqlite:///{TEST_DB_PATH}"
sync_engine = create_sync_engine(SYNC_DB_URL, echo=False)
SyncSession = sync_sessionmaker(bind=sync_engine)
test_engine = create_async_engine(ASYNC_DB_URL, echo=False)


def _seed():
    Base.metadata.create_all(bind=sync_engine)
    with SyncSession() as s:
        for row in s.query(AuditLog).all():
            s.delete(row)
        for row in s.query(User).all():
            s.delete(row)
        for row in s.query(StandardSpec).all():
            s.delete(row)
        s.commit()
        s.add(User(
            id=1, username="admin", role="admin", display_name="Admin",
            active=True, password_hash=rust_auth.hash_password("Pa$$w0rd"),
            token_version=0,
        ))
        s.add(User(
            id=2, username="operator", role="operator", display_name="Operator",
            active=True, password_hash=rust_auth.hash_password("Pa$$w0rd"),
            token_version=0,
        ))
        s.add(StandardSpec(
            id=1, grade="L80", outer_diameter_mm=88.9,
            min_wall_thickness_mm=6.0, max_wall_thickness_mm=10.0,
            length_tolerance_mm=0.5, source="API 5CT",
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
    _seed()
    from sqlalchemy.ext.asyncio import AsyncSession, async_sessionmaker as mk
    db.engine = create_async_engine(ASYNC_DB_URL, echo=False)
    db.async_session_factory = mk(
        db.engine, class_=AsyncSession, expire_on_commit=False
    )
    yield
    app.dependency_overrides.clear()
    Base.metadata.drop_all(bind=sync_engine)
    sync_engine.dispose()


# --- helpers ---------------------------------------------------------------


def _audit_count() -> int:
    with SyncSession() as s:
        return s.query(AuditLog).count()


def _last_audit() -> AuditLog | None:
    with SyncSession() as s:
        return s.query(AuditLog).order_by(AuditLog.id.desc()).first()


async def _post(client: AsyncClient, url: str, data: dict):
    return await client.post(
        url, data={**data, "csrf_token": "test-csrf"}, follow_redirects=False
    )


def _as_admin():
    app.dependency_overrides[get_current_user] = _stub_user(1)
    app.dependency_overrides[require_csrf] = _allow_csrf()


# --- user ops audit --------------------------------------------------------


def test_create_user_writes_audit(client):
    _as_admin()
    async def run():
        async with client as c:
            r = await _post(c, "/users/create", {
                "username": "alice", "password": "Secret1!",
                "display_name": "Alice", "role": "operator",
            })
        assert r.status_code == 303, r.text[:400]
    asyncio.run(run())
    row = _last_audit()
    assert row is not None
    assert row.action == "create_user"
    assert "username=alice" in row.detail
    assert "role=operator" in row.detail
    assert row.user_id == 1


def test_update_user_writes_audit(client):
    _as_admin()
    async def run():
        async with client as c:
            r = await _post(c, "/users/edit/2", {
                "display_name": "Operator2", "role": "admin", "password": "",
            })
        assert r.status_code == 303, r.text[:400]
    asyncio.run(run())
    row = _last_audit()
    assert row is not None
    assert row.action == "update_user"
    assert "role=admin" in row.detail
    assert row.user_id == 1


def test_update_user_password_hides_hash(client):
    _as_admin()
    async def run():
        async with client as c:
            r = await _post(c, "/users/edit/2", {
                "display_name": "Operator2", "role": "operator",
                "password": "NewPass123!",
            })
        assert r.status_code == 303, r.text[:400]
    asyncio.run(run())
    row = _last_audit()
    assert row is not None
    assert row.action == "update_user"
    assert "password_changed=true" in row.detail
    assert "$argon2" not in row.detail


def test_update_user_no_changes_no_audit():
    # service-level: no changed fields -> rollback, no audit row
    async def run():
        await update_user(user_id=2, engine=test_engine, actor_id=1)
    asyncio.run(run())
    assert _audit_count() == 0


def test_revoke_sessions_writes_audit(client):
    _as_admin()
    async def run():
        async with client as c:
            r = await _post(c, "/users/revoke/2", {})
        assert r.status_code == 303, r.text[:400]
    asyncio.run(run())
    row = _last_audit()
    assert row is not None
    assert row.action == "revoke_sessions"
    assert "user_id=2" in row.detail
    assert "token_version+1" in row.detail
    assert row.user_id == 1


def test_toggle_user_writes_audit(client):
    _as_admin()
    async def run():
        async with client as c:
            r = await _post(c, "/users/toggle/2", {"active": "false"})
        assert r.status_code == 303, r.text[:400]
    asyncio.run(run())
    row = _last_audit()
    assert row is not None
    assert row.action == "toggle_user"
    assert "active=False" in row.detail
    assert row.user_id == 1


def test_duplicate_username_no_audit(client):
    _as_admin()
    async def run():
        async with client as c:
            r = await _post(c, "/users/create", {
                "username": "admin", "password": "Secret1!",
                "display_name": "Dup", "role": "viewer",
            })
        assert r.status_code == 409, r.text[:400]
    asyncio.run(run())
    assert _audit_count() == 0


def test_toggle_last_admin_rejected_no_audit(client):
    _as_admin()
    async def run():
        async with client as c:
            r = await _post(c, "/users/toggle/1", {"active": "false"})
        assert r.status_code == 400, r.text[:400]
    asyncio.run(run())
    assert _audit_count() == 0


# --- standards ops audit ---------------------------------------------------


def test_create_standard_writes_audit(client):
    _as_admin()
    async def run():
        async with client as c:
            r = await _post(c, "/standards", {
                "steel_grade": "N80",
                "outer_diameter_mm": "114.3",
                "min_wall_thickness_mm": "7.0",
                "max_wall_thickness_mm": "11.0",
            })
        assert r.status_code == 303, r.text[:400]
    asyncio.run(run())
    row = _last_audit()
    assert row is not None
    assert row.action == "create_standard"
    assert "grade=N80" in row.detail
    assert "od=114.3" in row.detail
    assert "min_wt=7.0" in row.detail
    assert "max_wt=11.0" in row.detail
    assert row.user_id == 1


def test_update_standard_writes_audit(client):
    _as_admin()
    async def run():
        async with client as c:
            r = await _post(c, "/standards/1", {
                "steel_grade": "L80",
                "outer_diameter_mm": "90.0",
                "min_wall_thickness_mm": "6.0",
                "max_wall_thickness_mm": "10.0",
            })
        assert r.status_code == 303, r.text[:400]
    asyncio.run(run())
    row = _last_audit()
    assert row is not None
    assert row.action == "update_standard"
    assert "od=90.0" in row.detail
    assert row.user_id == 1


def test_delete_standard_writes_audit_with_snapshot(client):
    _as_admin()
    async def run():
        async with client as c:
            r = await _post(c, "/standards/1/delete", {})
        assert r.status_code == 303, r.text[:400]
    asyncio.run(run())
    row = _last_audit()
    assert row is not None
    assert row.action == "delete_standard"
    assert "grade=L80" in row.detail
    assert "od=88.9" in row.detail
    assert "min_wt=6.0" in row.detail
    assert "max_wt=10.0" in row.detail
    assert row.user_id == 1
    with SyncSession() as s:
        assert s.query(StandardSpec).filter(StandardSpec.id == 1).first() is None


def test_duplicate_grade_no_audit(client):
    _as_admin()
    async def run():
        async with client as c:
            r = await _post(c, "/standards", {
                "steel_grade": "L80",
                "outer_diameter_mm": "88.9",
                "min_wall_thickness_mm": "6.0",
                "max_wall_thickness_mm": "10.0",
            })
        assert r.status_code == 409, r.text[:400]
    asyncio.run(run())
    assert _audit_count() == 0


# --- hash-chain continuity -------------------------------------------------


def test_chain_verifies_after_mixed_ops(client):
    _as_admin()
    async def run():
        async with client as c:
            r = await _post(c, "/users/create", {
                "username": "alice", "password": "Secret1!",
                "display_name": "Alice", "role": "operator",
            })
            assert r.status_code == 303, r.text[:400]
            r = await _post(c, "/standards", {
                "steel_grade": "N80",
                "outer_diameter_mm": "114.3",
                "min_wall_thickness_mm": "7.0",
                "max_wall_thickness_mm": "11.0",
            })
            assert r.status_code == 303, r.text[:400]
            r = await _post(c, "/standards/1/delete", {})
            assert r.status_code == 303, r.text[:400]
            r = await _post(c, "/users/revoke/2", {})
            assert r.status_code == 303, r.text[:400]
            v = await _post(c, "/audit/verify", {})
        assert v.status_code == 200, v.text[:400]
        assert "哈希链完整" in v.text
    asyncio.run(run())
    assert _audit_count() == 4
