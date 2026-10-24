"""tubes router POST endpoint e2e (stock-out + stock-in + batch + transfer).

Covered:
* POST /tubes/stock-in  → 303 redirect (single new tube).
* POST /tubes/stock-in with conflicting custom pipe_id → 409.
* POST /tubes/stock-out → 303 ok.
* POST /tubes/stock-out on already-out tube → 409 (TOCTOU guard).
* POST /tubes/stock-in/batch → 200 JSON summary.
* POST /tubes/stock-out missing CSRF → 403.
* POST /tubes/transfer → 303 ok between two seeded locations.
* POST /tubes/transfer when source==target → 409 ConflictError.
* POST /tubes/transfer of an already-out (in_transfer) tube → 409.
* POST /tubes/transfer missing CSRF → 403.
* POST /location/{code}/deactivate → 303 + is_active=False (row kept).
* POST /tubes/import-excel → 405 (half-wired Excel route removed).

Each seeded StandardSpec must match L80 / 88.9 / wall in min..max range so
the API 5CT compliance gate in stock_in is satisfied.
"""

from __future__ import annotations

import asyncio
import os
import sys
from datetime import UTC, datetime

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

TEST_DB_PATH = "data/test_tubes_routes.db"
os.environ["DATABASE_URL"] = f"sqlite+aiosqlite:///{TEST_DB_PATH}"
os.environ.setdefault("JWT_SECRET_KEY", "test-secret-tubes-routes")

import pytest
from httpx import ASGITransport, AsyncClient
from sqlalchemy import create_engine as create_sync_engine
from sqlalchemy import text
from sqlalchemy.ext.asyncio import AsyncSession, create_async_engine
from sqlalchemy.orm import sessionmaker as sync_sessionmaker

from rust_core import auth as rust_auth
from src import database as db
from src.main import app
from src.models import Base, Location, StandardSpec, User
from src.security import get_current_user, require_csrf

SYNC_DB_URL = f"sqlite:///{TEST_DB_PATH}"
ASYNC_DB_URL = f"sqlite+aiosqlite:///{TEST_DB_PATH}"
sync_engine = create_sync_engine(SYNC_DB_URL, echo=False)
SyncSession = sync_sessionmaker(bind=sync_engine)
test_engine = create_async_engine(ASYNC_DB_URL, echo=False)


