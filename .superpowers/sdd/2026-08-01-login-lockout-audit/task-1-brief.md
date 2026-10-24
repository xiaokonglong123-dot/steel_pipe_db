## Task 1: 核心实现（security.py + auth.py + 6 项测试）

### Files
- `src/security.py` — 新增 2 个类常量 + `record_failure` 加返回值
- `src/modules/auth/routers/auth.py` — imports + 401 分支审计写入
- `tests/test_login_lockout_audit.py` — 新建（6 项测试）

### Interfaces

| 变更 | 签名 |
|---|---|
| `LoginRateLimiter.MAX_FAILURES` | `int` = `10`（新增类常量） |
| `LoginRateLimiter.LOCKOUT_SECONDS` | `float` = `300.0`（新增类常量） |
| `LoginRateLimiter.record_failure` | `(self, username: str) -> bool`——返回 True ⇔ 本次恰好触发锁定（`locked_until` 从非未来 → 未来） |
| `auth.py` 401 分支 | 保持现有 raise 不变；审计写在 raise 之前 |

### Step 1: 写失败测试

创建 `tests/test_login_lockout_audit.py`（完整内容）：

```python
"""Login lockout audit tests (D10).

Covers:
* 10th failed login triggers exactly one login_lockout audit row (user_id NULL)
* 9 failures write nothing
* locked attempts (429) do not add new audit rows
* IP-tier 429s write nothing (username tier never reached)
* audit hash chain stays verifiable after a lockout event
* successful login clears failure records without writing audit

Rate limiter is a module-level singleton — the autouse fx fixture resets
ip_hits / user_fail so tests never contaminate each other. 10-failure tests
use distinct X-Forwarded-For IPs: the IP tier (<=5/min) would 429 the 6th
same-IP attempt before the username tier ever locks.
"""

from __future__ import annotations

import asyncio
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

TEST_DB_PATH = "data/test_login_lockout_audit.db"
os.environ["DATABASE_URL"] = f"sqlite+aiosqlite:///{TEST_DB_PATH}"
os.environ.setdefault("JWT_SECRET_KEY", "test-secret-login-lockout")

from httpx import ASGITransport, AsyncClient
import pytest
from sqlalchemy import create_engine as create_sync_engine
from sqlalchemy.ext.asyncio import AsyncSession, async_sessionmaker, create_async_engine
from sqlalchemy.orm import sessionmaker as sync_sessionmaker

from rust_core import auth as rust_auth
rust_auth.init(os.environ["JWT_SECRET_KEY"])  # 成功登录路径 issue_auth_tokens 需要
from src import database as db
from src.main import app
from src.models import AuditLog, Base, User
from src.security import get_current_user, login_limiter, require_csrf


SYNC_DB_URL = f"sqlite:///{TEST_DB_PATH}"
ASYNC_DB_URL = f"sqlite+aiosqlite:///{TEST_DB_PATH}"
sync_engine = create_sync_engine(SYNC_DB_URL, echo=False)
SyncSession = sync_sessionmaker(bind=sync_engine)


def _seed():
    Base.metadata.create_all(bind=sync_engine)
    with SyncSession() as s:
        for row in s.query(AuditLog).all():
            s.delete(row)
        for row in s.query(User).all():
            s.delete(row)
        s.commit()
        s.add(User(
            id=1, username="admin", role="admin", display_name="Admin",
            active=True, password_hash=rust_auth.hash_password("Pa$$w0rd"),
            token_version=0,
        ))
        s.commit()


def _reset_db():
    if os.path.exists(TEST_DB_PATH):
        os.remove(TEST_DB_PATH)
    for ext in ("-wal", "-shm"):
        p = TEST_DB_PATH + ext
        if os.path.exists(p):
            os.remove(p)


def _allow_csrf():
    async def _dep():
        return None
    return _dep


def _stub_user(user_id: int):
    async def _dep():
        with SyncSession() as s:
            return s.get(User, user_id)
    return _dep


def _audit_rows() -> list[AuditLog]:
    with SyncSession() as s:
        return s.query(AuditLog).order_by(AuditLog.id).all()


def _audit_count() -> int:
    with SyncSession() as s:
        return s.query(AuditLog).count()


@pytest.fixture
def client():
    transport = ASGITransport(app=app)
    return AsyncClient(transport=transport, base_url="http://test")


@pytest.fixture(autouse=True)
def fx():
    _reset_db()
    _seed()
    login_limiter.ip_hits.clear()
    login_limiter.user_fail.clear()
    db.engine = create_async_engine(ASYNC_DB_URL, echo=False)
    db.async_session_factory = async_sessionmaker(
        db.engine, class_=AsyncSession, expire_on_commit=False
    )
    yield
    app.dependency_overrides.clear()
    Base.metadata.drop_all(bind=sync_engine)
    sync_engine.dispose()
    login_limiter.ip_hits.clear()
    login_limiter.user_fail.clear()


async def _post(client, url: str, data: dict, xff: str):
    return await client.post(
        url,
        data={**data, "csrf_token": "test-csrf"},
        headers={"X-Forwarded-For": xff},
        follow_redirects=False,
    )


async def _fail_n_times(client, n: int, start_ip: int = 1) -> str:
    """Send n failed logins (wrong password) from distinct XFF IPs.

    Returns the IP used for the n-th (last) attempt.
    """
    last_ip = ""
    for i in range(start_ip, start_ip + n):
        last_ip = f"203.0.113.{i}"
        r = await _post(
            client, "/login", {"username": "admin", "password": "WrongPass!"}, last_ip
        )
        assert r.status_code == 401, r.text[:200]
    return last_ip


def test_10th_failure_triggers_lockout_audit(client):
    app.dependency_overrides[require_csrf] = _allow_csrf()

    async def run():
        async with client as c:
            last_ip = await _fail_n_times(c, 10)
            return last_ip

    last_ip = asyncio.run(run())
    rows = _audit_rows()
    assert len(rows) == 1, f"expected exactly 1 audit row, got {len(rows)}"
    row = rows[0]
    assert row.action == "login_lockout"
    assert row.user_id is None
    assert "username=admin" in row.detail
    assert f"ip={last_ip}" in row.detail
    assert "failures=10" in row.detail
    assert "locked_for=300s" in row.detail


def test_9_failures_no_audit(client):
    app.dependency_overrides[require_csrf] = _allow_csrf()

    async def run():
        async with client as c:
            await _fail_n_times(c, 9)

    asyncio.run(run())
    assert _audit_count() == 0


def test_locked_attempt_no_new_audit(client):
    app.dependency_overrides[require_csrf] = _allow_csrf()

    async def run():
        async with client as c:
            await _fail_n_times(c, 10)
            r = await _post(
                c, "/login", {"username": "admin", "password": "WrongPass!"}, "203.0.113.99"
            )
            return r

    r = asyncio.run(run())
    assert r.status_code == 429, r.text[:200]
    assert _audit_count() == 1


def test_ip_429_no_audit(client):
    app.dependency_overrides[require_csrf] = _allow_csrf()

    async def run():
        async with client as c:
            for i in range(6):
                r = await _post(
                    c, "/login", {"username": "admin", "password": "WrongPass!"}, "203.0.113.50"
                )
                if i < 5:
                    assert r.status_code == 401, r.text[:200]
            return r

    r = asyncio.run(run())
    assert r.status_code == 429, r.text[:200]
    assert _audit_count() == 0


def test_chain_verifies_after_lockout(client):
    app.dependency_overrides[get_current_user] = _stub_user(1)
    app.dependency_overrides[require_csrf] = _allow_csrf()

    async def run():
        async with client as c:
            await _fail_n_times(c, 10)
            v = await _post(c, "/audit/verify", {}, "203.0.113.88")
            return v

    v = asyncio.run(run())
    assert v.status_code == 200, v.text[:400]
    assert "哈希链完整" in v.text


def test_success_clears_failures_no_audit(client):
    app.dependency_overrides[require_csrf] = _allow_csrf()

    async def run():
        async with client as c:
            await _fail_n_times(c, 3)
            r = await _post(
                c, "/login", {"username": "admin", "password": "Pa$$w0rd"}, "203.0.113.77"
            )
            return r

    r = asyncio.run(run())
    assert r.status_code == 303, r.text[:200]
    assert _audit_count() == 0
```

