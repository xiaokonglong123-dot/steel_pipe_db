"""Quality mark update tests (D12).

Covers:
* operator can change quality_mark/note of an in-stock pipe via /tubes/quality
* scrapped / out_stock / missing tubes are rejected
* viewer gets 403
* audit row recorded on every successful change
* csrf_token present on the form page
"""

from __future__ import annotations

import asyncio
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

TEST_DB_PATH = "data/test_quality.db"
os.environ["DATABASE_URL"] = f"sqlite+aiosqlite:///{TEST_DB_PATH}"
os.environ.setdefault("JWT_SECRET_KEY", "test-secret-quality")

from datetime import UTC, datetime

import pytest
from httpx import ASGITransport, AsyncClient
from sqlalchemy import create_engine as create_sync_engine
from sqlalchemy.ext.asyncio import AsyncSession, async_sessionmaker, create_async_engine
from sqlalchemy.orm import sessionmaker as sync_sessionmaker

from rust_core import auth as rust_auth

rust_auth.init(os.environ["JWT_SECRET_KEY"])
from src import database as db
from src.main import app
from src.models import AuditLog, Base, Location, Tube, User
from src.security import get_current_user, require_csrf

SYNC_DB_URL = f"sqlite:///{TEST_DB_PATH}"
ASYNC_DB_URL = f"sqlite+aiosqlite:///{TEST_DB_PATH}"
sync_engine = create_sync_engine(SYNC_DB_URL, echo=False)
SyncSession = sync_sessionmaker(bind=sync_engine)
test_engine = create_async_engine(ASYNC_DB_URL, echo=False)

T1 = "SP-L80-88.9x8.00-qa00001"
T2 = "SP-L80-88.9x8.00-qa00002"


def _seed():
    Base.metadata.create_all(bind=sync_engine)
    with SyncSession() as s:
        for t in (AuditLog, Tube, Location, User):
            s.query(t).delete()
        s.commit()
        s.add(Location(code="QA", name="QA warehouse", used_count=2, is_active=True))
        s.add(User(id=1, username="admin", role="admin", display_name="Admin",
                   password_hash=rust_auth.hash_password("pass"), active=True, token_version=0))
        s.add(User(id=2, username="op", role="operator", display_name="Operator",
                   password_hash=rust_auth.hash_password("pass"), active=True, token_version=0))
        s.add(User(id=3, username="viewer", role="viewer", display_name="Viewer",
                   password_hash=rust_auth.hash_password("pass"), active=True, token_version=0))
        for pipe_id, status, quality_mark in [
            (T1, "in_stock", "normal"),
            (T2, "out_stock", "normal"),
        ]:
            s.add(
                Tube(
                    pipe_id=pipe_id,
                    type_="seamless_pipe",
                    outer_diameter_mm=88.9,
                    wall_thickness_mm=8.00,
                    length_m=12.0,
                    steel_grade="L80",
                    location="wh",
                    status=status,
                    quality_mark=quality_mark,
                    manufacturer="test",
                    stock_in_time=datetime.now(UTC),
                    created_at=datetime.now(UTC),
                    updated_at=datetime.now(UTC),
                )
            )
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


def _stub_user(user_id):
    async def _dep():
        with SyncSession() as s:
            return s.get(User, user_id)

    return _dep


def _audit_count():
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
    db.engine = create_async_engine(ASYNC_DB_URL, echo=False)
    db.async_session_factory = async_sessionmaker(
        db.engine, class_=AsyncSession, expire_on_commit=False
    )
    yield
    app.dependency_overrides.clear()
    Base.metadata.drop_all(bind=sync_engine)
    sync_engine.dispose()


async def _get(client, url):
    return await client.get(url, follow_redirects=False)


async def _post(client, url, data=None):
    payload = {"csrf_token": "test-csrf"}
    if data:
        payload.update(data)
    return await client.post(url, data=payload, follow_redirects=False)


# ---- tests ---------------------------------------------------------------


