"""Inventory check (盘点) tests — D11.

Covers the check lifecycle: create, mark (ok/missing/moved), surplus
add/delete, cancel, role matrix. Confirm/recover atomicity tests live in
the confirm task (appended later in this file).
"""

from __future__ import annotations

import asyncio
import json
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

TEST_DB_PATH = "data/test_check.db"
os.environ["DATABASE_URL"] = f"sqlite+aiosqlite:///{TEST_DB_PATH}"
os.environ.setdefault("JWT_SECRET_KEY", "test-secret-check")

from datetime import datetime, timezone  # noqa: E402

from httpx import ASGITransport, AsyncClient  # noqa: E402
import pytest  # noqa: E402
from sqlalchemy import create_engine as create_sync_engine  # noqa: E402
from sqlalchemy.ext.asyncio import AsyncSession, async_sessionmaker, create_async_engine  # noqa: E402
from sqlalchemy.orm import sessionmaker as sync_sessionmaker  # noqa: E402

from rust_core import auth as rust_auth  # noqa: E402

rust_auth.init(os.environ["JWT_SECRET_KEY"])
from src import database as db  # noqa: E402
from src.main import app  # noqa: E402
from src.models import (  # noqa: E402
    AuditLog, Base, CheckItem, CheckSession, Location, StandardSpec,
    Transaction, TransactionItem, Tube, User,
)
from src.security import get_current_user, require_csrf  # noqa: E402

SYNC_DB_URL = f"sqlite:///{TEST_DB_PATH}"
ASYNC_DB_URL = f"sqlite+aiosqlite:///{TEST_DB_PATH}"
sync_engine = create_sync_engine(SYNC_DB_URL, echo=False)
SyncSession = sync_sessionmaker(bind=sync_engine)
test_engine = create_async_engine(ASYNC_DB_URL, echo=False)

TUBES = [
    ("SP-L80-88.9x8.00-aaaac001", "normal"),
    ("SP-L80-88.9x8.00-aaaac002", "normal"),
    ("SP-L80-88.9x8.00-aaaac003", "normal"),
]
WH_A = "WH-A"
WH_B = "WH-B"


def _seed():
    Base.metadata.create_all(bind=sync_engine)
    with SyncSession() as s:
        for m in (AuditLog, TransactionItem, Transaction, CheckItem,
                  CheckSession, Tube, StandardSpec, Location, User):
            s.query(m).delete()
        s.commit()
        s.add(Location(code=WH_A, name="Warehouse A", used_count=3, is_active=True))
        s.add(Location(code=WH_B, name="Warehouse B", used_count=0, is_active=True))
        s.add(StandardSpec(
            grade="L80", outer_diameter_mm=88.9,
            min_wall_thickness_mm=6.0, max_wall_thickness_mm=10.0,
            length_tolerance_mm=0.5, source="API 5CT",
        ))
        s.add(User(id=1, username="admin", role="admin", display_name="Admin",
                   password_hash=rust_auth.hash_password("Pa$$w0rd"), active=True, token_version=0))
        s.add(User(id=2, username="operator", role="operator", display_name="Operator",
                   password_hash=rust_auth.hash_password("Pa$$w0rd"), active=True, token_version=0))
        s.add(User(id=3, username="viewer", role="viewer", display_name="Viewer",
                   password_hash=rust_auth.hash_password("Pa$$w0rd"), active=True, token_version=0))
        now = datetime.now(timezone.utc)
        for pid, qm in TUBES:
            s.add(Tube(
                pipe_id=pid, type_="seamless_pipe",
                outer_diameter_mm=88.9, wall_thickness_mm=8.0, length_m=12.0,
                steel_grade="L80", location=WH_A, status="in_stock",
                quality_mark=qm, manufacturer="TestCo",
                stock_in_time=now, created_at=now, updated_at=now,
            ))
        s.commit()


def _reset_db():
    asyncio.run(test_engine.dispose())
    sync_engine.dispose()
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


def _audit_rows():
    with SyncSession() as s:
        return s.query(AuditLog).order_by(AuditLog.id).all()


def _audit_count():
    with SyncSession() as s:
        return s.query(AuditLog).count()


def _sessions():
    with SyncSession() as s:
        return s.query(CheckSession).order_by(CheckSession.id).all()