def _seed():
    Base.metadata.create_all(bind=sync_engine)
    with SyncSession() as s:
        for row in s.query(User).all():
            s.delete(row)
        for row in s.query(Location).all():
            s.delete(row)
        for row in s.query(StandardSpec).all():
            s.delete(row)
        s.commit()
        s.add(User(
            id=1, username="admin", role="admin", display_name="Admin",
            active=True, password_hash=rust_auth.hash_password("Pa$$w0rd"),
            token_version=0,
        ))
        s.add(Location(code="WH-A", name="Warehouse A", used_count=0, is_active=True))
        s.add(Location(code="WH-B", name="Warehouse B", used_count=0, is_active=True))
        s.add(StandardSpec(
            grade="L80",
            outer_diameter_mm=88.9,
            min_wall_thickness_mm=6.0,
            max_wall_thickness_mm=10.0,
            length_tolerance_mm=0.5,
            source="API 5CT",
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
    from sqlalchemy.ext.asyncio import async_sessionmaker as mk
    db.engine = create_async_engine(ASYNC_DB_URL, echo=False)
    db.async_session_factory = mk(
        db.engine, class_=AsyncSession, expire_on_commit=False
    )
    yield
    app.dependency_overrides.clear()
    Base.metadata.drop_all(bind=sync_engine)
    sync_engine.dispose()
    asyncio.run(db.engine.dispose())


def _stock_in_payload(pipe_id: str | None = None, **kw):
    """Build a basic compliant form payload for stock-in."""
    p = {
        "type": "seamless_pipe",
        "outer_diameter_mm": "88.9",
        "wall_thickness_mm": "8.0",
        "length_m": "12.0",
        "steel_grade": "L80",
        "location": "WH-A",
        "manufacturer": "Acme",
        "quality_mark": "normal",
        "csrf_token": "stub",
    }
    if pipe_id:
        p["pipe_id"] = pipe_id
    p.update(kw)
    return p


def test_post_stock_in_success_redirects_303(client):
    app.dependency_overrides[get_current_user] = _stub_user(1)
    app.dependency_overrides[require_csrf] = _allow_csrf()
    async def run():
        async with client as c:
            r = await c.post(
                "/tubes/stock-in",
                data=_stock_in_payload(),
                follow_redirects=False,
            )
        assert r.status_code == 303, (r.status_code, r.text[:400])
        assert r.headers["location"] == "/tubes"
    asyncio.run(run())


def test_post_stock_in_conflict_pipe_id_returns_409(client):
    app.dependency_overrides[get_current_user] = _stub_user(1)
    app.dependency_overrides[require_csrf] = _allow_csrf()
    async def run():
        async with client as c:
            payload = _stock_in_payload(pipe_id="SP-L80-88.9x8.0-deadbeef")
            r1 = await c.post("/tubes/stock-in", data=payload, follow_redirects=False)
            assert r1.status_code == 303
            r2 = await c.post("/tubes/stock-in", data=payload, follow_redirects=False)
        assert r2.status_code == 409
    asyncio.run(run())


def test_post_stock_in_malformed_manual_pipe_id_returns_422(client):
    """A manually supplied pipe_id failing format validation must re-render
    the stock-in form with 422 + the validation message — never a 500."""
    app.dependency_overrides[get_current_user] = _stub_user(1)
    app.dependency_overrides[require_csrf] = _allow_csrf()
    async def run():
        async with client as c:
            r = await c.post(
                "/tubes/stock-in",
                data=_stock_in_payload(pipe_id="NOT-A-VALID-ID"),
                follow_redirects=False,
            )
        assert r.status_code == 422, (r.status_code, r.text[:400])
        assert "text/html" in r.headers.get("content-type", "")
        assert "must start with" in r.text
    asyncio.run(run())


def test_post_stock_out_success_redirects_303(client):
    """Seed a tube via stock-in POST first, then stock-out it."""
    app.dependency_overrides[get_current_user] = _stub_user(1)
    app.dependency_overrides[require_csrf] = _allow_csrf()
    async def run():
        async with client as c:
            r_in = await c.post(
                "/tubes/stock-in",
                data=_stock_in_payload(),
                follow_redirects=False,
            )
            assert r_in.status_code == 303
            # Lookup pipe_id via /tubes
            r_list = await c.get("/tubes", follow_redirects=False)
            assert r_list.status_code == 200
            # Pipe id auto-generated as `{SP|SCP}-{grade}-{od:.1f}x{wt:.2f}-{uuid8}`.
            import re
            m = re.search(r"(SP-L80-88\.9x8\.00-[0-9a-f]{8})", r_list.text)
            assert m is not None, "auto-generated pipe_id must appear in /tubes page"
            pipe_id = m.group(1)
            r_out = await c.post(
                "/tubes/stock-out",
                data={"pipe_id": pipe_id, "csrf_token": "stub"},
                follow_redirects=False,
            )
        assert r_out.status_code == 303
        assert r_out.headers["location"] == "/tubes"
    asyncio.run(run())


def test_post_stock_out_already_out_returns_409(client):
    """First stock-out succeeds (303); second on same pipe_id → 409."""
    app.dependency_overrides[get_current_user] = _stub_user(1)
    app.dependency_overrides[require_csrf] = _allow_csrf()
    async def run():
        async with client as c:
            r_in = await c.post(
                "/tubes/stock-in",
                data=_stock_in_payload(),
                follow_redirects=False,
            )
            assert r_in.status_code == 303
            r_list = await c.get("/tubes", follow_redirects=False)
            import re
            m = re.search(r"(SP-L80-88\.9x8\.00-[0-9a-f]{8})", r_list.text)
            assert m is not None
            pipe_id = m.group(1)
            r1 = await c.post(
                "/tubes/stock-out",
                data={"pipe_id": pipe_id, "csrf_token": "stub"},
                follow_redirects=False,
            )
            assert r1.status_code == 303
            r2 = await c.post(
                "/tubes/stock-out",
                data={"pipe_id": pipe_id, "csrf_token": "stub"},
                follow_redirects=False,
            )
        assert r2.status_code == 409
    asyncio.run(run())


def test_post_stock_out_missing_csrf_rejected(client):
    app.dependency_overrides[get_current_user] = _stub_user(1)
    async def run():
        async with client as c:
            r = await c.post(
                "/tubes/stock-out",
                data={"pipe_id": "SP-L80-88.9x8.0-deadbeef", "csrf_token": "bad"},
                follow_redirects=False,
            )
        assert r.status_code == 403
    asyncio.run(run())


def test_post_batch_stock_in_returns_200(client):
    app.dependency_overrides[get_current_user] = _stub_user(1)
    app.dependency_overrides[require_csrf] = _allow_csrf()
    async def run():
        async with client as c:
            r = await c.post(
                "/tubes/stock-in/batch",
                json={
                    "tubes": [
                        {
                            "pipe_type": "seamless_pipe",
                            "outer_diameter_mm": 88.9,
                            "wall_thickness_mm": 8.0,
                            "length_m": 12.0,
                            "steel_grade": "L80",
                            "location": "WH-A",
                            "manufacturer": "Acme",
                            "quality_mark": "normal",
                        },
                        {
                            "pipe_type": "seamless_pipe",
                            "outer_diameter_mm": 88.9,
                            "wall_thickness_mm": 7.5,
                            "length_m": 11.0,
                            "steel_grade": "L80",
                            "location": "WH-A",
                            "manufacturer": "Acme",
                            "quality_mark": "normal",
                        },
                    ]
                },
                follow_redirects=False,
            )
        assert r.status_code == 200
        body = r.json()
        assert "ok" in body
        assert body.get("ok", 0) == 2, body
    asyncio.run(run())


def test_post_batch_stock_in_missing_csrf_rejected(client):
    """batch POST is JSON-only — require_csrf reads csrf_token from JSON body.
    Without override + without csrf_token in body → 403."""
    app.dependency_overrides[get_current_user] = _stub_user(1)
    async def run():
        async with client as c:
            r = await c.post(
                "/tubes/stock-in/batch",
                json={"tubes": []},
                follow_redirects=False,
            )
        assert r.status_code == 403
    asyncio.run(run())


def test_post_transfer_success_redirects_303(client):
    """transfer of an in-stock tube from WH-A → WH-B returns 303."""
    app.dependency_overrides[get_current_user] = _stub_user(1)
    app.dependency_overrides[require_csrf] = _allow_csrf()
    async def run():
        async with client as c:
            r_in = await c.post(
                "/tubes/stock-in",
                data=_stock_in_payload(),
                follow_redirects=False,
            )
            assert r_in.status_code == 303
            r_list = await c.get("/tubes", follow_redirects=False)
            import re
            m = re.search(r"(SP-L80-88\.9x8\.00-[0-9a-f]{8})", r_list.text)
            assert m is not None
            pipe_id = m.group(1)
            r_t = await c.post(
                "/tubes/transfer",
                data={
                    "pipe_id": pipe_id,
                    "target_location": "WH-B",
                    "remark": "",
                    "csrf_token": "stub",
                },
                follow_redirects=False,
            )
        assert r_t.status_code == 303, (r_t.status_code, r_t.text[:400])
        assert r_t.headers["location"] == "/tubes"
    asyncio.run(run())


def test_post_transfer_same_location_returns_409(client):
    """transfer where target == source → ConflictError 'same location'."""
    app.dependency_overrides[get_current_user] = _stub_user(1)
    app.dependency_overrides[require_csrf] = _allow_csrf()
    async def run():
        async with client as c:
            r_in = await c.post(
                "/tubes/stock-in",
                data=_stock_in_payload(),
                follow_redirects=False,
            )
            assert r_in.status_code == 303
            r_list = await c.get("/tubes", follow_redirects=False)
            import re
            m = re.search(r"(SP-L80-88\.9x8\.00-[0-9a-f]{8})", r_list.text)
            assert m is not None
            pipe_id = m.group(1)
            # tube was stocked into WH-A, so keep target = WH-A
            r_t = await c.post(
                "/tubes/transfer",
                data={
                    "pipe_id": pipe_id,
                    "target_location": "WH-A",
                    "remark": "",
                    "csrf_token": "stub",
                },
                follow_redirects=False,
            )
        assert r_t.status_code == 409
    asyncio.run(run())


def test_post_transfer_not_in_stock_returns_409(client):
    """transfer of a stock-out (non-in-stock) tube → ConflictError."""
    app.dependency_overrides[get_current_user] = _stub_user(1)
    app.dependency_overrides[require_csrf] = _allow_csrf()
    async def run():
        async with client as c:
            r_in = await c.post(
                "/tubes/stock-in",
                data=_stock_in_payload(),
                follow_redirects=False,
            )
            assert r_in.status_code == 303
            r_list = await c.get("/tubes", follow_redirects=False)
            import re
            m = re.search(r"(SP-L80-88\.9x8\.00-[0-9a-f]{8})", r_list.text)
            assert m is not None
            pipe_id = m.group(1)
            r_out = await c.post(
                "/tubes/stock-out",
                data={"pipe_id": pipe_id, "csrf_token": "stub"},
                follow_redirects=False,
            )
            assert r_out.status_code == 303
            r_t = await c.post(
                "/tubes/transfer",
                data={
                    "pipe_id": pipe_id,
                    "target_location": "WH-B",
                    "remark": "",
                    "csrf_token": "stub",
                },
                follow_redirects=False,
            )
        assert r_t.status_code == 409
    asyncio.run(run())


def test_post_transfer_missing_csrf_rejected(client):
    app.dependency_overrides[get_current_user] = _stub_user(1)
    async def run():
        async with client as c:
            r = await c.post(
                "/tubes/transfer",
                data={"pipe_id": "SP-L80-88.9x8.00-deadbeef", "target_location": "WH-B", "csrf_token": "bad"},
                follow_redirects=False,
            )
        assert r.status_code == 403
    asyncio.run(run())


def test_list_tubes_excludes_soft_deleted(client):
    """D6: a soft-deleted tube (deleted_at set) vanishes from GET /tubes,
    including under a status filter that would otherwise match it."""
    app.dependency_overrides[get_current_user] = _stub_user(1)
    app.dependency_overrides[require_csrf] = _allow_csrf()
    async def run():
        async with client as c:
            r_in = await c.post(
                "/tubes/stock-in",
                data=_stock_in_payload(),
                follow_redirects=False,
            )
            assert r_in.status_code == 303
            r_list = await c.get("/tubes", follow_redirects=False)
            import re
            m = re.search(r"(SP-L80-88\.9x8\.00-[0-9a-f]{8})", r_list.text)
            assert m is not None, "tube must appear before soft-delete"
            pipe_id = m.group(1)

            # Soft-delete directly via SQL — simulates the delete path.
            async with test_engine.begin() as conn:
                await conn.execute(
                    text("UPDATE tubes SET deleted_at=:t WHERE pipe_id=:p"),
                    {"t": datetime.now(UTC), "p": pipe_id},
                )

            r_after = await c.get("/tubes", follow_redirects=False)
            assert pipe_id not in r_after.text, "soft-deleted tube must be hidden"

            # status-filtered edge case: status='in_stock' matches the row,
            # but the deleted_at filter must still exclude it.
            r_filtered = await c.get("/tubes?status=in_stock", follow_redirects=False)
            assert pipe_id not in r_filtered.text
    asyncio.run(run())


def test_post_location_deactivate_soft_deletes(client):
    """POST /location/{code}/deactivate soft-deletes (is_active=False) and
    keeps the row. Regression: the handler previously used ``session.query``
    on an AsyncSession → AttributeError → 500."""
    app.dependency_overrides[get_current_user] = _stub_user(1)
    app.dependency_overrides[require_csrf] = _allow_csrf()
    async def run():
        async with client as c:
            r = await c.post(
                "/location/WH-B/deactivate",
                data={"csrf_token": "stub"},
                follow_redirects=False,
            )
        assert r.status_code == 303, (r.status_code, r.text[:400])
        assert r.headers["location"] == "/locations"
        with SyncSession() as s:
            row = s.get(Location, "WH-B")
            assert row is not None, "soft-delete must keep the row"
            assert row.is_active is False
    asyncio.run(run())


def test_post_import_excel_route_removed(client):
    """The broken, unimplemented Excel import route is gone. POSTing to it
    now yields 405 (the path only exists as the GET /tubes/{pipe_id} detail
    route) instead of a 500 from a half-wired handler."""
    app.dependency_overrides[get_current_user] = _stub_user(1)
    app.dependency_overrides[require_csrf] = _allow_csrf()
    async def run():
        async with client as c:
            r = await c.post(
                "/tubes/import-excel",
                files={"file": ("x.xlsx", b"not-an-xlsx", "application/vnd.ms-excel")},
                data={"csrf_token": "stub"},
                follow_redirects=False,
            )
        assert r.status_code == 405, (r.status_code, r.text[:200])
    asyncio.run(run())
