"""Inventory check (盘点) service — D11.

All functions use raw conn + ``begin_immediate`` (same pattern as
user_service). Check sessions are DB-persisted; ``confirm_check`` applies
all differences atomically in ONE ``BEGIN IMMEDIATE`` transaction
(spec §4.3) — any failure rolls the whole txn back and keeps the session
``open`` (zero partial application, zero audit on failure).
"""

from __future__ import annotations

import json
from datetime import UTC, datetime

from sqlalchemy import text

from src import database as db
from src.models import CheckItem, CheckSession, Transaction, TransactionItem
from src.modules.warehouse.services.audit_service import (
    ConflictError,
    _write_audit,
    begin_immediate,
)
from src.modules.warehouse.services.tube_service import (
    _generate_pipe_id,
    _insert_tube_with_compliance,
)


class CheckConflictError(ConflictError):
    """409 for check-flow state conflicts (non-open session, bad marks)."""


CHECK_STATUSES = ("open", "confirmed", "cancelled")
CHECK_RESULTS = ("ok", "missing", "moved", "surplus")


def _now() -> datetime:
    return datetime.now(UTC)


async def _load_session(conn, check_id: int) -> dict:
    row = (
        await conn.execute(
            text("SELECT * FROM check_sessions WHERE id=:i"), {"i": check_id}
        )
    ).fetchone()
    if row is None:
        raise CheckConflictError(f"check session {check_id} not found")
    return dict(row._mapping)


async def _require_open(conn, check_id: int) -> dict:
    sess = await _load_session(conn, check_id)
    if sess["status"] != "open":
        raise CheckConflictError(
            f"check session {check_id} is {sess['status']} (not open)"
        )
    return sess


async def create_check(
    location: str,
    operator_id: int,
    remark: str | None = None,
    engine: object = None,
) -> dict:
    eng = engine or db.engine
    async with eng.connect() as conn:
        await begin_immediate(conn)
        loc = (
            await conn.execute(
                text("SELECT 1 FROM locations WHERE code=:c AND is_active=1"),
                {"c": location},
            )
        ).fetchone()
        if loc is None:
            await conn.rollback()
            raise CheckConflictError(f"location '{location}' not found or inactive")
        res = await conn.execute(
            CheckSession.__table__.insert().values(
                location=location,
                status="open",
                created_by=operator_id,
                remark=remark,
                created_at=_now(),
            )
        )
        check_id = res.inserted_primary_key[0]
        await conn.commit()
        return {"id": check_id, "location": location}


async def get_check(check_id: int, engine: object = None) -> dict:
    eng = engine or db.engine
    async with eng.connect() as conn:
        return await _load_session(conn, check_id)


async def list_checks(engine: object = None) -> list[dict]:
    eng = engine or db.engine
    async with eng.connect() as conn:
        rows = (
            await conn.execute(
                text("SELECT * FROM check_sessions ORDER BY id DESC")
            )
        ).all()
        return [dict(r._mapping) for r in rows]


async def get_check_detail(check_id: int, engine: object = None) -> dict:
    eng = engine or db.engine
    async with eng.connect() as conn:
        sess = await _load_session(conn, check_id)
        items = (
            await conn.execute(
                text(
                    "SELECT * FROM check_items WHERE check_id=:c ORDER BY id"
                ),
                {"c": check_id},
            )
        ).all()
        tubes = (
            await conn.execute(
                text(
                    "SELECT pipe_id, steel_grade, outer_diameter_mm, "
                    "wall_thickness_mm, length_m, quality_mark, location "
                    "FROM tubes WHERE location=:l AND status='in_stock' "
                    "AND deleted_at IS NULL ORDER BY pipe_id"
                ),
                {"l": sess["location"]},
            )
        ).all()
        locations = (
            await conn.execute(
                text(
                    "SELECT code FROM locations WHERE is_active=1 ORDER BY code"
                )
            )
        ).all()
        return {
            "session": sess,
            "items": [dict(r._mapping) for r in items],
            "tubes": [dict(r._mapping) for r in tubes],
            "locations": [r._mapping["code"] for r in locations],
        }


