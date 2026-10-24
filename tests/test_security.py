"""Tests for src/security.py — JWT auth, CSRF, login rate limiting."""

import pytest
from fastapi import FastAPI, Form, Depends, HTTPException, Request, Response
from fastapi.testclient import TestClient
from sqlalchemy import create_engine as create_sync_engine
from sqlalchemy.orm import sessionmaker as sync_sessionmaker
from sqlalchemy.ext.asyncio import create_async_engine, AsyncSession, async_sessionmaker
from sqlalchemy import select

import sys

sys.path.insert(0, ".")

from src.models import Base, User
from src.database import get_session
from src.security import (
    get_current_user,
    require_csrf,
    ensure_csrf_token,
    issue_auth_tokens,
    set_auth_cookies,
    ACCESS_COOKIE,
    REFRESH_COOKIE,
    LoginRateLimiter,
    login_limiter,
)
from rust_core import auth as rust_auth

# Initialize Rust with a known test secret
rust_auth.init("test-secret")

# ---------------------------------------------------------------------------
# Test DB
# ---------------------------------------------------------------------------
TEST_DB_PATH = "data/test_security.db"
TEST_DB_URL_ASYNC = f"sqlite+aiosqlite:///{TEST_DB_PATH}"
TEST_DB_URL_SYNC = f"sqlite:///{TEST_DB_PATH}"

sync_engine = create_sync_engine(TEST_DB_URL_SYNC, echo=False)
SyncSession = sync_sessionmaker(bind=sync_engine)

async_engine = create_async_engine(TEST_DB_URL_ASYNC, echo=False)
AsyncSessionFactory = async_sessionmaker(
    async_engine, class_=AsyncSession, expire_on_commit=False
)


async def override_get_session():
    async with AsyncSessionFactory() as session:
        yield session


# ---------------------------------------------------------------------------
# Test app
# ---------------------------------------------------------------------------
app = FastAPI()
app.dependency_overrides[get_session] = override_get_session

# Real auth router: exercises POST /logout (D3) and POST /auth/refresh (D4).
from src.modules.auth.routers.auth import router as auth_router  # noqa: E402

app.include_router(auth_router)

# Store seeded user id for assertions
_admin_id: int | None = None


@app.get("/protected")
async def protected(user=Depends(get_current_user)):
    return {"user_id": user.id}


@app.post("/protected-action")
async def protected_action(
    csrf_token: str = Form(...),  # noqa: ARG001
    _csrf=Depends(require_csrf),
    user=Depends(get_current_user),
):
    return {"ok": True}


@app.get("/csrf")
async def csrf(request: Request, response: Response):
    token = await ensure_csrf_token(request, response)
    return {"csrf_token": token}


@app.post("/login-test")
async def login_test(
    request: Request,
    response: Response,
    username: str = Form(...),
    password: str = Form(...),
    session: AsyncSession = Depends(get_session),
):
    result = await session.execute(select(User).where(User.username == username))
    user = result.scalar_one_or_none()
    if user is None or not rust_auth.verify_password(password, user.password_hash):
        raise HTTPException(status_code=401, detail="invalid credentials")
    access, refresh, jti = issue_auth_tokens(user)
    user.last_refresh_jti = jti
    await session.commit()
    set_auth_cookies(response, access, refresh)
    return {"ok": True}


# ---------------------------------------------------------------------------
# Fixtures
# ---------------------------------------------------------------------------


@pytest.fixture(autouse=True)
def setup_db():
    """Create tables and seed admin user before each test."""
    global _admin_id

    Base.metadata.create_all(bind=sync_engine)

    with SyncSession() as session:
        user = User(
            username="admin",
            password_hash=rust_auth.hash_password("adminpw"),
            display_name="Admin",
            role="admin",
            active=True,
        )
        session.add(user)
        session.commit()
        _admin_id = user.id

    yield

    Base.metadata.drop_all(bind=sync_engine)

    # Reset rate limiter state between tests
    login_limiter.ip_hits.clear()
    login_limiter.user_fail.clear()


