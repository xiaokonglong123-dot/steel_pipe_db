## Task 2: 文档同步 + 版本 bump

### Files
- `docs/2026-07-13-api5ct-inventory-design.md` — v0.5.0 → v0.6.0
- `AGENTS.md` — v0.5.0 → v0.6.0
- `pyproject.toml` — 0.4.0 → 0.5.0
- `src/main.py` — `__version__` 0.4.0 → 0.5.0
- `docs/superpowers/specs/2026-08-01-login-lockout-audit-design.md` — 状态 → 已实现

### Steps

**1. 设计文档 v0.6.0**
- 头部：`> 版本：v0.5.0（...）` → `v0.6.0`，`> 日期：` → `2026-08-01`（按现状文本）
- §0.5 标题 `### 0.5 本次修订（v0.4.0 → v0.5.0）` → `### 0.5 历史修订（v0.4.0 → v0.5.0，2026-08-01 归档）`
- 在 §0.5 节之后、`### 0.6 实现状态总览` 之前插入新节：

```
### 0.6 本次修订（v0.5.0 → v0.6.0，2026-08-01）

| # | 类型 | 位置 | 变更说明 |
|---|---|---|---|
| 1 | 功能 | §7、§17 | D10 登录锁定审计落地：用户名第 10 次失败触发锁定时写 1 条审计行（action=login_lockout、user_id=NULL=系统事件、detail=username+ip+failures+locked_for）；锁定期间 429 / IP 层 429 / 单次失败均不审计（防刷）；设计文档 §17 缺口「登录锁定写审计日志」移除 |
| 2 | 对齐 | §13.1 | 测试矩阵：12 → 13 文件，基线 112/109+3 → 118/115+3；新增 test_login_lockout_audit.py（6 项） |
| 3 | 版本 | §0.1 头部 | 应用版本 0.4.0 → 0.5.0（pyproject.toml + main.py） |
```

- 原 `### 0.6 实现状态总览` → `### 0.7 实现状态总览`；`grep -n "§0\.6" docs/2026-07-13-api5ct-inventory-design.md` 修正指向状态总览的引用（**归档行除外**——历史记录保留当时的编号）
- §7 限流细节：把"锁定时不写审计日志"的表述改为"锁定触发（第 10 次失败）写审计日志（`login_lockout`）"——按现状文本定位 `grep -n "审计" docs/...design.md` 找 §7 相关行
- §17：`grep -n "登录锁定" docs/...design.md` 定位缺口行「登录锁定写审计日志」并删除
- §13.1 矩阵：加一行 `| test_login_lockout_audit.py | 登录锁定触发审计：10 次失败写 1 行（user_id NULL）/9 次不写/锁定后 429 不新增/IP 层不写/链完整/成功清除 |`；文件数 12 → 13；基线行 112/109+3 → 118/115+3
- 附录 A：文档版本引用 → v0.6.0

**2. AGENTS.md v0.6.0**
- `**Generated:**` → `2026-08-01`；`**Version:**` → `v0.6.0`
- NOTES 审计覆盖行补：`login lockout trigger（第 10 次失败）写审计（action=login_lockout、user_id=NULL、detail=username+ip+failures+locked_for）；锁定期间/IP 层 429 不审计`
- Test baseline：`109 passed + 3 deselected` → `115 passed + 3 deselected`

**3. 版本**
- `pyproject.toml`：`version = "0.4.0"` → `"0.5.0"`
- `src/main.py`：`__version__ = "0.4.0"` → `"0.5.0"`
- `rust-core/Cargo.toml` **不动**（0.1.0 独立模块版本）

**4. spec 状态**
- `docs/superpowers/specs/2026-08-01-login-lockout-audit-design.md` 状态行 → `已实现`

**5. 回归 + 自检**
- `.venv/bin/python -m pytest -m "not slow"` → 115 passed, 3 deselected（文档改动后应无变化）
- `grep -rn "0\.4\.0" pyproject.toml src/main.py` → 无匹配（确认版本已更新）
- `grep -n "登录锁定写审计日志" docs/...design.md` → 无匹配（确认缺口行已移除）