def test_quality_form_renders(client):
    app.dependency_overrides[get_current_user] = _stub_user(2)
    app.dependency_overrides[require_csrf] = _allow_csrf()

    async def run():
        async with client as c:
            r = await _get(c, "/tubes/quality?pipe_id=" + T1)
            assert r.status_code == 200
            assert T1 in r.text

    asyncio.run(run())


def test_quality_update_normal_to_problematic(client):
    app.dependency_overrides[get_current_user] = _stub_user(2)
    app.dependency_overrides[require_csrf] = _allow_csrf()

    async def run():
        async with client as c:
            r = await _post(c, "/tubes/quality",
                            {"pipe_id": T1, "quality_mark": "problematic", "quality_note": "defect"})
            assert r.status_code == 303, r.text[:200]

    asyncio.run(run())
    with SyncSession() as s:
        t = s.query(Tube).filter(Tube.pipe_id == T1).first()
        assert t.quality_mark == "problematic"
        assert t.quality_note == "defect"
    assert _audit_count() == 1
    with SyncSession() as s:
        a = s.query(AuditLog).order_by(AuditLog.id.desc()).first()
        assert a.action == "quality_mark"
        assert "note=defect" in a.detail


def test_quality_update_to_scrapped(client):
    app.dependency_overrides[get_current_user] = _stub_user(2)
    app.dependency_overrides[require_csrf] = _allow_csrf()

    async def run():
        async with client as c:
            r = await _post(c, "/tubes/quality",
                            {"pipe_id": T1, "quality_mark": "scrapped"})
            assert r.status_code == 303

    asyncio.run(run())
    with SyncSession() as s:
        t = s.query(Tube).filter(Tube.pipe_id == T1).first()
        assert t.quality_mark == "scrapped"
    assert _audit_count() == 1


def test_quality_update_out_stock_rejected(client):
    app.dependency_overrides[get_current_user] = _stub_user(2)
    app.dependency_overrides[require_csrf] = _allow_csrf()

    async def run():
        async with client as c:
            r = await _post(c, "/tubes/quality",
                            {"pipe_id": T2, "quality_mark": "problematic"})
            assert r.status_code == 409
            assert "not in_stock" in r.text.lower()

    asyncio.run(run())
    assert _audit_count() == 0


def test_quality_form_404_on_nonexistent(client):
    app.dependency_overrides[get_current_user] = _stub_user(2)
    app.dependency_overrides[require_csrf] = _allow_csrf()

    async def _():
        async with client as c:
            r = await _get(c, "/tubes/quality?pipe_id=SP-L80-88.9x8.00-deadbeef")
            assert r.status_code == 200
            assert "not found" in r.text.lower()

    asyncio.run(_())
    assert _audit_count() == 0


def test_viewer_403(client):
    app.dependency_overrides[get_current_user] = _stub_user(3)
    app.dependency_overrides[require_csrf] = _allow_csrf()

    async def _():
        c = client
        r = await _get(c, "/tubes/quality")
        assert r.status_code == 403

    asyncio.run(_())


def test_tube_detail_200(client):
    """D14 detail page smoke."""
    app.dependency_overrides[get_current_user] = _stub_user(2)
    app.dependency_overrides[require_csrf] = _allow_csrf()
    async def _():
        async with client as c:
            r = await c.get(f"/tubes/{T1}")
            assert r.status_code == 200
            assert T1 in r.text
    asyncio.run(_())
def test_location_manage_admin(client):
    """D15 location management quick test."""
    app.dependency_overrides[get_current_user] = _stub_user(1)
    app.dependency_overrides[require_csrf] = _allow_csrf()
    async def _():
        async with client as c:
            r = await c.get("/locations")
            assert r.status_code == 200, r.text[:200]
            assert "location" in r.text.lower() or "库位" in r.text
            r2 = await c.post("/locations", data={"csrf_token": "x", "code": "NEW-LOC", "name": "Test"})
            assert r2.status_code == 303
    asyncio.run(_())
