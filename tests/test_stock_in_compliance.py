"""Tests for the single-tube ``stock_in`` API 5CT compliance gate.

Covers project rule #67 ("all inbound tubes must pass the API 5CT check,
including manual stock-in and 盘盈 surplus cases") and the router-level
``ComplianceError`` mapping. Runs service-layer + HTTP-layer round trips:

  * pass path — spec matches, tube row persisted with ``compliance_pass=True``;
  * fail: wall thickness out of range → ``ComplianceError`` raised, NO tube
    row persisted (atomic rollback);
  * fail: no spec for grade → ``ComplianceError`` raised;
  * caller-supplied ``compliance_pass=True`` skips the Rust call and persists
    the value as-is (mirrors the CSV-import path);
  * HTTP: posting a non-compliant grade to ``/tubes/stock-in`` returns 422
    and re-renders the form with the error.
"""

from __future__ import annotations

import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

TEST_DB_PATH = "data/test_stock_in_compliance.db"
os.environ["DATABASE_URL"] = f"sqlite+aiosqlite:///{TEST_DB_PATH}"
os.environ.setdefault("JWT_SECRET_KEY", "test-secret-stock-in-compliance")

import asyncio

import pytest
from httpx import ASGITransport, AsyncClient
from sqlalchemy import create_engine as create_sync_engine, text
from sqlalchemy.ext.asyncio import create_async_engine
from sqlalchemy.orm import sessionmaker as sync_sessionmaker

from src import database as db
from src.main import app
from src.models import Base, Location, StandardSpec, User
from src.modules.warehouse.services.audit_service import ComplianceError
from src.modules.warehouse.services.tube_service import stock_in
from rust_core import auth as rust_auth
from src.security import get_current_user, require_csrf

SYNC_DB_URL = f"sqlite:///{TEST_DB_PATH}"
ASYNC_DB_URL = f"sqlite+aiosqlite:///{TEST_DB_PATH}"

sync_engine = create_sync_engine(SYNC_DB_URL, echo=False)
SyncSession = sync_sessionmaker(bind=sync_engine)
test_engine = create_async_engine(ASYNC_DB_URL, echo=False)


