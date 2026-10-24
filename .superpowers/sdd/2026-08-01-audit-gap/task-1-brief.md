### Task 1: user_service 审计接入（4 函数 + 路由传 actor）

**Files:**
- Modify: `src/modules/users/services/user_service.py`
- Modify: `src/modules/users/routers/users.py`
- Create: `tests/test_admin_audit.py`（本任务建骨架 + 测试 1-8）

**Interfaces:**
- Consumes: `_write_audit(conn, action, detail, user_id=None)` 与 `begin_immediate(conn)`，均来自 `src.modules.warehouse.services.audit_service`（`begin_immediate` 已导入）。
- Produces: `create_user(username, password, display_name, role, engine=None, *, actor_id) -> User`；`update_user(user_id, display_name=None, role=None, password=None, engine=None, *, actor_id) -> User`；`revoke_all_sessions(user_id, engine=None, *, actor_id) -> None`；`toggle_active(user_id, active, engine=None, *, actor_id) -> None`。users 路由 4 处调用传 `actor_id=user.id`。

- [ ] **Step 1: 写失败测试（文件骨架 + 测试 1-8）**

创建 `tests/test_admin_audit.py`，完整内容如下：

```python
"""Admin write-operation audit tests (D9: user + standards audit).

Covers:
* create_user / update_user / revoke_sessions / toggle_user -> audit rows
* create_standard / update_standard / delete_standard -> audit rows
* failure paths write zero audit rows (rollback)
* update_user password changes never leak the hash into detail
* mixed operations keep the audit hash chain verifiable

PROJECT_RULES: audit entries must live in the same BEGIN IMMEDIATE txn as
the operation they record; AuditLog.user_id is the actor.
"""

from __future__ import annotations

import asyncio
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

TEST_DB_PATH = "data/test_admin_audit.db"
os.environ["DATABASE_URL"] = f"sqlite+aiosqlite:///{TEST_DB_PATH}"
os.environ.setdefault("JWT_SECRET_KEY", "test-secret-admin-audit")

from httpx import ASGITransport, AsyncClient
import pytest
from sqlalchemy import create_engine as create_sync_engine
from sqlalchemy.ext.asyncio import AsyncSession, async_sessionmaker, create_async_engine
from sqlalchemy.orm import sessionmaker as sync_sessionmaker

from rust_core import auth as rust_auth
from src import database as db
from src.main import app
from src.models import AuditLog, Base, StandardSpec, User
from src.security import get_current_user, require_csrf
from src.modules.users.services.user_service import update_user

SYNC_DB_URL = f"sqlite:///{TEST_DB_PATH}"
ASYNC_DB_URL = f"sqlite+aiosqlite:///{TEST_DB_PATH}"
sync_engine = create_sync_engine(SYNC_DB_URL, echo=False)
SyncSession = sync_sessionmaker(bind=sync_engine)
test_engine = create_async_engine(ASYNC_DB_URL, echo=False)


def _seed():
    Base.metadata.create_all(bind=sync_engine)
    with SyncSession() as s:
        for row in s.query(AuditLog).all():
            s.delete(row)
        for row in s.query(User).all():
            s.delete(row)
        for row in s.query(StandardSpec).all():
            s.delete(row)
        s.commit()
        s.add(User(
            id=1, username="admin", role="admin", display_name="Admin",
            active=True, password_hash=rust_auth.hash_password("Pa$$w0rd"),
            token_version=0,
        ))
        s.add(User(
            id=2, username="operator", role="operator", display_name="Operator",
            active=True, password_hash=rust_auth.hash_password("Pa$$w0rd"),
            token_version=0,
        ))
        s.add(StandardSpec(
            id=1, grade="L80", outer_diameter_mm=88.9,
            min_wall_thickness_mm=6.0, max_wall_thickness_mm=10.0,
            length_tolerance_mm=0.5, source="API 5CT",
        ))
        s.commit()


def _reset_db():
    if os.path.exists(TEST_DB_PATH):
        os.remove(TEST_DB_PATH)
    for ext in ("-wal", "-shm"):
        p = TEST_DB_PATH + ext
        if os.path.exists(p):
            os.remove(p)


def _stub_user(user_id: int):
    async def _dep():
        with SyncSession() as s:
            return s.get(User, user_id)
    return _dep


def _allow_csrf():
    async def _dep():
        return None
    return _dep


@pytest.fixture
def client():
    transport = ASGITransport(app=app)
    return AsyncClient(transport=transport, base_url="http://test")


@pytest.fixture(autouse=True)
def fx():
    _reset_db()
    _seed()
    from sqlalchemy.ext.asyncio import AsyncSession, async_sessionmaker as mk
    db.engine = create_async_engine(ASYNC_DB_URL, echo=False)
    db.async_session_factory = mk(
        db.engine, class_=AsyncSession, expire_on_commit=False
    )
    yield
    app.dependency_overrides.clear()
    Base.metadata.drop_all(bind=sync_engine)
    sync_engine.dispose()


# --- helpers ---------------------------------------------------------------


def _audit_count() -> int:
    with SyncSession() as s:
        return s.query(AuditLog).count()


def _last_audit() -> AuditLog | None:
    with SyncSession() as s:
        return s.query(AuditLog).order_by(AuditLog.id.desc()).first()


async def _post(client: AsyncClient, url: str, data: dict):
    return await client.post(
        url, data={**data, "csrf_token": "test-csrf"}, follow_redirects=False
    )


def _as_admin():
    app.dependency_overrides[get_current_user] = _stub_user(1)
    app.dependency_overrides[require_csrf] = _allow_csrf()


# --- user ops audit --------------------------------------------------------


def test_create_user_writes_audit(client):
    _as_admin()
    async def run():
        async with client as c:
            r = await _post(c, "/users/create", {
                "username": "alice", "password": "Secret1!",
                "display_name": "Alice", "role": "operator",
            })
        assert r.status_code == 303, r.text[:400]
    asyncio.run(run())
    row = _last_audit()
    assert row is not None
    assert row.action == "create_user"
    assert "username=alice" in row.detail
    assert "role=operator" in row.detail
    assert row.user_id == 1


def test_update_user_writes_audit(client):
    _as_admin()
    async def run():
        async with client as c:
            r = await _post(c, "/users/edit/2", {
                "display_name": "Operator2", "role": "admin", "password": "",
            })
        assert r.status_code == 303, r.text[:400]
    asyncio.run(run())
    row = _last_audit()
    assert row is not None
    assert row.action == "update_user"
    assert "role=admin" in row.detail
    assert row.user_id == 1


def test_update_user_password_hides_hash(client):
    _as_admin()
    async def run():
        async with client as c:
            r = await _post(c, "/users/edit/2", {
                "display_name": "Operator2", "role": "operator",
                "password": "NewPass123!",
            })
        assert r.status_code == 303, r.text[:400]
    asyncio.run(run())
    row = _last_audit()
    assert row is not None
    assert row.action == "update_user"
    assert "password_changed=true" in row.detail
    assert "$argon2" not in row.detail


def test_update_user_no_changes_no_audit():
    # service-level: no changed fields -> rollback, no audit row
    async def run():
        await update_user(user_id=2, engine=test_engine, actor_id=1)
    asyncio.run(run())
    assert _audit_count() == 0


def test_revoke_sessions_writes_audit(client):
    _as_admin()
    async def run():
        async with client as c:
            r = await _post(c, "/users/revoke/2", {})
        assert r.status_code == 303, r.text[:400]
    asyncio.run(run())
    row = _last_audit()
    assert row is not None
    assert row.action == "revoke_sessions"
    assert "user_id=2" in row.detail
    assert "token_version+1" in row.detail
    assert row.user_id == 1


def test_toggle_user_writes_audit(client):
    _as_admin()
    async def run():
        async with client as c:
            r = await _post(c, "/users/toggle/2", {"active": "false"})
        assert r.status_code == 303, r.text[:400]
    asyncio.run(run())
    row = _last_audit()
    assert row is not None
    assert row.action == "toggle_user"
    assert "active=False" in row.detail
    assert row.user_id == 1


def test_duplicate_username_no_audit(client):
    _as_admin()
    async def run():
        async with client as c:
            r = await _post(c, "/users/create", {
                "username": "admin", "password": "Secret1!",
                "display_name": "Dup", "role": "viewer",
            })
        assert r.status_code == 409, r.text[:400]
    asyncio.run(run())
    assert _audit_count() == 0


def test_toggle_last_admin_rejected_no_audit(client):
    _as_admin()
    async def run():
        async with client as c:
            r = await _post(c, "/users/toggle/1", {"active": "false"})
        assert r.status_code == 400, r.text[:400]
    asyncio.run(run())
    assert _audit_count() == 0
```

