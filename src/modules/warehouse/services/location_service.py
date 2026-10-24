"""Location management service — create / rename / deactivate, audited.

Until 2026-10-24 these routes wrote through the ORM session with a bare
``session.commit()`` and recorded nothing, leaving the one warehouse write
path outside the audit chain. They now follow the same rule as every other
warehouse write (AGENTS.md rules #2 / #13): raw conn + ``begin_immediate``,
audit row written inside the same transaction.

Failure paths (duplicate code, missing code, already inactive, unchanged
name) roll back and write **no** audit row — only committed state changes
enter the chain.
"""

from __future__ import annotations

from sqlalchemy import select
from sqlalchemy.exc import IntegrityError

from src import database as db
from src.models import Location
from src.modules.warehouse.services.audit_service import (
    _write_audit,
    begin_immediate,
)


class LocationConflictError(Exception):
    """Duplicate code, or a state change that is already in effect."""


class LocationNotFoundError(Exception):
    """No location with the requested code."""


def _clean_name(name: str | None) -> str | None:
    """Normalise an optional display name: blank/whitespace → NULL."""
    return (name or "").strip() or None


async def create_location(
    code: str,
    name: str | None = None,
    engine: object = None,
    *,
    actor_id: int,
) -> None:
    """Insert a location; duplicate code raises ``LocationConflictError``."""
    eng = engine or db.engine
    code = code.strip()
    display = _clean_name(name)
    async with eng.connect() as conn:
        await begin_immediate(conn)
        try:
            await conn.execute(
                Location.__table__.insert().values(
                    code=code,
                    name=display,
                    used_count=0,
                    is_active=True,
                )
            )
        except IntegrityError:
            await conn.rollback()
            raise LocationConflictError(f"库位 '{code}' 已存在") from None
        await _write_audit(
            conn,
            "create_location",
            f"code={code} name={display}",
            user_id=actor_id,
        )
        await conn.commit()


async def update_location(
    code: str,
    name: str | None = None,
    engine: object = None,
    *,
    actor_id: int,
) -> None:
    """Rename a location. An unchanged name is a no-op (rollback, no audit)."""
    eng = engine or db.engine
    display = _clean_name(name)
    async with eng.connect() as conn:
        await begin_immediate(conn)
        row = (
            await conn.execute(
                select(Location.name).where(Location.code == code)
            )
        ).fetchone()
        if row is None:
            await conn.rollback()
            raise LocationNotFoundError(f"库位 '{code}' 不存在")
        if row.name == display:
            await conn.rollback()
            return
        await conn.execute(
            Location.__table__.update()
            .where(Location.code == code)
            .values(name=display)
        )
        await _write_audit(
            conn,
            "update_location",
            f"code={code} name={row.name}->{display}",
            user_id=actor_id,
        )
        await conn.commit()


async def deactivate_location(
    code: str,
    engine: object = None,
    *,
    actor_id: int,
) -> None:
    """Soft-disable a location. Already inactive → ``LocationConflictError``."""
    eng = engine or db.engine
    async with eng.connect() as conn:
        await begin_immediate(conn)
        row = (
            await conn.execute(
                select(Location.name, Location.is_active).where(
                    Location.code == code
                )
            )
        ).fetchone()
        if row is None:
            await conn.rollback()
            raise LocationNotFoundError(f"库位 '{code}' 不存在")
        if not row.is_active:
            await conn.rollback()
            raise LocationConflictError(f"库位 '{code}' 已停用")
        await conn.execute(
            Location.__table__.update()
            .where(Location.code == code)
            .where(Location.is_active.is_(True))
            .values(is_active=False)
        )
        await _write_audit(
            conn,
            "deactivate_location",
            f"code={code} name={row.name} is_active=true->false",
            user_id=actor_id,
        )
        await conn.commit()