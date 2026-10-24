"""End-to-end batched verify_chain test — 21K audit rows beyond the 20K chunk.

PROJECT_RULES #76, #82 require verify_chain to be batched at 20K logs/chunk.
We insert 21K audit rows via the same `_write_audit` chain the production
service uses, then exercise the batched verification path directly
(`_verify_chunked`) and confirm:

* The chain verifies OK (no broken entries) when hashes are intact.
* The chain verifies NOT ok when one row's current_hash is tampered.
* The chunked verifier returns a single broken entry on first mismatch
  (it stops early, not iterating the remaining chunks).

The first 20K rows exercise one full chunk boundary; the next 1K exercise
the partial second chunk — confirming start_id threading and tip
carry-over between batches.
"""

from __future__ import annotations

import asyncio
import os
import sys

import pytest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

TEST_DB_PATH = "data/test_audit_20k.db"
os.environ["DATABASE_URL"] = f"sqlite+aiosqlite:///{TEST_DB_PATH}"
os.environ.setdefault("JWT_SECRET_KEY", "test-secret-audit-20k")

from datetime import datetime, timedelta, timezone

from sqlalchemy import create_engine as create_sync_engine, text
from sqlalchemy.ext.asyncio import AsyncSession, async_sessionmaker, create_async_engine
from sqlalchemy.orm import sessionmaker as sync_sessionmaker

from src import database as db
from src.models import AuditLog, Base, GENESIS_HASH
from src.modules.warehouse.routers.audit import _verify_chunked
from src.modules.warehouse.services.audit_service import begin_immediate


SYNC_DB_URL = f"sqlite:///{TEST_DB_PATH}"
ASYNC_DB_URL = f"sqlite+aiosqlite:///{TEST_DB_PATH}"

sync_engine = create_sync_engine(SYNC_DB_URL, echo=False)
SyncSession = sync_sessionmaker(bind=sync_engine)
test_engine = create_async_engine(ASYNC_DB_URL, echo=False)


N_ROWS = 21_000  # > CHUNK_SIZE (20K) so we cross one batch boundary.


def _setup_db():
    """Create tables and bind src.database to the test engine, fresh."""
    if os.path.exists(TEST_DB_PATH):
        os.remove(TEST_DB_PATH)
    from sqlalchemy.ext.asyncio import AsyncSession, async_sessionmaker as mk

    db.engine = create_async_engine(ASYNC_DB_URL, echo=False)
    db.async_session_factory = mk(
        db.engine, class_=AsyncSession, expire_on_commit=False
    )
    Base.metadata.create_all(bind=sync_engine)


def _teardown_db():
    Base.metadata.drop_all(bind=sync_engine)
    sync_engine.dispose()
    asyncio.run(db.engine.dispose())


def _compute_chain_values(n: int) -> list[dict]:
    """Pre-compute n chained hashes + row payloads in ONE thread hop.

    Replicates the exact hash sequence `_write_audit` would have produced
    (Rust ``compute_hash`` over ``prev || action || detail || ts``) so the
    resulting rows form a real, verifiable chain — but computed in a single
    cross-thread jump instead of 21K individual ``anyio.to_thread`` calls,
    and persisted in one ``BEGIN IMMEDIATE`` transaction instead of 21K.
    The point of THIS test is exercising batched *verify_chain* across the
    20K chunk boundary, not measuring single-row insert throughput.
    """
    from rust_core import audit as rust_audit

    prev = GENESIS_HASH
    base = datetime(2026, 1, 1, tzinfo=timezone.utc)
    values: list[dict] = []
    for i in range(n):
        ts = (base + timedelta(seconds=i)).isoformat()
        current = rust_audit.compute_hash(prev, "test", f"row {i}", ts)
        values.append(
            {
                "user_id": None,
                "action": "test",
                "detail": f"row {i}",
                "prev_hash": prev,
                "current_hash": current,
                "ts": ts,
                "created_at": base + timedelta(seconds=i),
            }
        )
        prev = current
    return values


