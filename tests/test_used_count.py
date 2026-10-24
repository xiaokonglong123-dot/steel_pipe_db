"""used_count consistency tests — PROJECT_RULES #77 / #78.

Asserts locations.used_count stays in sync with the actual
``SELECT COUNT(*) FROM tubes WHERE status='in_stock'`` after each
business operation (stock-in, stock-out, transfer).  Also covers the
drift-repair path via ``refresh_used_count``.

Matches the style of test_concurrent_stockout.py (asyncio.run inside
sync tests; isolated per-file test DB).
"""

from __future__ import annotations

import asyncio
import os
import sys
from datetime import datetime, timezone

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

import pytest
from sqlalchemy import create_engine as create_sync_engine, text
from sqlalchemy.ext.asyncio import create_async_engine
from sqlalchemy.orm import sessionmaker as sync_sessionmaker

from src.models import Base, Location, StandardSpec, Tube
from src.modules.warehouse.services.tube_service import (
    stock_in,
    stock_out,
    transfer,
    refresh_used_count,
)

TEST_DB_PATH = "data/test_used_count.db"
SYNC_DB_URL = f"sqlite:///{TEST_DB_PATH}"
ASYNC_DB_URL = f"sqlite+aiosqlite:///{TEST_DB_PATH}"

sync_engine = create_sync_engine(SYNC_DB_URL, echo=False)
SyncSession = sync_sessionmaker(bind=sync_engine)
test_engine = create_async_engine(ASYNC_DB_URL, echo=False)


@pytest.fixture(autouse=True)
def setup_db():
    Base.metadata.create_all(bind=sync_engine)
    with SyncSession() as session:
        session.add(Location(code="WH-A", name="Warehouse A", used_count=0, is_active=True))
        session.add(Location(code="WH-B", name="Warehouse B", used_count=0, is_active=True))
        # API 5CT compliance spec used by every tube in this suite
        # (seamless L80, 88.9mm OD, 6.0–10.0mm wall). stock_in now
        # auto-verifies against standard_specs; without this row the
        # compliance gate raises ComplianceError.
        session.add(
            StandardSpec(
                grade="L80",
                outer_diameter_mm=88.9,
                min_wall_thickness_mm=6.0,
                max_wall_thickness_mm=10.0,
                length_tolerance_mm=0.5,
                source="test",
            )
        )
        session.commit()
    yield
    Base.metadata.drop_all(bind=sync_engine)


# ---------------------------------------------------------------------------
# Live count helper — the source of truth used in every assertion.
# ---------------------------------------------------------------------------


async def _live_count(code: str) -> int:
    async with test_engine.connect() as conn:
        return (
            await conn.execute(
                text(
                    "SELECT COUNT(*) FROM tubes "
                    "WHERE location=:c AND status='in_stock' "
                    "AND deleted_at IS NULL"
                ),
                {"c": code},
            )
        ).scalar_one()


async def _used_count(code: str) -> int:
    async with test_engine.connect() as conn:
        return (
            await conn.execute(
                text("SELECT used_count FROM locations WHERE code=:c"),
                {"c": code},
            )
        ).scalar_one()


# ---------------------------------------------------------------------------
# Tests
# ---------------------------------------------------------------------------


def test_used_count_after_stock_in():
    """Adding 3 tubes to WH-A drives its used_count to 3."""
    async def _run():
        for i in range(3):
            await stock_in(
                type_="seamless_pipe",
                outer_diameter_mm=88.9,
                wall_thickness_mm=8.0,
                length_m=12.0,
                steel_grade="L80",
                location="WH-A",
                operator_id=1,
                pipe_id=f"SP-L80-88.9x8.00-aaaab00{i}",
                thread_type="STC", manufacturer="TEST-MFG", engine=test_engine,
            )

        assert await _live_count("WH-A") == 3
        assert await _used_count("WH-A") == 3
        assert await _used_count("WH-B") == 0

    asyncio.run(_run())


