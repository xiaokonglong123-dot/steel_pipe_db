"""Pydantic v2 request/response schemas for all domain models.

All schemas are additive — existing routes that use raw ORM models can be
retrofitted incrementally without breaking changes.
"""

from __future__ import annotations

from datetime import datetime
from typing import Optional

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
    thread_type: Optional[str] = None
    api_5ct_cert: Optional[str] = None
    manufacturer: str
    batch_no: Optional[str] = None
    heat_no: Optional[str] = None
    heat_treat_method: Optional[str] = None
    location: str
    status: str
    quality_mark: str
    quality_note: Optional[str] = None
    stock_in_time: Optional[datetime] = None
    stock_out_time: Optional[datetime] = None
    compliance_pass: Optional[bool] = None
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

    surface_treatment: Optional[str] = None
    end_protector: Optional[str] = None


class ScreenPipeSpecIn(BaseModel):
    """Optional screen pipe specification on stock-in."""

    screen_type: Optional[str] = None  # wire_wrapped / slotted / precision
    slot_width_mm: Optional[float] = None
    slot_density: Optional[float] = None
    filtration_rating_um: Optional[float] = None
    base_pipe_grade: Optional[str] = None
    slot_pattern: Optional[str] = None


class StockInRequest(BaseModel):
    """Single-tube stock-in form fields."""

    pipe_id: Optional[str] = Field(
        default=None, description="Leave empty for auto-generation"
    )
    pipe_type: str = Field(alias="type", description="seamless_pipe / screen_pipe")
    outer_diameter_mm: float = Field(gt=0)
    wall_thickness_mm: float = Field(gt=0)
    length_m: float = Field(gt=0)
    steel_grade: str = Field(min_length=1)
    location: str = Field(min_length=1)
    manufacturer: str = Field(min_length=1)
    thread_type: Optional[str] = None
    api_5ct_cert: Optional[str] = None
    batch_no: Optional[str] = None
    heat_no: Optional[str] = None
    heat_treat_method: Optional[str] = None
    quality_mark: str = Field(default="normal")
    quality_note: Optional[str] = None
    seamless_specs: Optional[SeamlessPipeSpecIn] = None
    screen_specs: Optional[ScreenPipeSpecIn] = None

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
    remark: Optional[str] = None


class TransferRequest(BaseModel):
    """Transfer request (existing)."""

    pipe_id: str = Field(min_length=1)
    target_location: str = Field(min_length=1)
    remark: Optional[str] = None


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

    display_name: Optional[str] = Field(default=None, min_length=1, max_length=128)
    role: Optional[str] = None
    password: Optional[str] = Field(default=None, min_length=6, max_length=128)

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
    user_id: Optional[int] = None
    action: str
    detail: Optional[str] = None
    current_hash: str
    created_at: datetime

    model_config = {"from_attributes": True}


class ChainVerifyResult(BaseModel):
    """Result of a hash-chain verification."""

    ok: bool
    broken: list[dict] = []
    tip: Optional[str] = None


# ---------------------------------------------------------------------------
# Location schemas
# ---------------------------------------------------------------------------


class LocationOut(BaseModel):
    """Public location representation."""

    code: str
    name: Optional[str] = None
    used_count: int
    capacity: Optional[int] = None
    is_active: bool

    model_config = {"from_attributes": True}


# ---------------------------------------------------------------------------
# Standard spec schemas
# ---------------------------------------------------------------------------


class StandardSpecOut(BaseModel):
    """API 5CT standard specification for list/inspection."""

    id: int
    grade: str
    outer_diameter_mm: Optional[float] = None
    min_wall_thickness_mm: Optional[float] = None
    max_wall_thickness_mm: Optional[float] = None
    length_tolerance_mm: Optional[float] = None
    source: Optional[str] = None

    model_config = {"from_attributes": True}


class ComplianceCheckRequest(BaseModel):
    """Request to check a tube against a standard spec."""

    spec_id: int = Field(gt=0)
    pipe_id: Optional[str] = None
    pipe_type: str = Field(alias="type")
    outer_diameter_mm: float = Field(gt=0)
    wall_thickness_mm: float = Field(gt=0)
    length_m: float = Field(gt=0)
    steel_grade: str = Field(min_length=1)
    thread_type: Optional[str] = None
    screen_specs: Optional[ScreenPipeSpecIn] = None

    @field_validator("pipe_type")
    @classmethod
    def validate_pipe_type(cls, v: str) -> str:
        allowed = {"seamless_pipe", "screen_pipe"}
        if v not in allowed:
            raise ValueError(f"pipe_type must be one of {allowed}")
        return v

    model_config = {"populate_by_name": True}