"""Batch stock-out - simple tests."""

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
    _reset(); _seed(); db.engine = ae; db.async_session_factory = async_sessionmaker(db.engine, class_=AsyncSession, expire_on_commit=False)
    yield
    app.dependency_overrides.clear(); Base.metadata.drop_all(bind=se); se.dispose()

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