"""Pydantic v2 request/response schemas for all domain models.

All schemas are additive — existing routes that use raw ORM models can be
retrofitted incrementally without breaking changes.
"""

from __future__ import annotations

from datetime import datetime

from pydantic import BaseModel, Field, field_validator

# ---------------------------------------------------------------------------
# Tube schemas
# ---------------------------------------------------------------------------


class TubeOut(BaseModel):
    """Public tube representation for list views / API responses."""

    id: int
    pipe_id: str
    type_: str = Field(alias="type")
    outer_diameter_mm: float
    wall_thickness_mm: float
    length_m: float
    steel_grade: str
    thread_type: str | None = None
    api_5ct_cert: str | None = None
    manufacturer: str
    batch_no: str | None = None
    heat_no: str | None = None
    heat_treat_method: str | None = None
    location: str
    status: str
    quality_mark: str
    quality_note: str | None = None
    stock_in_time: datetime | None = None
    stock_out_time: datetime | None = None
    compliance_pass: bool | None = None
    created_at: datetime
    updated_at: datetime

    model_config = {"from_attributes": True, "populate_by_name": True}


class TubeListResponse(BaseModel):
    """Paginated tube list response."""

    tubes: list[TubeOut]
    page: int
    total_pages: int
    total: int


class SeamlessPipeSpecIn(BaseModel):
    """Optional seamless pipe specification on stock-in."""

    surface_treatment: str | None = None
    end_protector: str | None = None


class ScreenPipeSpecIn(BaseModel):
    """Optional screen pipe specification on stock-in."""

    screen_type: str | None = None  # wire_wrapped / slotted / precision
    slot_width_mm: float | None = None
    slot_density: float | None = None
    filtration_rating_um: float | None = None
    base_pipe_grade: str | None = None
    slot_pattern: str | None = None


class StockInRequest(BaseModel):
    """Single-tube stock-in form fields."""

    pipe_id: str | None = Field(
        default=None, description="Leave empty for auto-generation"
    )
    pipe_type: str = Field(alias="type", description="seamless_pipe / screen_pipe")
    outer_diameter_mm: float = Field(gt=0)
    wall_thickness_mm: float = Field(gt=0)
    length_m: float = Field(gt=0)
    steel_grade: str = Field(min_length=1)
    location: str = Field(min_length=1)
    manufacturer: str = Field(min_length=1)
    thread_type: str | None = None
    api_5ct_cert: str | None = None
    batch_no: str | None = None
    heat_no: str | None = None
    heat_treat_method: str | None = None
    quality_mark: str = Field(default="normal")
    quality_note: str | None = None
    seamless_specs: SeamlessPipeSpecIn | None = None
    screen_specs: ScreenPipeSpecIn | None = None

    @field_validator("pipe_type")
    @classmethod
    def validate_pipe_type(cls, v: str) -> str:
        allowed = {"seamless_pipe", "screen_pipe"}
        if v not in allowed:
            raise ValueError(f"pipe_type must be one of {allowed}")
        return v

    @field_validator("quality_mark")
    @classmethod
    def validate_quality_mark(cls, v: str) -> str:
        allowed = {"normal", "problematic", "scrapped"}
        if v not in allowed:
            raise ValueError(f"quality_mark must be one of {allowed}")
        return v

    model_config = {"populate_by_name": True}


class BatchStockInRequest(BaseModel):
    """Batch stock-in request body."""

    tubes: list[StockInRequest] = Field(min_length=1, max_length=500)


class BatchStockInResponse(BaseModel):
    """Batch stock-in result summary."""

    ok: int
    errors: list[dict] = []


class StockOutRequest(BaseModel):
    """Stock-out request (existing)."""

    pipe_id: str = Field(min_length=1)
    remark: str | None = None


class TransferRequest(BaseModel):
    """Transfer request (existing)."""

    pipe_id: str = Field(min_length=1)
    target_location: str = Field(min_length=1)
    remark: str | None = None


# ---------------------------------------------------------------------------
# User schemas
# ---------------------------------------------------------------------------


class UserOut(BaseModel):
    """Public user representation (never exposes password_hash)."""

    id: int
    username: str
    display_name: str
    role: str
    active: bool
    created_at: datetime

    model_config = {"from_attributes": True}


class UserCreate(BaseModel):
    """Create a new user. Password is required."""

    username: str = Field(min_length=1, max_length=64)
    password: str = Field(min_length=6, max_length=128)
    display_name: str = Field(min_length=1, max_length=128)
    role: str = Field(default="viewer")

    @field_validator("role")
    @classmethod
    def validate_role(cls, v: str) -> str:
        allowed = {"admin", "operator", "viewer"}
        if v not in allowed:
            raise ValueError(f"role must be one of {allowed}")
        return v


class UserUpdate(BaseModel):
    """Update an existing user. Password is optional."""

    display_name: str | None = Field(default=None, min_length=1, max_length=128)
    role: str | None = None
    password: str | None = Field(default=None, min_length=6, max_length=128)

    @field_validator("role")
    @classmethod
    def validate_role(cls, v: str | None) -> str | None:
        if v is None:
            return v
        allowed = {"admin", "operator", "viewer"}
        if v not in allowed:
            raise ValueError(f"role must be one of {allowed}")
        return v


# ---------------------------------------------------------------------------
# Audit schemas
# ---------------------------------------------------------------------------


class AuditLogOut(BaseModel):
    """Public audit log entry for list views."""

    id: int
    user_id: int | None = None
    action: str
    detail: str | None = None
    current_hash: str
    created_at: datetime

    model_config = {"from_attributes": True}


class ChainVerifyResult(BaseModel):
    """Result of a hash-chain verification."""

    ok: bool
    broken: list[dict] = []
    tip: str | None = None


# ---------------------------------------------------------------------------
# Location schemas
# ---------------------------------------------------------------------------


class LocationOut(BaseModel):
    """Public location representation."""

    code: str
    name: str | None = None
    used_count: int
    capacity: int | None = None
    is_active: bool

    model_config = {"from_attributes": True}


# ---------------------------------------------------------------------------
# Standard spec schemas
# ---------------------------------------------------------------------------


class StandardSpecOut(BaseModel):
    """API 5CT standard specification for list/inspection."""

    id: int
    grade: str
    outer_diameter_mm: float | None = None
    min_wall_thickness_mm: float | None = None
    max_wall_thickness_mm: float | None = None
    length_tolerance_mm: float | None = None
    source: str | None = None

    model_config = {"from_attributes": True}


class ComplianceCheckRequest(BaseModel):
    """Request to check a tube against a standard spec."""

    spec_id: int = Field(gt=0)
    pipe_id: str | None = None
    pipe_type: str = Field(alias="type")
    outer_diameter_mm: float = Field(gt=0)
    wall_thickness_mm: float = Field(gt=0)
    length_m: float = Field(gt=0)
    steel_grade: str = Field(min_length=1)
    thread_type: str | None = None
    screen_specs: ScreenPipeSpecIn | None = None

    @field_validator("pipe_type")
    @classmethod
    def validate_pipe_type(cls, v: str) -> str:
        allowed = {"seamless_pipe", "screen_pipe"}
        if v not in allowed:
            raise ValueError(f"pipe_type must be one of {allowed}")
        return v

    model_config = {"populate_by_name": True}