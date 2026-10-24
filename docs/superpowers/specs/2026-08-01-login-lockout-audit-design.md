# 登录锁定审计（D10）设计

**日期：** 2026-08-01
**状态：** 已实现（D10, 2026-08-01）
**前置：** D9 审计缺口修复（user/standards 写操作审计，2026-08-01 已落地）
**设计文档缺口来源：** `docs/2026-07-13-api5ct-inventory-design.md` §17 功能缺口表「登录锁定写审计日志 | §7 设计意图补全」；v0.3.0 归档行 11 记载"文档原写'写入审计日志'，实现未写，已按实现对齐"——即审计登录锁定是**原设计意图**，本 spec 补全。

---

## 1. 背景

当前 `security.py` 的 `LoginRateLimiter`（内存单例，单 worker）：
- **IP 层**：`check_ip(ip)` ≤5 次/分钟 → 429
- **用户名层**：`check_user(username)` 10 次失败/300s → 锁 300s（`locked_until` 未来时间戳）

`auth.py` POST /login 流程：
1. `check_ip(ip)` / `check_user(username)` → 429（被限/被锁）
2. 查用户；用户不存在 / 不活跃 / 密码错误 → `record_ip(ip)` + `record_failure(username)` → 401
3. 成功 → `record_success(username)` → 签发令牌

现状：login 路径**零审计写入**。`AuditLog.user_id` 可空（无 FK），`_write_audit(conn, action, detail, user_id=None)` 已有 `user_id=None` 默认——系统事件（无认证操作者）可用 NULL 表示。

## 2. 目标

1. 用户名锁定**触发事件**（第 10 次失败）写 1 条审计行
2. 满足项目约定：审计写入在 `begin_immediate` 事务内
3. 锁定期间被 429 拦截的尝试、IP 层 429、每次登录失败**均不写**（防审计表被爆破刷爆）
4. 设计文档  缺口行移除

## 3. 非目标

- **锁定状态持久化**：限流器保持内存单例（重启清空，与现状一致）——审计记录的是**事件**（history），不是状态
- 登录成功 / 每次失败 / IP 层 429 审计
- 解锁事件审计（解锁是时间驱动，无显式事件）
- `audit_logs` 加列 / 迁移

## 4. 设计

### 4.1 事件与数据

| 项 | 值 |
|---|---|
| action | `login_lockout` |
| user_id | `NULL`（系统事件；`user_id` 语义保持"操作者"不变，锁定是被系统施加的） |
| detail | `username={username} ip={client_ip} failures=10 locked_for=300s` |
| 触发条件 | `record_failure` 本次把 `locked_until` 从非有效未来 → 有效未来（即计数 9→10 的唯一那次） |
| 防刷 | 锁定期间 `check_user` 拦截 → 429，不再 `record_failure` → 不产生新审计行 |

detail 不含密码/哈希。`client_ip(request)` 已有（`X-Forwarded-For` 优先）。

### 4.2 实现（方案 A：信号返回值 + handler 写审计）

**`security.py` — `LoginRateLimiter`：** 新增两个类常量（当前阈值全部方法内硬编码，本次仅 `record_failure` 与审计 detail 引用，消除魔法数漂移）：

```python
class LoginRateLimiter:
    MAX_FAILURES = 10          # 用户名层锁定阈值
    LOCKOUT_SECONDS = 300.0    # 锁定时长 + 滚动窗口
```

`record_failure` 在**保持现有逻辑逐行不变**的前提下加返回值：

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

**`auth.py` — login 401 分支（方案 A 写入点，现有 401 raise 原样保留）：**
```python
login_limiter.record_ip(ip)
just_locked = login_limiter.record_failure(username)
if just_locked:
    detail = (
        f"username={username} ip={ip} "
        f"failures={LoginRateLimiter.MAX_FAILURES} "
        f"locked_for={LoginRateLimiter.LOCKOUT_SECONDS}s"
    )
    try:
        async with session.bind.connect() as conn:
            await begin_immediate(conn)
            await _write_audit(conn, "login_lockout", detail, user_id=None)
            await conn.commit()
    except Exception:
        logger.exception("login lockout audit write failed")
raise HTTPException(status_code=401, detail="invalid credentials")  # 原样保留
```

