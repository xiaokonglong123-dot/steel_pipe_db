# A — 认证与会话规格

来源：`tests/test_security.py`（15 项）、`tests/test_login_lockout_audit.py`（6 项）、
旧实现 `src/security.py`、`src/modules/auth/routers/auth.py`、`rust-core/src/auth.rs`。

约定：`401` 表示未认证，`403` 表示已认证但无权/CSRF 失败，`429` 表示限流。

## 口令

| # | 规格 | 来源 | 状态 |
| --- | --- | --- | --- |
| A1 | argon2id，参数 `m=19456 KiB, t=2, p=1`，输出 PHC 字符串（`$argon2id$v=19$m=19456,t=2,p=1$…`） | `auth.rs` / `test_security.py` 隐含 | ✅ `crates/auth` 单测锁定 |
| A2 | 校验口令时任何异常（错误口令、损坏哈希、空串）返回 `false`，不 panic | `auth.rs` | ✅ `crates/auth` |
| A3 | 同一口令两次哈希结果不同（随机盐），且都能通过校验 | `auth.rs` | ✅ `crates/auth` |

## Token

| # | 规格 | 来源 | 状态 |
| --- | --- | --- | --- |
| A4 | access token：HS256，TTL 15 分钟，载荷 `sub/role/token_version/exp`，无 `jti` | `auth.rs::issue_access_token` | ✅ `crates/auth` |
| A5 | refresh token：HS256，TTL 30 天，载荷含 `jti`（UUID v4），`role` 缺省 | `auth.rs::issue_refresh_token` | ✅ `crates/auth` |
| A6 | 签名被篡改、由其他密钥签发、格式非法（含 `alg=none`）一律拒绝 | `test_security.py` | ✅ `crates/auth` |
| A7 | `exp` 到期即拒绝；**宽限期 0 秒**（旧实现用 jsonwebtoken 默认 60s，属漂移，已修正） | `auth.rs` 默认 Validation | ✅ `crates/auth` |
| A8 | `sub` 非整数 → 明确错误，不静默按 0 处理 | `auth.rs` 旧实现 `unwrap_or(0)` 是缺陷 | ✅ `crates/auth` |
| A9 | `token_version` 只解码透出；是否与 DB 当前值一致由中间件比对，不一致 → `401`（吊销全体会话） | `security.py`、`test_security.py::test_protected_no_cookies` 相邻用例 | 待实现（M1 中间件） |

## 会话与 CSRF

| # | 规格 | 来源 | 状态 |
| --- | --- | --- | --- |
| A10 | 登录成功：下发 access + refresh 两个 httpOnly cookie（`SameSite=Lax`，无 `max-age`，有效期靠 JWT `exp`）+ 非 httpOnly 的 CSRF cookie；`303` 跳转 | `test_login_and_access` | 待实现 |
| A11 | 未认证访问受保护资源：API 客户端（`Accept: application/json`）→ `401` JSON；浏览器导航（`Accept: text/html`）→ 跳转 `/login` | `test_get_tubes_unauthenticated_returns_401`、`test_get_tubes_unauthenticated_browser_navigation_redirects_to_login` | 待实现 |
| A12 | CSRF double-submit：写请求缺匹配 token → `403`；匹配（cookie 值 = 表单字段或 `X-CSRF-Token` 头）→ 通过 | `test_csrf_missing`、`test_csrf_valid`、`test_post_revoke_missing_csrf_rejected` | 待实现 |
| A13 | 登录成功后 access 过期、refresh 有效时自动静默刷新并重放请求 | `test_auto_refresh` | 待实现（前端场景，M6 以 401→刷新→重试实现） |

## 登出与轮换

| # | 规格 | 来源 | 状态 |
| --- | --- | --- | --- |
| A14 | `POST /auth/logout`：需登录（无 cookie → `401`）+ 有效 CSRF（缺失 → `403`）；清空 `users.last_refresh_jti` 并删除 cookie，旧 refresh 立即失效 | `test_logout_requires_auth`、`test_logout_requires_csrf`、`test_logout_revokes_refresh_cookie` | 待实现 |
| A15 | `POST /auth/refresh`：需 refresh cookie（缺失 → `401`）+ CSRF（缺失 → `403`）；成功则签发新 access + 新 refresh，并把新 `jti` 写入该用户（单会话轮换） | `test_refresh_endpoint_requires_refresh_cookie`、`test_refresh_endpoint_requires_csrf`、`test_refresh_endpoint_rotates_tokens` | 待实现 |
| A16 | 已被轮换掉的 refresh（`jti` ≠ DB 当前值）→ `401`，且不轮换 cookie | `test_refresh_endpoint_rejects_stale_token` | 待实现 |

## 登录限流与审计

| # | 规格 | 来源 | 状态 |
| --- | --- | --- | --- |
| A17 | 用户名层：同一用户名 300 秒内失败 10 次 → 锁 300 秒，期间返回 `429` | `test_rate_limiter` | 待实现 |
| A18 | IP 层：同一 IP 60 秒内 >5 次 → `429`（先于用户名层触发） | `test_rate_limiter_ip` | 待实现 |
| A19 | 触发锁定的那一次（第 10 次失败）写**恰好 1 条** `login_lockout` 审计，`user_id = NULL` | `test_10th_failure_triggers_lockout_audit` | 待实现 |
| A20 | 前 9 次失败不写审计；锁定期间的 429 不写；IP 层 429 不写（用户名层未达） | `test_9_failures_no_audit`、`test_locked_attempt_no_new_audit`、`test_ip_429_no_audit` | 待实现 |
| A21 | 登录成功清空该用户的失败记录，**不**写审计 | `test_success_clears_failures_no_audit` | 待实现 |
| A22 | 锁定事件后审计链仍可验证（`login_lockout` 行参与链计算） | `test_chain_verifies_after_lockout` | 待实现（依赖 F 审计链） |
| A23 | 限流状态跨进程重启仍有效（旧实现是进程内 dict；改为 PG→SQLite 表 `login_attempts`） | 本项目的主动改造，非旧规格 | 待实现 |
| A24 | `STEELPIPE_JWT_SECRET` 为空或 <32 字节 → 拒绝启动 | 设计 §9.4 | ✅ M0 `config.rs` |

## 待你确认的取舍

1. **A7 宽限期 0 秒**：旧实现有 60 秒宽限。改成 0 更严格，但若前端时钟与服务器偏差较大，瞬时 401 会变多；当前单机部署无时钟漂移风险，按 0 执行。
2. **A11 的 401 与 302 双轨**：Rust 侧按 `Accept` 头分流，不用 UA 判断。
3. **A23 限流落库**：旧实现是模块级单例 dict，重启清零。落库后重启不清零，行为更严格；若你希望保留"重启即解锁"的旧行为，改回内存即可（单进程部署下没差别）。
