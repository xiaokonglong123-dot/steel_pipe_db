"""Audit hash-chain service — compute_hash, write_audit, begin_immediate.

All audit entries must be written inside the same ``BEGIN IMMEDIATE``
transaction as the business operation they record. Rust ``compute_hash``
is called via ``anyio.to_thread.run_sync`` to avoid blocking the async
event loop.

Shared helpers ``begin_immediate`` and ``_write_audit`` are used by
both tube_service and (future) import_service.
"""

from __future__ import annotations

from datetime import UTC, datetime

import anyio
from sqlalchemy import text

from rust_core import audit as rust_audit
from src.models import GENESIS_HASH, AuditLog

# ---------------------------------------------------------------------------
# Domain exceptions  (shared across all warehouse services)
# ---------------------------------------------------------------------------


class ConflictError(Exception):
    """The guarded status-update affected 0 rows — already moved / out / scrapped."""


class ForbiddenError(Exception):
    """A security policy was violated (e.g. problematic tube requires admin)."""


class ComplianceError(Exception):
    """API 5CT compliance check rejected the inbound tube.

    Raised by ``tube_service.stock_in`` when ``rust_compliance.validate_tube``
    fails or when no matching ``standard_specs`` row exists for the supplied
    grade.  Callers MUST run all inbound tubes — including manual single-tube
    stock-in and 盘盈 (surplus) cases — through compliance per project rule
    "All inbound tubes must pass API 5CT check".
    """


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


async def _now_iso() -> str:
    """Stable ISO-8601 timestamp string suitable for audit-hash input."""
    return datetime.now(UTC).isoformat()


async def _write_audit(
    conn: object,
    action: str,
    detail: str,
    user_id: int | None = None,
) -> None:
    """Read chain tip, compute new hash, insert audit row.

    **Must** be called inside the ``BEGIN IMMEDIATE`` transaction so that
    ``prev_hash`` is the latest committed tip.
    """
    # 1. Read current tip from DB
    row = (
        await conn.execute(
            text("SELECT current_hash FROM audit_logs ORDER BY id DESC LIMIT 1")
        )
    ).fetchone()
    prev = row[0] if row else GENESIS_HASH

    # 2. Compute hash via Rust (pure function, offload from event loop)
    ts = await _now_iso()
    current = await anyio.to_thread.run_sync(
        rust_audit.compute_hash, prev, action, detail, ts
    )

    # 3. Insert audit record
    now = datetime.now(UTC)
    await conn.execute(
        AuditLog.__table__.insert().values(
            user_id=user_id,
            action=action,
            detail=detail,
            prev_hash=prev,
            current_hash=current,
            ts=ts,
            created_at=now,
        )
    )


async def begin_immediate(conn: object) -> None:
    """Open a serialized transaction on *conn*.

    Use instead of the default deferred begin so that concurrent writers
    serialise on the first committer (others wait up to ``busy_timeout``
    then fail).
    """
    await conn.exec_driver_sql("BEGIN IMMEDIATE")