@pytest.fixture(autouse=True)
def setup_db():
    from sqlalchemy.ext.asyncio import async_sessionmaker, AsyncSession

    db.engine = test_engine
    db.async_session_factory = async_sessionmaker(
        test_engine, class_=AsyncSession, expire_on_commit=False,
    )

    Base.metadata.create_all(bind=sync_engine)
    with SyncSession() as session:
        session.add(Location(code="WH-A", name="Warehouse A", used_count=0, is_active=True))
        session.add(
            StandardSpec(
                grade="L80",
                outer_diameter_mm=88.9,
                min_wall_thickness_mm=6.0,
                max_wall_thickness_mm=10.0,
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
    asyncio.run(test_engine.dispose())


# ---------------------------------------------------------------------------
# Service-layer tests
# ---------------------------------------------------------------------------


async def _tube_exists(pipe_id: str) -> bool:
    async with test_engine.connect() as conn:
        row = (
            await conn.execute(
                text("SELECT compliance_pass FROM tubes WHERE pipe_id=:p"),
                {"p": pipe_id},
            )
        ).fetchone()
    return row is not None


async def _tube_compliance_pass(pipe_id: str):
    async with test_engine.connect() as conn:
        row = (
            await conn.execute(
                text("SELECT compliance_pass FROM tubes WHERE pipe_id=:p"),
                {"p": pipe_id},
            )
        ).fetchone()
    return row[0] if row else None


def test_stock_in_compliant_persists_true():
    """Spec matches → tube row has compliance_pass=True."""
    pipe_id = "SP-L80-88.9x8.00-aaaac001"

    async def _run():
        result = await stock_in(
            type_="seamless_pipe",
            outer_diameter_mm=88.9,
            wall_thickness_mm=8.0,
            length_m=12.0,
            steel_grade="L80",
            location="WH-A",
            operator_id=1,
            manufacturer="Acme",
            pipe_id=pipe_id,
            thread_type="STC",
            engine=test_engine,
        )
        assert result["compliance_pass"] is True
        assert await _tube_exists(pipe_id)
        assert await _tube_compliance_pass(pipe_id) == 1

    asyncio.run(_run())


def test_stock_in_wall_thickness_out_of_range_raises():
    """Tube with wall_thickness outside spec range → ComplianceError, no row."""
    pipe_id = "SP-L80-88.9x13.00-aaaac002"

    async def _run():
        with pytest.raises(ComplianceError) as ei:
            await stock_in(
                type_="seamless_pipe",
                outer_diameter_mm=88.9,
                wall_thickness_mm=13.0,  # > max 10.0
                length_m=12.0,
                steel_grade="L80",
                location="WH-A",
                operator_id=1,
                manufacturer="Acme",
                pipe_id=pipe_id,
                engine=test_engine,
            )
        assert "wall_thickness_range" in str(ei.value)
        assert not await _tube_exists(pipe_id)

    asyncio.run(_run())


def test_stock_in_outer_diameter_off_raises():
    """Outer diameter off by >1% → ComplianceError, no row."""
    pipe_id = "SP-L80-95.0x8.00-aaaac003"

    async def _run():
        with pytest.raises(ComplianceError):
            await stock_in(
                type_="seamless_pipe",
                outer_diameter_mm=95.0,  # off >1% from spec 88.9
                wall_thickness_mm=8.0,
                length_m=12.0,
                steel_grade="L80",
                location="WH-A",
                operator_id=1,
                manufacturer="Acme",
                pipe_id=pipe_id,
                engine=test_engine,
            )
        assert not await _tube_exists(pipe_id)

    asyncio.run(_run())


def test_stock_in_unknown_grade_raises():
    """No matching standard_specs row → ComplianceError, no row written."""
    pipe_id = "SP-X80-88.9x8.00-aaaac004"

    async def _run():
        with pytest.raises(ComplianceError) as ei:
            await stock_in(
                type_="seamless_pipe",
                outer_diameter_mm=88.9,
                wall_thickness_mm=8.0,
                length_m=12.0,
                steel_grade="X80",  # no spec seeded
                location="WH-A",
                operator_id=1,
                manufacturer="Acme",
                pipe_id=pipe_id,
                engine=test_engine,
            )
        assert "no standard spec" in str(ei.value)
        assert not await _tube_exists(pipe_id)

    asyncio.run(_run())


def test_stock_in_caller_supplied_compliance_skips_rust():
    """Explicit compliance_pass=True persists as-is even if tube doesn't match."""
    pipe_id = "SP-L80-88.9x13.00-aaaac005"

    async def _run():
        result = await stock_in(
            type_="seamless_pipe",
            outer_diameter_mm=88.9,
            wall_thickness_mm=13.0,  # would fail if validated
            length_m=12.0,
            steel_grade="L80",
            location="WH-A",
            operator_id=1,
            manufacturer="Acme",
            pipe_id=pipe_id,
            compliance_pass=True,  # caller asserts pre-validated
            engine=test_engine,
        )
        assert result["compliance_pass"] is True
        assert await _tube_compliance_pass(pipe_id) == 1

    asyncio.run(_run())


# ---------------------------------------------------------------------------
# HTTP-layer test — router catches ComplianceError → 422 + form re-renders
# ---------------------------------------------------------------------------


def test_post_stock_in_noncompliant_returns_422():
    """POST /tubes/stock-in with non-compliant params → 422, form re-rendered."""

    async def _run():
        transport = ASGITransport(app=app)
        async with AsyncClient(transport=transport, base_url="http://test") as client:
            r = await client.post(
                "/tubes/stock-in",
                data={
                    "type": "seamless_pipe",  # router alias: Form(alias="type")
                    "outer_diameter_mm": "88.9",
                    "wall_thickness_mm": "13.0",  # out of spec
                    "length_m": "12.0",
                    "steel_grade": "L80",
                    "location": "WH-A",
                    "manufacturer": "Acme",
                    "quality_mark": "normal",
                    "csrf_token": "stub",
                },
            )
            assert r.status_code == 422
            assert "text/html" in r.headers.get("content-type", "")
            assert "compliance" in r.text.lower() or "wall_thickness" in r.text.lower()

    asyncio.run(_run())
