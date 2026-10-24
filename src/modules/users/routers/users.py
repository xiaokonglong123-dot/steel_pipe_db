"""User management routes — list, create, edit, revoke-sessions, toggle-active.

All routes require admin role. Templates rendered via Jinja2 SSR.
"""

from fastapi import APIRouter, Depends, Form, HTTPException, Request
from fastapi.responses import RedirectResponse
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from src.database import get_session
from src.models import User
from src.modules.users.services.user_service import (
    LastAdminError,
    UserConflictError,
    create_user,
    revoke_all_sessions,
    toggle_active,
    update_user,
)
from src.security import (
    csrf_token_for,
    get_current_user,
    require_admin,
    require_csrf,
    set_csrf_cookie,
)
from src.templates_env import templates

router = APIRouter(prefix="/users", tags=["users"])


# ---------------------------------------------------------------------------
# List users
# ---------------------------------------------------------------------------


@router.get("")
@router.get("/")
async def list_users(
    request: Request,
    session: AsyncSession = Depends(get_session),
    user: User = Depends(get_current_user),
    _: User = Depends(require_admin),
):
    """List all users (admin only)."""
    result = await session.execute(
        select(User).order_by(User.created_at.desc())
    )
    users = result.scalars().all()

    return templates.TemplateResponse(
        "users/user_list.html",
        {"request": request, "user": user, "users": users},
    )


# ---------------------------------------------------------------------------
# Create user  —  GET form + POST handler
# ---------------------------------------------------------------------------


@router.get("/create")
async def create_user_form(
    request: Request,
    user: User = Depends(get_current_user),
    _: User = Depends(require_admin),
):
    """Render user creation form."""
    csrf_token = csrf_token_for(request)
    rendered = templates.TemplateResponse(
        "users/user_form.html",
        {
            "request": request,
            "user": user,
            "csrf_token": csrf_token,
            "editing": None,
            "error": None,
            "message": None,
            "roles": ["admin", "operator", "viewer"],
        },
    )
    set_csrf_cookie(rendered, csrf_token)
    return rendered


@router.post("/create")
async def create_user_handler(
    request: Request,
    username: str = Form(...),
    password: str = Form(...),
    display_name: str = Form(...),
    role: str = Form("viewer"),
    csrf_token: str = Form(...),
    user: User = Depends(get_current_user),
    _admin: User = Depends(require_admin),
    __: None = Depends(require_csrf),
):
    """Create a new user."""
    try:
        await create_user(
            username=username.strip(),
            password=password,
            display_name=display_name.strip(),
            role=role,
            actor_id=user.id,
        )
    except UserConflictError as e:
        csrf = csrf_token_for(request)
        rendered = templates.TemplateResponse(
            "users/user_form.html",
            {
                "request": request,
                "user": user,
                "csrf_token": csrf,
                "editing": None,
                "error": str(e),
                "message": None,
                "roles": ["admin", "operator", "viewer"],
            },
            status_code=409,
        )
        set_csrf_cookie(rendered, csrf)
        return rendered
    except ValueError as e:
        raise HTTPException(status_code=400, detail=str(e)) from e

    return RedirectResponse(url="/users", status_code=303)


# ---------------------------------------------------------------------------
# Edit user  —  GET form + POST handler
# ---------------------------------------------------------------------------


@router.get("/edit/{user_id}")
async def edit_user_form(
    request: Request,
    user_id: int,
    session: AsyncSession = Depends(get_session),
    user: User = Depends(get_current_user),
    _: User = Depends(require_admin),
):
    """Render user edit form pre-filled with current values."""
    target = await session.get(User, user_id)
    if target is None:
        raise HTTPException(status_code=404, detail="user not found")

    csrf_token = csrf_token_for(request)
    rendered = templates.TemplateResponse(
        "users/user_form.html",
        {
            "request": request,
            "user": user,
            "csrf_token": csrf_token,
            "editing": target,
            "error": None,
            "message": None,
            "roles": ["admin", "operator", "viewer"],
        },
    )
    set_csrf_cookie(rendered, csrf_token)
    return rendered


@router.post("/edit/{user_id}")
async def edit_user_handler(
    user_id: int,
    display_name: str = Form(...),
    role: str = Form(...),
    password: str = Form(""),
    csrf_token: str = Form(...),
    user: User = Depends(get_current_user),
    _admin: User = Depends(require_admin),
    __: None = Depends(require_csrf),
):
    """Update an existing user."""
    try:
        await update_user(
            user_id=user_id,
            display_name=display_name.strip(),
            role=role,
            password=password if password.strip() else None,
            actor_id=user.id,
        )
    except ValueError as e:
        raise HTTPException(status_code=404, detail=str(e)) from e

    return RedirectResponse(url="/users", status_code=303)


# ---------------------------------------------------------------------------
# Revoke all sessions
# ---------------------------------------------------------------------------


@router.post("/revoke/{user_id}")
async def revoke_user_sessions(
    user_id: int,
    csrf_token: str = Form(...),
    user: User = Depends(get_current_user),
    _admin: User = Depends(require_admin),
    __: None = Depends(require_csrf),
):
    """Revoke all active sessions for a user (increment token_version)."""
    try:
        await revoke_all_sessions(user_id=user_id, actor_id=user.id)
    except ValueError:
        raise HTTPException(status_code=404, detail="user not found") from None

    return RedirectResponse(url="/users", status_code=303)


# ---------------------------------------------------------------------------
# Toggle active status
# ---------------------------------------------------------------------------


@router.post("/toggle/{user_id}")
async def toggle_user_active(
    user_id: int,
    active: bool = Form(...),
    csrf_token: str = Form(...),
    user: User = Depends(get_current_user),
    _admin: User = Depends(require_admin),
    __: None = Depends(require_csrf),
):
    """Enable or disable a user account."""
    try:
        await toggle_active(user_id=user_id, active=active, actor_id=user.id)
    except LastAdminError as e:
        raise HTTPException(status_code=400, detail=str(e)) from e
    except ValueError:
        raise HTTPException(status_code=404, detail="user not found") from None

    return RedirectResponse(url="/users", status_code=303)
