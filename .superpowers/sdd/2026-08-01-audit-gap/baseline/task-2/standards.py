"""API 5CT standard specification management routes."""

from fastapi import APIRouter, Depends, Form, HTTPException, Query, Request
from fastapi.responses import JSONResponse, RedirectResponse
from sqlalchemy import func, select
from sqlalchemy.exc import IntegrityError
from sqlalchemy.ext.asyncio import AsyncSession

from src.database import get_session
from src.models import StandardSpec, User
from src.security import (
    generate_csrf_token,
    get_current_user,
    require_admin,
    require_csrf,
    set_csrf_cookie,
)
from src.templates_env import templates

router = APIRouter()

PAGE_SIZE = 20


@router.get("/standards")
async def list_standards(
    request: Request,
    session: AsyncSession = Depends(get_session),
    user=Depends(get_current_user),
    grade: str | None = Query(None),
    page: int = Query(1, ge=1),
):
    """List StandardSpecs with optional grade filter and pagination."""
    if user.role != "admin":
        raise HTTPException(status_code=403, detail="仅管理员可访问")

    stmt = select(StandardSpec)

    if grade:
        stmt = stmt.where(StandardSpec.grade == grade)

    # Total count
    count_stmt = select(func.count()).select_from(stmt.subquery())
    total = (await session.execute(count_stmt)).scalar_one()

    # Paginated query
    stmt = (
        stmt.order_by(StandardSpec.id.desc())
        .limit(PAGE_SIZE)
        .offset((page - 1) * PAGE_SIZE)
    )
    rows = (await session.execute(stmt)).scalars().all()

    total_pages = max(1, (total + PAGE_SIZE - 1) // PAGE_SIZE)

    csrf_token = request.cookies.get("csrf_token") or generate_csrf_token()

    rendered = templates.TemplateResponse(
        "standards/list.html",
        {
            "request": request,
            "user": user,
            "specs": rows,
            "page": page,
            "total_pages": total_pages,
            "total": total,
            "grade_filter": grade,
            "csrf_token": csrf_token,
            "error": None,
            "ok": None,
        },
    )
    set_csrf_cookie(rendered, csrf_token)
    return rendered


@router.post("/standards")
async def create_standard(
    request: Request,
    steel_grade: str = Form(...),
    outer_diameter_mm: float = Form(...),
    min_wall_thickness_mm: float = Form(...),
    max_wall_thickness_mm: float = Form(...),
    csrf_token: str = Form(...),  # noqa: ARG001
    session: AsyncSession = Depends(get_session),
    user=Depends(get_current_user),
    _admin: User = Depends(require_admin),
    _: None = Depends(require_csrf),
):
    """Insert a new StandardSpec."""
    try:
        await session.execute(
            StandardSpec.__table__.insert().values(
                grade=steel_grade.strip(),
                outer_diameter_mm=outer_diameter_mm,
                min_wall_thickness_mm=min_wall_thickness_mm,
                max_wall_thickness_mm=max_wall_thickness_mm,
            )
        )
        await session.commit()
    except IntegrityError:
        await session.rollback()
        csrf = request.cookies.get("csrf_token") or generate_csrf_token()
        rendered = templates.TemplateResponse(
            "standards/list.html",
            {
                "request": request,
                "user": user,
                "specs": [],
                "page": 1,
                "total_pages": 1,
                "total": 0,
                "grade_filter": None,
                "csrf_token": csrf,
                "error": "该钢级已存在",
                "ok": None,
            },
            status_code=409,
        )
        set_csrf_cookie(rendered, csrf)
        return rendered

    return RedirectResponse(url="/standards", status_code=303)


# ---------------------------------------------------------------------------
# Update routes
# ---------------------------------------------------------------------------


@router.get("/standards/{spec_id}/edit")
async def edit_standard_form(
    request: Request,
    spec_id: int,
    session: AsyncSession = Depends(get_session),
    user=Depends(get_current_user),
):
    if user.role != "admin":
        raise HTTPException(status_code=403, detail="仅管理员可访问")

    spec = await session.get(StandardSpec, spec_id)
    if spec is None:
        raise HTTPException(status_code=404, detail="standard spec not found")

    csrf_token = request.cookies.get("csrf_token") or generate_csrf_token()
    rendered = templates.TemplateResponse(
        "standards/edit.html",
        {
            "request": request,
            "user": user,
            "spec": spec,
            "csrf_token": csrf_token,
            "error": None,
        },
    )
    set_csrf_cookie(rendered, csrf_token)
    return rendered


@router.post("/standards/{spec_id}")
async def update_standard(
    spec_id: int,
    steel_grade: str = Form(...),
    outer_diameter_mm: float = Form(...),
    min_wall_thickness_mm: float = Form(...),
    max_wall_thickness_mm: float = Form(...),
    csrf_token: str = Form(...),  # noqa: ARG001
    session: AsyncSession = Depends(get_session),
    user=Depends(get_current_user),
    _admin: User = Depends(require_admin),
    _: None = Depends(require_csrf),
):
    spec = await session.get(StandardSpec, spec_id)
    if spec is None:
        raise HTTPException(status_code=404, detail="standard spec not found")

    try:
        spec.grade = steel_grade.strip()
        spec.outer_diameter_mm = outer_diameter_mm
        spec.min_wall_thickness_mm = min_wall_thickness_mm
        spec.max_wall_thickness_mm = max_wall_thickness_mm
        await session.commit()
    except IntegrityError:
        await session.rollback()
        return JSONResponse(
            status_code=409,
            content={"detail": "该钢级已存在"},
        )

    return RedirectResponse(url="/standards", status_code=303)


@router.post("/standards/{spec_id}/delete")
async def delete_standard(
    spec_id: int,
    csrf_token: str = Form(...),  # noqa: ARG001
    session: AsyncSession = Depends(get_session),
    user=Depends(get_current_user),
    _admin: User = Depends(require_admin),
    _: None = Depends(require_csrf),
):
    """Hard delete — API 5CT standards are reference data, not audited."""
    spec = await session.get(StandardSpec, spec_id)
    if spec is None:
        raise HTTPException(status_code=404, detail="standard spec not found")

    await session.delete(spec)
    await session.commit()
    return RedirectResponse(url="/standards", status_code=303)
