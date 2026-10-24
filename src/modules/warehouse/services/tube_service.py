"""Business logic for tube operations — stock-in, stock-out, transfer, used-count reconciliation.

ALL state-changing operations use ``BEGIN IMMEDIATE`` serialized transactions
with TOCTOU status guards + rows_affected checks to prevent overselling under
concurrency.  Audit-hash entries are written inside the **same** transaction so
the chain tip is always consistent.

Domain exceptions (ConflictError, ForbiddenError) are designed to be caught
by the HTTP router layer and mapped to 409 / 403 responses.
"""

from __future__ import annotations

import uuid
from datetime import datetime, timezone
from typing import Optional

import anyio
from sqlalchemy import text

from rust_core import audit as rust_audit
from rust_core import compliance as rust_compliance
from src import database as db
from src.models import (
    AuditLog,
    GENESIS_HASH,
    Location,
    SeamlessPipeSpec,
    ScreenPipeSpec,
    StandardSpec,
    Transaction,
    TransactionItem,
    Tube,
)
from src.modules.warehouse.services.audit_service import (
    ComplianceError,
    ConflictError,
    ForbiddenError,
    _write_audit,
    begin_immediate,
)


# ---------------------------------------------------------------------------
# Stock-out  (出库)
# ---------------------------------------------------------------------------


async def stock_out(
    pipe_id: str,
    operator_id: int,
    role: str,
    remark: str | None = None,
    engine: object = None,
) -> dict:
    """Remove a single tube from inventory.

    Steps inside one ``BEGIN IMMEDIATE`` transaction:
      1. Guard-select the tube (must be ``in_stock``, not scrapped).
      2. Reject ``problematic`` tubes for non-admin operators.
      3. ``UPDATE … WHERE status='in_stock' …`` — assert 1 row affected.
      4. Decrement ``locations.used_count``.
      5. Write ``Transaction`` + ``TransactionItem``.
      6. Write ``AuditLog`` entry.
    """
    eng = engine or db.engine

    async with eng.connect() as conn:
        await begin_immediate(conn)
        try:
            # -- 1. Guard-read ---------------------------------------------------
            row = (
                await conn.execute(
                    text(
                        "SELECT quality_mark, location FROM tubes "
                        "WHERE pipe_id=:pid AND status='in_stock' "
                        "AND deleted_at IS NULL "
                        "AND quality_mark != 'scrapped'"
                    ),
                    {"pid": pipe_id},
                )
            ).fetchone()

            if row is None:
                await conn.rollback()
                raise ConflictError(
                    f"tube {pipe_id} not available for stock-out"
                )

            quality_mark, location = row

            # -- 2. Admin gate for problematic tubes ----------------------------
            if quality_mark == "problematic" and role != "admin":
                await conn.rollback()
                raise ForbiddenError(
                    f"problematic tube {pipe_id} requires admin"
                )

            # -- 3. State change (TOCTOU guard) ---------------------------------
            res = await conn.execute(
                text(
                    "UPDATE tubes SET status='out_stock', stock_out_time=:t "
                    "WHERE pipe_id=:pid AND status='in_stock' "
                    "AND deleted_at IS NULL "
                    "AND quality_mark != 'scrapped'"
                ),
                {"t": datetime.now(timezone.utc), "pid": pipe_id},
            )
            if res.rowcount != 1:
                await conn.rollback()
                raise ConflictError(
                    f"tube {pipe_id} stock-out lost race"
                )

            # -- 4. Decrement location counter ----------------------------------
            await conn.execute(
                text(
                    "UPDATE locations SET used_count = used_count - 1 "
                    "WHERE code=:loc AND used_count > 0"
                ),
                {"loc": location},
            )

            # -- 5. Transaction header + item -----------------------------------
            ts_txn = datetime.now(timezone.utc)
            res = await conn.execute(
                Transaction.__table__.insert().values(
                    type="out",
                    operator_id=operator_id,
                    remark=remark,
                    created_at=ts_txn,
                )
            )
            txn_id = res.inserted_primary_key[0]

            await conn.execute(
                TransactionItem.__table__.insert().values(
                    transaction_id=txn_id,
                    pipe_id=pipe_id,
                    source_location=location,
                    target_location=None,
                )
            )

            # -- 6. Audit -------------------------------------------------------
            await _write_audit(
                conn, "stock_out", f"pipe_id={pipe_id} txn={txn_id}", operator_id
            )

            await conn.commit()
            return {
                "pipe_id": pipe_id,
                "transaction_id": txn_id,
                "location": location,
            }

        except Exception:
            await conn.rollback()
            raise


