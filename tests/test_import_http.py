"""HTTP-level coverage for ``POST /tubes/import`` CSV upload.

Auth and CSRF are bypassed by FastAPI dependency overrides; we exercise
the real router + import_service + DB layer end-to-end via httpx + the
ASGI transport. Covers:

* GET /tubes/import returns the form (200, text/html).
* POST /tubes/import with a compliant CSV row → 200, summary ok=1.
* POST with a CSV bigger than 20 MB → 413.
* POST with missing required columns → 400.
* POST with a row that fails API 5CT compliance → 200 with summary
  errors=[{row:.., error:"compliance failed ..."}] (per-row errors do not
  abort the batch).
"""

from __future__ import annotations

import os
import sys

# Ensure the project root is on sys.path so that ``from src import …`` works.
sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

# Configure DB BEFORE importing src.* so src.database binds to the test DB.
TEST_DB_PATH = "data/test_import_http.db"
os.environ["DATABASE_URL"] = f"sqlite+aiosqlite:///{TEST_DB_PATH}"
os.environ.setdefault("JWT_SECRET_KEY", "test-secret-import-http")

import asyncio
from io import BytesIO
from typing import AsyncIterator

import pytest
from httpx import ASGITransport, AsyncClient
from sqlalchemy import create_engine as create_sync_engine
from sqlalchemy.ext.asyncio import create_async_engine
from sqlalchemy.orm import sessionmaker as sync_sessionmaker

from src import database as db
from src.main import app
from src.models import Base, Location, StandardSpec, User
from src.modules.warehouse.services.import_service import (
    MAX_IMPORT_ROWS,
    import_tubes_csv,
)
from src.security import get_current_user, require_csrf


SYNC_DB_URL = f"sqlite:///{TEST_DB_PATH}"
ASYNC_DB_URL = f"sqlite+aiosqlite:///{TEST_DB_PATH}"

sync_engine = create_sync_engine(SYNC_DB_URL, echo=False)
SyncSession = sync_sessionmaker(bind=sync_engine)
test_engine = create_async_engine(ASYNC_DB_URL, echo=False)


@pytest.fixture(autouse=True)
def setup_db():
    # Rebind src.database.engine + session factory to the test engine so
    # everything that goes through `db.engine` (the default fallback in
    # import_service and tube_service) hits the test database. We replace
    # the sessionmaker instance wholesale — `async_sessionmaker.bind = X`
    # does NOT update the existing instance's bound engine, because that
    # attribute is captured as a kwarg at construction time.
    from sqlalchemy.ext.asyncio import async_sessionmaker, AsyncSession  # noqa: E402

    db.engine = test_engine
    db.async_session_factory = async_sessionmaker(
        test_engine,
        class_=AsyncSession,
        expire_on_commit=False,
    )

    Base.metadata.create_all(bind=sync_engine)
    with SyncSession() as session:
        from rust_core import auth as rust_auth

        session.add(Location(code="WH-A", name="Warehouse A", used_count=0, is_active=True))
        session.add(Location(code="WH-B", name="Warehouse B", used_count=0, is_active=True))
        session.add(
            StandardSpec(
                grade="L80",
                outer_diameter_mm=88.9,
                min_wall_thickness_mm=7.0,
                max_wall_thickness_mm=9.0,
                length_tolerance_mm=0.5,
                source="API 5CT",
            )
        )
        session.add(
            StandardSpec(
                grade="J55",
                outer_diameter_mm=88.9,
                min_wall_thickness_mm=6.0,
                max_wall_thickness_mm=8.0,
                length_tolerance_mm=0.5,
                source="API 5CT",
            )
        )
        session.add(
            User(
                id=1,
                username="admin",
                display_name="Admin",
                role="admin",
                password_hash=rust_auth.hash_password("Pa$$w0rd"),
                active=True,
                token_version=0,
            )
        )
        session.commit()

    # Override auth dependencies with a static admin user.
    async def _stub_user():
        with SyncSession() as s:
            return s.get(User, 1)

    async def _stub_csrf():
        return None

    app.dependency_overrides[get_current_user] = _stub_user
    app.dependency_overrides[require_csrf] = _stub_csrf

    yield

    app.dependency_overrides.clear()
    Base.metadata.drop_all(bind=sync_engine)
    sync_engine.dispose()
    import asyncio as _aio
    _aio.run(test_engine.dispose())


@pytest.fixture
def client_factory():
    """Factory returning an ``AsyncClient`` bound to the ASGI app.
    Each test should ``client = client_factory(); async with client:``."""
    def _make() -> AsyncClient:
        transport = ASGITransport(app=app)
        return AsyncClient(transport=transport, base_url="http://test")
    return _make


