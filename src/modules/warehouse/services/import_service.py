"""Batch CSV import service  —  §4.4 batch stock-in with API 5CT gate.

File-size cap (20 MB) enforced before parsing. Each row is run through
``rust_compliance.validate_tube`` first; only rows whose standard spec
exists and passes compliance proceed to ``stock_in``. Failures are
collected per-row and returned — the caller (router) renders the summary
without aborting the whole batch.

ALL Rust calls are offloaded via ``anyio.to_thread.run_sync`` to avoid
blocking the FastAPI async event loop (project rule #60).
"""

from __future__ import annotations

import io
from dataclasses import dataclass, field
from typing import Any

import anyio
import pandas as pd
from sqlalchemy import select

from rust_core import compliance as rust_compliance
from src import database as db
from src.models import StandardSpec as StdSpecRow
from src.modules.warehouse.services.tube_service import stock_in

MAX_IMPORT_BYTES = 20 * 1024 * 1024
MAX_IMPORT_ROWS = 10_000


REQUIRED_COLS = (
    "pipe_type",
    "outer_diameter_mm",
    "wall_thickness_mm",
    "length_m",
    "steel_grade",
    "location",
    "manufacturer",
)


class ImportTooLargeError(Exception):
    """Uploaded file exceeds the 20 MB cap."""


class ImportRowLimitError(Exception):
    """CSV has more data rows than MAX_IMPORT_ROWS per import."""


class ImportFormatError(Exception):
    """CSV is missing required columns or cannot be parsed."""


@dataclass
class ImportResult:
    ok: int = 0
    errors: list[dict] = field(default_factory=list)

    def as_dict(self) -> dict:
        return {"ok": self.ok, "errors": self.errors}


# ---------------------------------------------------------------------------


# StandardSpec rows cached per (engine, grade) for the duration of one import.
async def _load_std_specs(engine: object) -> dict[str, rust_compliance.StandardSpec]:
    """Return {grade: rust_compliance.StandardSpec} from DB.

    Uses an ``AsyncSession`` rather than ``engine.connect()`` so that
    ``select(StdSpecRow)`` is fully ORM-mapped (raw ``Connection.execute``
    on an entity query returns ``Row`` tuples whose ``.scalars()`` only
    yields the first column — i.e. the entity's id, not the entity itself).
    """
    from sqlalchemy.ext.asyncio import AsyncSession, async_sessionmaker

    factory = async_sessionmaker(engine, class_=AsyncSession, expire_on_commit=False)
    out: dict[str, rust_compliance.StandardSpec] = {}
    async with factory() as session:
        rows = (await session.execute(select(StdSpecRow))).scalars().all()
        for r in rows:
            out[r.grade] = rust_compliance.StandardSpec(
                grade=r.grade,
                outer_diameter_mm=r.outer_diameter_mm,
                min_wall_thickness_mm=r.min_wall_thickness_mm,
                max_wall_thickness_mm=r.max_wall_thickness_mm,
                length_tolerance_mm=r.length_tolerance_mm,
                source=r.source,
            )
    return out


def _build_tube_params(row: dict) -> tuple[rust_compliance.TubeParams, rust_compliance.ScreenPipeParams | None]:
    """Construct TubeParams (and optional ScreenPipeParams) from a CSV row."""
    screen = None
    if row["pipe_type"] == "screen_pipe":
        screen = rust_compliance.ScreenPipeParams(
            float(row.get("slot_width_mm") or 0.0),
            float(row.get("slot_density") or 0.0),
            float(row.get("filtration_rating_um") or 0.0),
            (row.get("slot_pattern") or None) or None,
        )

    params = rust_compliance.TubeParams(
        row["pipe_type"],
        float(row["outer_diameter_mm"]),
        float(row["wall_thickness_mm"]),
        float(row["length_m"]),
        row["steel_grade"],
        (row.get("thread_type") or None) or None,
        screen,
    )
    return params, screen


async def _check_row_compliance(
    row: dict,
    std_specs: dict[str, rust_compliance.StandardSpec],
) -> str | None:
    """Return None if row passes compliance, else a human-readable reason."""
    grade = row["steel_grade"]
    spec = std_specs.get(grade)
    if spec is None:
        return f"standard spec for grade '{grade}' not found"

    params, _ = _build_tube_params(row)
    result = await anyio.to_thread.run_sync(
        rust_compliance.validate_tube, params, spec
    )
    if result.passed:
        return None
    reasons = "; ".join(
        f"{c.check_name}: {c.reason}" for c in result.checks if not c.passed
    )
    return f"compliance failed — {reasons}"


def _row_to_seamless_specs(row: dict) -> dict | None:
    if row["pipe_type"] != "seamless_pipe":
        return None
    spec = {}
    if row.get("surface_treatment"):
        spec["surface_treatment"] = row["surface_treatment"]
    if row.get("end_protector"):
        spec["end_protector"] = row["end_protector"]
    return spec or None