def _items(check_id: int):
    with SyncSession() as s:
        return s.query(CheckItem).filter(CheckItem.check_id == check_id).order_by(CheckItem.id).all()


def _tube(pipe_id: str):
    with SyncSession() as s:
        return s.query(Tube).filter(Tube.pipe_id == pipe_id).first()


def _location(code: str):
    with SyncSession() as s:
        return s.query(Location).filter(Location.code == code).first()


def _txns():
    with SyncSession() as s:
        return s.query(Transaction).order_by(Transaction.id).all()


@pytest.fixture
def client():
    transport = ASGITransport(app=app)
    return AsyncClient(transport=transport, base_url="http://test")


@pytest.fixture(autouse=True)
def fx():
    _reset_db()
    _seed()
    db.engine = test_engine
    db.async_session_factory = async_sessionmaker(
        test_engine, class_=AsyncSession, expire_on_commit=False,
    )
    yield
    app.dependency_overrides.clear()
    Base.metadata.drop_all(bind=sync_engine)
    sync_engine.dispose()


async def _post(client, url: str, data: dict | None = None, follow: bool = False):
    return await client.post(
        url,
        data={**(data or {}), "csrf_token": "test-csrf"},
        follow_redirects=follow,
    )


def _as(user_id: int):
    app.dependency_overrides[get_current_user] = _stub_user(user_id)
    app.dependency_overrides[require_csrf] = _allow_csrf()


async def _create_session(client, location: str = WH_A) -> int:
    r = await _post(client, "/tubes/check", {"location": location, "remark": "r1"})
    assert r.status_code == 303, r.text[:200]
    return _sessions()[-1].id


# --- 1-2: role matrix -------------------------------------------------------

def test_1_viewer_get_check_list_forbidden(client):
    _as(3)
    async def run():
        async with client as c:
            r = await c.get("/tubes/check")
        assert r.status_code == 403, r.text[:200]
    asyncio.run(run())


def test_2_viewer_post_check_create_forbidden(client):
    _as(3)
    async def run():
        async with client as c:
            r = await _post(c, "/tubes/check", {"location": WH_A})
        assert r.status_code == 403, r.text[:200]
    asyncio.run(run())


# --- 3: create --------------------------------------------------------------

def test_3_operator_create_check(client):
    _as(2)
    async def run():
        async with client as c:
            r = await _post(c, "/tubes/check", {"location": WH_A, "remark": "r1"})
        assert r.status_code == 303, r.text[:200]
    asyncio.run(run())
    s = _sessions()
    assert len(s) == 1
    assert s[0].location == WH_A and s[0].status == "open" and s[0].created_by == 2


# --- 4-9: mark --------------------------------------------------------------

def test_4_mark_ok(client):
    _as(2)
    async def run():
        async with client as c:
            cid = await _create_session(c)
            r = await _post(c, f"/tubes/check/{cid}/items",
                            {"pipe_id": TUBES[0][0], "result": "ok"})
        assert r.status_code == 303, r.text[:200]
    asyncio.run(run())
    items = _items(_sessions()[-1].id)
    assert len(items) == 1 and items[0].result == "ok"


def test_5_mark_missing(client):
    _as(2)
    async def run():
        async with client as c:
            cid = await _create_session(c)
            r = await _post(c, f"/tubes/check/{cid}/items",
                            {"pipe_id": TUBES[0][0], "result": "missing"})
        assert r.status_code == 303, r.text[:200]
    asyncio.run(run())
    items = _items(_sessions()[-1].id)
    assert len(items) == 1 and items[0].result == "missing"


def test_6_mark_moved_without_location_422(client):
    _as(2)
    async def run():
        async with client as c:
            cid = await _create_session(c)
            r = await _post(c, f"/tubes/check/{cid}/items",
                            {"pipe_id": TUBES[0][0], "result": "moved",
                             "actual_location": ""})
        assert r.status_code == 422, r.text[:200]
    asyncio.run(run())
    assert len(_items(_sessions()[-1].id)) == 0


def test_7_mark_moved_same_location_409(client):
    _as(2)
    async def run():
        async with client as c:
            cid = await _create_session(c)
            r = await _post(c, f"/tubes/check/{cid}/items",
                            {"pipe_id": TUBES[0][0], "result": "moved",
                             "actual_location": WH_A})
        assert r.status_code == 409, r.text[:200]
    asyncio.run(run())
    assert len(_items(_sessions()[-1].id)) == 0


