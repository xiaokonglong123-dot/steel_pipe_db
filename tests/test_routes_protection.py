"""Route-level access-control tests for users/tubes endpoints.

PROJECT_RULES #78, #69:
* `admin` / `operator` / `viewer` are the only valid roles.
* All endpoints behind `get_current_user`; admin-only via `require_admin`.
* Mutating POSTs must validate CSRF double-submit token.

This file exercises the FastAPI route handlers via ASGITransport with
dependency_overrides, covering the matrix:

| Visit             | Auth | Role     | CSRF | Expected |
|-------------------|------|----------|------|----------|
| GET /tubes        | no   | -        | -    | 401      |
| GET /tubes        | yes  | admin    | -    | 200      |
| GET /tubes        | yes  | viewer   | -    | 200      |
| GET /users        | yes  | admin    | -    | 200      |
| GET /users        | yes  | operator | -    | 403      |
| POST /users/revoke/2  | yes | admin  | ok   | 302/200  |
| POST /users/revoke/2  | yes | admin  | bad  | 403      |

Notes:
* CSRF double-submit reads the cookie value and compares with the form
  field. With `app.dependency_overrides[require_csrf] = …` we test the
  real dependency only on the "valid" path; the "bad" path clears the
  override and submits a mismatched token.
* `get_current_user` is overridden with stubs that return pre-seeded
  User instances (admin/operator/viewer) without going through JWT.
* We do NOT bother hitting `/audit` or `/standards` here — those are
  covered by tests/test_audit_chain_20k.py and tests/test_used_count.py.
"""

from __future__ import annotations

import asyncio
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

TEST_DB_PATH = "data/test_routes.db"
os.environ["DATABASE_URL"] = f"sqlite+aiosqlite:///{TEST_DB_PATH}"
os.environ.setdefault("JWT_SECRET_KEY", "test-secret-routes")

import pytest
from httpx import ASGITransport, AsyncClient
from sqlalchemy import create_engine as create_sync_engine
from sqlalchemy.ext.asyncio import AsyncSession, create_async_engine
from sqlalchemy.orm import sessionmaker as sync_sessionmaker

from rust_core import auth as rust_auth
from src import database as db
from src.main import app
from src.models import Base, Location, StandardSpec, User
from src.security import get_current_user, require_csrf

SYNC_DB_URL = f"sqlite:///{TEST_DB_PATH}"
ASYNC_DB_URL = f"sqlite+aiosqlite:///{TEST_DB_PATH}"
sync_engine = create_sync_engine(SYNC_DB_URL, echo=False)
SyncSession = sync_sessionmaker(bind=sync_engine)
test_engine = create_async_engine(ASYNC_DB_URL, echo=False)


USERS_SEED = [
    {"id": 1, "username": "admin", "role": "admin", "display_name": "Admin"},
    {"id": 2, "username": "operator", "role": "operator", "display_name": "Operator"},
    {"id": 3, "username": "viewer", "role": "viewer", "display_name": "Viewer"},
]


def _seed():
    Base.metadata.create_all(bind=sync_engine)
    with SyncSession() as s:
        # Clear any stale rows so re-running is idempotent.
        for row in s.query(User).all():
            s.delete(row)
        for row in s.query(Location).all():
            s.delete(row)
        for row in s.query(StandardSpec).all():
            s.delete(row)
        s.commit()
        for u in USERS_SEED:
            s.add(User(
                id=u["id"], username=u["username"], role=u["role"],
                display_name=u["display_name"], active=True,
                password_hash=rust_auth.hash_password("Pa$$w0rd"),
                token_version=0,
            ))
        # Location + standard spec needed by the operator-allowed stock-in
        # tests. Grade C90 avoids colliding with grades used in the
        # /standards POST tests (P110/N80/J55/K55/L80/Q125).
        s.add(Location(code="WH-A", name="Warehouse A", used_count=0, is_active=True))
        s.add(StandardSpec(
            grade="C90",
            outer_diameter_mm=88.9,
            min_wall_thickness_mm=6.0,
            max_wall_thickness_mm=10.0,
            length_tolerance_mm=0.5,
            source="API 5CT",
        ))
        s.commit()