- [ ] **Step 2: 运行确认测试失败**

Run: `.venv/bin/python -m pytest tests/test_admin_audit.py -v`
Expected: **6 failed, 2 passed**——
- FAIL（断言 `row is not None` / detail，当前无审计写入）：`test_create_user_writes_audit`、`test_update_user_writes_audit`、`test_update_user_password_hides_hash`、`test_revoke_sessions_writes_audit`、`test_toggle_user_writes_audit`
- FAIL（`TypeError: update_user() got an unexpected keyword argument 'actor_id'`，签名未改）：`test_update_user_no_changes_no_audit`
- PASS（零审计守卫在实现前即成立）：`test_duplicate_username_no_audit`、`test_toggle_last_admin_rejected_no_audit`

- [ ] **Step 3: 实现 user_service 审计**

修改 `src/modules/users/services/user_service.py`：

导入追加（现有 `from src.modules.warehouse.services.audit_service import begin_immediate` 改为）：

```python
from src.modules.warehouse.services.audit_service import (
    _write_audit,
    begin_immediate,
)
```

四个函数改动如下（保留原签名参数顺序，仅追加 `*, actor_id: int`）：

`create_user` 签名改为 `(username, password, display_name, role, engine=None, *, actor_id)`；在 `await conn.commit()` 之前插入：

