"""SQLAlchemy ORM models — single file (split at ~300 lines).

All 9 tables from the design doc §4.1–§4.8 are defined here with
SQLAlchemy 2.0 typed-mapping style (Mapped / mapped_column).
"""

import hashlib
from datetime import UTC, datetime

from sqlalchemy import Boolean, DateTime, Float, ForeignKey, Integer, String, Text, UniqueConstraint
from sqlalchemy.orm import DeclarativeBase, Mapped, mapped_column

# ---------------------------------------------------------------------------
# Base & constants
# ---------------------------------------------------------------------------

GENESIS_HASH: str = hashlib.sha256(b"AYANAMI_REI_GENESIS").hexdigest()


class Base(DeclarativeBase):
    pass


# ---------------------------------------------------------------------------
# tubes  —  pipe base table (§4.1)
# ---------------------------------------------------------------------------

class Tube(Base):
    __tablename__ = "tubes"

    id: Mapped[int] = mapped_column(Integer, primary_key=True, autoincrement=True)
    pipe_id: Mapped[str] = mapped_column(String, unique=True, nullable=False)
    type_: Mapped[str] = mapped_column("type", String(32), nullable=False)  # seamless_pipe / screen_pipe
    outer_diameter_mm: Mapped[float] = mapped_column(Float, nullable=False)
    wall_thickness_mm: Mapped[float] = mapped_column(Float, nullable=False)
    length_m: Mapped[float] = mapped_column(Float, nullable=False)
    steel_grade: Mapped[str] = mapped_column(String(32), nullable=False)
    thread_type: Mapped[str | None] = mapped_column(String(32), nullable=True)
    api_5ct_cert: Mapped[str | None] = mapped_column(String, nullable=True)
    manufacturer: Mapped[str] = mapped_column(String, nullable=False)
    batch_no: Mapped[str | None] = mapped_column(String, nullable=True)
    heat_no: Mapped[str | None] = mapped_column(String, nullable=True)
    heat_treat_method: Mapped[str | None] = mapped_column(String, nullable=True)
    location: Mapped[str] = mapped_column(
        String(32),
        ForeignKey("locations.code", ondelete="RESTRICT"),
        nullable=False,
    )
    status: Mapped[str] = mapped_column(String(16), nullable=False)  # in_stock / out_stock / in_transfer
    quality_mark: Mapped[str] = mapped_column(
        String(16), default="normal", nullable=False
    )  # normal / problematic / scrapped
    quality_note: Mapped[str | None] = mapped_column(Text, nullable=True)
    stock_in_time: Mapped[datetime | None] = mapped_column(DateTime, nullable=True)
    stock_out_time: Mapped[datetime | None] = mapped_column(DateTime, nullable=True)
    compliance_pass: Mapped[bool | None] = mapped_column(Boolean, nullable=True)
    deleted_at: Mapped[datetime | None] = mapped_column(DateTime, nullable=True)
    created_at: Mapped[datetime] = mapped_column(
        DateTime, default=lambda: datetime.now(UTC), nullable=False
    )
    updated_at: Mapped[datetime] = mapped_column(
        DateTime,
        default=lambda: datetime.now(UTC),
        onupdate=lambda: datetime.now(UTC),
        nullable=False,
    )


# ---------------------------------------------------------------------------
# seamless_pipe_specs  —  1:1 extension table (§4.2)
# ---------------------------------------------------------------------------

class SeamlessPipeSpec(Base):
    __tablename__ = "seamless_pipe_specs"

    pipe_id: Mapped[str] = mapped_column(
        String,
        ForeignKey("tubes.pipe_id", ondelete="CASCADE"),
        primary_key=True,
    )
    surface_treatment: Mapped[str | None] = mapped_column(String, nullable=True)
    end_protector: Mapped[str | None] = mapped_column(String, nullable=True)


# ---------------------------------------------------------------------------
# screen_pipe_specs  —  1:1 extension table (§4.2)
# ---------------------------------------------------------------------------

class ScreenPipeSpec(Base):
    __tablename__ = "screen_pipe_specs"

    pipe_id: Mapped[str] = mapped_column(
        String,
        ForeignKey("tubes.pipe_id", ondelete="CASCADE"),
        primary_key=True,
    )
    screen_type: Mapped[str | None] = mapped_column(String, nullable=True)  # wire_wrapped / slotted / precision
    slot_width_mm: Mapped[float | None] = mapped_column(Float, nullable=True)
    slot_density: Mapped[float | None] = mapped_column(Float, nullable=True)
    filtration_rating_um: Mapped[float | None] = mapped_column(Float, nullable=True)
    base_pipe_grade: Mapped[str | None] = mapped_column(String, nullable=True)
    slot_pattern: Mapped[str | None] = mapped_column(String, nullable=True)


# ---------------------------------------------------------------------------
# users  —  authentication & role management (§4.3)
# ---------------------------------------------------------------------------

class User(Base):
    __tablename__ = "users"

    id: Mapped[int] = mapped_column(Integer, primary_key=True, autoincrement=True)
    username: Mapped[str] = mapped_column(String, unique=True, nullable=False)
    password_hash: Mapped[str] = mapped_column(String, nullable=False)
    display_name: Mapped[str] = mapped_column(String, nullable=False)
    role: Mapped[str] = mapped_column(String(16), nullable=False)  # admin / operator / viewer
    token_version: Mapped[int] = mapped_column(Integer, default=0, nullable=False)
    last_refresh_jti: Mapped[str | None] = mapped_column(String, nullable=True)
    active: Mapped[bool] = mapped_column(Boolean, default=True, nullable=False)
    created_at: Mapped[datetime] = mapped_column(
        DateTime, default=lambda: datetime.now(UTC), nullable=False
    )