- **尽力而为**：审计失败仅记日志，不阻塞登录（登录本就 401；审计是唯一 DB 写，无业务事务可并入 → 独立 `begin_immediate` 事务）
- `auth.py` 补 `from src.modules.warehouse.services.audit_service import _write_audit` + `import logging`；`LoginRateLimiter` 需从 `src.security` 的 import 块中显式导入（当前仅导入 `login_limiter`）
- detail 恒为 `failures=10 locked_for=300s`（由类常量推导，与限流逻辑单一来源）

### 4.3 测试（新 `tests/test_login_lockout_audit.py`）

复用既有 fixture 模式（独立 DB、env 预置、`_allow_csrf`、`client`）：

**关键基建**
- `login_limiter` 是模块级单例 → `fx` fixture 重置 `login_limiter.ip_hits.clear()` / `login_limiter.user_fail.clear()`
- **10 次失败必须不同 `X-Forwarded-For`**：IP 层 ≤5 次/分，同 IP 第 6 次就被 429 拦截，username 锁定永远触发不了——用不同 IP 是真实场景的正确建模（IP 层保护同源爆破，username 层抓分布式爆破）
- 审计行断言直接查测试 DB（SyncSession）

| # | 测试 | 断言 |
|---|---|---|
| 1 | `test_10th_failure_triggers_lockout_audit` | 10 次错误密码（不同 XFF）→ audit 行存在：`action=login_lockout`、detail 含 `username=`/`ip=`/`failures=10`/`locked_for=300s`、`user_id IS NULL` |
| 2 | `test_9_failures_no_audit` | 9 次 → 无 audit 行 |
| 3 | `test_locked_attempt_no_new_audit` | 锁定后第 11 次 → 429 且 audit 行数仍为 1（仅触发一次） |
| 4 | `test_ip_429_no_audit` | 同 IP 5 次失败（第 6 次 429 先行）→ 无 username 锁定 → 无 audit 行 |
| 5 | `test_chain_verifies_after_lockout` | 锁定事件后 `POST /audit/verify`（admin stub）→ 200 + 链完整 |
| 6 | `test_success_clears_failures_no_audit` | 失败几次后成功登录 → 无 audit 行（`record_success` 清除记录） |

### 4.4 文档同步（实现完成后）

- 设计文档 **v0.6.0**： 缺口行「登录锁定写审计日志」移除、§7 限流细节补「锁定触发写审计」、新修订记录节 + 状态总览重编号（沿用 v0.5.0 先例：本次修订 → 归档，新增本次修订节，状态总览后移）、测试矩阵
- AGENTS.md **v0.6.0**：NOTES 补锁定审计说明、测试基线
- 应用版本 **0.5.0**（pyproject.toml + main.py，沿用 D9 模式）
- rust-core Cargo.toml 不动（独立模块版本 0.1.0）

## 5. 文件清单

```
改 src/security.py                                # record_failure 返回 bool
改 src/modules/auth/routers/auth.py               # 锁定触发审计写入（_write_audit + begin_immediate + logger）
新 tests/test_login_lockout_audit.py              # 6 项测试
文档同步（实现后）：design doc v0.6.0 / AGENTS.md v0.6.0 / pyproject + main.py 版本 0.5.0
```

## 6. 风险与回滚

- **审计写入失败**：best-effort（log exception），登录流程不受影响——测试 1 断言审计行存在，若写入路径有误会立刻暴露
- **限流器语义变化**：`record_failure` 返回值从 `None` → `bool`，现有调用点仅 auth.py 一处（已核查），无破坏面
- **测试时序敏感**：限流基于 `time.monotonic`，300s 窗口跨测试不重置会串扰——fx fixture 强制重置，消除时序依赖

## 7. 审阅记录

- 2026-08-01 设计确认（范围：仅锁定触发；语义：NULL=系统事件；方案：A）