### Step 2: 运行，验证失败（RED）

```bash
.venv/bin/python -m pytest tests/test_login_lockout_audit.py -v
```

**预期：2 failed, 4 passed**

| 测试 | RED 状态 | 原因 |
|---|---|---|
| `test_10th_failure_triggers_lockout_audit` | **FAIL** | `record_failure` 返回 None → `just_locked` 假值 → 无审计行（`len(rows)==1` 断言，实际 0） |
| `test_locked_attempt_no_new_audit` | **FAIL** | 审计行数 `==1` 断言，实际 0（429 部分本身通过——锁定内存态已触发） |
| `test_9_failures_no_audit` | PASS | 守卫测试，实现前即成立 |
| `test_ip_429_no_audit` | PASS | 守卫测试（username 层未达阈值） |
| `test_chain_verifies_after_lockout` | PASS | 空链 verify → ok（实现后验证含锁定行的链） |
| `test_success_clears_failures_no_audit` | PASS | 守卫测试 |

若任一测试报 ImportError/TypeError（而非断言失败），停下检查 fixture 模式与 test_admin_audit.py 的差异后重跑。

### Step 3: 实现 `src/security.py`

**3a. 类常量**（插在 `class LoginRateLimiter` docstring 之后、`def __init__` 之前）：

```python
    MAX_FAILURES = 10          # 用户名层锁定阈值（触发审计写入）
    LOCKOUT_SECONDS = 300.0    # 锁定时长 + 滚动窗口
```

**3b. `record_failure` 整体替换**（保持现有逻辑逐行不变，仅加返回值；`None → bool`）：

```python
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
```

> 语义保持：原实现 `count >= 10` 即无条件刷新 `locked_until`；改造后仅当此前非未来才返回 True，`locked_until` 的刷新行为不变。`check_ip`/`check_user`/`record_ip`/`record_success` 零改动。

### Step 4: 实现 `src/modules/auth/routers/auth.py`

**4a. imports：**
- 行 12：`from src.modules.warehouse.services.audit_service import begin_immediate` → 加 `_write_audit`：
  `from src.modules.warehouse.services.audit_service import _write_audit, begin_immediate`
- 行 13-24 的 `from src.security import (...)` 块：在 `issue_auth_tokens,` 与 `login_limiter,` 之间插入 `LoginRateLimiter,`（保持字母序）
- 文件顶部（`import anyio` 之后）加 `import logging`；模块级 `logger = logging.getLogger(__name__)`

**4b. login 401 分支替换**（现 86-88 行）：

```python
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
```

> 现有 401 raise 原样保留；审计失败仅记日志（尽力而为）。`session.bind.connect()` 开独立连接（与 76 行 select 的读事务互不干扰；WAL 下并发安全）。

### Step 5: 运行，验证通过（GREEN）

```bash
.venv/bin/python -m pytest tests/test_login_lockout_audit.py -v
```
预期 6 passed。

### Step 6: 全量回归

```bash
.venv/bin/python -m pytest -m "not slow"
```
预期 **115 passed, 3 deselected**（109 + 6）。重点确认 test_security.py（限流/登录）与 test_routes_protection.py（401/429）无回归。

---