def _reset_db():
    if os.path.exists(TEST_DB_PATH):
        os.remove(TEST_DB_PATH)
    # Also remove -wal / -shm sidecars left by the previous run.
    for ext in ("-wal", "-shm"):
        p = TEST_DB_PATH + ext
        if os.path.exists(p):
            os.remove(p)


def _stub_user(user_id: int):
    """Return an async dep that yields the seeded user by id."""
    async def _dep():
        with SyncSession() as s:
            return s.get(User, user_id)
    return _dep


def _allow_csrf():
    """Bypass CSRF double-submit — simulates an already-validated token."""
    async def _dep():
        return None
    return _dep


def _enforce_csrf_with_token(token: str):
    """Standalone CSRF dep that compares against *token* (no cookie lookup).

    Not strictly necessary — the real `require_csrf` reads cookie+form —
    but using the real dependency with form fields requires a CSRF cookie
    AND matching form field, which is more setup noise than this unit
    cares about. We just need to confirm "no override → bad token rejected"
    vs "valid token → passes"; `_allow_csrf` covers the passing case and
    clearing the override covers the rejecting case.
    """
    raise NotImplementedError  # not used; see module docstring.


@pytest.fixture(autouse=True)
def fx():
    _reset_db()
    _seed()
    from sqlalchemy.ext.asyncio import async_sessionmaker as mk
    db.engine = create_async_engine(ASYNC_DB_URL, echo=False)
    db.async_session_factory = mk(
        db.engine, class_=AsyncSession, expire_on_commit=False
    )
    yield
    app.dependency_overrides.clear()
    Base.metadata.drop_all(bind=sync_engine)
    sync_engine.dispose()
    asyncio.run(db.engine.dispose())


@pytest.fixture
def client():
    """A single AsyncClient bound to the ASGI app; lifetime-scoped."""
    transport = ASGITransport(app=app)
    return AsyncClient(transport=transport, base_url="http://test")


# ---------------------------------------------------------------------------
# Tests
# ---------------------------------------------------------------------------


def test_get_tubes_unauthenticated_returns_401(client):
    async def run():
        async with client as c:
            r = await c.get("/tubes")
        assert r.status_code == 401
    asyncio.run(run())


def test_get_tubes_admin_returns_200(client):
    app.dependency_overrides[get_current_user] = _stub_user(1)
    async def run():
        async with client as c:
            r = await c.get("/tubes", follow_redirects=False)
        assert r.status_code == 200
    asyncio.run(run())


def test_get_tubes_viewer_returns_200(client):
    app.dependency_overrides[get_current_user] = _stub_user(3)
    async def run():
        async with client as c:
            r = await c.get("/tubes", follow_redirects=False)
        assert r.status_code == 200
    asyncio.run(run())


def test_get_users_admin_returns_200(client):
    app.dependency_overrides[get_current_user] = _stub_user(1)
    async def run():
        async with client as c:
            r = await c.get("/users", follow_redirects=False)
        assert r.status_code == 200
    asyncio.run(run())


def test_get_users_operator_forbidden_returns_403(client):
    app.dependency_overrides[get_current_user] = _stub_user(2)
    async def run():
        async with client as c:
            r = await c.get("/users", follow_redirects=False)
        assert r.status_code == 403
    asyncio.run(run())


def test_post_revoke_with_csrf_passes(client):
    app.dependency_overrides[get_current_user] = _stub_user(1)
    app.dependency_overrides[require_csrf] = _allow_csrf()
    async def run():
        async with client as c:
            r = await c.post(
                "/users/revoke/2",
                data={"csrf_token": "stub"},
                follow_redirects=False,
            )
        # revoke path either redirects (302) or returns 200 with template.
        assert r.status_code in (200, 302, 303), r.status_code
    asyncio.run(run())


def test_post_revoke_missing_csrf_rejected(client):
    """No CSRF override → real require_csrf; no cookie → 403."""
    app.dependency_overrides[get_current_user] = _stub_user(1)
    # NOT overriding require_csrf → real impl checks cookie+form → 403
    async def run():
        async with client as c:
            r = await c.post(
                "/users/revoke/2",
                data={"csrf_token": "bad"},
                follow_redirects=False,
            )
        assert r.status_code == 403
    asyncio.run(run())


