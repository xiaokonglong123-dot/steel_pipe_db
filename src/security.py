"""Security module — JWT cookie auth + CSRF double-submit + login rate limiting.

Uses ONLY Rust (rust_core.auth) for JWT sign/verify/decode — never a Python JWT library.
"""

import secrets
import time
from typing import Any

import anyio
from fastapi import Depends, HTTPException, Request, Response
from sqlalchemy.ext.asyncio import AsyncSession

from rust_core import auth as rust_auth
from src.database import get_session
from src.models import User

# ---------------------------------------------------------------------------
# Cookie names
# ---------------------------------------------------------------------------
ACCESS_COOKIE = "access_token"
REFRESH_COOKIE = "refresh_token"
CSRF_COOKIE = "csrf_token"

# Default cookie options for LAN deployments (no HTTPS).
# Set secure=True when deploying behind HTTPS.
COOKIE_OPTS: dict[str, Any] = {
    "httponly": True,
    "samesite": "lax",
    "path": "/",
}

# ---------------------------------------------------------------------------
# Token issuance helpers
# ---------------------------------------------------------------------------


def issue_auth_tokens(user: User) -> tuple[str, str, str]:
    """Issue access+refresh token pair and return (access, refresh, jti)."""
    access = rust_auth.issue_access_token(user.id, user.role, user.token_version)
    refresh = rust_auth.issue_refresh_token(user.id, user.token_version)
    claims = rust_auth.decode_claims(refresh)
    assert claims is not None and "jti" in claims, "refresh token must have jti"
    return access, refresh, claims["jti"]


def set_auth_cookies(response: Response, access: str, refresh: str) -> None:
    """Set httpOnly auth cookies on the response."""
    response.set_cookie(key=ACCESS_COOKIE, value=access, **COOKIE_OPTS)
    response.set_cookie(key=REFRESH_COOKIE, value=refresh, **COOKIE_OPTS)


def clear_auth_cookies(response: Response) -> None:
    """Delete auth cookies."""
    response.delete_cookie(key=ACCESS_COOKIE, path="/")
    response.delete_cookie(key=REFRESH_COOKIE, path="/")


# ---------------------------------------------------------------------------
# User authentication dependency
# ---------------------------------------------------------------------------


async def get_current_user(
    request: Request,
    response: Response,
    session: AsyncSession = Depends(get_session),
) -> User:
    """Resolve the authenticated user from access or refresh cookie.

    Priority:
      1. Valid access token with matching token_version -> use user.
      2. Valid refresh token with matching token_version + jti ->
         auto-issue new access token (silent refresh), set cookie.
      3. Neither -> 401.
    """
    user: User | None = None

    # --- Try access token ---
    access_token = request.cookies.get(ACCESS_COOKIE)
    if access_token:
        claims = rust_auth.decode_claims(access_token)
        if claims:
            u = await session.get(User, claims["user_id"])
            if (
                u is not None
                and u.active
                and claims["token_version"] == u.token_version
            ):
                user = u

    # --- Try refresh token (auto-refresh) ---
    if user is None:
        refresh_token = request.cookies.get(REFRESH_COOKIE)
        if refresh_token:
            rclaims = rust_auth.decode_claims(refresh_token)
            if rclaims:
                u = await session.get(User, rclaims["user_id"])
                if (
                    u is not None
                    and u.active
                    and rclaims["token_version"] == u.token_version
                    and u.last_refresh_jti is not None
                    and rclaims.get("jti") == u.last_refresh_jti
                ):
                    # HS256 signing runs off the event loop (see D5).
                    new_access = await anyio.to_thread.run_sync(
                        rust_auth.issue_access_token,
                        u.id,
                        u.role,
                        u.token_version,
                    )
                    response.set_cookie(
                        key=ACCESS_COOKIE, value=new_access, **COOKIE_OPTS
                    )
                    user = u

    if user is None:
        raise HTTPException(status_code=401, detail="not authenticated")

    return user


# ---------------------------------------------------------------------------
# Role-based access
# ---------------------------------------------------------------------------


def require_roles(*allowed: str) -> Any:
    """Return a FastAPI dependency that restricts access to given roles."""
    async def _role_dependency(
        user: User = Depends(get_current_user),
    ) -> User:
        if user.role not in allowed:
            raise HTTPException(status_code=403, detail="insufficient permissions")
        return user

    return _role_dependency


# Pre-baked role dependencies for direct injection via FastAPI Depends().
# Avoid constructing a fresh closure on every request.
require_admin = require_roles("admin")
require_operator = require_roles("admin", "operator")


# ---------------------------------------------------------------------------
# CSRF double-submit
# ---------------------------------------------------------------------------


def generate_csrf_token() -> str:
    """Generate a cryptographically random CSRF token."""
    return secrets.token_urlsafe(32)


def set_csrf_cookie(response: Response, token: str) -> None:
    """Set the CSRF cookie (httpOnly, sameSite=Lax) on the given response."""
    response.set_cookie(
        key=CSRF_COOKIE,
        value=token,
        httponly=True,
        samesite="lax",
        path="/",
    )