async def stock_out_batch(
    pipe_ids: list[str],
    operator_id: int,
    role: str,
    remark: str | None = None,
    engine: object = None,
) -> dict:
    """Remove several tubes in ONE ``BEGIN IMMEDIATE`` transaction (all-or-nothing).

    Unlike the per-row partial-success batch flows (batch stock-in / CSV
    import), this is atomic: the first guard failure rolls the entire batch
    back, leaving inventory and used_count untouched. Use when a set of
    tubes must be outed together or not at all.

    Same TOCTOU guards as :func:`stock_out`, applied per tube. A duplicate
    pipe_id in *pipe_ids* fails on its second pass (status is now
    ``out_stock``) and aborts the whole batch. One ``Transaction`` header
    covers all tubes; each tube gets its own ``TransactionItem`` and audit
    row so per-tube audit history stays searchable.
    """
    eng = engine or db.engine
    if not pipe_ids:
        raise ConflictError("no pipe_ids supplied")

    async with eng.connect() as conn:
        await begin_immediate(conn)
        try:
            location_by_pipe: dict[str, str] = {}
            for pipe_id in pipe_ids:
                # -- 1. Guard-read -------------------------------------------
                row = (
                    await conn.execute(
                        text(
                            "SELECT quality_mark, location FROM tubes "
                            "WHERE pipe_id=:pid AND status='in_stock' "
                            "AND deleted_at IS NULL "
                            "AND quality_mark != 'scrapped'"
                        ),
                        {"pid": pipe_id},
                    )
                ).fetchone()
                if row is None:
                    raise ConflictError(
                        f"tube {pipe_id} not available for stock-out"
                    )

                quality_mark, location = row

                # -- 2. Admin gate for problematic tubes ---------------------
                if quality_mark == "problematic" and role != "admin":
                    raise ForbiddenError(
                        f"problematic tube {pipe_id} requires admin"
                    )

                # -- 3. State change (TOCTOU guard) --------------------------
                res = await conn.execute(
                    text(
                        "UPDATE tubes SET status='out_stock', stock_out_time=:t "
                        "WHERE pipe_id=:pid AND status='in_stock' "
                        "AND deleted_at IS NULL "
                        "AND quality_mark != 'scrapped'"
                    ),
                    {"t": datetime.now(timezone.utc), "pid": pipe_id},
                )
                if res.rowcount != 1:
                    raise ConflictError(f"tube {pipe_id} stock-out lost race")

                location_by_pipe[pipe_id] = location

            # -- 4. Decrement location counters ------------------------------
            for location in location_by_pipe.values():
                await conn.execute(
                    text(
                        "UPDATE locations SET used_count = used_count - 1 "
                        "WHERE code=:loc AND used_count > 0"
                    ),
                    {"loc": location},
                )

            # -- 5. Transaction header + one item per tube -------------------
            n = len(location_by_pipe)
            ts_txn = datetime.now(timezone.utc)
            res = await conn.execute(
                Transaction.__table__.insert().values(
                    type="out",
                    operator_id=operator_id,
                    remark=remark or f"batch({n})",
                    created_at=ts_txn,
                )
            )
            txn_id = res.inserted_primary_key[0]

            # -- 6. Items + audit (per tube, same txn) -----------------------
            for pipe_id, location in location_by_pipe.items():
                await conn.execute(
                    TransactionItem.__table__.insert().values(
                        transaction_id=txn_id,
                        pipe_id=pipe_id,
                        source_location=location,
                        target_location=None,
                    )
                )
                await _write_audit(
                    conn,
                    "stock_out",
                    f"pipe_id={pipe_id} txn={txn_id} batch={n}",
                    operator_id,
                )

            await conn.commit()
            return {
                "pipe_ids": list(location_by_pipe),
                "transaction_id": txn_id,
                "count": n,
            }

        except Exception:
            await conn.rollback()
            raise