# ---------------------------------------------------------------------------
# Tests
# ---------------------------------------------------------------------------


def test_protected_no_cookies():
    """GET /protected without cookies -> 401."""
    client = TestClient(app)
    resp = client.get("/protected")
    assert resp.status_code == 401
    assert resp.json()["detail"] == "not authenticated"


def test_login_and_access():
    """Login then access protected resource."""
    client = TestClient(app)

    resp = client.post(
        "/login-test", data={"username": "admin", "password": "adminpw"}
    )
    assert resp.status_code == 200
    assert ACCESS_COOKIE in client.cookies
    assert REFRESH_COOKIE in client.cookies

    resp = client.get("/protected")
    assert resp.status_code == 200
    assert resp.json()["user_id"] == _admin_id


def test_auto_refresh():
    """Auto-refresh access token from refresh cookie."""
    client = TestClient(app)
    client.post("/login-test", data={"username": "admin", "password": "adminpw"})

    # Remove access cookie, keep refresh
    client.cookies.pop(ACCESS_COOKIE, None)

    resp = client.get("/protected")
    assert resp.status_code == 200
    assert resp.json()["user_id"] == _admin_id
    # A new access cookie should have been set
    assert ACCESS_COOKIE in client.cookies


def test_csrf_missing():
    """POST without matching CSRF cookie -> 403."""
    client = TestClient(app)
    client.post("/login-test", data={"username": "admin", "password": "adminpw"})

    # POST with a csrf_token value but no CSRF cookie yet -> 403
    resp = client.post(
        "/protected-action", data={"csrf_token": "some-token"}
    )
    assert resp.status_code == 403
    assert "csrf" in resp.json()["detail"]


def test_csrf_valid():
    """POST with valid CSRF token -> 200."""
    client = TestClient(app)
    client.post("/login-test", data={"username": "admin", "password": "adminpw"})

    # Get CSRF token (sets cookie)
    resp = client.get("/csrf")
    assert resp.status_code == 200
    csrf_token = resp.json()["csrf_token"]
    assert csrf_token

    # POST with the valid CSRF token
    resp = client.post(
        "/protected-action",
        data={"csrf_token": csrf_token},
    )
    assert resp.status_code == 200
    assert resp.json()["ok"] is True


def test_rate_limiter():
    """Unit test for LoginRateLimiter username-level lockout."""
    limiter = LoginRateLimiter()

    assert limiter.check_user("bob") is True

    for _ in range(10):
        limiter.record_failure("bob")

    assert limiter.check_user("bob") is False

    limiter.record_success("bob")
    assert limiter.check_user("bob") is True


def test_rate_limiter_ip():
    """Unit test for LoginRateLimiter IP-level limiting."""
    limiter = LoginRateLimiter()

    for _ in range(5):
        assert limiter.check_ip("10.0.0.1") is True, "first 5 should be allowed"
        limiter.record_ip("10.0.0.1")

    assert limiter.check_ip("10.0.0.1") is False, "6th should be blocked"


def test_logout_revokes_refresh_cookie():
    """POST /logout clears last_refresh_jti so old refresh cookies die (D3)."""
    client = TestClient(app)
    client.post("/login-test", data={"username": "admin", "password": "adminpw"})
    old_refresh = client.cookies.get(REFRESH_COOKIE)
    assert old_refresh

    csrf_token = client.get("/csrf").json()["csrf_token"]

    resp = client.post(
        "/logout", data={"csrf_token": csrf_token}, follow_redirects=False
    )
    assert resp.status_code == 303
    assert resp.headers["location"] == "/login"
    assert ACCESS_COOKIE not in client.cookies
    assert REFRESH_COOKIE not in client.cookies

    with SyncSession() as s:
        assert s.get(User, _admin_id).last_refresh_jti is None

    # A stolen old refresh cookie can no longer be exchanged for an access token.
    client.cookies.set(REFRESH_COOKIE, old_refresh)
    client.cookies.pop(ACCESS_COOKIE, None)
    resp = client.get("/protected")
    assert resp.status_code == 401
    assert resp.json()["detail"] == "not authenticated"


