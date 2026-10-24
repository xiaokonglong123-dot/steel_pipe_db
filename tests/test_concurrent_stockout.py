"""Concurrent stock-out test — proves BEGIN IMMEDIATE serialization.

Pattern: 5 concurrent ``stock_out`` calls on the same tube.
Expected: exactly 1 success, 4 ``ConflictError``.

Also tests:
- ``used_count`` is decremented after stock-out.
- ``problematic`` quality_mark requires admin role.
"""

from __future__ import annotations

import asyncio
import os
import sys

# Ensure the project root is on sys.path so that ``from src import …`` works.
sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

# ---------------------------------------------------------------------------
# Test DB — use a SEPARATE file, never touch the real inventory.
# ---------------------------------------------------------------------------
TEST_DB_PATH = "data/test_stockout.db"

os.environ["DATABASE_URL"] = f"sqlite+aiosqlite:///{TEST_DB_PATH}"

from datetime import datetime, timezone

import pytest
from sqlalchemy import create_engine as create_sync_engine, text
from sqlalchemy.ext.asyncio import create_async_engine
from sqlalchemy.orm import sessionmaker as sync_sessionmaker

from src.models import Base, Location, Tube
from src.modules.warehouse.services.tube_service import (
    ConflictError,
    ForbiddenError,
    stock_out,
)

# ---------------------------------------------------------------------------
# Engines  (sync for setup/teardown, async for the service)
# ---------------------------------------------------------------------------
SYNC_DB_URL = f"sqlite:///{TEST_DB_PATH}"
ASYNC_DB_URL = f"sqlite+aiosqlite:///{TEST_DB_PATH}"

sync_engine = create_sync_engine(SYNC_DB_URL, echo=False)
SyncSession = sync_sessionmaker(bind=sync_engine)

test_engine = create_async_engine(ASYNC_DB_URL, echo=False)


# ---------------------------------------------------------------------------
# Fixtures
# ---------------------------------------------------------------------------


@pytest.fixture(autouse=True)
def setup_db():
    """Create tables, seed one Location + two Tubes before every test."""
    Base.metadata.create_all(bind=sync_engine)

    with SyncSession() as session:
        # Location with used_count=1 so the decrement is observable.
        loc = Location(code="DEFAULT", name="Default", used_count=1)
        session.add(loc)

        # Tube 1: normal quality — for concurrent stock-out test.
        tube1 = Tube(
            pipe_id="SP-TEST",
            type_="seamless_pipe",
            outer_diameter_mm=88.9,
            wall_thickness_mm=8.0,
            length_m=12.0,
            steel_grade="L80",
            manufacturer="X",
            location="DEFAULT",
            status="in_stock",
            quality_mark="normal",
            created_at=datetime.now(timezone.utc),
            updated_at=datetime.now(timezone.utc),
        )
        session.add(tube1)

        # Tube 2: problematic quality — for the admin-gate test.
        tube2 = Tube(
            pipe_id="SP-PROBLEMATIC",
            type_="seamless_pipe",
            outer_diameter_mm=88.9,
            wall_thickness_mm=8.0,
            length_m=12.0,
            steel_grade="L80",
            manufacturer="X",
            location="DEFAULT",
            status="in_stock",
            quality_mark="problematic",
            created_at=datetime.now(timezone.utc),
            updated_at=datetime.now(timezone.utc),
        )
        session.add(tube2)

        session.commit()

    yield

    Base.metadata.drop_all(bind=sync_engine)


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


async def _read_tube(pipe_id: str) -> dict | None:
    """Read a tube row as a dict (used for assertions after stock-out)."""
    async with test_engine.connect() as conn:
        row = (
            await conn.execute(
                text(
                    "SELECT pipe_id, status, location FROM tubes "
                    "WHERE pipe_id=:pid"
                ),
                {"pid": pipe_id},
            )
        ).fetchone()
        if row is None:
            return None
        return {"pipe_id": row[0], "status": row[1], "location": row[2]}


async def _read_location(code: str) -> dict | None:
    """Read a location row."""
    async with test_engine.connect() as conn:
        row = (
            await conn.execute(
                text(
                    "SELECT code, used_count FROM locations WHERE code=:c"
                ),
                {"c": code},
            )
        ).fetchone()
        if row is None:
            return None
        return {"code": row[0], "used_count": row[1]}


# ---------------------------------------------------------------------------
# Tests
# ---------------------------------------------------------------------------


def test_concurrent_stock_out():
    """5 concurrent stock-out calls on the same tube → 1 success, 4 conflicts."""
    async def _run():
        results = await asyncio.gather(
            stock_out("SP-TEST", operator_id=1, role="operator", engine=test_engine),
            stock_out("SP-TEST", operator_id=1, role="operator", engine=test_engine),
            stock_out("SP-TEST", operator_id=1, role="operator", engine=test_engine),
            stock_out("SP-TEST", operator_id=1, role="operator", engine=test_engine),
            stock_out("SP-TEST", operator_id=1, role="operator", engine=test_engine),
            return_exceptions=True,
        )

        successes = [r for r in results if not isinstance(r, Exception)]
        conflicts = [r for r in results if isinstance(r, ConflictError)]

        assert len(successes) == 1, (
            f"expected 1 success, got {len(successes)}: "
            f"{[s for s in successes]}"
        )
        assert len(conflicts) == 4, (
            f"expected 4 conflicts, got {len(conflicts)}"
        )

    asyncio.run(_run())


def test_used_count_decremented():
    """After stock-out, used_count goes from 1 → 0."""
    async def _run():
        # Pre-condition: used_count is 1
        loc_before = await _read_location("DEFAULT")
        assert loc_before is not None
        assert loc_before["used_count"] == 1

        # Stock out the only tube
        await stock_out(
            "SP-TEST", operator_id=1, role="operator", engine=test_engine
        )

        # Post-condition: used_count is 0
        loc_after = await _read_location("DEFAULT")
        assert loc_after is not None
        assert loc_after["used_count"] == 0

        # Tube status is out_stock
        tube = await _read_tube("SP-TEST")
        assert tube is not None
        assert tube["status"] == "out_stock"

    asyncio.run(_run())


def test_problematic_requires_admin():
    """A problematic tube can only be stocked out by an admin."""
    async def _run():
        # Operator → ForbiddenError
        with pytest.raises(ForbiddenError):
            await stock_out(
                "SP-PROBLEMATIC",
                operator_id=1,
                role="operator",
                engine=test_engine,
            )

        # Admin → succeeds
        result = await stock_out(
            "SP-PROBLEMATIC",
            operator_id=2,
            role="admin",
            engine=test_engine,
        )
        assert result["pipe_id"] == "SP-PROBLEMATIC"

        # Tube is now out_stock
        tube = await _read_tube("SP-PROBLEMATIC")
        assert tube is not None
        assert tube["status"] == "out_stock"

    asyncio.run(_run())