def test_post_revoke_operator_forbidden_by_role(client):
    app.dependency_overrides[get_current_user] = _stub_user(2)
    app.dependency_overrides[require_csrf] = _allow_csrf()
    async def run():
        async with client as c:
            r = await c.post(
                "/users/revoke/3",
                data={"csrf_token": "stub"},
                follow_redirects=False,
            )
        assert r.status_code == 403
    asyncio.run(run())


# ---------------------------------------------------------------------------
# /standards — admin-only enforced by `user.role != "admin"` check.
# Pattern: GET renders standards/list.html (200) for admin, 403 otherwise;
# POST insert redirects 303 on success, 409 on duplicate grade.
# ---------------------------------------------------------------------------


def test_get_standards_unauthenticated_returns_401(client):
    async def run():
        async with client as c:
            r = await c.get("/standards", follow_redirects=False)
        assert r.status_code == 401
    asyncio.run(run())


def test_get_standards_admin_returns_200(client):
    app.dependency_overrides[get_current_user] = _stub_user(1)
    async def run():
        async with client as c:
            r = await c.get("/standards", follow_redirects=False)
        assert r.status_code == 200
    asyncio.run(run())


def test_get_standards_operator_returns_403(client):
    """`user.role != "admin"` guard rejects operator (403, not just render)."""
    app.dependency_overrides[get_current_user] = _stub_user(2)
    async def run():
        async with client as c:
            r = await c.get("/standards", follow_redirects=False)
        assert r.status_code == 403
    asyncio.run(run())


def test_post_standard_with_csrf_redirects_303(client):
    """Insert as admin + valid CSRF → 303 redirect to /standards."""
    app.dependency_overrides[get_current_user] = _stub_user(1)
    app.dependency_overrides[require_csrf] = _allow_csrf()
    async def run():
        async with client as c:
            # StandardSpec table is empty — the POST creates the first row.
            r = await c.post(
                "/standards",
                data={
                    "steel_grade": "P110",
                    "outer_diameter_mm": "88.9",
                    "min_wall_thickness_mm": "7.5",
                    "max_wall_thickness_mm": "10.0",
                    "csrf_token": "stub",
                },
                follow_redirects=False,
            )
        assert r.status_code == 303
        assert r.headers["location"] == "/standards"
    asyncio.run(run())


def test_post_standard_duplicate_grade_returns_409(client):
    """First POST inserts N80; second POST with same grade raises
    IntegrityError and the handler returns template with status_code=409."""
    app.dependency_overrides[get_current_user] = _stub_user(1)
    app.dependency_overrides[require_csrf] = _allow_csrf()
    async def run():
        async with client as c:
            await c.post(
                "/standards",
                data={
                    "steel_grade": "N80",
                    "outer_diameter_mm": "88.9",
                    "min_wall_thickness_mm": "7.5",
                    "max_wall_thickness_mm": "10.0",
                    "csrf_token": "stub",
                },
                follow_redirects=False,
            )
            r = await c.post(
                "/standards",
                data={
                    "steel_grade": "N80",
                    "outer_diameter_mm": "88.9",
                    "min_wall_thickness_mm": "7.5",
                    "max_wall_thickness_mm": "10.0",
                    "csrf_token": "stub",
                },
                follow_redirects=False,
            )
        assert r.status_code == 409
    asyncio.run(run())


def test_get_standard_edit_admin_returns_200(client):
    app.dependency_overrides[get_current_user] = _stub_user(1)
    app.dependency_overrides[require_csrf] = _allow_csrf()
    async def run():
        async with client as c:
            # Insert a spec first so we have an id
            r_in = await c.post(
                "/standards",
                data={
                    "steel_grade": "J55",
                    "outer_diameter_mm": "88.9",
                    "min_wall_thickness_mm": "7.0",
                    "max_wall_thickness_mm": "10.0",
                    "csrf_token": "stub",
                },
                follow_redirects=False,
            )
            assert r_in.status_code == 303
            r_list = await c.get("/standards", follow_redirects=False)
            import re
            m = re.search(r'href="/standards/(\d+)/edit"', r_list.text)
            assert m is not None, "edit href must appear in list"
            spec_id = m.group(1)
            r_edit = await c.get(f"/standards/{spec_id}/edit", follow_redirects=False)
        assert r_edit.status_code == 200, (r_edit.status_code, r_edit.text[:400])
        assert "J55" in r_edit.text
    asyncio.run(run())