def test_logout_requires_csrf():
    """POST /logout without a valid CSRF token -> 403."""
    client = TestClient(app)
    client.post("/login-test", data={"username": "admin", "password": "adminpw"})
    resp = client.post("/logout", data={}, follow_redirects=False)
    assert resp.status_code == 403


def test_logout_requires_auth():
    """POST /logout without any auth cookie -> 401."""
    client = TestClient(app)
    resp = client.post("/logout", data={"csrf_token": "stub"}, follow_redirects=False)
    assert resp.status_code == 401


def test_refresh_endpoint_rotates_tokens():
    """POST /auth/refresh rotates refresh token and updates last_refresh_jti (D4)."""
    client = TestClient(app)
    client.post("/login-test", data={"username": "admin", "password": "adminpw"})
    old_refresh = client.cookies.get(REFRESH_COOKIE)
    assert old_refresh
    old_jti = rust_auth.decode_claims(old_refresh)["jti"]

    csrf_token = client.get("/csrf").json()["csrf_token"]
    client.cookies.pop(ACCESS_COOKIE, None)

    resp = client.post(
        "/auth/refresh", data={"csrf_token": csrf_token}, follow_redirects=False
    )
    assert resp.status_code == 303
    assert resp.headers["location"] == "/dashboard"
    assert ACCESS_COOKIE in client.cookies

    new_refresh = client.cookies.get(REFRESH_COOKIE)
    assert new_refresh and new_refresh != old_refresh
    new_jti = rust_auth.decode_claims(new_refresh)["jti"]
    assert new_jti != old_jti

    with SyncSession() as s:
        assert s.get(User, _admin_id).last_refresh_jti == new_jti

    # Old refresh cookie is now rejected.
    client.cookies.set(REFRESH_COOKIE, old_refresh)
    client.cookies.pop(ACCESS_COOKIE, None)
    resp = client.get("/protected")
    assert resp.status_code == 401


def test_refresh_endpoint_rejects_stale_token():
    """POST /auth/refresh with an unknown jti -> 401, no cookie rotation."""
    client = TestClient(app)
    client.post("/login-test", data={"username": "admin", "password": "adminpw"})
    old_refresh = client.cookies.get(REFRESH_COOKIE)

    # Manually forge a refresh cookie whose jti does not match the DB.
    forged = rust_auth.issue_refresh_token(_admin_id, 0)
    client.cookies.set(REFRESH_COOKIE, forged)
    client.cookies.pop(ACCESS_COOKIE, None)

    csrf_token = client.get("/csrf").json()["csrf_token"]
    resp = client.post(
        "/auth/refresh", data={"csrf_token": csrf_token}, follow_redirects=False
    )
    assert resp.status_code == 401

    with SyncSession() as s:
        assert s.get(User, _admin_id).last_refresh_jti == rust_auth.decode_claims(
            old_refresh
        )["jti"]


def test_refresh_endpoint_requires_csrf():
    """POST /auth/refresh without a valid CSRF token -> 403."""
    client = TestClient(app)
    client.post("/login-test", data={"username": "admin", "password": "adminpw"})
    resp = client.post("/auth/refresh", data={}, follow_redirects=False)
    assert resp.status_code == 403


def test_refresh_endpoint_requires_refresh_cookie():
    """POST /auth/refresh without a refresh cookie -> 401."""
    client = TestClient(app)
    csrf_token = client.get("/csrf").json()["csrf_token"]
    resp = client.post(
        "/auth/refresh", data={"csrf_token": csrf_token}, follow_redirects=False
    )
    assert resp.status_code == 401