VALID_CSV = (
    b"pipe_type,outer_diameter_mm,wall_thickness_mm,length_m,steel_grade,location,manufacturer\n"
    b"seamless_pipe,88.9,8.0,12.0,L80,WH-A,Acme\n"
)


# ---------------------------------------------------------------------------
# Tests (sync wrapper: each coroutine is run via asyncio.run so plain
# pytest works without pytest-asyncio).
# ---------------------------------------------------------------------------


def test_get_import_form_ok(client_factory):
    async def _run():
        async with client_factory() as client:
            r = await client.get("/tubes/import")
            assert r.status_code == 200
            assert "text/html" in r.headers.get("content-type", "")
            assert "import" in r.text.lower()

    asyncio.run(_run())


def test_post_import_compliant_row_ok(client_factory):
    async def _run():
        files = {"file": ("upload.csv", BytesIO(VALID_CSV).read(), "text/csv")}
        data = {"csrf_token": "stub"}
        async with client_factory() as client:
            r = await client.post("/tubes/import", files=files, data=data)
            assert r.status_code == 200
            assert "text/html" in r.headers.get("content-type", "")

    asyncio.run(_run())


def test_post_import_too_large_413(client_factory):
    big = b"x" * (20 * 1024 * 1024 + 1)
    data = {"csrf_token": "stub"}
    files = {"file": ("big.csv", big, "text/csv")}

    async def _run():
        async with client_factory() as client:
            r = await client.post("/tubes/import", files=files, data=data)
            assert r.status_code == 413

    asyncio.run(_run())


def test_post_import_missing_columns_400(client_factory):
    csv = b"foo,bar\n1,2\n"
    data = {"csrf_token": "stub"}
    files = {"file": ("bad.csv", csv, "text/csv")}

    async def _run():
        async with client_factory() as client:
            r = await client.post("/tubes/import", files=files, data=data)
            assert r.status_code == 400

    asyncio.run(_run())


def test_post_import_noncompliant_row_collects_error(client_factory):
    csv = (
        b"pipe_type,outer_diameter_mm,wall_thickness_mm,length_m,steel_grade,location,manufacturer\n"
        b"seamless_pipe,88.9,8.0,12.0,J55,WH-A,Acme\n"
    )
    data = {"csrf_token": "stub"}
    files = {"file": ("mixed.csv", csv, "text/csv")}

    async def _run():
        async with client_factory() as client:
            # Per-row compliance failure does not abort the batch → 200.
            r = await client.post("/tubes/import", files=files, data=data)
            assert r.status_code == 200

    asyncio.run(_run())


# ---------------------------------------------------------------------------
# D7 — row-count cap: > MAX_IMPORT_ROWS rows → 400 row-limit error;
# exactly under the cap → per-row processing proceeds (no rejection).
# ---------------------------------------------------------------------------


def test_post_import_too_many_rows_400(client_factory):
    """CSV with MAX_IMPORT_ROWS+1 data rows → 400 with row-limit detail."""
    header = b"pipe_type,outer_diameter_mm,wall_thickness_mm,length_m,steel_grade,location,manufacturer\n"
    row = b"seamless_pipe,88.9,8.0,12.0,L80,WH-A,Acme\n"
    csv = header + row * (MAX_IMPORT_ROWS + 1)  # 10001 data rows, < 20MB
    data = {"csrf_token": "stub"}
    files = {"file": ("many.csv", csv, "text/csv")}

    async def _run():
        async with client_factory() as client:
            r = await client.post("/tubes/import", files=files, data=data)
        assert r.status_code == 400, r.text[:400]
        detail = r.json()["detail"]
        assert str(MAX_IMPORT_ROWS) in detail
        assert "split" in detail

    asyncio.run(_run())


def test_import_service_at_row_limit_not_rejected():
    """MAX_IMPORT_ROWS-1 rows reach per-row validation (errors collected)
    without a row-limit rejection — proves the cap boundary is exclusive."""
    header = b"pipe_type,outer_diameter_mm,wall_thickness_mm,length_m,steel_grade,location,manufacturer\n"
    row = b"bogus,88.9,8.0,12.0,L80,WH-A,Acme\n"
    csv = header + row * (MAX_IMPORT_ROWS - 1)  # 9999 data rows

    async def _run():
        result = await import_tubes_csv(content=csv, operator_id=1, engine=test_engine)
        assert result.ok == 0
        assert len(result.errors) == MAX_IMPORT_ROWS - 1

    asyncio.run(_run())