def test_post_standard_update_admin_redirect_303(client):
    app.dependency_overrides[get_current_user] = _stub_user(1)
    app.dependency_overrides[require_csrf] = _allow_csrf()
    async def run():
        async with client as c:
            r_in = await c.post(
                "/standards",
                data={
                    "steel_grade": "K55",
                    "outer_diameter_mm": "88.9",
                    "min_wall_thickness_mm": "7.0",
                    "max_wall_thickness_mm": "10.0",
                    "csrf_token": "stub",
                },
                follow_redirects=False,
            )
            assert r_in.status_code == 303
            r_list = await c.get("/standards", follow_redirects=False)
            import re
            m = re.search(r'href="/standards/(\d+)/edit"', r_list.text)
            assert m is not None
            spec_id = m.group(1)
            r_up = await c.post(
                f"/standards/{spec_id}",
                data={
                    "steel_grade": "K55",
                    "outer_diameter_mm": "88.9",
                    "min_wall_thickness_mm": "6.0",
                    "max_wall_thickness_mm": "11.0",
                    "csrf_token": "stub",
                },
                follow_redirects=False,
            )
        assert r_up.status_code == 303, (r_up.status_code, r_up.text[:400])
        assert r_up.headers["location"] == "/standards"
    asyncio.run(run())


def test_post_standard_update_duplicate_grade_returns_409(client):
    """Update spec A's grade to match existing spec B's grade → IntegrityError → JSON 409."""
    app.dependency_overrides[get_current_user] = _stub_user(1)
    app.dependency_overrides[require_csrf] = _allow_csrf()
    async def run():
        async with client as c:
            for grade in ("L80", "P110"):
                r = await c.post(
                    "/standards",
                    data={
                        "steel_grade": grade,
                        "outer_diameter_mm": "88.9",
                        "min_wall_thickness_mm": "7.0",
                        "max_wall_thickness_mm": "10.0",
                        "csrf_token": "stub",
                    },
                    follow_redirects=False,
                )
                assert r.status_code == 303
            # Try changing spec_id=1 (L80) to 'P110' — conflicts with spec_id=2's existing grade
            r_up = await c.post(
                "/standards/1",
                data={
                    "steel_grade": "P110",
                    "outer_diameter_mm": "88.9",
                    "min_wall_thickness_mm": "7.0",
                    "max_wall_thickness_mm": "10.0",
                    "csrf_token": "stub",
                },
                follow_redirects=False,
            )
        assert r_up.status_code == 409, (r_up.status_code, r_up.text[:400])
    asyncio.run(run())


def test_post_standard_delete_admin_redirect_303(client):
    app.dependency_overrides[get_current_user] = _stub_user(1)
    app.dependency_overrides[require_csrf] = _allow_csrf()
    async def run():
        async with client as c:
            r_in = await c.post(
                "/standards",
                data={
                    "steel_grade": "Q125",
                    "outer_diameter_mm": "88.9",
                    "min_wall_thickness_mm": "7.0",
                    "max_wall_thickness_mm": "10.0",
                    "csrf_token": "stub",
                },
                follow_redirects=False,
            )
            assert r_in.status_code == 303
            r_list = await c.get("/standards", follow_redirects=False)
            import re
            m = re.search(r'href="/standards/(\d+)/edit"', r_list.text)
            spec_id = m.group(1)
            r_del = await c.post(
                f"/standards/{spec_id}/delete",
                data={"csrf_token": "stub"},
                follow_redirects=False,
            )
        assert r_del.status_code == 303, (r_del.status_code, r_del.text[:400])
        assert r_del.headers["location"] == "/standards"
    asyncio.run(run())


def test_post_standard_update_missing_csrf_rejected(client):
    app.dependency_overrides[get_current_user] = _stub_user(1)
    async def run():
        async with client as c:
            r = await c.post(
                "/standards/1",
                data={"steel_grade": "L80", "outer_diameter_mm": "88.9",
                      "min_wall_thickness_mm": "7.0", "max_wall_thickness_mm": "10.0",
                      "csrf_token": "bad"},
                follow_redirects=False,
            )
        assert r.status_code == 403
    asyncio.run(run())