async def mark_item(
    check_id: int,
    pipe_id: str,
    result: str,
    actual_location: str | None = None,
    actor_id: int = 0,
    engine: object = None,
) -> dict:
    if result not in CHECK_RESULTS:
        raise CheckConflictError(f"invalid result '{result}'")
    eng = engine or db.engine
    async with eng.connect() as conn:
        await begin_immediate(conn)
        sess = await _require_open(conn, check_id)
        tube = (
            await conn.execute(
                text("SELECT * FROM tubes WHERE pipe_id=:p AND deleted_at IS NULL"),
                {"p": pipe_id},
            )
        ).fetchone()
        if tube is None:
            await conn.rollback()
            raise CheckConflictError(f"pipe '{pipe_id}' not found")
        tm = dict(tube._mapping)
        if tm["location"] != sess["location"] or tm["status"] != "in_stock":
            await conn.rollback()
            raise CheckConflictError(
                f"pipe '{pipe_id}' not in_stock at '{sess['location']}'"
            )
        act_loc = actual_location.strip() if actual_location else None
        if result == "moved":
            if not act_loc:
                raise ValueError("actual_location required for moved")
            if act_loc == sess["location"]:
                await conn.rollback()
                raise CheckConflictError(
                    "actual_location must differ from session location"
                )
            loc = (
                await conn.execute(
                    text("SELECT 1 FROM locations WHERE code=:c AND is_active=1"),
                    {"c": act_loc},
                )
            ).fetchone()
            if loc is None:
                await conn.rollback()
                raise CheckConflictError(
                    f"location '{act_loc}' not found or inactive"
                )
        existing = (
            await conn.execute(
                text(
                    "SELECT id FROM check_items "
                    "WHERE check_id=:c AND pipe_id=:p"
                ),
                {"c": check_id, "p": pipe_id},
            )
        ).fetchone()
        if existing is not None:
            await conn.execute(
                CheckItem.__table__.update()
                .where(CheckItem.__table__.c.id == existing[0])
                .values(
                    result=result,
                    actual_location=act_loc,
                    surplus_data=None,
                )
            )
        else:
            await conn.execute(
                CheckItem.__table__.insert().values(
                    check_id=check_id,
                    pipe_id=pipe_id,
                    result=result,
                    actual_location=act_loc,
                    created_at=_now(),
                )
            )
        await conn.commit()
        return {"pipe_id": pipe_id, "result": result}


async def add_surplus(
    check_id: int,
    surplus_data: dict,
    actor_id: int = 0,
    engine: object = None,
) -> dict:
    eng = engine or db.engine
    async with eng.connect() as conn:
        await begin_immediate(conn)
        await _require_open(conn, check_id)
        pipe_type = surplus_data["type"]
        pipe_id = _generate_pipe_id(
            pipe_type,
            surplus_data["steel_grade"],
            surplus_data["outer_diameter_mm"],
            surplus_data["wall_thickness_mm"],
        )
        existing = (
            await conn.execute(
                text(
                    "SELECT 1 FROM tubes "
                    "WHERE pipe_id=:p AND deleted_at IS NULL"
                ),
                {"p": pipe_id},
            )
        ).fetchone()
        if existing is not None:
            await conn.rollback()
            raise CheckConflictError(f"pipe_id '{pipe_id}' already exists")
        res = await conn.execute(
            CheckItem.__table__.insert().values(
                check_id=check_id,
                pipe_id=pipe_id,
                result="surplus",
                surplus_data=json.dumps(surplus_data, ensure_ascii=False),
                created_at=_now(),
            )
        )
        await conn.commit()
        return {"pipe_id": pipe_id, "item_id": res.inserted_primary_key[0]}


async def delete_surplus_item(
    check_id: int,
    item_id: int,
    actor_id: int = 0,
    engine: object = None,
) -> None:
    eng = engine or db.engine
    async with eng.connect() as conn:
        await begin_immediate(conn)
        await _require_open(conn, check_id)
        item = (
            await conn.execute(
                text(
                    "SELECT * FROM check_items WHERE id=:i AND check_id=:c"
                ),
                {"i": item_id, "c": check_id},
            )
        ).fetchone()
        if item is None:
            await conn.rollback()
            raise CheckConflictError(f"check item {item_id} not found")
        if dict(item._mapping)["result"] != "surplus":
            await conn.rollback()
            raise CheckConflictError("only surplus items can be deleted")
        await conn.execute(
            text("DELETE FROM check_items WHERE id=:i"), {"i": item_id}
        )
        await conn.commit()


