"""API 5CT standard spec service — create, update, delete with audit.

All operations use ``raw conn + begin_immediate`` and write the audit
entry inside the same transaction (project convention #13). The deleted
row's values are snapshotted into the audit detail before the hard delete
so the audit chain remains the only record of removed reference data.
"""

from __future__ import annotations

from sqlalchemy import select
from sqlalchemy.exc import IntegrityError

from src import database as db
from src.models import StandardSpec
from src.modules.warehouse.services.audit_service import (
    _write_audit,
    begin_immediate,
)


class StandardConflictError(Exception):
    """A StandardSpec with the same grade already exists."""


def _detail(grade: str, od: float, min_wt: float, max_wt: float) -> str:
    return (
        f"grade={grade} od={od} min_wt={min_wt} max_wt={max_wt}"
    )


async def create_standard(
    steel_grade: str,
    outer_diameter_mm: float,
    min_wall_thickness_mm: float,
    max_wall_thickness_mm: float,
    engine: object = None,
    *,
    actor_id: int,
) -> None:
    eng = engine or db.engine
    async with eng.connect() as conn:
        await begin_immediate(conn)
        grade = steel_grade.strip()
        try:
            await conn.execute(
                StandardSpec.__table__.insert().values(
                    grade=grade,
                    outer_diameter_mm=outer_diameter_mm,
                    min_wall_thickness_mm=min_wall_thickness_mm,
                    max_wall_thickness_mm=max_wall_thickness_mm,
                )
            )
        except IntegrityError:
            await conn.rollback()
            raise StandardConflictError(
                f"grade '{grade}' already exists"
            ) from None
        await _write_audit(
            conn, "create_standard",
            _detail(grade, outer_diameter_mm, min_wall_thickness_mm,
                    max_wall_thickness_mm),
            user_id=actor_id,
        )
        await conn.commit()


async def update_standard(
    spec_id: int,
    steel_grade: str,
    outer_diameter_mm: float,
    min_wall_thickness_mm: float,
    max_wall_thickness_mm: float,
    engine: object = None,
    *,
    actor_id: int,
) -> None:
    eng = engine or db.engine
    async with eng.connect() as conn:
        await begin_immediate(conn)
        row = (
            await conn.execute(
                select(StandardSpec.id).where(StandardSpec.id == spec_id)
            )
        ).fetchone()
        if row is None:
            await conn.rollback()
            raise ValueError(f"standard spec {spec_id} not found")
        grade = steel_grade.strip()
        try:
            await conn.execute(
                StandardSpec.__table__.update()
                .where(StandardSpec.id == spec_id)
                .values(
                    grade=grade,
                    outer_diameter_mm=outer_diameter_mm,
                    min_wall_thickness_mm=min_wall_thickness_mm,
                    max_wall_thickness_mm=max_wall_thickness_mm,
                )
            )
        except IntegrityError:
            await conn.rollback()
            raise StandardConflictError(
                f"grade '{grade}' already exists"
            ) from None
        await _write_audit(
            conn, "update_standard",
            _detail(grade, outer_diameter_mm, min_wall_thickness_mm,
                    max_wall_thickness_mm),
            user_id=actor_id,
        )
        await conn.commit()


async def delete_standard(
    spec_id: int,
    engine: object = None,
    *,
    actor_id: int,
) -> None:
    eng = engine or db.engine
    async with eng.connect() as conn:
        await begin_immediate(conn)
        row = (
            await conn.execute(
                select(
                    StandardSpec.grade,
                    StandardSpec.outer_diameter_mm,
                    StandardSpec.min_wall_thickness_mm,
                    StandardSpec.max_wall_thickness_mm,
                ).where(StandardSpec.id == spec_id)
            )
        ).fetchone()
        if row is None:
            await conn.rollback()
            raise ValueError(f"standard spec {spec_id} not found")
        await conn.execute(
            StandardSpec.__table__.delete().where(StandardSpec.id == spec_id)
        )
        await _write_audit(
            conn, "delete_standard",
            _detail(row.grade, row.outer_diameter_mm,
                    row.min_wall_thickness_mm, row.max_wall_thickness_mm),
            user_id=actor_id,
        )
        await conn.commit()