def test_post_standard_delete_missing_csrf_rejected(client):
    app.dependency_overrides[get_current_user] = _stub_user(1)
    async def run():
        async with client as c:
            r = await c.post(
                "/standards/1/delete",
                data={"csrf_token": "bad"},
                follow_redirects=False,
            )
        assert r.status_code == 403
    asyncio.run(run())


# ---------------------------------------------------------------------------
# D1 — warehouse write paths must be role-gated (viewer=403, operator=ok on
# tubes, admin-only on /audit/verify + /standards + /users).
# ---------------------------------------------------------------------------


def _stock_in_payload(**kw):
    p = {
        "type": "seamless_pipe",
        "outer_diameter_mm": "88.9",
        "wall_thickness_mm": "8.0",
        "length_m": "12.0",
        "steel_grade": "C90",
        "location": "WH-A",
        "manufacturer": "Acme",
        "quality_mark": "normal",
        "csrf_token": "stub",
    }
    p.update(kw)
    return p


# --- viewer: every warehouse write → 403 -----------------------------------


def test_post_stock_out_viewer_forbidden_403(client):
    app.dependency_overrides[get_current_user] = _stub_user(3)
    app.dependency_overrides[require_csrf] = _allow_csrf()
    async def run():
        async with client as c:
            r = await c.post(
                "/tubes/stock-out",
                data={"pipe_id": "SP-L80-88.9x8.0-deadbeef", "csrf_token": "stub"},
                follow_redirects=False,
            )
        assert r.status_code == 403
    asyncio.run(run())


def test_post_transfer_viewer_forbidden_403(client):
    app.dependency_overrides[get_current_user] = _stub_user(3)
    app.dependency_overrides[require_csrf] = _allow_csrf()
    async def run():
        async with client as c:
            r = await c.post(
                "/tubes/transfer",
                data={"pipe_id": "SP-L80-88.9x8.0-deadbeef", "target_location": "WH-B", "csrf_token": "stub"},
                follow_redirects=False,
            )
        assert r.status_code == 403
    asyncio.run(run())


def test_post_stock_in_viewer_forbidden_403(client):
    app.dependency_overrides[get_current_user] = _stub_user(3)
    app.dependency_overrides[require_csrf] = _allow_csrf()
    async def run():
        async with client as c:
            r = await c.post(
                "/tubes/stock-in",
                data=_stock_in_payload(),
                follow_redirects=False,
            )
        assert r.status_code == 403
    asyncio.run(run())


def test_post_batch_stock_in_viewer_forbidden_403(client):
    app.dependency_overrides[get_current_user] = _stub_user(3)
    app.dependency_overrides[require_csrf] = _allow_csrf()
    async def run():
        async with client as c:
            r = await c.post(
                "/tubes/stock-in/batch",
                json={"tubes": [], "csrf_token": "stub"},
                follow_redirects=False,
            )
        assert r.status_code == 403
    asyncio.run(run())


def test_post_import_viewer_forbidden_403(client):
    app.dependency_overrides[get_current_user] = _stub_user(3)
    app.dependency_overrides[require_csrf] = _allow_csrf()
    async def run():
        csv = b"pipe_type,outer_diameter_mm,wall_thickness_mm,length_m,steel_grade,location,manufacturer\nseamless_pipe,88.9,8.0,12.0,C90,WH-A,Acme\n"
        files = {"file": ("v.csv", csv, "text/csv")}
        async with client as c:
            r = await c.post("/tubes/import", files=files, data={"csrf_token": "stub"}, follow_redirects=False)
        assert r.status_code == 403
    asyncio.run(run())


def test_post_audit_verify_viewer_forbidden_403(client):
    app.dependency_overrides[get_current_user] = _stub_user(3)
    app.dependency_overrides[require_csrf] = _allow_csrf()
    async def run():
        async with client as c:
            r = await c.post("/audit/verify", data={"csrf_token": "stub"}, follow_redirects=False)
        assert r.status_code == 403
    asyncio.run(run())


def test_post_standard_viewer_forbidden_403(client):
    app.dependency_overrides[get_current_user] = _stub_user(3)
    app.dependency_overrides[require_csrf] = _allow_csrf()
    async def run():
        async with client as c:
            r = await c.post(
                "/standards",
                data={"steel_grade": "T95", "outer_diameter_mm": "88.9",
                      "min_wall_thickness_mm": "7.0", "max_wall_thickness_mm": "10.0",
                      "csrf_token": "stub"},
                follow_redirects=False,
            )
        assert r.status_code == 403
    asyncio.run(run())