# ---------------------------------------------------------------------------
# Stock-in  (入库)
# ---------------------------------------------------------------------------


def _generate_pipe_id(
    type_: str,
    steel_grade: str,
    od: float,
    wt: float,
) -> str:
    """Auto-generate a pipe_id: {SP|SCP}-{grade}-{od}x{wt}-{uuid8}.

    SP for seamless_pipe, SCP for screen_pipe.
    """
    prefix = "SP" if type_ == "seamless_pipe" else "SCP"
    uid = uuid.uuid4().hex[:8]
    return f"{prefix}-{steel_grade}-{od:.1f}x{wt:.2f}-{uid}"


def _validate_pipe_id_format(pipe_id: str, type_: str) -> None:
    """Validate pipe_id matches the expected format for the given pipe type.

    Raises ValueError on malformed IDs.
    """
    expected_prefix = "SP-" if type_ == "seamless_pipe" else "SCP-"
    if not pipe_id.startswith(expected_prefix):
        raise ValueError(
            f"pipe_id '{pipe_id}' must start with '{expected_prefix[:-1]}' "
            f"for type '{type_}'"
        )
    parts = pipe_id.split("-")
    if len(parts) != 4:
        raise ValueError(
            f"pipe_id '{pipe_id}' does not match format "
            f"{{SP|SCP}}-{{grade}}-{{od}}x{{wt}}-{{uuid8}}"
        )


async def _load_std_spec_for_grade(
    conn: object,
    grade: str,
) -> Optional[rust_compliance.StandardSpec]:
    """Read the ``standard_specs`` row for *grade* on the given async conn.

    Returns ``None`` when no row exists.  Uses raw SQL (not ORM) so it runs on
    a plain ``AsyncConnection`` without a session.
    """
    from sqlalchemy import text

    row = (
        await conn.execute(
            text(
                "SELECT grade, outer_diameter_mm, min_wall_thickness_mm, "
                "max_wall_thickness_mm, length_tolerance_mm, source "
                "FROM standard_specs WHERE grade=:g LIMIT 1"
            ),
            {"g": grade},
        )
    ).fetchone()
    if row is None:
        return None
    return rust_compliance.StandardSpec(
        grade=row[0],
        outer_diameter_mm=row[1],
        min_wall_thickness_mm=row[2],
        max_wall_thickness_mm=row[3],
        length_tolerance_mm=row[4],
        source=row[5],
    )


def _build_tube_params_for_compliance(
    type_: str,
    outer_diameter_mm: float,
    wall_thickness_mm: float,
    length_m: float,
    steel_grade: str,
    thread_type: str | None,
    screen_specs: dict | None,
) -> rust_compliance.TubeParams:
    """Construct a ``TubeParams`` for ``rust_compliance.validate_tube``."""
    screen = None
    if type_ == "screen_pipe" and screen_specs is not None:
        screen = rust_compliance.ScreenPipeParams(
            float(screen_specs.get("slot_width_mm") or 0.0),
            float(screen_specs.get("slot_density") or 0.0),
            float(screen_specs.get("filtration_rating_um") or 0.0),
            screen_specs.get("slot_pattern") or None,
        )
    return rust_compliance.TubeParams(
        type_,
        float(outer_diameter_mm),
        float(wall_thickness_mm),
        float(length_m),
        steel_grade,
        thread_type,
        screen,
    )