async def cancel_check(
    check_id: int,
    actor_id: int = 0,
    engine: object = None,
) -> None:
    eng = engine or db.engine
    async with eng.connect() as conn:
        await begin_immediate(conn)
        res = await conn.execute(
            text(
                "UPDATE check_sessions SET status='cancelled' "
                "WHERE id=:i AND status='open'"
            ),
            {"i": check_id},
        )
        if res.rowcount != 1:
            await conn.rollback()
            raise CheckConflictError(
                f"check session {check_id} cannot be cancelled (not open)"
            )
        await conn.commit()


async def confirm_check(
    check_id: int,
    actor_id: int,
    engine: object = None,
) -> dict:
    """Atomically apply all differences of an open check session.

    Every item, transaction, audit row, and session transition is committed
    together. Any conflict or compliance failure rolls back the entire
    transaction and leaves the session open for correction.
    """
    eng = engine or db.engine
    async with eng.connect() as conn:
        await begin_immediate(conn)
        try:
            sess = await _require_open(conn, check_id)
            location = sess["location"]
            rows = (
                await conn.execute(
                    text("SELECT * FROM check_items WHERE check_id=:c ORDER BY id"),
                    {"c": check_id},
                )
            ).all()
            items = [dict(row._mapping) for row in rows]
            missing_n = surplus_n = moved_n = 0
            now = _now()

            for item in items:
                result = item["result"]
                if result == "ok":
                    continue
                if result == "missing":
                    missing_n += 1
                    result_update = await conn.execute(
                        text(
                            "UPDATE tubes SET status='missing', updated_at=:n "
                            "WHERE pipe_id=:p AND location=:l AND status='in_stock' "
                            "AND deleted_at IS NULL AND quality_mark != 'scrapped'"
                        ),
                        {"p": item["pipe_id"], "l": location, "n": now},
                    )
                    if result_update.rowcount != 1:
                        raise CheckConflictError(
                            f"pipe '{item['pipe_id']}' not in_stock at "
                            f"'{location}' — re-check"
                        )
                    await conn.execute(
                        text(
                            "UPDATE locations SET used_count=used_count-1 "
                            "WHERE code=:l AND used_count>0"
                        ),
                        {"l": location},
                    )
                elif result == "moved":
                    moved_n += 1
                    target = item["actual_location"]
                    target_location = (
                        await conn.execute(
                            text(
                                "SELECT 1 FROM locations "
                                "WHERE code=:c AND is_active=1"
                            ),
                            {"c": target},
                        )
                    ).fetchone()
                    if target_location is None:
                        raise CheckConflictError(
                            f"location '{target}' not found or inactive"
                        )
                    result_update = await conn.execute(
                        text(
                            "UPDATE tubes SET location=:t, updated_at=:n "
                            "WHERE pipe_id=:p AND location=:l AND status='in_stock' "
                            "AND deleted_at IS NULL AND quality_mark != 'scrapped'"
                        ),
                        {
                            "p": item["pipe_id"],
                            "l": location,
                            "t": target,
                            "n": now,
                        },
                    )
                    if result_update.rowcount != 1:
                        raise CheckConflictError(
                            f"pipe '{item['pipe_id']}' not in_stock at "
                            f"'{location}' — re-check"
                        )
                    await conn.execute(
                        text(
                            "UPDATE locations SET used_count=used_count-1 "
                            "WHERE code=:l AND used_count>0"
                        ),
                        {"l": location},
                    )
                    await conn.execute(
                        text("UPDATE locations SET used_count=used_count+1 WHERE code=:t"),
                        {"t": target},
                    )
                elif result == "surplus":
                    surplus_n += 1
                    data = json.loads(item["surplus_data"])
                    await _insert_tube_with_compliance(
                        conn,
                        type_=data["type"],
                        outer_diameter_mm=data["outer_diameter_mm"],
                        wall_thickness_mm=data["wall_thickness_mm"],
                        length_m=data["length_m"],
                        steel_grade=data["steel_grade"],
                        location=location,
                        operator_id=actor_id,
                        manufacturer=data["manufacturer"],
                        pipe_id=item["pipe_id"],
                        thread_type=data.get("thread_type"),
                        api_5ct_cert=data.get("api_5ct_cert"),
                        batch_no=data.get("batch_no"),
                        heat_no=data.get("heat_no"),
                        heat_treat_method=data.get("heat_treat_method"),
                        quality_mark=data.get("quality_mark", "normal"),
                        quality_note=data.get("quality_note"),
                        seamless_specs=data.get("seamless_specs"),
                        screen_specs=data.get("screen_specs"),
                        compliance_pass=None,
                    )

            transaction_result = await conn.execute(
                Transaction.__table__.insert().values(
                    type="check",
                    operator_id=actor_id,
                    remark=f"check {check_id}",
                    created_at=now,
                )
            )
            transaction_id = transaction_result.inserted_primary_key[0]
            for item in items:
                result = item["result"]
                if result == "ok":
                    continue
                source = location if result in ("missing", "moved") else None
                target = (
                    item["actual_location"]
                    if result == "moved"
                    else location if result == "surplus" else None
                )
                await conn.execute(
                    TransactionItem.__table__.insert().values(
                        transaction_id=transaction_id,
                        pipe_id=item["pipe_id"],
                        source_location=source,
                        target_location=target,
                    )
                )

            await _write_audit(
                conn,
                "check",
                f"check_id={check_id} location={location} "
                f"missing={missing_n} surplus={surplus_n} moved={moved_n}",
                actor_id,
            )
            await conn.execute(
                text(
                    "UPDATE check_sessions SET status='confirmed', "
                    "confirmed_by=:a, confirmed_at=:t WHERE id=:i"
                ),
                {"a": actor_id, "t": now, "i": check_id},
            )
            await conn.commit()
            return {
                "check_id": check_id,
                "missing": missing_n,
                "surplus": surplus_n,
                "moved": moved_n,
            }
        except Exception:
            await conn.rollback()
            raise