def test_8_mark_moved_valid(client):
    _as(2)
    async def run():
        async with client as c:
            cid = await _create_session(c)
            r = await _post(c, f"/tubes/check/{cid}/items",
                            {"pipe_id": TUBES[0][0], "result": "moved",
                             "actual_location": WH_B})
        assert r.status_code == 303, r.text[:200]
    asyncio.run(run())
    items = _items(_sessions()[-1].id)
    assert len(items) == 1
    assert items[0].result == "moved" and items[0].actual_location == WH_B


def test_9_mark_upsert_updates_single_row(client):
    _as(2)
    async def run():
        async with client as c:
            cid = await _create_session(c)
            await _post(c, f"/tubes/check/{cid}/items",
                        {"pipe_id": TUBES[0][0], "result": "missing"})
            r = await _post(c, f"/tubes/check/{cid}/items",
                            {"pipe_id": TUBES[0][0], "result": "ok"})
        assert r.status_code == 303, r.text[:200]
    asyncio.run(run())
    items = _items(_sessions()[-1].id)
    assert len(items) == 1 and items[0].result == "ok"


# --- 10-11: surplus ---------------------------------------------------------

def _surplus_payload():
    return {
        "type": "seamless_pipe", "outer_diameter_mm": 88.9,
        "wall_thickness_mm": 8.0, "length_m": 12.0, "steel_grade": "L80",
        "manufacturer": "TestCo", "quality_mark": "normal",
    }


def test_10_add_surplus_creates_item_with_preview_pipe_id(client):
    _as(2)
    async def run():
        async with client as c:
            cid = await _create_session(c)
            r = await _post(c, f"/tubes/check/{cid}/surplus", _surplus_payload())
        assert r.status_code == 303, r.text[:200]
    asyncio.run(run())
    items = _items(_sessions()[-1].id)
    assert len(items) == 1
    it = items[0]
    assert it.result == "surplus"
    import re
    assert re.match(r"SP-L80-88\.9x8\.00-[0-9a-f]{8}$", it.pipe_id)
    data = json.loads(it.surplus_data)
    assert data["steel_grade"] == "L80" and data["outer_diameter_mm"] == 88.9


def test_11_delete_surplus_item(client):
    _as(2)
    async def run():
        async with client as c:
            cid = await _create_session(c)
            await _post(c, f"/tubes/check/{cid}/surplus", _surplus_payload())
            await _post(c, f"/tubes/check/{cid}/items",
                        {"pipe_id": TUBES[0][0], "result": "ok"})
            r = await _post(c, f"/tubes/check/{cid}/surplus/1/delete")
            assert r.status_code == 303, r.text[:200]
            r2 = await _post(c, f"/tubes/check/{cid}/surplus/2/delete")  # ok 项不可删
        assert r2.status_code == 409, r2.text[:200]
    asyncio.run(run())
    items = _items(_sessions()[-1].id)
    assert len(items) == 1 and items[0].result == "ok"


# --- 12 + 18a: cancel / non-open --------------------------------------------

def test_18a_cancel_open_session(client):
    _as(2)
    async def run():
        async with client as c:
            cid = await _create_session(c)
            r = await _post(c, f"/tubes/check/{cid}/cancel")
        assert r.status_code == 303, r.text[:200]
    asyncio.run(run())
    assert _sessions()[-1].status == "cancelled"


def test_12_mark_non_open_session_409(client):
    _as(2)
    async def run():
        async with client as c:
            cid = await _create_session(c)
            await _post(c, f"/tubes/check/{cid}/cancel")
            r = await _post(c, f"/tubes/check/{cid}/items",
                            {"pipe_id": TUBES[0][0], "result": "ok"})
        assert r.status_code == 409, r.text[:200]
    asyncio.run(run())
    assert len(_items(_sessions()[-1].id)) == 0


# --- 13: operator cannot confirm ---------------------------------------------

def test_13_operator_confirm_forbidden(client):
    _as(2)
    async def run():
        async with client as c:
            cid = await _create_session(c)
            r = await _post(c, f"/tubes/check/{cid}/confirm")
        assert r.status_code == 403, r.text[:200]
    asyncio.run(run())