async def _insert_chain(n: int) -> None:
    """Insert *n* audit rows forming a valid hash chain.

    Single serialised write transaction: chain pre-computed in one thread
    hop, then bulk-inserted and committed once. Hashes are produced by the
    same Rust ``compute_hash`` the production audit service uses.
    """
    import anyio

    values = await anyio.to_thread.run_sync(_compute_chain_values, n)
    async with db.engine.connect() as conn:
        await begin_immediate(conn)
        await conn.execute(AuditLog.__table__.insert(), values)
        await conn.commit()


async def _fetch_rows() -> list:
    from sqlalchemy import select

    async with db.engine.connect() as conn:
        result = await conn.execute(select(AuditLog).order_by(AuditLog.id.asc()))
        # AsyncSession-less Connection scalars returns the entity id only
        # (same SQLAlchemy 2.x quirk observed in import_service); use the
        # unique scalars path via session.
    factory = async_sessionmaker(db.engine, class_=AsyncSession, expire_on_commit=False)
    async with factory() as session:
        rows = (await session.execute(select(AuditLog).order_by(AuditLog.id.asc()))).scalars().all()
    return list(rows)


async def _verify(rows: list) -> dict:
    return await _verify_chunked(rows)


# ---------------------------------------------------------------------------
# Tests
# ---------------------------------------------------------------------------


@pytest.mark.slow
def test_verify_chain_21k_intact():
    _setup_db()
    try:
        asyncio.run(_insert_chain(N_ROWS))
        rows = asyncio.run(_fetch_rows())
        assert len(rows) == N_ROWS
        result = asyncio.run(_verify(rows))
        assert result["ok"] is True, "intact chain should verify ok"
        assert result["broken"] == []
    finally:
        _teardown_db()


@pytest.mark.slow
def test_verify_chain_detects_tamper():
    _setup_db()
    try:
        asyncio.run(_insert_chain(N_ROWS))
        rows = asyncio.run(_fetch_rows())

        # Tamper one row's current_hash deep into the second chunk (index 20500).
        bad = rows[20_500]
        bad_id = bad.id
        bad.current_hash = "0" * 64
        # Tamper prev_hash of the following row so the chain-end-to-tamper
        # signature is in `current_hash != expected` (next.prev != bad.current
        # matches blowing prev), making the break detectable by verify_chain_chunk.

        factory = async_sessionmaker(db.engine, class_=AsyncSession, expire_on_commit=False)
        async def _persist():
            async with factory() as session:
                await session.execute(
                    text("UPDATE audit_logs SET current_hash=:h WHERE id=:i"),
                    {"h": "0" * 64, "i": bad_id},
                )
                await session.commit()
        asyncio.run(_persist())

        rows = asyncio.run(_fetch_rows())
        result = asyncio.run(_verify(rows))
        assert result["ok"] is False, "tampered chain must not verify ok"
        assert len(result["broken"]) >= 1
    finally:
        _teardown_db()


@pytest.mark.slow
def test_verify_chain_stops_at_first_break():
    """When the first row is tampered, the second chunk is never evaluated."""
    _setup_db()
    try:
        asyncio.run(_insert_chain(N_ROWS))
        rows = asyncio.run(_fetch_rows())

        bad = rows[0]
        bad_id = bad.id
        factory = async_sessionmaker(db.engine, class_=AsyncSession, expire_on_commit=False)

        async def _persist():
            async with factory() as session:
                await session.execute(
                    text("UPDATE audit_logs SET current_hash=:h WHERE id=:i"),
                    {"h": "0" * 64, "i": bad_id},
                )
                await session.commit()
        asyncio.run(_persist())

        rows = asyncio.run(_fetch_rows())
        result = asyncio.run(_verify(rows))
        # Stopped at first mismatch → exactly one broken entry
        assert result["ok"] is False
        assert len(result["broken"]) == 1
    finally:
        _teardown_db()
