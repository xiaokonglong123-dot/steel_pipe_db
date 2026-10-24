"""Authentication routes: login, logout, dashboard."""

import anyio
import logging
from fastapi import APIRouter, Depends, Form, HTTPException, Request
from fastapi.responses import RedirectResponse
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from rust_core import auth as rust_auth
from src.database import get_session
from src.models import User
from src.modules.warehouse.services.audit_service import _write_audit, begin_immediate
from src.security import (
    CSRF_COOKIE,
    REFRESH_COOKIE,
    clear_auth_cookies,
    generate_csrf_token,
    get_current_user,
    issue_auth_tokens,
    LoginRateLimiter,
    login_limiter,
    require_csrf,
    set_auth_cookies,
    set_csrf_cookie,
)
from src.templates_env import templates

logger = logging.getLogger(__name__)

router = APIRouter()


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


def client_ip(request: Request) -> str:
    """Extract client IP from X-Forwarded-For or direct connection."""
    forwarded = request.headers.get("X-Forwarded-For", "")
    if forwarded:
        return forwarded.split(",")[0].strip()
    return request.client.host if request.client else "unknown"


# ---------------------------------------------------------------------------
# Routes
# ---------------------------------------------------------------------------


@router.get("/login")
async def login_page(request: Request):
    """Render the login form."""
    token = request.cookies.get(CSRF_COOKIE) or generate_csrf_token()
    rendered = templates.TemplateResponse(
        "auth/login.html",
        {"request": request, "csrf_token": token, "error": None},
    )
    set_csrf_cookie(rendered, token)
    return rendered


@router.post("/login")
async def login(
    request: Request,
    username: str = Form(...),
    password: str = Form(...),
    csrf_token: str = Form(...),  # noqa: ARG001
    session: AsyncSession = Depends(get_session),
    _: None = Depends(require_csrf),
):
    """Authenticate user and set auth cookies."""
    ip = client_ip(request)

    # Rate limit checks
    if not login_limiter.check_ip(ip) or not login_limiter.check_user(username):
        raise HTTPException(status_code=429, detail="too many attempts")

    result = await session.execute(select(User).where(User.username == username))
    user = result.scalar_one_or_none()

    if (
        user is None
        or not user.active
        or not await anyio.to_thread.run_sync(
            rust_auth.verify_password, password, user.password_hash
        )
    ):
        login_limiter.record_ip(ip)
        just_locked = login_limiter.record_failure(username)
        if just_locked:
            detail = (
                f"username={username} ip={ip} "
                f"failures={LoginRateLimiter.MAX_FAILURES} "
                f"locked_for={LoginRateLimiter.LOCKOUT_SECONDS:g}s"
            )
            try:
                async with session.bind.connect() as conn:
                    await begin_immediate(conn)
                    await _write_audit(conn, "login_lockout", detail, user_id=None)
                    await conn.commit()
            except Exception:
                logger.exception("login lockout audit write failed")
        raise HTTPException(status_code=401, detail="invalid credentials")

    login_limiter.record_success(username)

    access, refresh, jti = await anyio.to_thread.run_sync(issue_auth_tokens, user)
    user.last_refresh_jti = jti
    await session.commit()

    redirect = RedirectResponse(url="/dashboard", status_code=303)
    set_auth_cookies(redirect, access, refresh)
    return redirect


@router.post("/logout")
async def logout(
    user: User = Depends(get_current_user),
    session: AsyncSession = Depends(get_session),
    _: None = Depends(require_csrf),
):
    """Revoke the current session's refresh token and clear auth cookies.

    Clears ``users.last_refresh_jti`` inside a ``BEGIN IMMEDIATE`` transaction
    so every already-issued refresh cookie becomes invalid immediately
    (``get_current_user`` requires ``jti == last_refresh_jti`` and non-NULL).
    Does NOT bump ``token_version`` — other active sessions stay signed in.
    """
    user_id = user.id
    # End the implicit read transaction left by get_current_user so the
    # BEGIN IMMEDIATE below starts clean on the session's connection.
    await session.rollback()
    conn = await session.connection()
    await begin_immediate(conn)
    await session.execute(
        User.__table__.update()
        .where(User.id == user_id)
        .values(last_refresh_jti=None)
    )
    await session.commit()

    redirect = RedirectResponse(url="/login", status_code=303)
    clear_auth_cookies(redirect)
    return redirect


@router.post("/auth/refresh")
async def refresh(
    request: Request,
    session: AsyncSession = Depends(get_session),
    _: None = Depends(require_csrf),
):
    """Rotate the refresh token and mint a fresh access token.

    Validates the refresh cookie (signature + exp + token_version + jti
    match), then inside one ``BEGIN IMMEDIATE`` transaction issues a new
    access + refresh pair and persists the new jti to ``last_refresh_jti``.
    The old refresh cookie is thereby invalidated immediately.
    """
    refresh_token = request.cookies.get(REFRESH_COOKIE)
    if not refresh_token:
        raise HTTPException(status_code=401, detail="not authenticated")
    rclaims = rust_auth.decode_claims(refresh_token)
    if not rclaims:
        raise HTTPException(status_code=401, detail="not authenticated")

    user_id = rclaims["user_id"]
    bind = session.bind
    async with bind.connect() as conn:
        await begin_immediate(conn)
        row = (
            await conn.execute(select(User).where(User.id == user_id))
        ).fetchone()
        if (
            row is None
            or not row.active
            or rclaims["token_version"] != row.token_version
            or row.last_refresh_jti is None
            or rclaims.get("jti") != row.last_refresh_jti
        ):
            raise HTTPException(status_code=401, detail="not authenticated")

        access, refresh, jti = await anyio.to_thread.run_sync(issue_auth_tokens, row)
        await conn.execute(
            User.__table__.update()
            .where(User.id == user_id)
            .values(last_refresh_jti=jti)
        )
        await conn.commit()

    redirect = RedirectResponse(url="/dashboard", status_code=303)
    set_auth_cookies(redirect, access, refresh)
    return redirect


@router.get("/")
@router.get("/dashboard")
async def dashboard(
    request: Request,
    user: User = Depends(get_current_user),
    session: AsyncSession = Depends(get_session),
):
    """Render the dashboard home page with summary stats."""
    from sqlalchemy import func, select

    from src.models import Tube, AuditLog

    total = await session.scalar(select(func.count(Tube.id)))
    in_stock = await session.scalar(
        select(func.count(Tube.id)).where(Tube.status == "in_stock")
    )
    out_stock = await session.scalar(
        select(func.count(Tube.id)).where(Tube.status == "out_stock")
    )
    in_transfer = await session.scalar(
        select(func.count(Tube.id)).where(Tube.status == "in_transfer")
    )
    pending_check = await session.scalar(
        select(func.count(Tube.id)).where(Tube.compliance_pass.is_(None))
    )
    audit_total = await session.scalar(select(func.count(AuditLog.id)))

    recent_audits_result = await session.execute(
        select(AuditLog)
        .order_by(AuditLog.id.desc())
        .limit(5)
    )
    recent_audits = recent_audits_result.scalars().all()

    return templates.TemplateResponse(
        "dashboard/index.html",
        {
            "request": request,
            "user": user,
            "stats": {
                "total": total or 0,
                "in_stock": in_stock or 0,
                "out_stock": out_stock or 0,
                "in_transfer": in_transfer or 0,
                "pending_check": pending_check or 0,
                "audit_total": audit_total or 0,
            },
            "recent_audits": recent_audits,
        },
    )