async def _insert_tube_with_compliance(
    conn: object,
    type_: str,
    outer_diameter_mm: float,
    wall_thickness_mm: float,
    length_m: float,
    steel_grade: str,
    location: str,
    operator_id: int,
    manufacturer: str,
    pipe_id: str | None = None,
    thread_type: str | None = None,
    api_5ct_cert: str | None = None,
    batch_no: str | None = None,
    heat_no: str | None = None,
    heat_treat_method: str | None = None,
    quality_mark: str = "normal",
    quality_note: str | None = None,
    seamless_specs: dict | None = None,
    screen_specs: dict | None = None,
    compliance_pass: bool | None = None,
) -> dict:
    # -- 0. API 5CT compliance gate --------------------------------------
    if compliance_pass is None:
        spec = await _load_std_spec_for_grade(conn, steel_grade)
        if spec is None:
            raise ComplianceError(
                f"no standard spec for grade '{steel_grade}'"
            )
        params = _build_tube_params_for_compliance(
            type_,
            outer_diameter_mm,
            wall_thickness_mm,
            length_m,
            steel_grade,
            thread_type,
            screen_specs,
        )
        result = await anyio.to_thread.run_sync(
            rust_compliance.validate_tube, params, spec
        )
        if not result.passed:
            reasons = "; ".join(
                f"{c.check_name}: {c.reason}"
                for c in result.checks
                if not c.passed and c.reason
            )
            raise ComplianceError(
                f"API 5CT compliance failed — {reasons}"
            )
        compliance_pass = True
    else:
        compliance_pass = bool(compliance_pass)

    # -- 1. Uniqueness check --------------------------------------------
    existing = (
        await conn.execute(
            text("SELECT 1 FROM tubes WHERE pipe_id=:pid AND deleted_at IS NULL"),
            {"pid": pipe_id},
        )
    ).fetchone()
    if existing is not None:
        raise ConflictError(f"pipe_id '{pipe_id}' already exists")

    # -- 2. Verify location exists and is active ------------------------
    loc = (
        await conn.execute(
            text("SELECT 1 FROM locations WHERE code=:c AND is_active=1"),
            {"c": location},
        )
    ).fetchone()
    if loc is None:
        raise ConflictError(f"location '{location}' not found or inactive")

    # -- 3. INSERT tube --------------------------------------------------
    now = datetime.now(timezone.utc)
    await conn.execute(
        Tube.__table__.insert().values(
            pipe_id=pipe_id,
            **{Tube.type_.name: type_},
            outer_diameter_mm=outer_diameter_mm,
            wall_thickness_mm=wall_thickness_mm,
            length_m=length_m,
            steel_grade=steel_grade,
            thread_type=thread_type,
            api_5ct_cert=api_5ct_cert,
            manufacturer=manufacturer,
            batch_no=batch_no,
            heat_no=heat_no,
            heat_treat_method=heat_treat_method,
            location=location,
            status="in_stock",
            quality_mark=quality_mark,
            quality_note=quality_note,
            compliance_pass=compliance_pass,
            stock_in_time=now,
            created_at=now,
            updated_at=now,
        )
    )

    # -- 4. INSERT extension specs ---------------------------------------
    if type_ == "seamless_pipe" and seamless_specs:
        await conn.execute(
            SeamlessPipeSpec.__table__.insert().values(
                pipe_id=pipe_id,
                **seamless_specs,
            )
        )
    elif type_ == "screen_pipe" and screen_specs:
        await conn.execute(
            ScreenPipeSpec.__table__.insert().values(
                pipe_id=pipe_id,
                **screen_specs,
            )
        )

    # -- 5. Increment location counter -----------------------------------
    await conn.execute(
        text("UPDATE locations SET used_count = used_count + 1 WHERE code=:c"),
        {"c": location},
    )

    # -- 6. Transaction header + item ------------------------------------
    ts_txn = datetime.now(timezone.utc)
    res = await conn.execute(
        Transaction.__table__.insert().values(
            type="in",
            operator_id=operator_id,
            remark="stock_in",
            created_at=ts_txn,
        )
    )
    txn_id = res.inserted_primary_key[0]

    await conn.execute(
        TransactionItem.__table__.insert().values(
            transaction_id=txn_id,
            pipe_id=pipe_id,
            source_location=None,
            target_location=location,
        )
    )

    # -- 7. Audit --------------------------------------------------------
    await _write_audit(
        conn,
        "stock_in",
        f"pipe_id={pipe_id} txn={txn_id} compliant={compliance_pass}",
        operator_id,
    )

    return {
        "pipe_id": pipe_id,
        "transaction_id": txn_id,
        "location": location,
        "compliance_pass": compliance_pass,
    }