async def ensure_csrf_token(request: Request, response: Response) -> str:
    """Ensure a CSRF cookie exists; return the token to render in the form.

    The cookie set on `response` only propagates to the client when `response`
    is the SAME instance returned by the route handler (FastAPI does NOT merge
    cookies from an injected `Response` parameter onto a later-returned
    `Response` subclass like TemplateResponse or RedirectResponse).

    Prefer `csrf_token_for(request)` + `set_csrf_cookie(returned_response, token)`
    in route handlers that return a Response subclass directly.
    """
    token = request.cookies.get(CSRF_COOKIE)
    if not token:
        token = generate_csrf_token()
        set_csrf_cookie(response, token)
    return token


def csrf_token_for(request: Request) -> str:
    """Return the existing CSRF cookie value, or generate a fresh one."""
    return request.cookies.get(CSRF_COOKIE) or generate_csrf_token()


async def require_csrf(request: Request) -> None:
    """Validate CSRF token for mutating methods (POST/PUT/DELETE/PATCH).

    Submitted-token priority:
      1. ``X-CSRF-Token`` request header  (the batch-JSON frontend sends this;
         the body may be a raw JSON array, which is not safe to read here)
      2. ``csrf_token`` field in a JSON object body (``application/json``)
      3. ``csrf_token`` field in form data (HTML forms / multipart uploads)

    The header is read BEFORE any body access: the batch stock-in frontend
    POSTs a raw JSON array body, so attempting ``request.json()`` on it would
    raise ``AttributeError`` (a list has no ``.get``) → 500. JSON body reads
    are additionally guarded with an ``isinstance(body, dict)`` check so a
    raw-array body is rejected as a plain 403, never a crash.
    All comparisons use ``secrets.compare_digest`` for constant-time equality.
    """
    if request.method not in ("POST", "PUT", "DELETE", "PATCH"):
        return

    cookie = request.cookies.get(CSRF_COOKIE)
    if cookie is None:
        raise HTTPException(status_code=403, detail="csrf validation failed")

    # 1) Header first — works even when the body is a raw JSON array.
    submitted = request.headers.get("X-CSRF-Token")

    if submitted is None:
        content_type = request.headers.get("content-type", "")
        if "application/json" in content_type:
            body = await request.json()
            submitted = body.get("csrf_token") if isinstance(body, dict) else None
        else:
            form = await request.form()
            submitted = form.get("csrf_token")

    if submitted is None or not isinstance(submitted, str):
        raise HTTPException(status_code=403, detail="csrf validation failed")

    if not secrets.compare_digest(cookie, submitted):
        raise HTTPException(status_code=403, detail="csrf validation failed")


# ---------------------------------------------------------------------------
# Login rate limiter (in-memory, single-worker only - restart clears state)
# ---------------------------------------------------------------------------


class LoginRateLimiter:
    """Two-tier in-memory login throttle.

    Tier 1 - IP-level: <=5 attempts/minute per source IP -> 429.
    Tier 2 - Username-level: <=10 failures in 5 minutes -> 5-minute lockout.
    """

    MAX_FAILURES = 10          # 用户名层锁定阈值（触发审计写入）
    LOCKOUT_SECONDS = 300.0    # 锁定时长 + 滚动窗口

    def __init__(self) -> None:
        self.ip_hits: dict[str, list[float]] = {}
        self.user_fail: dict[str, dict[str, float | int | None]] = {}

    # ---- Tier 1: IP-level ----

    def check_ip(self, ip: str) -> bool:
        """Return True if this IP is allowed, False if rate-limited."""
        now = time.time()
        hits = [t for t in self.ip_hits.get(ip, []) if now - t < 60.0]
        self.ip_hits[ip] = hits
        return len(hits) < 5

    def record_ip(self, ip: str) -> None:
        """Record a login attempt from this IP."""
        now = time.time()
        if ip not in self.ip_hits:
            self.ip_hits[ip] = []
        self.ip_hits[ip].append(now)

    # ---- Tier 2: Username-level ----

    def check_user(self, username: str) -> bool:
        """Return True if the username is allowed to attempt login.

        Returns False if currently locked out.
        """
        record = self.user_fail.get(username)
        if record is None:
            return True
        locked_until = record.get("locked_until")
        return not (
            locked_until is not None
            and isinstance(locked_until, float)
            and locked_until > time.time()
        )

    def record_failure(self, username: str) -> bool:
        """Record a failed login attempt for this username.

        Returns True iff this failure just triggered a lockout.

        Within a rolling 300s window, after 10 failures the username
        is locked out for 300 seconds.
        """
        now = time.time()
        if username not in self.user_fail:
            self.user_fail[username] = {
                "count": 0,
                "window_start": now,
                "locked_until": None,
            }
        rec = self.user_fail[username]

        window_start = rec.get("window_start")
        if window_start is None or now - (window_start if isinstance(window_start, float) else 0.0) > self.LOCKOUT_SECONDS:
            rec["count"] = 0
            rec["window_start"] = now

        rec["count"] = (rec.get("count") or 0) + 1
        locked_before = rec.get("locked_until")
        if rec["count"] is not None and rec["count"] >= self.MAX_FAILURES:
            rec["locked_until"] = now + self.LOCKOUT_SECONDS
            # 刚触发 ⇔ locked_until 从非未来（None/已过期）→ 未来
            if not (isinstance(locked_before, float) and locked_before > now):
                return True
        return False

    def record_success(self, username: str) -> None:
        """Clear failure record on successful login."""
        self.user_fail.pop(username, None)


login_limiter = LoginRateLimiter()
