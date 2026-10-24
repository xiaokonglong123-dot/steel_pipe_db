"""Tube routes: list, stock-in, stock-out, import (minimal slice)."""

from urllib.parse import urlencode

from fastapi import APIRouter, Depends, File, Form, HTTPException, Query, Request, UploadFile
from fastapi.responses import RedirectResponse
from sqlalchemy import func, select, text
from sqlalchemy.ext.asyncio import AsyncSession

from src.database import get_session
from src.models import (
    AuditLog,
    Location,
    ScreenPipeSpec,
    SeamlessPipeSpec,
    Transaction,
    TransactionItem,
    Tube,
    User,
)
from src.modules.warehouse.services.audit_service import (
    ComplianceError,
    ConflictError,
)
from src.modules.warehouse.services.check_service import (
    CheckConflictError,
    add_surplus,
    cancel_check,
    confirm_check,
    create_check,
    delete_surplus_item,
    get_check_detail,
    list_checks,
    mark_item,
    recover_missing,
)
from src.modules.warehouse.services.import_service import (
    ImportFormatError,
    ImportRowLimitError,
    ImportTooLargeError,
    import_tubes_csv,
)
from src.modules.warehouse.services.location_service import (
    LocationConflictError,
    LocationNotFoundError,
    create_location,
    deactivate_location,
    update_location,
)
from src.modules.warehouse.services.tube_service import (
    stock_in,
    stock_out,
    stock_out_batch,
    transfer,
    update_quality_mark,
)
from src.schemas import BatchStockInRequest, BatchStockInResponse
from src.security import (
    generate_csrf_token,
    get_current_user,
    require_admin,
    require_csrf,
    require_operator,
    set_csrf_cookie,
)
from src.templates_env import templates

router = APIRouter()

PAGE_SIZE = 20


async def _render_tube_list(
    request: Request,
    session: AsyncSession,
    user: User,
    *,
    page: int = 1,
    status: str | None = None,
    location: str | None = None,
    steel_grade: str | None = None,
    quality_mark: str | None = None,
    batch_no: str | None = None,
    heat_no: str | None = None,
    error: str | None = None,
    status_code: int = 200,
):
    """Render the tube list with filters, candidate values and pagination.

    Reused by the GET route and by error paths of other routes that used to
    render ``warehouse/tubes_list.html`` with a partial context.
    """
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
    if batch_no:
        stmt = stmt.where(Tube.batch_no == batch_no)
    if heat_no:
        stmt = stmt.where(Tube.heat_no == heat_no)

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

    # Filter candidate values: the form offers pickers instead of free text.
    async def _distinct(column) -> list[str]:
        values_stmt = (
            select(column)
            .where(Tube.deleted_at.is_(None), column.is_not(None))
            .distinct()
            .order_by(column)
            .limit(200)
        )
        return list((await session.execute(values_stmt)).scalars().all())

    location_rows = (
        await session.execute(
            select(Location.code, Location.name)
            .where(Location.is_active.is_(True))
            .order_by(Location.code)
            .limit(200)
        )
    ).all()

    filter_values = {
        "status": status,
        "location": location,
        "steel_grade": steel_grade,
        "quality_mark": quality_mark,
        "batch_no": batch_no,
        "heat_no": heat_no,
    }
    # Pagination must carry every active filter, not just the four original ones.
    filter_query = urlencode({k: v for k, v in filter_values.items() if v})
    active_filters = {k: v for k, v in filter_values.items() if v}
    # One-click remove for each active filter chip.
    remove_links = {
        k: "/tubes"
        + (
            "?" + urlencode({f: v for f, v in active_filters.items() if f != k})
            if len(active_filters) > 1
            else ""
        )
        for k in active_filters
    }

    token = request.cookies.get("csrf_token") or generate_csrf_token()
    rendered = templates.TemplateResponse(
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
            "batch_no_filter": batch_no,
            "heat_no_filter": heat_no,
            "filter_query": filter_query,
            "filter_values": active_filters,
            "remove_links": remove_links,
            "locations": location_rows,
            "steel_grades": await _distinct(Tube.steel_grade),
            "batch_nos": await _distinct(Tube.batch_no),
            "heat_nos": await _distinct(Tube.heat_no),
            "csrf_token": token,
            "error": error,
        },
        status_code=status_code,
    )
    set_csrf_cookie(rendered, token)
    return rendered