def test_used_count_after_stock_out():
    """Stock-out drops used_count but only for the affected location."""
    async def _run():
        await stock_in(
            type_="seamless_pipe",
            outer_diameter_mm=88.9, wall_thickness_mm=8.0, length_m=12.0,
            steel_grade="L80", location="WH-A", operator_id=1,
            pipe_id="SP-L80-88.9x8.00-aaaaa001", thread_type="STC", manufacturer="TEST-MFG", engine=test_engine,
        )
        await stock_in(
            type_="seamless_pipe",
            outer_diameter_mm=88.9, wall_thickness_mm=8.0, length_m=12.0,
            steel_grade="L80", location="WH-A", operator_id=1,
            pipe_id="SP-L80-88.9x8.00-aaaaa002", thread_type="STC", manufacturer="TEST-MFG", engine=test_engine,
        )
        # Two in stock
        assert await _used_count("WH-A") == 2

        await stock_out("SP-L80-88.9x8.00-aaaaa001", operator_id=1, role="admin", engine=test_engine)

        # One remains
        assert await _live_count("WH-A") == 1
        assert await _used_count("WH-A") == 1

    asyncio.run(_run())


def test_used_count_after_transfer():
    """Moving a tube decrements source, increments target — both match live."""
    async def _run():
        await stock_in(
            type_="seamless_pipe",
            outer_diameter_mm=88.9, wall_thickness_mm=8.0, length_m=12.0,
            steel_grade="L80", location="WH-A", operator_id=1,
            pipe_id="SP-L80-88.9x8.00-aaaaa011", thread_type="STC", manufacturer="TEST-MFG", engine=test_engine,
        )
        await stock_in(
            type_="seamless_pipe",
            outer_diameter_mm=88.9, wall_thickness_mm=8.0, length_m=12.0,
            steel_grade="L80", location="WH-A", operator_id=1,
            pipe_id="SP-L80-88.9x8.00-aaaaa012", thread_type="STC", manufacturer="TEST-MFG", engine=test_engine,
        )

        await transfer("SP-L80-88.9x8.00-aaaaa011", "WH-B", operator_id=1, role="admin", engine=test_engine)

        assert await _live_count("WH-A") == 1
        assert await _live_count("WH-B") == 1
        assert await _used_count("WH-A") == 1
        assert await _used_count("WH-B") == 1

    asyncio.run(_run())


def test_used_count_drift_repair():
    """A deliberately corrupted used_count is repaired by refresh_used_count."""
    async def _run():
        await stock_in(
            type_="seamless_pipe",
            outer_diameter_mm=88.9, wall_thickness_mm=8.0, length_m=12.0,
            steel_grade="L80", location="WH-A", operator_id=1,
            pipe_id="SP-L80-88.9x8.00-aaaaa021", thread_type="STC", manufacturer="TEST-MFG", engine=test_engine,
        )
        # Inject drift: claim 999 used tubes when there's only 1
        async with test_engine.begin() as conn:
            await conn.execute(
                text(
                    "UPDATE locations SET used_count=999 WHERE code='WH-A'"
                )
            )
        assert await _used_count("WH-A") == 999
        assert await _live_count("WH-A") == 1

        repaired = await refresh_used_count("WH-A", engine=test_engine)

        assert repaired == 1
        assert await _used_count("WH-A") == 1

    asyncio.run(_run())


def test_used_count_consistent_after_mixed_ops():
    """Run a mixed sequence and assert used_count == live count at the end."""
    async def _run():
        await stock_in(
            type_="seamless_pipe",
            outer_diameter_mm=88.9, wall_thickness_mm=8.0, length_m=12.0,
            steel_grade="L80", location="WH-A", operator_id=1,
            pipe_id="SP-L80-88.9x8.00-aaaaa031", thread_type="STC", manufacturer="TEST-MFG", engine=test_engine,
        )
        await stock_in(
            type_="seamless_pipe",
            outer_diameter_mm=88.9, wall_thickness_mm=8.0, length_m=12.0,
            steel_grade="L80", location="WH-A", operator_id=1,
            pipe_id="SP-L80-88.9x8.00-aaaaa032", thread_type="STC", manufacturer="TEST-MFG", engine=test_engine,
        )
        await stock_in(
            type_="seamless_pipe",
            outer_diameter_mm=88.9, wall_thickness_mm=8.0, length_m=12.0,
            steel_grade="L80", location="WH-B", operator_id=1,
            pipe_id="SP-L80-88.9x8.00-aaaaa033", thread_type="STC", manufacturer="TEST-MFG", engine=test_engine,
        )
        await transfer("SP-L80-88.9x8.00-aaaaa031", "WH-B", operator_id=1, role="admin", engine=test_engine)
        await stock_out("SP-L80-88.9x8.00-aaaaa032", operator_id=1, role="admin", engine=test_engine)
        await stock_out("SP-L80-88.9x8.00-aaaaa033", operator_id=1, role="admin", engine=test_engine)

        for code in ("WH-A", "WH-B"):
            assert await _used_count(code) == await _live_count(code)

    asyncio.run(_run())