```python
        await _write_audit(
            conn, "create_user", f"username={username} role={role}",
            user_id=actor_id,
        )
```

`update_user` 签名改为 `(user_id, display_name=None, role=None, password=None, engine=None, *, actor_id)`；在 `if values:` 分支内、`await conn.commit()` 之前插入：

```python
            parts = []
            if display_name is not None:
                parts.append(f"display_name={display_name}")
            if role is not None:
                parts.append(f"role={role}")
            if password is not None:
                parts.append("password_changed=true")
            await _write_audit(
                conn, "update_user", " ".join(parts), user_id=actor_id,
            )
```

`revoke_all_sessions` 签名改为 `(user_id, engine=None, *, actor_id)`；在 `await conn.commit()` 之前插入：

```python
        await _write_audit(
            conn, "revoke_sessions", f"user_id={user_id} token_version+1",
            user_id=actor_id,
        )
```

`toggle_active` 签名改为 `(user_id, active, engine=None, *, actor_id)`；在 `await conn.commit()` 之前插入：

```python
        await _write_audit(
            conn, "toggle_user", f"user_id={user_id} active={active}",
            user_id=actor_id,
        )
```

- [ ] **Step 4: 修改 users 路由传 actor**

`src/modules/users/routers/users.py` 4 处调用追加 `actor_id=user.id`：
- `create_user_handler`（line ~101）：`role=role, actor_id=user.id,`
- `edit_user_handler`（line ~178）：`password=password if password.strip() else None, actor_id=user.id,`
- `revoke_user_sessions`（line ~205）：`await revoke_all_sessions(user_id=user_id, actor_id=user.id)`
- `toggle_user_active`（line ~228）：`await toggle_active(user_id=user_id, active=active, actor_id=user.id)`

- [ ] **Step 5: 运行测试确认通过**

Run: `.venv/bin/python -m pytest tests/test_admin_audit.py -v`
Expected: 8 passed。

- [ ] **Step 6: 全量回归**

Run: `.venv/bin/python -m pytest -m "not slow" -q`
Expected: 104 passed（96 + 8），3 deselected，零失败。任何既有测试失败（尤其 test_routes_protection.py / test_security.py 中涉及 users 路由的用例）→ 停下检查签名/传参是否遗漏。

---