async def stock_in(
    type_: str,
    outer_diameter_mm: float,
    wall_thickness_mm: float,
    length_m: float,
    steel_grade: str,
    location: str,
    operator_id: int,
    manufacturer: str,
    pipe_id: str | None = None,
    thread_type: str | None = None,
    api_5ct_cert: str | None = None,
    batch_no: str | None = None,
    heat_no: str | None = None,
    heat_treat_method: str | None = None,
    quality_mark: str = "normal",
    quality_note: str | None = None,
    seamless_specs: dict | None = None,
    screen_specs: dict | None = None,
    compliance_pass: bool | None = None,
    engine: object = None,
) -> dict:
    """Register a single tube into inventory.

    Steps inside one ``BEGIN IMMEDIATE`` transaction:
      1. Generate or validate pipe_id.
      2. Check pipe_id uniqueness.
      3. Resolve API 5CT compliance for the grade (unless caller passed
         ``compliance_pass`` explicitly):
         - Read the matching ``standard_specs`` row.
         - Run ``rust_compliance.validate_tube`` via ``anyio``.
         - On pass → persist ``compliance_pass=True``.
         - On failure / missing spec → raise ``ComplianceError`` (per
           project rule: all inbound tubes must pass the API 5CT gate,
           including manual stock-in and 盘盈 surplus cases).
      4. INSERT into tubes.
      5. INSERT into seamless_pipe_specs / screen_pipe_specs if applicable.
      6. Increment ``locations.used_count``.
      7. Write ``Transaction`` + ``TransactionItem``.
      8. Write ``AuditLog`` entry.

    ``compliance_pass`` semantics:
      - ``None``  → service auto-verifies against ``standard_specs`` (default
        for the manual stock-in router path).
      - ``True``/``False`` → caller has already verified (e.g. CSV import)
        and the value is persisted as-is; no Rust call is made.

    核心逻辑见 `_insert_tube_with_compliance`
    """
    eng = engine or db.engine

    # Auto-generate pipe_id if not provided
    if pipe_id is None:
        pipe_id = _generate_pipe_id(type_, steel_grade, outer_diameter_mm, wall_thickness_mm)
    else:
        _validate_pipe_id_format(pipe_id, type_)

    async with eng.connect() as conn:
        await begin_immediate(conn)
        try:
            result = await _insert_tube_with_compliance(
                conn,
                type_,
                outer_diameter_mm,
                wall_thickness_mm,
                length_m,
                steel_grade,
                location,
                operator_id,
                manufacturer,
                pipe_id=pipe_id,
                thread_type=thread_type,
                api_5ct_cert=api_5ct_cert,
                batch_no=batch_no,
                heat_no=heat_no,
                heat_treat_method=heat_treat_method,
                quality_mark=quality_mark,
                quality_note=quality_note,
                seamless_specs=seamless_specs,
                screen_specs=screen_specs,
                compliance_pass=compliance_pass,
            )
            await conn.commit()
            return result

        except Exception:
            await conn.rollback()
            raise


# ---------------------------------------------------------------------------
# Transfer  (调拨 / cross-location move)
# ---------------------------------------------------------------------------