async def recover_missing(
    pipe_id: str,
    actor_id: int,
    engine: object = None,
) -> dict:
    """Restore a missing tube to inventory atomically."""
    eng = engine or db.engine
    async with eng.connect() as conn:
        await begin_immediate(conn)
        try:
            tube = (
                await conn.execute(
                    text("SELECT * FROM tubes WHERE pipe_id=:p AND deleted_at IS NULL"),
                    {"p": pipe_id},
                )
            ).fetchone()
            if tube is None:
                raise CheckConflictError(f"pipe '{pipe_id}' not found")
            tube_data = dict(tube._mapping)
            if tube_data["status"] != "missing":
                raise CheckConflictError(
                    f"pipe '{pipe_id}' is not missing "
                    f"(status={tube_data['status']})"
                )
            now = _now()
            result_update = await conn.execute(
                text(
                    "UPDATE tubes SET status='in_stock', updated_at=:n "
                    "WHERE pipe_id=:p AND status='missing' AND deleted_at IS NULL"
                ),
                {"p": pipe_id, "n": now},
            )
            if result_update.rowcount != 1:
                raise CheckConflictError(f"pipe '{pipe_id}' recovery failed")
            location = tube_data["location"]
            await conn.execute(
                text("UPDATE locations SET used_count=used_count+1 WHERE code=:l"),
                {"l": location},
            )
            transaction_result = await conn.execute(
                Transaction.__table__.insert().values(
                    type="check",
                    operator_id=actor_id,
                    remark="recover",
                    created_at=now,
                )
            )
            transaction_id = transaction_result.inserted_primary_key[0]
            await conn.execute(
                TransactionItem.__table__.insert().values(
                    transaction_id=transaction_id,
                    pipe_id=pipe_id,
                    source_location=location,
                    target_location=None,
                )
            )
            await _write_audit(
                conn,
                "recover",
                f"pipe_id={pipe_id} location={location}",
                actor_id,
            )
            await conn.commit()
            return {"pipe_id": pipe_id, "location": location}
        except Exception:
            await conn.rollback()
            raise
