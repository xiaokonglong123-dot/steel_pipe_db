"""Tube routes: list, stock-in, stock-out, import (minimal slice)."""

from fastapi import APIRouter, Depends, File, Form, HTTPException, Query, Request, UploadFile
from fastapi.responses import RedirectResponse
from sqlalchemy import func, select
from sqlalchemy.ext.asyncio import AsyncSession

from src.database import get_session
from src.models import Tube, Location, User
from src.modules.warehouse.services.tube_service import stock_in, stock_out, transfer
from src.modules.warehouse.services.audit_service import (
    ComplianceError,
    ConflictError,
    ForbiddenError,
)
from src.modules.warehouse.services.import_service import (
    ImportFormatError,
    ImportRowLimitError,
    ImportTooLargeError,
    import_tubes_csv,
)
from src.schemas import BatchStockInRequest, BatchStockInResponse
from src.security import (
    generate_csrf_token,
    get_current_user,
    require_csrf,
    require_operator,
    set_csrf_cookie,
)
from src.templates_env import templates

router = APIRouter()

PAGE_SIZE = 20


@router.get("/tubes")
async def list_tubes(
    request: Request,
    session: AsyncSession = Depends(get_session),
    user=Depends(get_current_user),
    status: str | None = Query(None),
    location: str | None = Query(None),
    steel_grade: str | None = Query(None),
    quality_mark: str | None = Query(None),
    page: int = Query(1, ge=1),
):
    """List tubes with optional filters and pagination."""
    stmt = select(Tube)
    # D6: soft-deleted tubes must never appear in the listing.
    stmt = stmt.where(Tube.deleted_at.is_(None))

    if status:
        stmt = stmt.where(Tube.status == status)
    if location:
        stmt = stmt.where(Tube.location == location)
    if steel_grade:
        stmt = stmt.where(Tube.steel_grade == steel_grade)
    if quality_mark:
        stmt = stmt.where(Tube.quality_mark == quality_mark)

    # Total count
    count_stmt = select(func.count()).select_from(stmt.subquery())
    total = (await session.execute(count_stmt)).scalar_one()

    # Paginated query
    stmt = (
        stmt.order_by(Tube.id.desc())
        .limit(PAGE_SIZE)
        .offset((page - 1) * PAGE_SIZE)
    )
    rows = (await session.execute(stmt)).scalars().all()

    total_pages = max(1, (total + PAGE_SIZE - 1) // PAGE_SIZE)

    return templates.TemplateResponse(
        "warehouse/tubes_list.html",
        {
            "request": request,
            "user": user,
            "tubes": rows,
            "page": page,
            "total_pages": total_pages,
            "total": total,
            "status_filter": status,
            "location_filter": location,
            "steel_grade_filter": steel_grade,
            "quality_mark_filter": quality_mark,
        },
    )


@router.get("/tubes/stock-out/{pipe_id}")
async def stock_out_form(
    request: Request,
    pipe_id: str,
    user=Depends(get_current_user),
):
    """Render the stock-out confirmation form."""
    token = request.cookies.get("csrf_token") or generate_csrf_token()
    rendered = templates.TemplateResponse(
        "warehouse/stock_out.html",
        {
            "request": request,
            "user": user,
            "pipe_id": pipe_id,
            "csrf_token": token,
        },
    )
    set_csrf_cookie(rendered, token)
    return rendered


@router.post("/tubes/stock-out")
async def do_stock_out(
    pipe_id: str = Form(...),
    csrf_token: str = Form(...),  # noqa: ARG001
    session: AsyncSession = Depends(get_session),
    user=Depends(get_current_user),
    _role: User = Depends(require_operator),
    _: None = Depends(require_csrf),
):
    """Execute stock-out for a single tube.

    Raises 409 ConflictError or 403 ForbiddenError on failure
    (handled by app-level exception handlers).
    """
    await stock_out(pipe_id, user.id, user.role, engine=session.bind)
    return RedirectResponse(url="/tubes", status_code=303)


# ---------------------------------------------------------------------------
# Transfer routes  (§3.2 — tube_service.transfer)
# ---------------------------------------------------------------------------


@router.get("/tubes/transfer/{pipe_id}")
async def transfer_form(
    request: Request,
    pipe_id: str,
    session: AsyncSession = Depends(get_session),
    user=Depends(get_current_user),
):
    token = request.cookies.get("csrf_token") or generate_csrf_token()

    tube = (
        await session.execute(
            select(Tube).where(
                Tube.pipe_id == pipe_id,
                Tube.deleted_at.is_(None),
            )
        )
    ).scalars().first()
    if tube is None:
        raise HTTPException(status_code=404, detail="tube not found")

    locations = [
        loc
        for loc in (
            await session.execute(
                select(Location).where(Location.is_active == True)  # noqa: E712
            )
        ).scalars().all()
        if loc.code != tube.location and loc.is_active
    ]

    rendered = templates.TemplateResponse(
        "warehouse/transfer.html",
        {
            "request": request,
            "user": user,
            "tube": tube,
            "locations": locations,
            "csrf_token": token,
            "error": None,
        },
    )
    set_csrf_cookie(rendered, token)
    return rendered


@router.post("/tubes/transfer")
async def do_transfer(
    pipe_id: str = Form(...),
    target_location: str = Form(...),
    remark: str = Form(""),
    csrf_token: str = Form(...),  # noqa: ARG001
    user=Depends(get_current_user),
    _role: User = Depends(require_operator),
    _: None = Depends(require_csrf),
):
    """Raises ConflictError/ForbiddenError → JSON 409/403 via app handlers."""
    await transfer(
        pipe_id=pipe_id,
        target_location=target_location,
        operator_id=user.id,
        role=user.role,
        remark=remark or None,
    )
    return RedirectResponse(url="/tubes", status_code=303)


# ---------------------------------------------------------------------------
# Stock-in routes
# ---------------------------------------------------------------------------


@router.get("/tubes/stock-in")
async def stock_in_form(
    request: Request,
    session: AsyncSession = Depends(get_session),
    user=Depends(get_current_user),
):
    """Render the stock-in form page."""
    token = request.cookies.get("csrf_token") or generate_csrf_token()
    result = await session.execute(
        select(Location).where(Location.is_active == True)  # noqa: E712
    )
    locations = result.scalars().all()

    rendered = templates.TemplateResponse(
        "warehouse/stock_in.html",
        {
            "request": request,
            "user": user,
            "csrf_token": token,
            "locations": locations,
            "pipe_types": [("seamless_pipe", "无缝管"), ("screen_pipe", "筛管")],
            "quality_marks": [
                ("normal", "正常"),
                ("problematic", "问题管"),
                ("scrapped", "报废"),
            ],
            "error": None,
            "message": None,
        },
    )
    set_csrf_cookie(rendered, token)
    return rendered


@router.post("/tubes/stock-in")
async def do_stock_in(
    request: Request,
    pipe_id: str = Form(""),
    pipe_type: str = Form(..., alias="type"),
    outer_diameter_mm: float = Form(...),
    wall_thickness_mm: float = Form(...),
    length_m: float = Form(...),
    steel_grade: str = Form(...),
    location: str = Form(...),
    manufacturer: str = Form(...),
    thread_type: str = Form(""),
    api_5ct_cert: str = Form(""),
    batch_no: str = Form(""),
    heat_no: str = Form(""),
    heat_treat_method: str = Form(""),
    quality_mark: str = Form("normal"),
    quality_note: str = Form(""),
    csrf_token: str = Form(...),  # noqa: ARG001
    surface_treatment: str = Form(""),
    end_protector: str = Form(""),
    screen_type: str = Form(""),
    slot_width_mm: float = Form(0.0),
    slot_density: float = Form(0.0),
    filtration_rating_um: float = Form(0.0),
    base_pipe_grade: str = Form(""),
    slot_pattern: str = Form(""),
    session: AsyncSession = Depends(get_session),
    user=Depends(get_current_user),
    _role: User = Depends(require_operator),
    _: None = Depends(require_csrf),
):
    """Execute stock-in for a single tube."""
    pipe_id_val = pipe_id.strip() if pipe_id.strip() else None

    seamless_specs = None
    if pipe_type == "seamless_pipe":
        spec = {}
        if surface_treatment.strip():
            spec["surface_treatment"] = surface_treatment.strip()
        if end_protector.strip():
            spec["end_protector"] = end_protector.strip()
        seamless_specs = spec if spec else None

    screen_specs = None
    if pipe_type == "screen_pipe":
        spec = {}
        if screen_type.strip():
            spec["screen_type"] = screen_type.strip()
        if slot_width_mm > 0:
            spec["slot_width_mm"] = slot_width_mm
        if slot_density > 0:
            spec["slot_density"] = slot_density
        if filtration_rating_um > 0:
            spec["filtration_rating_um"] = filtration_rating_um
        if base_pipe_grade.strip():
            spec["base_pipe_grade"] = base_pipe_grade.strip()
        if slot_pattern.strip():
            spec["slot_pattern"] = slot_pattern.strip()
        screen_specs = spec if spec else None

    try:
        await stock_in(
            type_=pipe_type,
            outer_diameter_mm=outer_diameter_mm,
            wall_thickness_mm=wall_thickness_mm,
            length_m=length_m,
            steel_grade=steel_grade.strip(),
            location=location.strip(),
            operator_id=user.id,
            manufacturer=manufacturer.strip(),
            pipe_id=pipe_id_val,
            thread_type=thread_type.strip() if thread_type.strip() else None,
            api_5ct_cert=api_5ct_cert.strip() if api_5ct_cert.strip() else None,
            batch_no=batch_no.strip() if batch_no.strip() else None,
            heat_no=heat_no.strip() if heat_no.strip() else None,
            heat_treat_method=heat_treat_method.strip() if heat_treat_method.strip() else None,
            quality_mark=quality_mark,
            quality_note=quality_note.strip() if quality_note.strip() else None,
            seamless_specs=seamless_specs,
            screen_specs=screen_specs,
            engine=session.bind,
        )
    except ConflictError as e:
        csrf = request.cookies.get("csrf_token") or generate_csrf_token()
        locations_result = await session.execute(
            select(Location).where(Location.is_active == True)  # noqa: E712
        )
        locations = locations_result.scalars().all()
        rendered = templates.TemplateResponse(
            "warehouse/stock_in.html",
            {
                "request": request,
                "user": user,
                "csrf_token": csrf,
                "locations": locations,
                "pipe_types": [("seamless_pipe", "无缝管"), ("screen_pipe", "筛管")],
                "quality_marks": [
                    ("normal", "正常"),
                    ("problematic", "问题管"),
                    ("scrapped", "报废"),
                ],
                "error": str(e),
                "message": None,
            },
            status_code=409,
        )
        set_csrf_cookie(rendered, csrf)
        return rendered

    except ComplianceError as e:
        csrf = request.cookies.get("csrf_token") or generate_csrf_token()
        locations_result = await session.execute(
            select(Location).where(Location.is_active == True)  # noqa: E712
        )
        locations = locations_result.scalars().all()
        rendered = templates.TemplateResponse(
            "warehouse/stock_in.html",
            {
                "request": request,
                "user": user,
                "csrf_token": csrf,
                "locations": locations,
                "pipe_types": [("seamless_pipe", "无缝管"), ("screen_pipe", "筛管")],
                "quality_marks": [
                    ("normal", "正常"),
                    ("problematic", "问题管"),
                    ("scrapped", "报废"),
                ],
                "error": str(e),
                "message": None,
            },
            status_code=422,
        )
        set_csrf_cookie(rendered, csrf)
        return rendered

    except ValueError as e:
        # Malformed manual pipe_id (e.g. wrong prefix / wrong part count).
        # Re-render the form with the validation message instead of a 500.
        csrf = request.cookies.get("csrf_token") or generate_csrf_token()
        locations_result = await session.execute(
            select(Location).where(Location.is_active == True)  # noqa: E712
        )
        locations = locations_result.scalars().all()
        rendered = templates.TemplateResponse(
            "warehouse/stock_in.html",
            {
                "request": request,
                "user": user,
                "csrf_token": csrf,
                "locations": locations,
                "pipe_types": [("seamless_pipe", "无缝管"), ("screen_pipe", "筛管")],
                "quality_marks": [
                    ("normal", "正常"),
                    ("problematic", "问题管"),
                    ("scrapped", "报废"),
                ],
                "error": str(e),
                "message": None,
            },
            status_code=422,
        )
        set_csrf_cookie(rendered, csrf)
        return rendered

    return RedirectResponse(url="/tubes", status_code=303)


@router.post("/tubes/stock-in/batch")
async def do_batch_stock_in(
    body: BatchStockInRequest,
    session: AsyncSession = Depends(get_session),
    user=Depends(get_current_user),
    _role: User = Depends(require_operator),
    _: None = Depends(require_csrf),
):
    """Execute batch stock-in from JSON body. Returns partial-success summary."""
    ok_count = 0
    errors = []

    for i, entry in enumerate(body.tubes):
        try:
            await stock_in(
                type_=entry.pipe_type,
                outer_diameter_mm=entry.outer_diameter_mm,
                wall_thickness_mm=entry.wall_thickness_mm,
                length_m=entry.length_m,
                steel_grade=entry.steel_grade,
                location=entry.location,
                operator_id=user.id,
                manufacturer=entry.manufacturer,
                pipe_id=entry.pipe_id,
                thread_type=entry.thread_type,
                api_5ct_cert=entry.api_5ct_cert,
                batch_no=entry.batch_no,
                heat_no=entry.heat_no,
                heat_treat_method=entry.heat_treat_method,
                quality_mark=entry.quality_mark,
                quality_note=entry.quality_note,
                seamless_specs=entry.seamless_specs.model_dump(exclude_none=True) if entry.seamless_specs else None,
                screen_specs=entry.screen_specs.model_dump(exclude_none=True) if entry.screen_specs else None,
                engine=session.bind,
            )
            ok_count += 1
        except Exception as e:
            errors.append({"index": i, "error": str(e), "pipe_id": entry.pipe_id or ""})

    return BatchStockInResponse(ok=ok_count, errors=errors)


# ---------------------------------------------------------------------------
# CSV import routes
# ---------------------------------------------------------------------------


@router.get("/tubes/import")
async def import_form(
    request: Request,
    user=Depends(get_current_user),
):
    """Render the CSV import page."""
    token = request.cookies.get("csrf_token") or generate_csrf_token()
    rendered = templates.TemplateResponse(
        "warehouse/import.html",
        {"request": request, "user": user, "csrf_token": token},
    )
    set_csrf_cookie(rendered, token)
    return rendered


@router.post("/tubes/import")
async def do_import(
    request: Request,
    file: UploadFile = File(...),
    csrf_token: str = Form(...),  # noqa: ARG001
    session: AsyncSession = Depends(get_session),
    user=Depends(get_current_user),
    _role: User = Depends(require_operator),
    _: None = Depends(require_csrf),
):
    """Accept a CSV upload and import each compliant row."""
    content = await file.read()
    try:
        summary = await import_tubes_csv(
            content=content,
            operator_id=user.id,
            filename=file.filename or "upload.csv",
            engine=session.bind,
        )
    except ImportTooLargeError as e:
        raise HTTPException(status_code=413, detail=str(e)) from e
    except ImportFormatError as e:
        raise HTTPException(status_code=400, detail=str(e)) from e
    except ImportRowLimitError as e:
        raise HTTPException(status_code=400, detail=str(e)) from e

    csrf = request.cookies.get("csrf_token") or generate_csrf_token()
    rendered = templates.TemplateResponse(
        "warehouse/import.html",
        {
            "request": request,
            "user": user,
            "csrf_token": csrf,
            "import_result": summary.as_dict(),
        },
    )
    set_csrf_cookie(rendered, csrf)
    return rendered