@router.get("/tubes")
async def list_tubes(
    request: Request,
    session: AsyncSession = Depends(get_session),
    user=Depends(get_current_user),
    status: str | None = Query(None),
    location: str | None = Query(None),
    steel_grade: str | None = Query(None),
    quality_mark: str | None = Query(None),
    batch_no: str | None = Query(None),
    heat_no: str | None = Query(None),
    page: int = Query(1, ge=1),
):
    """List tubes with optional filters and pagination."""
    return await _render_tube_list(
        request,
        session,
        user,
        page=page,
        status=status,
        location=location,
        steel_grade=steel_grade,
        quality_mark=quality_mark,
        batch_no=batch_no,
        heat_no=heat_no,
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
    csrf_token: str = Form(...),
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


# D13 — batch stock-out endpoint (atomic: all pipes or none)
@router.post("/tubes/stock-out/batch", response_model=None)
async def do_batch_stock_out(
    request: Request,
    session: AsyncSession = Depends(get_session),
    user=Depends(get_current_user),
    _role=Depends(require_operator),
    _csrf: None = Depends(require_csrf),
):
    """Stock out every selected tube in one atomic transaction.

    Any guard failure (not in_stock / scrapped / problematic without admin)
    rolls the whole batch back and surfaces as 409/403 via the app-level
    exception handlers — same contract as the single stock-out route.
    Submitting with no selection is a no-op redirect.
    """
    form = await request.form()
    pipe_ids = [pid.strip() for pid in form.getlist("pipe_ids") if pid.strip()]
    if not pipe_ids:
        return RedirectResponse(url="/tubes", status_code=303)

    await stock_out_batch(
        pipe_ids, operator_id=user.id, role=user.role, engine=session.bind
    )
    return RedirectResponse(url="/tubes", status_code=303)


# ---------------------------------------------------------------------------


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
    csrf_token: str = Form(...),
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
    csrf_token: str = Form(...),
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
        # Per-row partial success is the documented contract for this endpoint
        # (AGENTS.md: batch stock-in collects per-row errors, unlike the atomic
        # batch *stock-out* route).
        except Exception as e:  # noqa: BLE001
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
    csrf_token: str = Form(...),
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


# ---------------------------------------------------------------------------
# Inventory check (盘点, D11)
# ---------------------------------------------------------------------------


async def _active_location_codes(session: AsyncSession = None) -> list[str]:
    from src import database as db_mod
    if session is None:
        async with db_mod.async_session_factory() as s:
            rows = (await s.execute(text("SELECT code FROM locations WHERE is_active=1 ORDER BY code"))).scalars().all()
            return list(rows)
    rows = (await session.execute(text("SELECT code FROM locations WHERE is_active=1 ORDER BY code"))).scalars().all()
    return list(rows)


def _render_check_template(request, template_name, context, status_code=200):
    token = request.cookies.get("csrf_token") or generate_csrf_token()
    rendered = templates.TemplateResponse(
        request,
        template_name,
        {**context, "csrf_token": token},
        status_code=status_code,
    )
    set_csrf_cookie(rendered, token)
    return rendered


async def _render_check_list_error(
    request,
    user,
    error,
    status_code=409,
    form=None,
):
    context = {
        "request": request,
        "user": user,
        "checks": await list_checks(),
        "locations": await _active_location_codes(),
        "error": error,
    }
    if form is not None:
        context["form"] = form
    return _render_check_template(request, "warehouse/check_list.html", context, status_code)


async def _render_check_detail_error(
    request,
    check_id,
    user,
    error,
    status_code=409,
):
    try:
        detail = await get_check_detail(check_id)
    except CheckConflictError:
        return await _render_check_list_error(request, user, error, status_code)
    return _render_check_template(
        request,
        "warehouse/check_detail.html",
        {"request": request, "detail": detail, "user": user, "error": error},
        status_code,
    )


async def _render_check_report_error(
    request,
    check_id,
    user,
    error,
    status_code=409,
):
    try:
        detail = await get_check_detail(check_id)
    except CheckConflictError:
        return await _render_check_list_error(request, user, error, status_code)
    return _render_check_template(
        request,
        "warehouse/check_report.html",
        {
            "request": request,
            "check": detail,
            "detail": detail,
            "counts": _report_counts(detail),
            "user": user,
            "error": error,
        },
        status_code,
    )


def _report_counts(detail: dict) -> dict:
    counts = {"ok": 0, "missing": 0, "moved": 0, "surplus": 0}
    for item in detail["items"]:
        counts[item["result"]] = counts.get(item["result"], 0) + 1
    return counts


@router.get("/tubes/check", dependencies=[Depends(require_operator)])
async def check_list(
    request: Request,
    session: AsyncSession = Depends(get_session),
    _: None = Depends(require_operator),
):
    checks = await list_checks()
    locations = (
        await session.execute(
            text("SELECT code FROM locations WHERE is_active=1 ORDER BY code")
        )
    ).scalars().all()
    return _render_check_template(
        request,
        "warehouse/check_list.html",
        {"request": request, "user": _, "checks": checks, "locations": locations},
    )


@router.post("/tubes/check", dependencies=[Depends(require_csrf), Depends(require_operator)])
async def check_create(
    request: Request,
    location: str = Form(...),
    remark: str = Form(""),
    csrf_token: str = Form(...),
    user: User = Depends(get_current_user),
):
    try:
        created = await create_check(location=location, operator_id=user.id, remark=remark or None)
    except CheckConflictError as e:
        return await _render_check_list_error(
            request,
            user,
            str(e),
            form={"location": location, "remark": remark},
        )
    return RedirectResponse(url=f"/tubes/check/{created['id']}", status_code=303)


@router.get("/tubes/check/{check_id}", dependencies=[Depends(require_operator)])
async def check_detail(
    request: Request,
    check_id: int,
    user: User = Depends(get_current_user),
):
    try:
        detail = await get_check_detail(check_id)
    except CheckConflictError as e:
        return await _render_check_list_error(
            request,
            user,
            str(e),
            status_code=200,
        )
    return _render_check_template(
        request,
        "warehouse/check_detail.html",
        {"request": request, "detail": detail, "user": user},
    )


@router.post("/tubes/check/{check_id}/items", dependencies=[Depends(require_csrf), Depends(require_operator)])
async def check_mark_item(
    request: Request,
    check_id: int,
    pipe_id: str = Form(...),
    result: str = Form(...),
    actual_location: str = Form(""),
    csrf_token: str = Form(...),
    user: User = Depends(get_current_user),
):
    try:
        await mark_item(
            check_id=check_id,
            pipe_id=pipe_id,
            result=result,
            actual_location=actual_location or None,
            actor_id=user.id,
        )
    except ValueError as e:
        return await _render_check_detail_error(
            request, check_id, user, str(e), status_code=422
        )
    except CheckConflictError as e:
        return await _render_check_detail_error(
            request, check_id, user, str(e)
        )
    return RedirectResponse(url=f"/tubes/check/{check_id}", status_code=303)


@router.post("/tubes/check/{check_id}/surplus", dependencies=[Depends(require_csrf), Depends(require_operator)])
async def check_add_surplus(
    request: Request,
    check_id: int,
    type: str = Form(..., alias="type"),
    outer_diameter_mm: float = Form(...),
    wall_thickness_mm: float = Form(...),
    length_m: float = Form(...),
    steel_grade: str = Form(...),
    manufacturer: str = Form(...),
    thread_type: str = Form(""),
    api_5ct_cert: str = Form(""),
    batch_no: str = Form(""),
    heat_no: str = Form(""),
    heat_treat_method: str = Form(""),
    quality_mark: str = Form("normal"),
    quality_note: str = Form(""),
    csrf_token: str = Form(...),
    user: User = Depends(get_current_user),
):
    surplus_data = {
        "type": type,
        "outer_diameter_mm": outer_diameter_mm,
        "wall_thickness_mm": wall_thickness_mm,
        "length_m": length_m,
        "steel_grade": steel_grade,
        "manufacturer": manufacturer,
        "thread_type": thread_type or None,
        "api_5ct_cert": api_5ct_cert or None,
        "batch_no": batch_no or None,
        "heat_no": heat_no or None,
        "heat_treat_method": heat_treat_method or None,
        "quality_mark": quality_mark,
        "quality_note": quality_note or None,
    }
    try:
        await add_surplus(check_id=check_id, surplus_data=surplus_data, actor_id=user.id)
    except (ValueError, KeyError) as e:
        return await _render_check_detail_error(
            request,
            check_id,
            user,
            f"盘盈信息不完整: {e}",
            status_code=422,
        )
    except CheckConflictError as e:
        return await _render_check_detail_error(
            request, check_id, user, str(e)
        )
    return RedirectResponse(url=f"/tubes/check/{check_id}", status_code=303)


@router.post("/tubes/check/{check_id}/surplus/{item_id}/delete", dependencies=[Depends(require_csrf), Depends(require_operator)])
async def check_delete_surplus(
    request: Request,
    check_id: int,
    item_id: int,
    csrf_token: str = Form(...),
    user: User = Depends(get_current_user),
):
    try:
        await delete_surplus_item(check_id=check_id, item_id=item_id, actor_id=user.id)
    except CheckConflictError as e:
        return await _render_check_detail_error(
            request, check_id, user, str(e)
        )
    return RedirectResponse(url=f"/tubes/check/{check_id}", status_code=303)


@router.get("/tubes/check/{check_id}/report", dependencies=[Depends(require_operator)])
async def check_report(
    request: Request,
    check_id: int,
    user: User = Depends(get_current_user),
):
    try:
        detail = await get_check_detail(check_id)
    except CheckConflictError as e:
        return await _render_check_list_error(
            request,
            user,
            str(e),
            status_code=200,
        )
    return _render_check_template(
        request,
        "warehouse/check_report.html",
        {
            "request": request,
            "detail": detail,
            "counts": _report_counts(detail),
            "user": user,
        },
    )


@router.post("/tubes/check/{check_id}/confirm", dependencies=[Depends(require_csrf), Depends(require_admin)])
async def check_confirm(
    request: Request,
    check_id: int,
    csrf_token: str = Form(...),
    user: User = Depends(require_admin),
):
    try:
        await confirm_check(check_id=check_id, actor_id=user.id)
    except (ComplianceError, ValueError) as e:
        return await _render_check_report_error(
            request, check_id, user, str(e), status_code=422
        )
    except CheckConflictError as e:
        return await _render_check_report_error(
            request, check_id, user, str(e), status_code=409
        )
    return RedirectResponse(url=f"/tubes/check/{check_id}/report", status_code=303)


@router.post("/tubes/recover/{pipe_id}", dependencies=[Depends(require_csrf), Depends(require_operator)])
async def tube_recover(
    request: Request,
    pipe_id: str,
    csrf_token: str = Form(...),
    session: AsyncSession = Depends(get_session),
    user: User = Depends(get_current_user),
):
    try:
        await recover_missing(pipe_id=pipe_id, actor_id=user.id)
    except CheckConflictError as e:
        return await _render_tube_list(
            request, session, user, error=str(e), status_code=409
        )
    return RedirectResponse(url="/tubes", status_code=303)


@router.post("/tubes/check/{check_id}/cancel", dependencies=[Depends(require_csrf), Depends(require_operator)])
async def check_cancel(
    request: Request,
    check_id: int,
    csrf_token: str = Form(...),
    user: User = Depends(get_current_user),
):
    try:
        await cancel_check(check_id=check_id, actor_id=user.id)
    except CheckConflictError as e:
        return await _render_check_detail_error(
            request, check_id, user, str(e)
        )
    return RedirectResponse(url="/tubes/check", status_code=303)


# ---------------------------------------------------------------------------
# Quality mark
# ---------------------------------------------------------------------------


@router.get("/tubes/quality", dependencies=[Depends(require_operator)])
async def quality_form(
    request: Request,
    pipe_id: str = Query(None),
    user: User = Depends(get_current_user),
    session: AsyncSession = Depends(get_session),
):
    tube = None
    error = ""
    if pipe_id:
        result = await session.execute(
            text(
                "SELECT * FROM tubes WHERE pipe_id=:p "
                "AND status='in_stock' AND deleted_at IS NULL "
                "AND quality_mark != 'scrapped'"
            ),
            {"p": pipe_id},
        )
        row = result.fetchone()
        if row is None:
            error = f"pipe '{pipe_id}' not found (in_stock / not scrapped)"
        else:
            tube = dict(row._mapping)
    return _render_check_template(
        request,
        "warehouse/quality.html",
        {
            "request": request,
            "user": user,
            "tube": tube,
            "pipe_id": pipe_id,
            "error": error,
        },
    )


@router.post(
    "/tubes/quality",
    dependencies=[Depends(require_csrf), Depends(require_operator)],
)
async def quality_update(
    request: Request,
    pipe_id: str = Form(...),
    quality_mark: str = Form(...),
    quality_note: str = Form(""),
    csrf_token: str = Form(...),
    user: User = Depends(get_current_user),
    session: AsyncSession = Depends(get_session),
):
    try:
        await update_quality_mark(
            pipe_id=pipe_id,
            quality_mark=quality_mark,
            quality_note=quality_note.strip() if quality_note.strip() else None,
            actor_id=user.id,
        )
    except ConflictError as e:
        token = request.cookies.get("csrf_token") or generate_csrf_token()
        result = await session.execute(
            text("SELECT * FROM tubes WHERE pipe_id=:p AND status='in_stock' AND deleted_at IS NULL"),
            {"p": pipe_id},
        )
        tube = result.fetchone()
        rendered = templates.TemplateResponse(
            request,
            "warehouse/quality.html",
            {
                "request": request,
                "user": user,
                "csrf_token": token,
                "tube": dict(tube._mapping) if tube else None,
                "pipe_id": pipe_id,
                "error": str(e),
            },
            status_code=409,
        )
        set_csrf_cookie(rendered, token)
        return rendered
    return RedirectResponse(url=f"/tubes/quality?pipe_id={pipe_id}", status_code=303)


async def _render_locations(
    request: Request,
    session: AsyncSession,
    user: User,
    *,
    error: str | None = None,
    status_code: int = 200,
):
    """Render the location list, optionally with an error banner (409 paths)."""
    locs = (
        await session.execute(select(Location).order_by(Location.code))
    ).scalars().all()
    token = request.cookies.get("csrf_token") or generate_csrf_token()
    rendered = templates.TemplateResponse(
        "warehouse/location_list.html",
        {
            "request": request,
            "user": user,
            "csrf_token": token,
            "locations": locs,
            "error": error,
        },
        status_code=status_code,
    )
    set_csrf_cookie(rendered, token)
    return rendered


@router.get("/locations")
async def location_list(
    request: Request,
    session: AsyncSession = Depends(get_session),
    user=Depends(get_current_user),
    _role=Depends(require_admin),
):
    return await _render_locations(request, session, user)


@router.post("/locations")
async def location_create(
    request: Request,
    code: str = Form(...),
    name: str = Form(""),
    csrf_token: str = Form(...),
    session: AsyncSession = Depends(get_session),
    user=Depends(get_current_user),
    _role=Depends(require_admin),
    _: None = Depends(require_csrf),
):
    """Create a location — audited in the same BEGIN IMMEDIATE txn."""
    try:
        await create_location(code, name, engine=session.bind, actor_id=user.id)
    except LocationConflictError as e:
        return await _render_locations(
            request, session, user, error=str(e), status_code=409
        )
    return RedirectResponse(url="/locations", status_code=303)


@router.post("/location/{code}/edit")
async def location_edit(
    code: str,
    request: Request,
    name: str = Form(""),
    csrf_token: str = Form(...),
    session: AsyncSession = Depends(get_session),
    user=Depends(get_current_user),
    _role=Depends(require_admin),
    _: None = Depends(require_csrf),
):
    """Rename a location — audited; an unchanged name is a silent no-op."""
    try:
        await update_location(code, name, engine=session.bind, actor_id=user.id)
    except LocationNotFoundError:
        raise HTTPException(status_code=404) from None
    return RedirectResponse(url="/locations", status_code=303)


@router.post("/location/{code}/deactivate")
async def location_deactivate(
    code: str,
    request: Request,
    csrf_token: str = Form(...),
    session: AsyncSession = Depends(get_session),
    user=Depends(get_current_user),
    _role=Depends(require_admin),
    _: None = Depends(require_csrf),
):
    """Soft-disable a location — audited; already-inactive → 409."""
    try:
        await deactivate_location(code, engine=session.bind, actor_id=user.id)
    except LocationNotFoundError:
        raise HTTPException(status_code=404) from None
    except LocationConflictError as e:
        return await _render_locations(
            request, session, user, error=str(e), status_code=409
        )
    return RedirectResponse(url="/locations", status_code=303)


@router.get("/tubes/{pipe_id}")
async def tube_detail(
    request: Request,
    pipe_id: str,
    session: AsyncSession = Depends(get_session),
    user=Depends(get_current_user),
    _role=Depends(require_operator),
):
    tube = (await session.execute(
        select(Tube).where(Tube.pipe_id == pipe_id, Tube.deleted_at.is_(None))
    )).scalars().first()
    if tube is None:
        raise HTTPException(status_code=404, detail="pipe not found")
    specs = None
    if tube.type_ == "seamless_pipe":
        specs = (await session.execute(
            select(SeamlessPipeSpec).where(SeamlessPipeSpec.pipe_id == pipe_id)
        )).scalars().first()
    elif tube.type_ == "screen_pipe":
        specs = (await session.execute(
            select(ScreenPipeSpec).where(ScreenPipeSpec.pipe_id == pipe_id)
        )).scalars().first()
    items = (await session.execute(
        select(TransactionItem).join(Transaction).where(TransactionItem.pipe_id == pipe_id)
        .order_by(Transaction.created_at.desc()).limit(20)
    )).scalars().all()
    audits = (await session.execute(
        select(AuditLog).where(AuditLog.detail.like(f"%pipe_id={pipe_id}%"))
        .order_by(AuditLog.id.desc()).limit(5)
    )).scalars().all()
    token = request.cookies.get("csrf_token") or generate_csrf_token()
    rendered = templates.TemplateResponse(
        "warehouse/tube_detail.html",
        {"request": request, "user": user, "csrf_token": token, "tube": tube, "specs": specs, "items": items, "audits": audits},
    )
    set_csrf_cookie(rendered, token)
    return rendered