# ---------------------------------------------------------------------------
# transactions  —  stock-in/out/move/check header (§4.4)
# ---------------------------------------------------------------------------

class Transaction(Base):
    __tablename__ = "transactions"

    id: Mapped[int] = mapped_column(Integer, primary_key=True, autoincrement=True)
    type_: Mapped[str] = mapped_column("type", String(16), nullable=False)  # in / out / move / check
    operator_id: Mapped[int] = mapped_column(Integer, nullable=False)
    reference_no: Mapped[str | None] = mapped_column(String, nullable=True)
    remark: Mapped[str | None] = mapped_column(Text, nullable=True)
    created_at: Mapped[datetime] = mapped_column(
        DateTime, default=lambda: datetime.now(UTC), nullable=False
    )


# ---------------------------------------------------------------------------
# transaction_items  —  line items for each transaction (§4.5)
# ---------------------------------------------------------------------------

class TransactionItem(Base):
    __tablename__ = "transaction_items"

    id: Mapped[int] = mapped_column(Integer, primary_key=True, autoincrement=True)
    transaction_id: Mapped[int] = mapped_column(
        Integer,
        ForeignKey("transactions.id", ondelete="CASCADE"),
        nullable=False,
    )
    pipe_id: Mapped[str] = mapped_column(String, nullable=False)
    source_location: Mapped[str | None] = mapped_column(String, nullable=True)
    target_location: Mapped[str | None] = mapped_column(String, nullable=True)


# ---------------------------------------------------------------------------
# locations  —  storage locations (§4.6)
# ---------------------------------------------------------------------------

class Location(Base):
    __tablename__ = "locations"

    code: Mapped[str] = mapped_column(String(32), primary_key=True)
    name: Mapped[str | None] = mapped_column(String, nullable=True)
    used_count: Mapped[int] = mapped_column(Integer, default=0, nullable=False)
    capacity: Mapped[int | None] = mapped_column(Integer, nullable=True)
    is_active: Mapped[bool] = mapped_column(Boolean, default=True, nullable=False)


# ---------------------------------------------------------------------------
# audit_logs  —  tamper-evident hash chain (§4.7)
# ---------------------------------------------------------------------------

class AuditLog(Base):
    __tablename__ = "audit_logs"

    id: Mapped[int] = mapped_column(Integer, primary_key=True, autoincrement=True)
    user_id: Mapped[int | None] = mapped_column(Integer, nullable=True)
    action: Mapped[str] = mapped_column(String, nullable=False)
    detail: Mapped[str | None] = mapped_column(Text, nullable=True)
    prev_hash: Mapped[str] = mapped_column(String, nullable=False)
    current_hash: Mapped[str] = mapped_column(String, nullable=False)
    ts: Mapped[str] = mapped_column("ts", String, nullable=False)
    created_at: Mapped[datetime] = mapped_column(
        DateTime, default=lambda: datetime.now(UTC), nullable=False
    )


# ---------------------------------------------------------------------------
# standard_specs  —  API 5CT compliance reference data (§4.8)
# ---------------------------------------------------------------------------

class StandardSpec(Base):
    __tablename__ = "standard_specs"
    __table_args__ = (
        UniqueConstraint("grade", name="uq_standard_specs_grade"),
    )

    id: Mapped[int] = mapped_column(Integer, primary_key=True, autoincrement=True)
    grade: Mapped[str] = mapped_column(String, nullable=False)
    outer_diameter_mm: Mapped[float | None] = mapped_column(Float, nullable=True)
    min_wall_thickness_mm: Mapped[float | None] = mapped_column(Float, nullable=True)
    max_wall_thickness_mm: Mapped[float | None] = mapped_column(Float, nullable=True)
    length_tolerance_mm: Mapped[float | None] = mapped_column(Float, nullable=True)
    source: Mapped[str | None] = mapped_column(String, nullable=True)


# ---------------------------------------------------------------------------
# check_sessions / check_items  —  inventory check (盘点, D11)
# ---------------------------------------------------------------------------


class CheckSession(Base):
    __tablename__ = "check_sessions"

    id: Mapped[int] = mapped_column(Integer, primary_key=True, autoincrement=True)
    location: Mapped[str] = mapped_column(String(32), nullable=False)
    status: Mapped[str] = mapped_column(String(16), nullable=False, default="open")  # open/confirmed/cancelled
    created_by: Mapped[int] = mapped_column(Integer, nullable=False)
    confirmed_by: Mapped[int | None] = mapped_column(Integer, nullable=True)
    created_at: Mapped[datetime] = mapped_column(
        DateTime, default=lambda: datetime.now(UTC), nullable=False
    )
    confirmed_at: Mapped[datetime | None] = mapped_column(DateTime, nullable=True)
    remark: Mapped[str | None] = mapped_column(Text, nullable=True)


class CheckItem(Base):
    __tablename__ = "check_items"

    id: Mapped[int] = mapped_column(Integer, primary_key=True, autoincrement=True)
    check_id: Mapped[int] = mapped_column(
        Integer,
        ForeignKey("check_sessions.id", ondelete="CASCADE"),
        nullable=False,
        index=True,
    )
    pipe_id: Mapped[str] = mapped_column(String, nullable=False)
    result: Mapped[str] = mapped_column(String(16), nullable=False)  # ok/missing/moved/surplus
    actual_location: Mapped[str | None] = mapped_column(String(32), nullable=True)
    surplus_data: Mapped[str | None] = mapped_column(Text, nullable=True)
    created_at: Mapped[datetime] = mapped_column(
        DateTime, default=lambda: datetime.now(UTC), nullable=False
    )
