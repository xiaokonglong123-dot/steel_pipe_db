"""Batch stock-out tests.

``POST /tubes/stock-out/batch`` is atomic: all selected tubes are outed in
one ``BEGIN IMMEDIATE`` transaction, or none are. These tests cover the
success path plus the rollback guarantees (missing pipe_id, duplicate
pipe_id) that the previous per-tube implementation did not provide.
"""

import asyncio, os, sys
sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))
TEST_DB_PATH = "data/test_batch_stockout.db"
os.environ["DATABASE_URL"] = f"sqlite+aiosqlite:///{TEST_DB_PATH}"
os.environ.setdefault("JWT_SECRET_KEY", "test-batch")

from datetime import datetime, timezone
from httpx import ASGITransport, AsyncClient
import pytest
from sqlalchemy import create_engine as sync_engine
from sqlalchemy.ext.asyncio import AsyncSession, async_sessionmaker, create_async_engine
from sqlalchemy.orm import sessionmaker as sync_sessionmaker
from rust_core import auth as rust_auth
rust_auth.init(os.environ["JWT_SECRET_KEY"])
from src import database as db
from src.main import app
from src.models import AuditLog, Base, Location, Tube, User
from src.security import get_current_user, require_csrf

_SYNC = f"sqlite:///{TEST_DB_PATH}"
se = sync_engine(_SYNC, echo=False)
SS = sync_sessionmaker(bind=se)
ae = create_async_engine(f"sqlite+aiosqlite:///{TEST_DB_PATH}")

T1, T2, T3 = "SP-bs1", "SP-bs2", "SP-bs3"

def _seed():
    Base.metadata.create_all(bind=se)
    with SS() as s:
        for t in [AuditLog, Tube, Location, User]: s.query(t).delete()
        s.commit()
        s.add(Location(code="BATCH", name="B", used_count=3, is_active=True))
        s.add(User(id=2, username="op", role="operator", display_name="O", password_hash=rust_auth.hash_password("p"), active=True, token_version=0))
        now = datetime.now(timezone.utc)
        for pid in [T1, T2, T3]:
            s.add(Tube(pipe_id=pid, type_="seamless_pipe", outer_diameter_mm=80.0, wall_thickness_mm=6.0, length_m=10.0, steel_grade="H40", location="BATCH", status="in_stock", quality_mark="normal", manufacturer="m", stock_in_time=now, created_at=now, updated_at=now))
        s.commit()

def _reset():
    for p in [TEST_DB_PATH, TEST_DB_PATH+"-wal", TEST_DB_PATH+"-shm"]:
        if os.path.exists(p): os.remove(p)

def _stub(uid):
    async def f():
        with SS() as s: return s.query(User).get(uid)
    return f

def _allow_csrf():
    async def f(): return None
    return f

@pytest.fixture
def client():
    return AsyncClient(transport=ASGITransport(app=app), base_url="http://x")

@pytest.fixture(autouse=True)
def fx():
    app.dependency_overrides.clear()
    # Dispose both engines BEFORE unlinking the DB file. A pooled connection to
    # the previous test's file keeps that inode alive after unlink, so the next
    # test's fresh file is a *different* inode and a stale async connection
    # would report "no such table" instead of the freshly seeded schema.
    se.dispose()
    asyncio.run(ae.dispose())
    _reset()
    _seed()
    db.engine = ae
    db.async_session_factory = async_sessionmaker(db.engine, class_=AsyncSession, expire_on_commit=False)
    yield
    app.dependency_overrides.clear()
    se.dispose()
    asyncio.run(ae.dispose())

# ----------------------------------------------------------------------
def test_batch_out(client):
    app.dependency_overrides[get_current_user] = _stub(2)
    app.dependency_overrides[require_csrf] = _allow_csrf
    async def r():
        async with client as c:
            resp = await c.post("/tubes/stock-out/batch", data={"pipe_ids": [T1, T2], "csrf_token": "1"})
            assert resp.status_code == 303
    asyncio.run(r())
    with SS() as s:
        for pid, st in [(T1,"out_stock"), (T2,"out_stock"), (T3,"in_stock")]:
            assert s.query(Tube).filter(Tube.pipe_id==pid).first().status == st


def test_batch_out_decrements_used_count_and_writes_one_txn(client):
    """A successful batch is one transaction with per-tube audit rows."""
    app.dependency_overrides[get_current_user] = _stub(2)
    app.dependency_overrides[require_csrf] = _allow_csrf
    async def r():
        async with client as c:
            resp = await c.post("/tubes/stock-out/batch", data={"pipe_ids": [T1, T2], "csrf_token": "1"})
            assert resp.status_code == 303
    asyncio.run(r())
    with SS() as s:
        # used_count 3 -> 1 (two tubes out of BATCH)
        assert s.query(Location).filter(Location.code=="BATCH").first().used_count == 1
        logs = s.query(AuditLog).all()
        assert len(logs) == 2
        assert all(l.action == "stock_out" for l in logs)
        assert all("batch=2" in (l.detail or "") for l in logs)
        assert {l.detail.split("pipe_id=")[1].split()[0] for l in logs} == {T1, T2}


def test_batch_out_unknown_pipe_id_rolls_back_whole_batch(client):
    """A missing pipe_id aborts the batch: no tube changes, no used_count
    drift, no audit rows — all-or-nothing."""
    app.dependency_overrides[get_current_user] = _stub(2)
    app.dependency_overrides[require_csrf] = _allow_csrf
    async def r():
        async with client as c:
            resp = await c.post("/tubes/stock-out/batch", data={"pipe_ids": [T1, "SP-nope"], "csrf_token": "1"})
            assert resp.status_code == 409
    asyncio.run(r())
    with SS() as s:
        for pid in [T1, T2, T3]:
            assert s.query(Tube).filter(Tube.pipe_id==pid).first().status == "in_stock"
        assert s.query(Location).filter(Location.code=="BATCH").first().used_count == 3
        assert s.query(AuditLog).count() == 0


def test_batch_out_duplicate_pipe_id_rolls_back_whole_batch(client):
    """Same pipe_id twice: the second guard-read sees it already out and the
    transaction rolls back, so the first one is restored too."""
    app.dependency_overrides[get_current_user] = _stub(2)
    app.dependency_overrides[require_csrf] = _allow_csrf
    async def r():
        async with client as c:
            resp = await c.post("/tubes/stock-out/batch", data={"pipe_ids": [T1, T1], "csrf_token": "1"})
            assert resp.status_code == 409
    asyncio.run(r())
    with SS() as s:
        assert s.query(Tube).filter(Tube.pipe_id==T1).first().status == "in_stock"
        assert s.query(Location).filter(Location.code=="BATCH").first().used_count == 3
        assert s.query(AuditLog).count() == 0


def test_batch_out_empty_selection_is_noop(client):
    app.dependency_overrides[get_current_user] = _stub(2)
    app.dependency_overrides[require_csrf] = _allow_csrf
    async def r():
        async with client as c:
            resp = await c.post("/tubes/stock-out/batch", data={"pipe_ids": ["   "], "csrf_token": "1"})
            assert resp.status_code == 303
    asyncio.run(r())
    with SS() as s:
        assert s.query(Location).filter(Location.code=="BATCH").first().used_count == 3
        assert s.query(AuditLog).count() == 0