# --- operator: allowed on tubes POSTs, 403 on admin-only --------------------


def test_post_stock_in_operator_allowed_303(client):
    app.dependency_overrides[get_current_user] = _stub_user(2)
    app.dependency_overrides[require_csrf] = _allow_csrf()
    async def run():
        async with client as c:
            r = await c.post("/tubes/stock-in", data=_stock_in_payload(), follow_redirects=False)
        assert r.status_code == 303, (r.status_code, r.text[:400])
        assert r.headers["location"] == "/tubes"
    asyncio.run(run())


def test_post_stock_out_operator_allowed_not_403(client):
    """Operator passes the role gate; nonexistent pipe → 409 ConflictError."""
    app.dependency_overrides[get_current_user] = _stub_user(2)
    app.dependency_overrides[require_csrf] = _allow_csrf()
    async def run():
        async with client as c:
            r = await c.post(
                "/tubes/stock-out",
                data={"pipe_id": "SP-L80-88.9x8.0-deadbeef", "csrf_token": "stub"},
                follow_redirects=False,
            )
        assert r.status_code == 409
    asyncio.run(run())


def test_post_batch_stock_in_operator_allowed_200(client):
    app.dependency_overrides[get_current_user] = _stub_user(2)
    app.dependency_overrides[require_csrf] = _allow_csrf()
    async def run():
        async with client as c:
            r = await c.post(
                "/tubes/stock-in/batch",
                json={"tubes": [{
                    "pipe_type": "seamless_pipe",
                    "outer_diameter_mm": 88.9,
                    "wall_thickness_mm": 8.0,
                    "length_m": 12.0,
                    "steel_grade": "C90",
                    "location": "WH-A",
                    "manufacturer": "Acme",
                    "quality_mark": "normal",
                }], "csrf_token": "stub"},
                follow_redirects=False,
            )
        assert r.status_code == 200, (r.status_code, r.text[:400])
        body = r.json()
        assert body["ok"] == 1, body
        assert body["errors"] == []
    asyncio.run(run())


def test_post_import_operator_allowed_200(client):
    """Operator import passes the gate; per-row error collected → 200 page."""
    app.dependency_overrides[get_current_user] = _stub_user(2)
    app.dependency_overrides[require_csrf] = _allow_csrf()
    async def run():
        csv = b"pipe_type,outer_diameter_mm,wall_thickness_mm,length_m,steel_grade,location,manufacturer\nbogus,88.9,8.0,12.0,C90,WH-A,Acme\n"
        files = {"file": ("op.csv", csv, "text/csv")}
        async with client as c:
            r = await c.post("/tubes/import", files=files, data={"csrf_token": "stub"}, follow_redirects=False)
        assert r.status_code == 200
    asyncio.run(run())


def test_post_audit_verify_operator_forbidden_403(client):
    app.dependency_overrides[get_current_user] = _stub_user(2)
    app.dependency_overrides[require_csrf] = _allow_csrf()
    async def run():
        async with client as c:
            r = await c.post("/audit/verify", data={"csrf_token": "stub"}, follow_redirects=False)
        assert r.status_code == 403
    asyncio.run(run())


def test_post_standard_operator_forbidden_403(client):
    app.dependency_overrides[get_current_user] = _stub_user(2)
    app.dependency_overrides[require_csrf] = _allow_csrf()
    async def run():
        async with client as c:
            r = await c.post(
                "/standards",
                data={"steel_grade": "T95", "outer_diameter_mm": "88.9",
                      "min_wall_thickness_mm": "7.0", "max_wall_thickness_mm": "10.0",
                      "csrf_token": "stub"},
                follow_redirects=False,
            )
        assert r.status_code == 403
    asyncio.run(run())


# --- admin: allowed everywhere -------------------------------------------------


def test_post_audit_verify_admin_allowed_200(client):
    app.dependency_overrides[get_current_user] = _stub_user(1)
    app.dependency_overrides[require_csrf] = _allow_csrf()
    async def run():
        async with client as c:
            r = await c.post("/audit/verify", data={"csrf_token": "stub"}, follow_redirects=False)
        assert r.status_code == 200
    asyncio.run(run())
