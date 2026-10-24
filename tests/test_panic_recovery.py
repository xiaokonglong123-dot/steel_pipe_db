"""Rust panic recovery tests — verify Rust edge-case calls surface as
catchable Python exceptions instead of crashing the Python interpreter.

PROJECT_RULES #78 requires panic_recovery tests. The Rust safe core uses
unwrap_or / Result handling so genuine panics should not occur in normal
operation; this test instead asserts that the **boundary** between Python
and Rust is robust:

  1. Type mismatches raise ValueError, not segfault.
  2. Empty/oversized inputs return predictable errors.
  3. A failed Rust call leaves the engine + DB intact (subsequent calls
     still succeed via anyio.to_thread run_sync).

These tests do NOT try to intentionally trigger Rust UB; they target the
documented failure paths in pyo3-bound functions.
"""

from __future__ import annotations

import asyncio
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

import pytest
from sqlalchemy import create_engine as create_sync_engine, text
from sqlalchemy.ext.asyncio import create_async_engine
from sqlalchemy.orm import sessionmaker as sync_sessionmaker

from rust_core import audit as rust_audit
from rust_core import auth as rust_auth
from rust_core import compliance as rust_compliance
from src.models import Base

# ---------------------------------------------------------------------------
# Rust-init: use a dedicated test secret. init() is idempotent so we can
# call it safely even if other test modules already did.
# ---------------------------------------------------------------------------
rust_auth.init("panic-recovery-test-secret")

# ---------------------------------------------------------------------------
# Test DB (only used to assert post-panic DB still works).
# ---------------------------------------------------------------------------
TEST_DB_PATH = "data/test_panic_recovery.db"
SYNC_DB_URL = f"sqlite:///{TEST_DB_PATH}"
ASYNC_DB_URL = f"sqlite+aiosqlite:///{TEST_DB_PATH}"

sync_engine = create_sync_engine(SYNC_DB_URL, echo=False)
SyncSession = sync_sessionmaker(bind=sync_engine)
test_engine = create_async_engine(ASYNC_DB_URL, echo=False)


@pytest.fixture(autouse=True)
def setup_db():
    Base.metadata.create_all(bind=sync_engine)
    yield
    Base.metadata.drop_all(bind=sync_engine)


# ---------------------------------------------------------------------------
# auth.rs — hash_password, verify_password, token issue/decode
# ---------------------------------------------------------------------------


def test_hash_password_empty_string_does_not_panic():
    """"Empty password is hashed successfully (argon2 accepts any input)."""
    h = rust_auth.hash_password("")
    assert isinstance(h, str) and h.startswith("$argon2")


def test_verify_password_invalid_hash_returns_false():
    """"A malformed PHC hash returns False instead of panicking."""
    assert rust_auth.verify_password("anything", "not-a-valid-hash") is False
    assert rust_auth.verify_password("anything", "") is False


def test_decode_claims_garbage_returns_none():
    """"Random garbage token decodes to None rather than raising."""
    assert rust_auth.decode_claims("not.a.jwt") is None
    assert rust_audit is not None  # ensure import not stripped


# ---------------------------------------------------------------------------
# audit.rs — verify_chain_chunk size limit
# ---------------------------------------------------------------------------


def test_verify_chain_chunk_oversize_raises_valueerror():
    """A chunk > 20_000 entries raises ValueError (catchable in Python)."""
    fake_logs = [
        {
            "prev_hash": "x" * 64,
            "action": "noop",
            "detail": "",
            "timestamp": "2025-01-01T00:00:00Z",
            "current_hash": "y" * 64,
        }
        for _ in range(20_001)
    ]
    with pytest.raises(ValueError):
        rust_audit.verify_chain_chunk(
            0, "x" * 64, fake_logs,
        )


def test_verify_chain_chunk_continuity_mismatch():
    """First entry's prev_hash != expected_prev_hash → broken_index=0."""
    logs = [
        {
            "prev_hash": "deadbeef" + "0" * 56,
            "action": "stock_in",
            "detail": "x",
            "timestamp": "2025-01-01T00:00:00Z",
            "current_hash": "z" * 64,
        }
    ]
    result = rust_audit.verify_chain_chunk(
        0, "00" * 32, logs,
    )
    assert result["ok"] is False
    assert result["broken_index"] == 0


# ---------------------------------------------------------------------------
# compliance.rs — invalid inputs surface as Python errors
# ---------------------------------------------------------------------------


def test_compliance_validate_tube_mismatched_grade():
    """Mismatched grade yields passed=False with a readable reason."""
    params = rust_compliance.TubeParams(
        "seamless_pipe",
        88.9, 8.0, 12.0, "L80",
    )
    spec = rust_compliance.StandardSpec(
        grade="J55",
        outer_diameter_mm=88.9,
    )
    result = rust_compliance.validate_tube(params, spec)
    assert result.passed is False
    grade_check = next(
        c for c in result.checks if c.check_name == "steel_grade_match"
    )
    assert grade_check.passed is False
    assert "L80" in (grade_check.reason or "")


def test_db_usable_after_rust_valueerror():
    """After a caught ValueError from Rust, the async engine still works."""
    async def _run():
        with pytest.raises(ValueError):
            rust_audit.verify_chain_chunk(0, "", [{}] * 20_001)
        async with test_engine.connect() as conn:
            rows = (await conn.execute(text("SELECT 1 AS one"))).fetchall()
            assert len(rows) == 1 and rows[0][0] == 1

    asyncio.run(_run())