def _row_to_screen_specs(row: dict) -> dict | None:
    if row["pipe_type"] != "screen_pipe":
        return None
    spec = {}
    if row.get("screen_type"):
        spec["screen_type"] = row["screen_type"]
    if row.get("slot_width_mm") and float(row["slot_width_mm"]) > 0:
        spec["slot_width_mm"] = float(row["slot_width_mm"])
    if row.get("slot_density") and float(row["slot_density"]) > 0:
        spec["slot_density"] = float(row["slot_density"])
    if row.get("filtration_rating_um") and float(row["filtration_rating_um"]) > 0:
        spec["filtration_rating_um"] = float(row["filtration_rating_um"])
    if row.get("base_pipe_grade"):
        spec["base_pipe_grade"] = row["base_pipe_grade"]
    if row.get("slot_pattern"):
        spec["slot_pattern"] = row["slot_pattern"]
    return spec or None


async def import_tubes_csv(
    content: bytes,
    operator_id: int,
    filename: str = "upload.csv",
    engine: object | None = None,
) -> ImportResult:
    """Parse CSV bytes and import each compliant row as a new tube.

    Returns an :class:`ImportResult` summarising the per-row outcome.
    Failure on one row does not abort the whole batch — the remaining
    rows are still processed.
    """
    eng = engine or db.engine

    if len(content) > MAX_IMPORT_BYTES:
        raise ImportTooLargeError(
            f"file size {len(content)} bytes exceeds {MAX_IMPORT_BYTES} bytes"
        )

    try:
        df = pd.read_csv(io.BytesIO(content), dtype=str)
    except Exception as exc:
        raise ImportFormatError(f"CSV parse failed: {exc}") from exc

    missing = [c for c in REQUIRED_COLS if c not in df.columns]
    if missing:
        raise ImportFormatError(f"missing required columns: {missing}")

    if len(df) > MAX_IMPORT_ROWS:
        raise ImportRowLimitError(
            f"row count {len(df)} exceeds the limit of "
            f"{MAX_IMPORT_ROWS} rows per import — please split the file"
        )

    # Normalise to plain dicts and strip whitespace on string fields.
    rows: list[dict[str, Any]] = []
    for _, raw in df.iterrows():
        row: dict[str, Any] = {}
        for col in df.columns:
            val = raw[col]
            if pd.isna(val):
                row[col] = None
            elif isinstance(val, str):
                row[col] = val.strip()
            else:
                row[col] = val
        rows.append(row)

    std_specs = await _load_std_specs(eng)
    result = ImportResult()

    for idx, row in enumerate(rows, start=2):  # CSV row 1 is header
        # Required-field presence / non-empty
        empty = [c for c in REQUIRED_COLS if not row.get(c)]
        if empty:
            result.errors.append(
                {"row": idx, "error": f"missing required fields: {empty}",
                 "pipe_id": row.get("pipe_id") or ""}
            )
            continue

        if row["pipe_type"] not in ("seamless_pipe", "screen_pipe"):
            result.errors.append(
                {"row": idx, "error": f"invalid pipe_type '{row['pipe_type']}'",
                 "pipe_id": row.get("pipe_id") or ""}
            )
            continue

        # -- API 5CT compliance gate (盘盈入库也必须通过) --------------------
        reason = await _check_row_compliance(row, std_specs)
        if reason is not None:
            result.errors.append(
                {"row": idx, "error": reason,
                 "pipe_id": row.get("pipe_id") or ""}
            )
            continue

        # -- stock_in --------------------------------------------------------
        pipe_id_val = row.get("pipe_id") or None
        try:
            await stock_in(
                type_=row["pipe_type"],
                outer_diameter_mm=float(row["outer_diameter_mm"]),
                wall_thickness_mm=float(row["wall_thickness_mm"]),
                length_m=float(row["length_m"]),
                steel_grade=row["steel_grade"],
                location=row["location"],
                operator_id=operator_id,
                manufacturer=row["manufacturer"],
                pipe_id=pipe_id_val,
                thread_type=row.get("thread_type") or None,
                api_5ct_cert=row.get("api_5ct_cert") or None,
                batch_no=row.get("batch_no") or None,
                heat_no=row.get("heat_no") or None,
                heat_treat_method=row.get("heat_treat_method") or None,
                quality_mark=row.get("quality_mark") or "normal",
                quality_note=row.get("quality_note") or None,
                seamless_specs=_row_to_seamless_specs(row),
                screen_specs=_row_to_screen_specs(row),
                # import_service already validated via _check_row_compliance above;
                # pass True so stock_in skips the (now-redundant) Rust call and
                # persists compliance_pass=True on the row.
                compliance_pass=True,
                engine=eng,
            )
            result.ok += 1
        except Exception as exc:  # noqa: BLE001
            result.errors.append(
                {"row": idx, "error": str(exc),
                 "pipe_id": pipe_id_val or ""}
            )

    return result