async def transfer(
    pipe_id: str,
    target_location: str,
    operator_id: int,
    role: str,
    remark: str | None = None,
    engine: object = None,
) -> dict:
    """Move a tube to a different location.

    The tube stays ``in_stock``.  Source and target ``used_count`` are
    adjusted atomically inside the same ``BEGIN IMMEDIATE`` transaction.
    """
    eng = engine or db.engine

    async with eng.connect() as conn:
        await begin_immediate(conn)
        try:
            # -- 1. Guard-read ---------------------------------------------------
            row = (
                await conn.execute(
                    text(
                        "SELECT quality_mark, location FROM tubes "
                        "WHERE pipe_id=:pid AND status='in_stock' "
                        "AND deleted_at IS NULL "
                        "AND quality_mark != 'scrapped'"
                    ),
                    {"pid": pipe_id},
                )
            ).fetchone()

            if row is None:
                await conn.rollback()
                raise ConflictError(
                    f"tube {pipe_id} not available for transfer"
                )

            quality_mark, source = row

            # -- 2. Admin gate + no-op guard ------------------------------------
            if quality_mark == "problematic" and role != "admin":
                await conn.rollback()
                raise ForbiddenError(
                    f"problematic tube {pipe_id} requires admin"
                )
            if source == target_location:
                await conn.rollback()
                raise ConflictError("source and target locations are the same")

            # -- 3. State change (TOCTOU guard) ---------------------------------
            res = await conn.execute(
                text(
                    "UPDATE tubes SET location=:loc "
                    "WHERE pipe_id=:pid AND status='in_stock' "
                    "AND deleted_at IS NULL "
                    "AND quality_mark != 'scrapped'"
                ),
                {"loc": target_location, "pid": pipe_id},
            )
            if res.rowcount != 1:
                await conn.rollback()
                raise ConflictError(
                    f"tube {pipe_id} transfer lost race"
                )

            # -- 4. Adjust both location counters -------------------------------
            await conn.execute(
                text(
                    "UPDATE locations SET used_count = used_count - 1 "
                    "WHERE code=:s AND used_count > 0"
                ),
                {"s": source},
            )
            await conn.execute(
                text(
                    "UPDATE locations SET used_count = used_count + 1 "
                    "WHERE code=:t"
                ),
                {"t": target_location},
            )

            # -- 5. Transaction header + item -----------------------------------
            ts_txn = datetime.now(timezone.utc)
            res = await conn.execute(
                Transaction.__table__.insert().values(
                    type="move",
                    operator_id=operator_id,
                    remark=remark,
                    created_at=ts_txn,
                )
            )
            txn_id = res.inserted_primary_key[0]

            await conn.execute(
                TransactionItem.__table__.insert().values(
                    transaction_id=txn_id,
                    pipe_id=pipe_id,
                    source_location=source,
                    target_location=target_location,
                )
            )

            # -- 6. Audit -------------------------------------------------------
            await _write_audit(
                conn,
                "transfer",
                f"pipe_id={pipe_id} {source}->{target_location} txn={txn_id}",
                operator_id,
            )

            await conn.commit()
            return {
                "pipe_id": pipe_id,
                "from": source,
                "to": target_location,
                "transaction_id": txn_id,
            }

        except Exception:
            await conn.rollback()
            raise


# ---------------------------------------------------------------------------
# used_count reconciliation  (§12.2.1)
# ---------------------------------------------------------------------------


async def update_quality_mark(
    pipe_id: str,
    quality_mark: str,
    quality_note: str | None,
    actor_id: int,
    engine=None,
) -> dict[str, str | None]:
    """Modify the quality_mark / quality_note of an in-stock pipe."""
    from src.modules.warehouse.services.audit_service import begin_immediate, _write_audit
    eng = engine or db.engine
    async with eng.connect() as conn:
        await begin_immediate(conn)
        try:
            now = datetime.now(timezone.utc)
            r = await conn.execute(
                text(
                    "UPDATE tubes SET quality_mark=:m, quality_note=:n, updated_at=:t "
                    "WHERE pipe_id=:p AND status='in_stock' "
                    "AND deleted_at IS NULL AND quality_mark != 'scrapped'"
                ),
                {"m": quality_mark, "n": quality_note, "t": now, "p": pipe_id},
            )
            if r.rowcount != 1:
                raise ConflictError(
                    f"pipe '{pipe_id}' not in_stock / scrapped — quality update refused"
                )
            detail = f"pipe_id={pipe_id} mark={quality_mark}"
            if quality_note:
                detail += f" note={quality_note}"
            await _write_audit(conn, "quality_mark", detail, actor_id)
            await conn.commit()
            return {"pipe_id": pipe_id, "quality_mark": quality_mark}
        except Exception:
            await conn.rollback()
            raise


async def refresh_used_count(
    location_code: str,
    engine: object = None,
) -> int:
    """Recalculate and persist the accurate ``used_count`` for *location_code*.

    Returns the new count.  Uses ``BEGIN IMMEDIATE`` to prevent concurrent
    drift.
    """
    eng = engine or db.engine

    async with eng.connect() as conn:
        await begin_immediate(conn)
        try:
            actual = (
                await conn.execute(
                    text(
                        "SELECT COUNT(*) FROM tubes "
                        "WHERE location=:loc AND status='in_stock' "
                        "AND deleted_at IS NULL"
                    ),
                    {"loc": location_code},
                )
            ).scalar_one()

            await conn.execute(
                text("UPDATE locations SET used_count=:c WHERE code=:loc"),
                {"c": actual, "loc": location_code},
            )

            await conn.commit()
            return actual

        except Exception:
            await conn.rollback()
            raise
