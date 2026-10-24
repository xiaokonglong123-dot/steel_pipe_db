# SteelPipe — 无缝钢管与筛管库存管理系统 设计文档（修订版）

> 版本：v0.6.0（v0.5.0 功能版：登录锁定触发审计落地）
> 日期：2026-08-01
> 标准：API 5CT
> 技术栈：Python (FastAPI) + Rust (PyO3) + SQLite
> 状态标注约定：`[已实现]` / `[部分实现]` / `[未实现-规划中]` / `[缺陷修正-待实现]` —— 各章节均标注当前实现状态，便于对照代码

---

## 0. 修订说明

### 0.1 背景与战略决策

`Ayanami_Rei` 是**独立新项目**，采用与既有 `Ikari_Shinji`（Rust Axum + React 19 + SQLite）不同的技术栈（FastAPI + Rust/PyO3 + Jinja2），系有意的栈选择，**非**对 Ikari_Shinji 的替代。两者功能域重合，但本设计不依赖 Ikari_Shinji 代码。

角色命名采用 `admin / operator / viewer`（与 Ikari_Shinji 的 `admin/warehouse/qc/sales` 不同，因本项目范围更聚焦库存本身，质检作为 `quality_mark` 内置于管子记录而非独立角色）。

### 0.2 历史修订摘要（v0.1 → v0.3.0，保留归档）

| 版本 | 关键修正 |
|---|---|
| v0.1 → v0.2 | 审计哈希链 tip 移入 DB（Rust 无状态）；`BEGIN IMMEDIATE` 防 TOCTOU；自实现 CSRF；类型扩展表拆分；`pipe_id` 生成规则；`used_count` 反范式计数；软删除；`workers=1`；JWT + `token_version` 吊销；API 5CT 合规入库审批；xlsx/csv 拆分；Alembic 迁移；盘盈走合规 |
| v0.2 → v0.2.1 | 登录限流/锁号（§7）；初始管理员 env 引导；refresh token 轮换；access token 也存 httpOnly cookie；quality_mark 事务守卫；WAL + 异步驱动 + PyO3 阻塞调用入线程池；导入加固（大小/必需列/编码）；tubes 不硬删语义；审计链增量验证 |
| v0.2.1 → v0.2.2 | refresh 轮换改用 `last_refresh_jti`（`token_version` 仅全局吊销）；补充 Rust 数据结构定义；`rust_auth.init(secret_key)` 启动协议 + fail-fast；`verify_chain_chunk()` 增量验证接口（单批 20K） |

### 0.3 历史修订（v0.2.2 → v0.3.0，2026-07-31 归档）

基于 2026-07-31 对代码库的全量深读，本版做四类修订：

| # | 类型 | 位置 | 变更说明 |
|---|---|---|---|
| 1 | 对齐 | §5.1 | 页面清单标注实现状态：8 个页面未实现（盘点/质量标注/管子详情/库位管理×4/批次查询） |
| 2 | 对齐 | §3.2 | Rust API 签名与实现核对：`verify_token` 标注弃用（Python 未调用）；`get_standard_specs` 从未实现，实际为 Python 读表构造 `StandardSpec` |
| 3 | 对齐 | §3.2、§7 | refresh 轮换实际行为澄清：无 `/refresh` 端点，`last_refresh_jti` 仅在登录时写入；现状为"重放闸"，真轮换列为 D4 缺陷修正 |
| 4 | 对齐 | §7 | CSRF cookie 实际为**未签名裸 token** 双提交（httpOnly + SameSite=Lax），非"签名 cookie"；修正描述 |
| 5 | 对齐 | §5.2 | 导入事务语义修正：实际为**每行独立 BEGIN IMMEDIATE**（行级部分成功），非"单事务可整体回滚"；Excel 导入未实现（仅 CSV） |
| 6 | 对齐 | §9.1 | 上传限制实现方式修正：`RequestBodySizeLimitMiddleware`（raw ASGI，413），非配置覆写 |
| 7 | 对齐 | §9.4 | 初始管理员环境变量名对齐：`ADMIN_USERNAME` / `ADMIN_PASSWORD`（文档原写 INIT_ADMIN_*） |
| 8 | 对齐 | §4.7 | 创世哈希常量位置明确：`src/models.py`（`sha256("AYANAMI_REI_GENESIS")`），非 Rust；Rust `audit.rs` 纯函数接收 prev_hash |
| 9 | 对齐 | §4.7 | `audit_logs.ts` 已知漂移：ORM 模型 NOT NULL vs 迁移 nullable=True |
| 10 | 对齐 | §5.1 | 审计日志页实际路径 `/audit`（文档原写 /audit-logs）；仪表盘同时存在 `/` 与 `/dashboard` |
| 11 | 对齐 | §7 | 登录限流实现细节：IP ≤5 次/60s、用户名 10 次/300s 锁 300s；**锁定时不写审计日志**（文档原写"写入审计日志"） |
| 12 | 对齐 | §5.4、§11 | "出库不足提示临近库位"、"强制入库 compliance_pass=false + forced_intake 审计"均未实现，标注规划中 |
| 13 | 缺陷修正→✅ 已实现 | §7（D1） | viewer 角色越权：tubes 5 写路由补 `require_operator`、audit verify POST 补 `require_admin`、standards 3 POST 内联检查换 `Depends(require_admin)`；新增 14 项角色矩阵测试 |
| 14 | 缺陷修正→✅ 已实现 | §7（D2） | batch JSON 入库 CSRF 断裂：`require_csrf` 增读 `X-CSRF-Token` header（优先于 body），JSON body 加 `isinstance(dict)` 防护，改用 `secrets.compare_digest` |
| 15 | 缺陷修正→✅ 已实现 | §7（D3） | logout 改 `POST /logout`（CSRF 保护），`BEGIN IMMEDIATE` 内清 `last_refresh_jti`；仅 POST（无 GET 兼容页） |
| 16 | 缺陷修正→✅ 已实现 | §7（D4） | 新增 `POST /auth/refresh` 显式轮换端点：校验 refresh → 同事务签发新 access+refresh 并更新 `last_refresh_jti` |
| 17 | 缺陷修正→✅ 已实现 | §2.2（D5） | auth 路径 Rust 调用全部 `anyio.to_thread.run_sync`（security 静默刷新、login、refresh、user_service 哈希、main 种子）；`decode_claims`/`init` 保持同步 |
| 18 | 缺陷修正→✅ 已实现 | §5.1（D6） | `list_tubes` 补 `deleted_at IS NULL` 过滤；新增软删管不出现在列表的测试 |
| 18b | 缺陷修正→✅ 已实现 | §5.2（D7） | CSV 导入行数上限 `MAX_IMPORT_ROWS=10_000` + `ImportRowLimitError`→400；新增 10001/9999 行边界测试 |
| 19 | 补全 | §14 | 新增 API 端点总表（约 30 端点：方法/路径/权限/CSRF/请求/响应/错误码） |
| 20 | 补全 | §15 | 新增统一异常映射表（Conflict→409 / Forbidden→403 / Compliance→422） |
| 21 | 补全 | §16 | 新增运维实操：对账 cron 脚本、备份/恢复脚本（含 7 天保留、busy_timeout、integrity_check 演练）、部署清单与 env 表、构建步骤 |
| 22 | 补全 | §13 | 测试矩阵对齐实测：11 文件 / 56 项（53 通过 + 3 slow 可选），无 conftest、每文件自足 |
| 23 | 结构 | §0.5 | 新增"实现状态总览"对照表，作为版本演进基线（历史章节引用，保留原编号） |
| 24 | 结构 | 全文 | 每章统一标注 `[已实现]/[部分实现]/[未实现]/[缺陷修正-待实现]` |

### 0.4 历史修订（v0.3.0 → v0.4.0，2026-08-01 归档）

基于 2026-08-01 对代码库的**全量逐节核对**（迁移逐列、11 个测试文件逐个、34 处 Rust 调用点、全部路由），本版做同步修正：

| # | 类型 | 位置 | 变更说明 |
|---|---|---|---|
| 1 | 对齐 | §13 | 测试矩阵更新为实测基线：**99 项（96 通过 + 3 slow）**；修正 v0.3.0"共 56 项"与 D1-D7 声称新增 24 项的算术矛盾 |
| 2 | 对齐 | §14 | 角色列同步 D1 落地现状：POST /tubes/stock-out、/transfer、/stock-in、/import → admin, operator（require_operator）；/audit/verify 拆分 GET（任意）/POST（admin）；GET /standards → admin（内联检查，已知偏差）；POST /standards → admin（require_admin 依赖） |
| 3 | 对齐 | §14 | POST /tubes/stock-out 表单字段修正：实际仅 pipe_id + csrf_token（service 支持 remark 但路由未接，移除 `remark?`） |
| 4 | 缺陷修正→✅ | §4.1、§17（D8） | 手动 pipe_id 弱校验（前缀 + 4 段）描述修正；单管表单路径非法 pipe_id 曾冒泡 **500**，现捕获 ValueError → **422 表单重渲染** + 1 项测试 |
| 5 | 对齐 | §5.1、§6、§8 | 前端资源：**无 `src/static/` 目录**，Bootstrap 5.3.3 + Icons 走 jsdelivr CDN；模板实为 base.html + **12 个页面**（含 transfer.html、standards/edit.html） |
| 6 | 对齐 | §15 | 异常表补 `ImportRowLimitError` → 400（D7 落地时漏列） |
| 7 | 缺口记录 | §4.7、§17 | 用户管理（create_user/update_user/revoke/toggle）与 standards 写操作**不入审计链**——§4.7 action 示例含 create_user 系设计意图，实现仅 warehouse 管操作写审计；列入 §17 功能缺口 |
| 8 | 对齐 | 附录 A | AGENTS.md 已同步（v0.4.0，2026-08-01），移除"待同步"备注 |
| 9 | 版本 | §0.1 头部 | 应用版本号 0.2.2 → **0.3.0**（pyproject.toml + main.py），与文档 v0.3.0 对齐；rust-core 保持 0.1.0（独立模块版本） |

### 0.5 历史修订（v0.4.0 → v0.5.0，2026-08-01 归档）

用户管理与标准管理写操作审计落地（D9，对应 spec `docs/superpowers/specs/2026-08-01-audit-gap-design.md`）：

| # | 类型 | 位置 | 变更说明 |
|---|---|---|---|
| 1 | 功能 | §4.7 | 审计 action 清单补全：create_user / update_user / revoke_sessions / toggle_user / create_standard / update_standard / delete_standard；AuditLog.user_id = 操作者 |
| 2 | 功能 | §17 | 移除「用户管理/标准管理写操作审计」缺口行；standards 硬删改为**审计 + 删前快照**（不再「不审计」） |
| 3 | 结构 | §6 | 新增 `warehouse/services/standards_service.py` |
| 4 | 测试 | §13 | 新增 test_admin_audit.py（13 项）；基线 96 → 109 通过 + 3 slow |
| 5 | 版本 | §0.1 | 应用版本号 0.3.0 → 0.4.0 |

### 0.6 本次修订（v0.5.0 → v0.6.0，2026-08-01）

| # | 类型 | 位置 | 变更说明 |
|---|---|---|---|
| 1 | 功能 | §7、§17 | D10 登录锁定审计落地：用户名第 10 次失败触发锁定时写 1 条审计行（action=login_lockout、user_id=NULL=系统事件、detail=username+ip+failures+locked_for）；锁定期间 429 / IP 层 429 / 单次失败均不审计（防刷）；设计文档 §17 缺口移除 |
| 2 | 对齐 | §13.1 | 测试矩阵：12 → 13 文件，基线 112/109+3 → 118/115+3；新增 test_login_lockout_audit.py（6 项） |
| 3 | 版本 | §0.1 头部 | 应用版本 0.4.0 → 0.5.0（pyproject.toml + main.py） |

### 0.7 实现状态总览（2026-08-01 实测基线）

| 设计项 | 状态 | 说明 |
|---|---|---|
| JWT 认证（access 15min + refresh 30d，httpOnly cookie） | ✅ 已实现 | `src/security.py` + `rust_core.auth` |
| 静默刷新（refresh cookie → 新 access） | ✅ 已实现 | `get_current_user`，不轮换 refresh |
| refresh 真轮换（/auth/refresh 端点） | ✅ 已实现 | `POST /auth/refresh`（2026-07-31 落地） |
| CSRF 双提交 | ✅ 已实现 | `require_csrf` + 表单隐藏字段；支持 `X-CSRF-Token` header（batch JSON 已修复） |
| 登录限流（IP + 用户名双层） | ✅ 已实现 | `LoginRateLimiter` |
| 角色权限 | ✅ 已实现 | tubes 写路径 `require_operator`、audit verify + standards 写路径 `require_admin`、users `require_admin`（2026-07-31 补齐） |
| 手动入库 + 合规校验 | ✅ 已实现 | `stock_in` + `rust_compliance.validate_tube` |
| CSV 批量导入 | ✅ 已实现 | `import_service.py`（pandas，20MB 上限，行级部分成功） |
| Excel 批量导入 | ❌ 未实现 | 设计有 openpyxl 路径，代码仅 CSV |
| 按编号出库 | ✅ 已实现 | TOCTOU 守卫 + `problematic` 需 admin |
| 批量筛选出库 | ❌ 未实现 | 仅单根出库 |
| 跨库位调拨 | ✅ 已实现 | `transfer` + used_count 同事务增减 |
| 出库不足提示临近库位 | ❌ 未实现 | §5.4 规划 |
| 库存盘点（/tubes/check） | ❌ 未实现 | §5.3 规划 |
| 质量标注（/tubes/quality） | ❌ 未实现 | 仅入库时可填 quality_mark |
| 管子详情（/tubes/{pipe_id}） | ❌ 未实现 | 规划 |
| 库位管理（/locations*） | ❌ 未实现 | 仅 DEFAULT 种子 + transfer 候选列表 |
| 批次查询（/batches） | ❌ 未实现 | 规划 |
| 用户管理 CRUD + 会话吊销 | ✅ 已实现 | `users` 模块 |
| 审计哈希链（写入同事务 + 20K 分段验证） | ✅ 已实现 | `audit_service._write_audit` + `_verify_chunked` |
| 强制入库（compliance_pass=false + forced_intake 审计） | ❌ 未实现 | §11 规划 |
| used_count 反范式计数 | ✅ 已实现 | 同事务 ±1 + `refresh_used_count` |
| used_count 每日对账 cron | ❌ 未实现 | 需部署方按 §16 配置 |
| 数据备份/恢复脚本 | ❌ 未实现 | 需部署方按 §16 配置 |
| API 5CT 标准管理（/standards） | ✅ 已实现 | CRUD + 删除（参考数据硬删） |

---

## 1. 系统概述

### 1.1 目标

开发一套面向局域网环境的无缝钢管与筛管（符合 API 5CT 标准）单根级别库存管理系统，实现管材的入库、出库、库存查询、盘点等全流程管理。

### 1.2 范围

**本期范围（库存管理）**：

| 功能 | 状态 |
|---|---|
| 单根管材的信息录入（手动 + CSV 批量导入） | ✅ 已实现 |
| 按编号出库 | ✅ 已实现 |
| 按条件筛选批量出库 | ❌ 未实现（规划） |
| 跨库位调拨（从其他库位调货满足出库需求） | ✅ 已实现 |
| 库存查询与筛选（含质量问题/报废管单独筛选） | ⚠️ 部分实现（筛选字段：status/location/steel_grade/quality_mark） |
| 库存盘点 | ❌ 未实现（规划） |
| 问题管/报废管标注与管理 | ⚠️ 部分实现（入库时标记 + 出库/调拨守卫；无独立标注页面） |
| 基础角色权限（管理员、仓库操作员、查看者） | ✅ 已实现（D1 已落地：tubes 写路径 require_operator、audit verify + standards 写路径 require_admin、users require_admin） |
| 操作审计日志（防篡改哈希链） | ✅ 已实现 |

**本期不包含**：
- 采购/销售管理
- 生产质量检测管理（仅做质量标记，不做全项检测报告）
- 全生命周期追溯（后续迭代）

---

## 2. 系统架构

### 2.1 整体架构

```
┌─────────────────────────────────────────────────┐
│                  用户浏览器                        │
│          (管理员 / 仓库操作员 / 查看者)              │
└────────────────────┬────────────────────────────┘
                      │ HTTP（局域网）
┌────────────────────▼────────────────────────────┐
│           Python FastAPI Web 服务                 │
│  路由 / Jinja2 模板渲染 / 表单处理 / 业务编排      │
│  负责：UI交互、CSV导入解析、会话管理               │
│  中间件：RequestBodySizeLimitMiddleware(20MB→413) │
└────────────────────┬────────────────────────────┘
                      │ PyO3 FFI 调用（仅安全敏感逻辑）
┌────────────────────▼────────────────────────────┐
│         Rust 安全核心 (native .so 模块)           │
│  ┌─────────────────────────────────────────┐     │
│  │ auth 模块：密码哈希、JWT 签发/校验        │     │
│  ├─────────────────────────────────────────┤     │
│  │ compliance 模块：API 5CT 合规校验引擎     │     │
│  ├─────────────────────────────────────────┤     │
│  │ audit 模块：哈希链计算、链完整性验证       │     │
│  └─────────────────────────────────────────┘     │
└────────────────────┬────────────────────────────┘
                      │ SQLAlchemy async（仅经 service 层访问）
┌────────────────────▼────────────────────────────┐
│               SQLite 数据库（WAL）                │
│  tubes / seamless_pipe_specs / screen_pipe_specs │
│  users / transactions / transaction_items /      │
│  audit_logs / locations / standard_specs          │
└─────────────────────────────────────────────────┘
```

实现补充说明（对齐 v0.3.0）：
- 模板环境独立为 `src/templates_env.py` 单例（`CompatTemplates` 兼容 Starlette 0.x/1.x 两代 `TemplateResponse` 签名），路由统一 `from src.templates_env import templates`，避免 `main.py` 导入循环。
- 请求体大小限制由 raw ASGI 中间件 `RequestBodySizeLimitMiddleware` 实现：Content-Length 预检 + chunked 流式计数，超 `MAX_UPLOAD_SIZE` 直接 413（在路由/UploadFile 读取之前）。

### 2.2 架构原则

| 原则 | 说明 | 状态 |
|---|---|---|
| **分层隔离** | Python 只做 Web 层和数据库访问；密码哈希、JWT 校验、合规判定、审计哈希计算下沉 Rust | ✅ 已实现 |
| **Rust 安全核心作为强制约定** | 所有密码操作、Token 校验、合规判定、审计哈希计算**必须**经 Rust 模块。⚠️ Python 经 SQLAlchemy 仍物理可访问 DB，"不可绕过"是**架构约定 + code review 约束**而非运行时强制 | ✅ 已实现（约定） |
| **单进程部署** | FastAPI + uvicorn **单 worker** 启动；SQLite 文件数据库（见 §9 部署约束） | ✅ 已实现 |
| **无前端构建** | Jinja2 模板 + Bootstrap 5，无需 Node.js | ✅ 已实现 |
| **异步与阻塞调用隔离** | Rust PyO3 调用为**同步阻塞**，须经 `anyio.to_thread.run_sync` / `run_in_threadpool` 执行，避免阻塞事件循环 | ✅ 已实现 | warehouse 模块（`audit_service`/`tube_service`/`import_service`）正确 offload；**auth 路径已补齐**（D5 落地）：`security.py` 静默刷新、`auth.py` 登录/refresh、`user_service.py` 哈希、`main.py` admin 种子均经 `anyio.to_thread.run_sync`；`decode_claims`（µs 级）与 `rust_auth.init`（启动一次性）保持同步 |

---

## 3. Rust 安全核心模块设计

### 3.1 模块划分

| 模块 | 职责 | 为什么放 Rust |
|---|---|---|
| `auth` | 密码哈希 (argon2)、JWT 签发/校验、token 吊销版本号校验 | 密码与签名安全是红线 |
| `compliance` | API 5CT 标准规格校验（纯函数，无状态） | 合规错误后果严重，强类型保证 |
| `audit` | 哈希链**计算**（纯函数）与**链完整性验证**（纯函数） | 防篡改要求，哈希算法由 Rust 保证一致 |

> 关键约束：Rust 模块**全部为无状态纯函数**（唯一例外：`auth.init()` 内的 `OnceLock` JWT 密钥）。任何"链 tip""会话状态"都不保存在 Rust 内存中（见 §4.7、§12）。

### 3.2 API 接口（Rust 导出给 Python 调用 —— v0.3.0 与实现核对版）

```python
# auth 模块  [已实现]
rust_auth.init(secret_key: str) -> None               # 启动时调用一次；OnceLock 幂等；密钥缺失 fail-fast
rust_auth.hash_password(password: str) -> str         # Argon2id 默认参数 m=19456KiB, t=2, p=1，PHC 编码
rust_auth.verify_password(password: str, hash: str) -> bool   # 畸形/空 hash 返回 False，永不 panic
rust_auth.issue_access_token(user_id: int, role: str, token_version: int) -> str  # HS256, exp=15min, jti=""
rust_auth.issue_refresh_token(user_id: int, token_version: int) -> str            # HS256, exp=30d, jti=UUIDv4
rust_auth.decode_claims(token: str) -> dict | None    # 校验签名+exp；不校验 token_version；非法返回 None
rust_auth.verify_token(token: str, current_token_version: int) -> dict | None     # ⚠️ 弃用：Python 从未调用（死代码），
                                                      #   逻辑由 get_current_user 用 decode_claims+手动比对替代

# compliance 模块  [已实现]（get_standard_specs 从未实现 —— 规格由 Python 从 standard_specs 表读行构造）
rust_compliance.validate_tube(params: TubeParams, spec: StandardSpec) -> ComplianceResult
    # 5 项逐项检查：steel_grade_match（精确）/ wall_thickness_range（[min,max] 闭区间，缺省边跳过）/
    #   outer_diameter_match（相对偏差 ≤ ±1%）/ length_positive（>0）/
    #   screen_specs_valid（仅 screen_pipe：须有 screen_specs 且 slot_width>0 且 filtration_rating>0）

# audit 模块  [已实现]
rust_audit.compute_hash(prev_hash: str, action: str, detail: str, timestamp: str) -> str
    # sha256(prev_hash ‖ action ‖ detail ‖ timestamp) hex；确定性、无失败
rust_audit.verify_chain(logs: list[dict]) -> list[VerifyResult]
    # 全量扫描：逐条重算比对存储值，返回断点列表；无链连续性检查；数据畸形抛 PyValueError
rust_audit.verify_chain_chunk(start_id: int, expected_prev_hash: str, logs: list[dict]) -> ChunkResult
    # 增量验证，单批次硬上限 20K 条（超限 PyValueError）；首条 prev_hash 必须等于 expected_prev_hash，
    # 后续每条 prev_hash 必须等于前一条重算哈希；返回 {ok, broken_index, tip}；start_id 仅占位未用
# 注意：v0.1 的 get_chain_tip() 已移除。tip 由 Python 从 DB 读取（见 §4.7）。
```

```rust
// Rust 数据结构定义（供 compliance 模块）
#[derive(Deserialize)]
pub struct TubeParams {
    pub pipe_type: String,
    pub outer_diameter_mm: f64,
    pub wall_thickness_mm: f64,
    pub length_m: f64,
    pub steel_grade: String,
    pub thread_type: Option<String>,
    pub screen_specs: Option<ScreenPipeParams>,
}

#[derive(Deserialize)]
pub struct ScreenPipeParams {
    pub slot_width_mm: f64,
    pub slot_density: f64,
    pub filtration_rating_um: f64,
    pub slot_pattern: Option<String>,
}

#[derive(Serialize)]
pub struct ComplianceResult {
    pub passed: bool,
    pub checks: Vec<ComplianceCheck>,
}

#[derive(Serialize)]
pub struct ComplianceCheck {
    pub check_name: String,
    pub passed: bool,
    pub reason: Option<String>,
}
```

> **初始化协议**：`rust_auth.init(secret_key: str)` 必须在应用启动时调用一次，用于设置 JWT 签名密钥（HS256）。若密钥缺失，应用应拒绝启动（fail-fast）。

```python
# main.py 启动片段（lifespan 内，与实现一致）
from src.config import settings
from rust_core import auth as rust_auth

async def lifespan(app):
    if not settings.JWT_SECRET_KEY:
        raise RuntimeError("JWT_SECRET_KEY must be set (fail-fast at startup). Set via .env or environment variable.")
    rust_auth.init(settings.JWT_SECRET_KEY)
    # ... 种子 DEFAULT 库位、upsert 管理员（ADMIN_USERNAME/ADMIN_PASSWORD）
```

**Token 设计（v0.3.0 对齐实现）**：
- access token：JWT，HS256，`exp=15min`，payload 含 `sub`(user_id 字符串)/`role`/`token_version`/`jti`(空串)。
- refresh token：JWT，`exp=30d`，payload 含 `sub`/`role`(空串)/`token_version`/`jti`(UUIDv4)，存 httpOnly cookie。
- 吊销（全局）：用户表 `token_version` 整数；`revoke_all_sessions` 时 `token_version += 1` 且清空 `last_refresh_jti`，所有已签发 token 立即失效。无需服务端会话存储。
- 静默刷新 `[已实现]`：`get_current_user` 在 access 失效时校验 refresh（签名+exp+`token_version`+`jti == users.last_refresh_jti`），通过则签发**新 access** 并 set_cookie；**不签发新 refresh、不更新 last_refresh_jti**。
- **refresh 轮换 `✅ 已实现`**：设计意图为"每次使用 refresh 即轮换新 refresh 并更新 `last_refresh_jti`，旧 refresh 立即失效"。**实现（2026-07-31 D4 落地）**：`get_current_user` 静默刷新只换 access（不轮换）；显式 `POST /auth/refresh` 完成真轮换——校验 refresh（active + token_version + `jti==last_refresh_jti`）→ `BEGIN IMMEDIATE` 签发新 access+refresh 并写新 jti → 旧 refresh 因 jti 失配立即失效。

---

## 4. 数据模型设计

> 全模型无 `relationship()` 定义——FK 均为裸列，联表查询在 service 层手写。

### 4.1 管子基础表 (tubes)  `[已实现]`

每一根管子一条记录。共有字段放基础表，**类型特有字段放扩展表**（见 §4.2）。

| 字段 | 类型 | 说明 |
|---|---|---|
| id | INTEGER PK | 自增主键 |
| pipe_id | TEXT UNIQUE NOT NULL | 管子编号（自动生成或手动，规则见下） |
| type | TEXT NOT NULL | seamless_pipe / screen_pipe |
| outer_diameter_mm | REAL NOT NULL | 外径 (mm) |
| wall_thickness_mm | REAL NOT NULL | 壁厚 (mm) |
| length_m | REAL NOT NULL | 长度 (m) |
| steel_grade | TEXT NOT NULL | 钢级 (L80 / N80 / P110 / ...) |
| thread_type | TEXT | 螺纹类型 |
| api_5ct_cert | TEXT | API 5CT 证书号 |
| manufacturer | TEXT NOT NULL | 生产厂家 |
| batch_no | TEXT | 生产批号 |
| heat_no | TEXT | 炉号 |
| heat_treat_method | TEXT | 热处理方法 |
| location | TEXT NOT NULL REFERENCES locations(code) | 当前库位（FK `ON DELETE RESTRICT`，见 §4.6） |
| status | TEXT NOT NULL | in_stock / out_stock / in_transfer |
| quality_mark | TEXT DEFAULT 'normal' | normal / problematic / scrapped（与在库状态独立） |
| quality_note | TEXT | 质量问题说明 |
| stock_in_time | DATETIME | 入库时间 |
| stock_out_time | DATETIME | 出库时间 |
| compliance_pass | BOOLEAN | 是否通过 API 5CT 合规校验 |
| deleted_at | DATETIME NULL | 软删除位。正常业务不置，仅通过 `status` 表达生命周期；仅极端数据纠正场景使用且须写审计 |
| created_at | DATETIME | 创建时间（UTC） |
| updated_at | DATETIME | 更新时间（UTC，onupdate） |

**`pipe_id` 生成规则（对齐实现）**：入库时若未提供则自动生成
`{SP|SCP}-{grade}-{od:.1f}x{wt:.2f}-{uuid8}`，其中 `SP`=seamless、`SCP`=screen，`uuid8` 为 8 位 uuid 短码，`od` 保留 1 位小数、`wt` 保留 2 位小数（printf 风格，如 `88.9x8.00`）。允许手动覆盖，但须满足 UNIQUE；手动值做**弱格式校验**（`_validate_pipe_id_format`：前缀须匹配 `SP-`/`SCP-` 且段数为 4；od/wt/uuid 段内容不校验）。校验失败 → **422 表单重渲染**（v0.4.0 修复，此前冒泡为 500，见 §17 D8）。

### 4.2 类型扩展表  `[已实现]`

**seamless_pipe_specs**（1:1，按 pipe_id）

| 字段 | 类型 | 说明 |
|---|---|---|
| pipe_id | TEXT PK FK | 关联 tubes.pipe_id（ON DELETE CASCADE） |
| surface_treatment | TEXT | 表面处理（如磷化/涂漆） |
| end_protector | TEXT | 管端保护器类型 |

**screen_pipe_specs**（1:1，按 pipe_id）

| 字段 | 类型 | 说明 |
|---|---|---|
| pipe_id | TEXT PK FK | 关联 tubes.pipe_id（ON DELETE CASCADE） |
| screen_type | TEXT | 筛管类型：wire_wrapped / slotted / precision |
| slot_width_mm | REAL | 缝宽 (mm) |
| slot_density | REAL | 缝密度（缝/单位周长） |
| filtration_rating_um | REAL | 过滤精度 (μm) |
| base_pipe_grade | TEXT | 基管钢级（可能与外层 steel_grade 一致） |
| slot_pattern | TEXT | 缝隙排布（如梯形/桥式） |

> 后续新增管材类型时，仅加一张扩展表，不动基础表结构。

### 4.3 用户表 (users)  `[已实现]`

| 字段 | 类型 | 说明 |
|---|---|---|
| id | INTEGER PK | 自增主键 |
| username | TEXT UNIQUE NOT NULL | 登录名（唯一约束 + 应用层预检） |
| password_hash | TEXT NOT NULL | 密码哈希（Rust argon2id PHC 格式） |
| display_name | TEXT NOT NULL | 显示名称 |
| role | TEXT NOT NULL | admin / operator / viewer（**无 DB CHECK**，应用层校验，且仅 users 模块强校验） |
| token_version | INTEGER DEFAULT 0 | 全局吊销计数（revoke_all_sessions 时 +1） |
| last_refresh_jti | TEXT | 末次 refresh token 的 JTI，静默刷新重放闸（见 §3.2） |
| active | BOOLEAN DEFAULT 1 | 是否启用（登录与 get_current_user 均检查） |
| created_at | DATETIME | 创建时间（UTC） |

### 4.4 出入库单据表 (transactions)  `[已实现]`

| 字段 | 类型 | 说明 |
|---|---|---|
| id | INTEGER PK | 自增主键 |
| type | TEXT NOT NULL | in / out / move / check |
| operator_id | INTEGER NOT NULL | 操作人（无 FK，普通整数） |
| reference_no | TEXT | 单据号 |
| remark | TEXT | 备注 |
| created_at | DATETIME | 操作时间（UTC） |

### 4.5 单据明细表 (transaction_items)  `[已实现]`

| 字段 | 类型 | 说明 |
|---|---|---|
| id | INTEGER PK | 自增主键 |
| transaction_id | INTEGER NOT NULL FK | 关联 transactions.id（ON DELETE CASCADE） |
| pipe_id | TEXT NOT NULL | 管子编号（**无 FK**，反范式设计，防级联删管） |
| source_location | TEXT | 来源库位 |
| target_location | TEXT | 目标库位 |

### 4.6 库位表 (locations)  `[已实现]`

| 字段 | 类型 | 说明 |
|---|---|---|
| code | TEXT PK | 库位编码 |
| name | TEXT | 库位名称 |
| used_count | INTEGER DEFAULT 0 | 在库管子数（反范式计数，见 §12） |
| capacity | INTEGER | 容量（可选） |
| is_active | BOOLEAN DEFAULT 1 | 是否启用；停用库位不出现在出库/调拨候选列表 |

> `tubes.location` 通过 FK 约束保证引用完整性：`FOREIGN KEY (location) REFERENCES locations(code) ON DELETE RESTRICT`。`database.py` 的 connect 事件强制 `PRAGMA foreign_keys=ON`（连同 `journal_mode=WAL`、`synchronous=NORMAL`、`busy_timeout=5000`、`wal_autocheckpoint=1000`）。

### 4.7 审计日志表 (audit_logs)  `[已实现]`

| 字段 | 类型 | 说明 |
|---|---|---|
| id | INTEGER PK | 自增主键 |
| user_id | INTEGER | 操作人 |
| action | TEXT NOT NULL | 操作类型（stock_in / stock_out / transfer / create_user / update_user / revoke_sessions / toggle_user / create_standard / update_standard / delete_standard / login_lockout） |
| detail | TEXT | 详细描述 |
| prev_hash | TEXT NOT NULL | 上一条日志的哈希 |
| current_hash | TEXT NOT NULL | SHA-256(prev_hash ‖ action ‖ detail ‖ ts) |
| ts | TEXT NOT NULL* | 确定性 ISO-8601 时间戳字符串（哈希输入之一） |
| created_at | DATETIME | 落库时间（UTC） |

> ⚠️ **已知漂移（v0.3.0 记录）**：ORM 模型声明 `ts` NOT NULL（`models.py`），但迁移 `20260713_084000_add_audit_ts.py` 以 nullable=True 添加。现有行可能为 NULL；service 写入时总填值。后续迁移可补数据回填与 NOT NULL 收紧（见 §17 待办）。
> 数据库层无 FK（user_id 为普通整数）。
> ✅ **实现现状（v0.6.0）**：审计覆盖 warehouse 管操作 + 用户管理 4 操作 + standards 3 写操作（v0.5.0 D9 落地）+ 登录锁定触发（第 10 次失败，`action=login_lockout`、`user_id=NULL`；v0.6.0 D10 落地）。锁定期间 / IP 层 429 / 单次登录失败均不写审计。

**哈希链写入协议（对齐实现）**：
1. `prev_hash` = `SELECT current_hash FROM audit_logs ORDER BY id DESC LIMIT 1`；首条记录用固定创世常量 **`GENESIS_HASH = sha256("AYANAMI_REI_GENESIS")`**，定义于 **`src/models.py`**（`2e3874e8f41f106cc35d41312497bf350acebadc6ab2ebf6728631a9c180cd40`），**不在 Rust**。
2. Python 调用 `rust_audit.compute_hash(prev_hash, action, detail, ts)` 得到 `current_hash`（经 `anyio.to_thread.run_sync`）。
3. 审计写入与对应业务操作**在同一 `BEGIN IMMEDIATE` 事务内**提交（见 §12），杜绝并发插入算出相同 `prev_hash` 导致链分叉。
4. 验证：`verify_chain`（全量）或 `verify_chain_chunk`（增量）逐条重算比对，返回断点列表。
5. **可扩展性**：单次验证上限 20K 条日志；超出用 `verify_chain_chunk(start_id, expected_prev_hash, logs)` 分段，跨段传递 tip（`_verify_chunked` 已在 `/audit/verify` 实现）。定期快照 current_hash 落库（如每月，脚本未实现）。

### 4.8 标准规格表 (standard_specs)  `[已实现]`

| 字段 | 类型 | 说明 |
|---|---|---|
| id | INTEGER PK | |
| grade | TEXT NOT NULL | 钢级（**唯一约束 `uq_standard_specs_grade`**，迁移 3 添加） |
| outer_diameter_mm | REAL | 适用外径 |
| min_wall_thickness_mm | REAL | 该规格允许最小壁厚 |
| max_wall_thickness_mm | REAL | 允许最大壁厚 |
| length_tolerance_mm | REAL | 长度公差 |
| source | TEXT | 数据来源（API 5CT 版本/章节） |

---

## 5. Web 页面设计

### 5.1 页面清单（v0.3.0 对齐实现状态）

| 页面 | 路径 | 设计角色限制 | 实现角色限制（现状） | 状态 |
|---|---|---|---|---|
| 登录页 | /login | 无 | 无 | ✅ 已实现 |
| 登出 | /logout | 无 | POST（CSRF 保护，`BEGIN IMMEDIATE` 清 last_refresh_jti——D3 已落地） | ✅ 已实现 |
| 仪表盘 | / 与 /dashboard | 全部 | 全部（get_current_user） | ✅ 已实现 |
| 管子列表 | /tubes | 全部 | 全部（`deleted_at IS NULL` 已过滤——D6 已落地） | ✅ 已实现 |
| 入库页面 | /tubes/stock-in | admin, operator | admin, operator（require_operator——D1 已落地） | ✅ 已实现 |
| 批量入库(JSON) | /tubes/stock-in/batch | admin, operator | admin, operator（CSRF header 已支持——D2 已落地） | ✅ 已实现 |
| 出库页面 | /tubes/stock-out | admin, operator | admin, operator（require_operator） | ✅ 已实现 |
| 调拨页面 | /tubes/transfer | admin, operator | admin, operator（require_operator） | ✅ 已实现 |
| CSV 导入 | /tubes/import | admin, operator | admin, operator（require_operator；10K 行上限——D7 已落地） | ✅ 已实现 |
| 审计日志 | /audit（设计原写 /audit-logs） | admin | **全部** | ✅ 已实现 |
| 审计链验证 | /audit/verify | admin | 全部（GET+POST 双处理器） | ✅ 已实现 |
| API 5CT 标准 | /standards | 全部 | 全部（写操作**内联 admin 检查**，403） | ✅ 已实现 |
| 用户管理 | /users | admin | admin（require_admin） | ✅ 已实现 |
| 质量标注 | /tubes/quality | admin, operator | — | ❌ 未实现（规划） |
| 盘点页面 | /tubes/check | admin, operator | — | ❌ 未实现（规划） |
| 管子详情 | /tubes/{pipe_id} | 全部 | — | ❌ 未实现（规划） |
| 库位列表/新增/编辑/详情 | /locations* | 见设计 | — | ❌ 未实现（规划） |
| 批次查询 | /batches | 全部 | — | ❌ 未实现（规划） |

> 前端样式：v0.2.x 起全部模板（base.html + **12 个页面**，含 transfer.html、standards/edit.html）采用统一的轻量设计令牌体系（`--accent` indigo、`.surface` 卡片、`.btn-accent` 等），Bootstrap Icons 内联，无 emoji。

### 5.2 入库流程  `[已实现，导入语义与设计有差异]`

```
手动录入:  [已实现]
  [表单] → [Rust compliance 校验]
     ├── 通过 → 写 tubes + 扩展表, status=in_stock, 记录 transaction + 审计
     └── 失败 → 422 提示具体不合规项（ComplianceError，表单重渲染）

批量导入:  [已实现——仅 CSV；Excel(xlsx) 未实现]
  CSV 选择 → pandas 解析（UTF-8 含 BOM 容错, dtype=str）
   → 必需列校验 → 逐行校验（合规 + 字段） → 显示导入结果: 成功/跳过/失败原因
```

**导入加固（v0.3.0 对齐实现）**：
- 文件大小上限 **20MB**：raw ASGI 中间件在 body 解析前拒绝（HTTP 413）+ service 层 `MAX_IMPORT_BYTES` 二次防御。
- 必需列：`pipe_type / outer_diameter_mm / wall_thickness_mm / length_m / steel_grade / location / manufacturer`，任一缺失整行跳过并标注。
- CSV 编码：pandas `dtype=str` 解析，UTF-8 含 BOM 容错。
- **事务语义（对齐实际）**：**每行独立 `BEGIN IMMEDIATE` 事务**——行级部分成功（错误行收集到 `errors` 列表，成功行提交，整体不回滚）。设计原稿"单事务可整体回滚"未实现。**行数上限（2026-07-31 D7 落地）**：`MAX_IMPORT_ROWS = 10_000` 行/次，超限抛 `ImportRowLimitError` → HTTP 400（提示分批导入）；检查发生在逐行处理之前。
- 批量 JSON 入库（`/tubes/stock-in/batch`）：Pydantic `BatchStockInRequest` 上限 **500 条**，逐条调用 `stock_in`，异常入 errors 列表，部分成功。

**合规校验语义（对齐实现）**：`stock_in(compliance_pass=None)` → service 按钢级查 `standard_specs`（无规格抛 `ComplianceError`）→ Rust `validate_tube` → 失败抛 `ComplianceError`（含逐项原因）。调用方传入 `compliance_pass=True/False` 时**跳过** Rust 校验直接持久化（CSV 导入路径预校验后传 True 复用，避免二次校验）。强制入库审批（admin 确认 + `forced_intake` 审计）**未实现**（见 §11）。

### 5.3 盘点流程  `[未实现-规划中]`

```
[选择库位] → [列出在库管子] → [逐根核对]
   ├── 一致 → 已核对
   ├── 实物缺失 → 缺失（生成调整：标记 out/盘亏）
   ├── 实物存在但系统无 → 盘盈 → 录入实际信息 + 走 compliance 校验（同入库）
   └── 位置不对 → 位置异常（触发移库事务）
→ [生成盘点报告] → [管理员确认, 差异项生成调整记录 + 审计]
```

### 5.4 调拨流程  `[已实现（单根）；临近库位提示未实现]`

```
[选目标库位 + 原因] → [确认] → [单根调拨]
→ 单事务: BEGIN IMMEDIATE; UPDATE tubes SET location; rows_affected 校验
→ 同事务：源库位 used_count - 1（AND used_count > 0 防下溢）、目标库位 + 1、写 transaction(type=move) + 审计
守卫：status='in_stock' AND deleted_at IS NULL AND quality_mark != 'scrapped'；
      problematic 需 admin；源=目标 拒绝（409）
```

设计稿中"出库库存不足时提示可用调拨的临近库位（基于 locations.used_count）"**未实现**（规划中）。

### 5.5 出库流程  `[已实现（单根）；批量筛选出库未实现]`

```
按编号: [输入 pipe_id] → [确认] → [备注] → [出库]
批量: [筛选] → [勾选] → [单据号/备注] → [批量出库]   ← 未实现（规划）
所有出库走 §12 的串行事务，防止并发超卖。
```

### 5.6 问题管/报废管管理  `[部分实现]`

`quality_mark` 与 `status` 独立；入库时可标记；`problematic` 出库/调拨需 admin（service 层 `ForbiddenError`）；`scrapped` 不可出库/调拨（守卫 `quality_mark != 'scrapped'`）。**独立标注页面（/tubes/quality）未实现**，盘点单列不计入可用库存（盘点未实现）。

### 5.7 库位管理流程  `[部分实现]`

- **物理删除限制**：已关联管子的库位不可物理删除（FK `ON DELETE RESTRICT`）——约束已实现。
- **停用机制**：`is_active` 标记；停用库位不参与出库/调拨候选列表——**候选列表逻辑已实现（transfer 表单），库位管理页面未实现**。
- **首次部署**：启动 lifespan 幂等种子 `DEFAULT` 库位——已实现。
- **容量监控**：`used_count` + `capacity` 展示——**页面未实现**（数据模型就绪）。

---

## 6. 项目结构（v0.3.0 对齐实际）

```
Ayanami_Rei/
├── pyproject.toml              # 依赖 + pytest 配置（asyncio_mode=auto, markers: slow/requires_rust）
├── rust-core/Cargo.toml        # PyO3 0.23, argon2 0.5, jsonwebtoken 9, sha2 0.10, uuid
├── migrations/                 # Alembic 迁移（flat, timestamped）
│   ├── alembic.ini
│   └── versions/
│       ├── 20260713_083945_initial.py              # 9 表 + 8 索引
│       ├── 20260713_084000_add_audit_ts.py         # audit_logs.ts（batch_alter_table）
│       └── 8ce24f163996_add_unique_on_standard_specs_grade.py  # grade 唯一约束
├── src/
│   ├── main.py                 # App 工厂、lifespan（fail-fast + rust_auth.init + 种子）、
│   │                           # RequestBodySizeLimitMiddleware、异常处理器、register_all_routers
│   ├── config.py               # pydantic-settings（JWT_SECRET_KEY 空默认 → 运行时 fail-fast）
│   ├── database.py             # SQLAlchemy async 引擎 + PRAGMA 监听器 + get_session
│   ├── models.py               # 9 个 ORM 模型 + GENESIS_HASH
│   ├── schemas.py              # Pydantic v2 请求/响应 schema
│   ├── security.py             # JWT cookie 认证、require_roles/admin/operator、CSRF、LoginRateLimiter
│   ├── templates_env.py        # CompatTemplates 单例（Starlette 0.x/1.x 签名兼容）
│   ├── modules/
│   │   ├── __init__.py         # register_all_routers(app)（硬编码 5 个 router）
│   │   ├── auth/routers/auth.py
│   │   ├── warehouse/
│   │   │   ├── routers/ (tubes.py, audit.py, standards.py)
│   │   │   └── services/ (tube_service.py, import_service.py, audit_service.py, standards_service.py)
│   │   └── users/
│   │       ├── routers/users.py
│   │       └── services/user_service.py
│   ├── templates/              # 按模块分组：base, auth/, dashboard/, warehouse/, users/, audit/, standards/
│   │                           # （无 static/ —— Bootstrap 5.3.3 + Icons 走 jsdelivr CDN，见 §8）
├── rust-core/src/ (lib.rs, auth.rs, compliance.rs, audit.rs)   # 保持扁平（跨领域纯函数）
├── tests/                      # 13 个测试文件（见 §13）
└── docs/
```

未来模块：`src/modules/sales/`（销售）、`procurement/`（采购）、`qc/`（质检）—— 各为 modules/ 下的同级目录，在 `__init__.py` 的 `register_all_routers` 中追加一行。

---

## 7. 安全设计（v0.3.0 对齐 + 缺陷修正）

| 安全措施 | 实现位置 | 状态 | 说明 |
|---|---|---|---|
| 密码 argon2 哈希 | Rust auth | ✅ 已实现 | Argon2id 默认参数（m=19456KiB, t=2, p=1） |
| JWT 签发/校验 + 全局吊销 | Rust auth + users.token_version | ✅ 已实现 | 吊销经 `revoke_all_sessions` |
| 防篡改审计（哈希链） | Python 调 Rust audit（纯函数） | ✅ 已实现 | tip 状态在 DB（§4.7） |
| API 5CT 合规校验 | Rust compliance | ✅ 已实现 | 纯函数，5 项检查 |
| 角色校验 | FastAPI 依赖注入 | ✅ 已实现 | users 模块 require_admin；tubes 写路径 require_operator；audit verify + standards 写路径 require_admin（2026-07-31 D1 落地） |
| CSRF 防护 | 自实现 `require_csrf` | ✅ 已实现 | 表单路径正常；支持 `X-CSRF-Token` header（batch JSON 已修复，2026-07-31 D2 落地） |
| 登录限流 | 自实现 `LoginRateLimiter` | ✅ 已实现 | 双层：IP ≤5 次/60s（429）；用户名 10 次/300s → 锁 300s。锁定触发（第 10 次失败）写 1 条 `login_lockout` 审计（`user_id=NULL`）；锁定期间 / IP 层 429 / 单次失败不写审计 |
| Cookie 会话 | httpOnly 签名 cookie | ✅ 已实现 | access+refresh 均 httpOnly + SameSite=Lax；**无 secure**（LAN 部署有意）；**无 max-age**（浏览器会话级，有效期靠 JWT exp） |
| 登出 | POST /logout | ✅ 已实现 | `BEGIN IMMEDIATE` 内清 `last_refresh_jti`（已发 refresh 立即失效）+ 删 cookie（2026-07-31 D3 落地） |
| SQLite 文件权限 | OS 层 | ✅ | — |

> 关于"Rust 不可绕过"：因 Python 持有 DB 连接，运行时无法强制所有写经 Rust。本期以**约定 + code review + 集成测试**保证；若未来需运行时强制，应将 DB 写收敛到 Rust 暴露的存储过程接口（不在本期范围）。

### D1 — 角色门禁缺陷（✅ 已实现 2026-07-31）

**现状**：`require_operator`（security.py 定义）曾经是**死代码**，全仓库无引用。tubes.py / standards.py / audit.py 的所有 POST 路由只挂 `get_current_user` + `require_csrf`，即 **viewer 角色可以执行入库、出库、调拨、CSV 导入、审计验证**。standards.py 用内联 `HTTPException(403)` 检查 admin（未复用依赖）。

**修正设计（已落地）**：
| 端点组 | 修正后权限 |
|---|---|
| GET /tubes、/audit、/standards、/、/dashboard | 全部（viewer 可读） |
| POST /tubes/stock-in、/stock-out、/transfer、/import、/stock-in/batch | admin, operator（`require_operator`） |
| POST /standards*（含 delete） | admin（`require_admin` 替换内联检查） |
| POST /audit/verify | admin（设计意图） |
| /users* | admin（现状保持） |

落地细节：tubes.py 5 个写路由补 `Depends(require_operator)`；audit.py 拆分 GET/POST 双处理器，POST /audit/verify 挂 `Depends(require_admin)`（GET 保持开放）；standards.py 3 个 POST 内联检查替换为 `Depends(require_admin)`（GET 保留内联检查，与既有测试断言一致）。新增 14 项角色矩阵测试（viewer→403、operator 允许 tubes/禁止 audit verify、admin 全通过）。偏差：设计表"GET /standards → 全部"未落地（GET 仍 admin 内联检查），因既有测试 `test_get_standards_operator_returns_403` 断言 operator→403；如需开放可后续改测试。

### D2 — batch JSON 入库 CSRF 断裂（✅ 已实现 2026-07-31）

**现状**：`import.html` 的批量 JSON 前端以 `fetch` 发送 `X-CSRF-Token` header + 裸 JSON 数组 body；而 `require_csrf` **只读 body 的 `csrf_token`（application/json）或表单字段**，从不读 header → 浏览器端批量入库永远 403（body 为数组时还会 `AttributeError` → 500）。测试因 stub `require_csrf` 未暴露。

**修正设计（方案 A 已落地）**：
- **A** ✅：`require_csrf` 增加对 `X-CSRF-Token` header 的读取（在 body 读取之前），与 cookie 恒定时间比对；JSON 路径优先 header、回退 body 字段（兼容现有 API 调用）。JSON body 读取加 `isinstance(body, dict)` 防护（裸数组 body 拒绝为 403 而非 500）；比对改用 `secrets.compare_digest`。
- **B**：前端改发 `{"tubes": [...], "csrf_token": "..."}` 结构（未采用）。

### D3 — logout 不吊销（✅ 已实现 2026-07-31）

**现状**：`GET /logout` 只删 cookie；被截获的 refresh cookie 在 30 天内仍有效（`last_refresh_jti` 未清、`token_version` 未 bump）。

**修正设计（已落地）**：改 `POST /logout`（`Depends(get_current_user)` + `Depends(require_csrf)`）：`BEGIN IMMEDIATE` 内 `last_refresh_jti = NULL`（使已发 refresh 全部失效——`get_current_user` 要求 `jti == last_refresh_jti` 且非 NULL，天然拒绝），**不 bump `token_version`**（避免误伤其它已登录会话；如需全端登出仍走 revoke）。**未保留 GET 兼容页**（GET /logout 现为 405，不再存在安全登出入口——与任务指示一致，偏差于设计原稿）。base.html 登出链接改为 POST 表单 + csrf hidden field。**权衡**：jti 清空后该用户 refresh 静默刷新失效，access 15min 内仍有效（可接受）；无需引入服务端黑名单。新增 3 项测试（吊销后旧 refresh→401、缺 CSRF→403、未认证→401）。

### D4 — refresh 真轮换（✅ 已实现 2026-07-31）

**现状**：曾无 `/refresh` 端点；`last_refresh_jti` 仅登录时写；`get_current_user` 静默刷新只换 access。

**修正设计（已落地）**：
```
POST /auth/refresh  [Depends(require_csrf)]
  → 读 refresh cookie → decode_claims（签名+exp）
  → 单连接 BEGIN IMMEDIATE：校验用户 active + token_version + jti==last_refresh_jti
  → 签发新 access + 新 refresh（新 jti），UPDATE users SET last_refresh_jti=新jti
  → 303 → /dashboard + set_cookies；旧 refresh 因 jti 失配立即失效
```
采用"保持现状静默刷新（只换 access）+ 显式 `/refresh` 端点轮换"方案（与设计建议一致）。校验失败一律 401。新增 4 项测试（轮换成功+旧 token 401、stale 拒绝、缺 CSRF→403、缺 refresh cookie→401）。

### D5 — auth 路径 Rust 调用线程池化（✅ 已实现 2026-07-31）

**现状**：`security.py`（`verify_password`/`decode_claims`）、`auth.py` 登录、`user_service.py`（`hash_password`）、`main.py` 启动曾为**直接同步调用**。argon2 verify（m=19MiB，~30-100ms）在单 worker 事件循环上阻塞所有并发请求。warehouse 模块已正确使用 `anyio.to_thread.run_sync`。

**修正设计（已落地）**：auth 路径全部改 `await anyio.to_thread.run_sync(...)`：security.py 静默刷新 `issue_access_token`、auth.py 登录 `verify_password` + `issue_auth_tokens`、refresh 端点 `issue_auth_tokens`、user_service `create_user`/`update_user` 的 `hash_password`、main.py lifespan admin 种子 `hash_password`。`decode_claims`（~µs）与 `rust_auth.init`（启动一次性）保持同步。同步函数签名不变。测试无需改动（函数签名未变）。

---

## 8. 技术选型

| 组件 | 选择 | 理由 |
|---|---|---|
| Web 框架 | FastAPI | 异步、类型校验、OpenAPI |
| 模板 | Jinja2（CompatTemplates） | 无前后端分离 |
| CSS | Bootstrap 5.3.3（jsdelivr CDN） | 无需构建，无本地静态资源 |
| 数据库 | SQLite（WAL） | 局域网零运维 |
| ORM | SQLAlchemy 2.0（async） | 成熟 |
| 迁移 | Alembic | 管理 schema 演进 |
| CSV | pandas | 解析健壮（含 BOM 容错） |
| Excel | openpyxl（xlsx） | 设计选型；**本期未实现**（未来补 xlsx 导入时启用） |
| Rust 集成 | maturin + PyO3 | 构建原生模块 |
| 密码学 | Rust argon2 + sha2 + jsonwebtoken | 标准，下沉安全核心 |

---

## 9. 部署方案

### 9.1 启动（对齐实现）

```bash
# 构建 Rust 模块（仅构建时）
maturin develop --release

# 开发/生产（⚠️ 必须单 worker：SQLite 单写者 + Rust 无状态）
uvicorn src.main:app --host 0.0.0.0 --port 8000 --workers 1
```

SQLite PRAGMA（`database.py` 连接事件自动设置）：`journal_mode=WAL`、`synchronous=NORMAL`、`foreign_keys=ON`、`busy_timeout=5000`、`wal_autocheckpoint=1000`。批量导入等写入密集场景可周期 `PRAGMA wal_checkpoint(TRUNCATE)` 回收 WAL 空间。

### 9.2 环境要求

- Python 3.11+
- Rust 工具链（仅构建时）
- OS：Linux / Windows / macOS

### 9.3 数据备份

备份与恢复脚本（未随仓库提供，部署方按本设计部署——见 §16）：

- 使用 `sqlite3 .backup`（**禁止 `cp`**），设置 `busy_timeout=5000` 防误判死锁。
- 保留策略：最近 7 天每日备份 + 4 周每周快照。
- 每月恢复演练：还原至临时目录 → `PRAGMA integrity_check` 验证。
- 审计哈希链定期 `verify_chain`（可调用 `/audit/verify`）。

### 9.4 初始管理员引导（v0.3.0 对齐实现）

首次部署无用户。启动 lifespan 从环境变量 **`ADMIN_USERNAME` / `ADMIN_PASSWORD`**（v0.2.2 文档误写 `INIT_ADMIN_USER/INIT_ADMIN_PASS`）读取，幂等 upsert 首个 admin（已存在则重置其密码哈希）；**不写死默认密码**。两变量同时提供才执行。管理员创建后建议清理环境变量（当前版本未自动清理，部署方注意）。

### 9.5 部署清单（新增）

| 步骤 | 命令/说明 |
|---|---|
| 1. 安装 Python 依赖 | `pip install -e .`（导入功能需 `pip install -e ".[imports]"`） |
| 2. 构建 Rust 模块 | `maturin develop --release` |
| 3. 配置环境变量 | `.env`：`JWT_SECRET_KEY`（必填，>=32 字符随机串）、`DATABASE_URL`（默认 `sqlite+aiosqlite:///./data/inventory.db`）、`ADMIN_USERNAME`/`ADMIN_PASSWORD`（首启可选）、`MAX_UPLOAD_SIZE`（默认 20MB） |
| 4. 执行迁移 | `alembic upgrade head` |
| 5. 启动 | `uvicorn src.main:app --host 0.0.0.0 --port 8000 --workers 1` |
| 6. 配置每日对账 cron | 见 §16.1 |
| 7. 配置备份 cron | 见 §16.2 |
| 8. 验证 | 登录 → 建标准规格 → 入库 → 审计链验证 |

---

## 10. 后续扩展

| 迭代 | 功能 |
|---|---|
| v0.3（本版） | 设计文档对齐与缺陷修正（D1-D7 已全部落地，2026-07-31） |
| v0.4 | 生产质量检测管理（qc 模块） |
| v0.5 | 采购与销售管理（procurement/sales 模块） |
| v0.6 | 全生命周期追溯 |

---

## 11. 附录：compliance 标准数据来源

- 依据 **API 5CT 第 10 版**（如适用请注明具体版本）。
- `standard_specs` 表由脚本从策展数据集（CSV/JSON，附来源章节）种子化；校验返回**逐项**通过/不通过及原因。
- 强制入库（`compliance_pass=false`）需 **admin 确认**并写入审计日志（action=`forced_intake`）——**未实现**（见 §17 待办）。

---

## 12. 并发与库存一致性设计

### 12.1 串行事务模式（防 TOCTOU 超卖）  `[已实现]`

所有改变管子状态的操作（出库 / 调拨 / 质量导致不可用）使用：

```
BEGIN IMMEDIATE;
  -- 1. 锁定并校验（守卫谓词三件套）
  UPDATE tubes SET status='out_stock', stock_out_time=:t
    WHERE pipe_id=:pid AND status='in_stock' AND deleted_at IS NULL
      AND quality_mark != 'scrapped';   -- 报废管不可出库
  -- 2. 检查影响行数
  IF rows_affected != 1 → ROLLBACK + ConflictError（已被他人移动/出库，或该管已报废）
  -- 2b. quality_mark='problematic' 需管理员：service 层 role 检查，否则 ForbiddenError
  -- 3. 更新 locations.used_count（同事务内，防下溢：AND used_count > 0）
  -- 4. 写入 transaction + transaction_items
  -- 5. 写入 audit_logs（prev_hash 取当前 tip，同事务）
COMMIT;
```

批量出库：同一事务内对选中集合逐一执行守卫，任一失败则整体回滚（**批量出库未实现**；当前单根路径即上述模式）。并发验证：`asyncio.gather` 5 并发出库 → 恰 1 成功 + 4 ConflictError（测试已覆盖）。

### 12.2 used_count 维护  `[已实现]`

`locations.used_count` 为反范式计数，每次移库/出入库后在**同一事务**内 `±1`（`refresh_used_count(location)` 重算 `COUNT` 已实现，供对账修复）。查询库存余量直接读 `used_count`，避免大表 `COUNT(*)`。

### 12.2.1 used_count 对账机制  `[脚本未随仓库提供——部署方按 §16.1 配置]`

每日 cron 执行（SQL 见 §16.1）：`HAVING l.used_count != actual_count` 查出漂移库位 → 调用 `refresh_used_count` 修正 + 记录偏差值用于根因分析。

### 12.3 审计写入串行化  `[已实现]`

审计写入与业务操作同事务（§4.7），保证 `prev_hash` 取自已提交的最新 tip，杜绝并发插入算出相同 `prev_hash` 导致链分叉。验证性能见 §4.7 第 5 条（20K 增量/快照）。

---

## 13. 测试与迁移策略

### 13.0 Alembic 迁移约束（SQLite）  `[已实现]`

SQLite ALTER TABLE 功能极其有限，所有结构性迁移（添加 NOT NULL 列、添加 FK、删除列、修改列）**必须**使用 `with op.batch_alter_table("table_name")`：

```python
def upgrade():
    with op.batch_alter_table("tubes") as batch_op:
        batch_op.add_column(sa.Column("new_field", sa.String(), nullable=False, server_default=""))
        batch_op.alter_column("existing_col", nullable=False)
```

初始迁移无此问题（直接 `create_table` 创建全部表）。现有迁移链：`20260713_083945_initial`（9 表 8 索引）→ `20260713_084000_add_audit_ts`（audit_logs.ts）→ `8ce24f163996`（standard_specs.grade 唯一约束）。已知漂移：audit_logs.ts nullable 不一致（§4.7）。

### 13.1 测试矩阵（v0.6.0 实测基线，2026-08-01）

**13 个测试文件，无 conftest（每文件自足：独立 test DB、`sys.path.insert`、env 预置、依赖覆盖 stub）**。pytest 配置：`asyncio_mode=auto`，markers `slow` / `requires_rust`（后者注册未用）。共 **118 项：115 通过 + 3 项 slow**（`-m "not slow"` 可跳过 21K 行审计压力测试；基线：115 passed, 3 deselected）。

| 文件 | 覆盖内容 |
|---|---|
| test_security.py | JWT 401/登录/静默刷新/CSRF 403-200/LoginRateLimiter（IP 5 次、用户名 10 次锁定） |
| test_concurrent_stockout.py | 5 并发出库 → 1 成功 4 ConflictError；used_count 递减；problematic 需 admin |
| test_audit_route.py | /audit 401/200；GET+POST /audit/verify；CSRF 403 |
| test_audit_chain_20k.py | 3× slow：21K 行跨 20K chunk 边界验证（完整/篡改/首断即停） |
| test_panic_recovery.py | Rust 边界：畸形 hash/JWT → False/None；chunk>20K → ValueError；DB 后续可用 |
| test_import_http.py | CSV 导入：200/413（>20MB）/400（缺列）/逐行错误收集 |
| test_routes_protection.py | 路由矩阵：/tubes 401-200-200；/users 403（operator）；/standards CRUD 全链路 + CSRF |
| test_admin_audit.py | 管理写操作审计：用户 4 操作 + standards 3 操作 → 审计行；失败路径零审计；密码哈希不入 detail；混合操作后链验证 ok |
| test_login_lockout_audit.py | 登录锁定触发审计：10 次失败写 1 行（user_id NULL）/9 次不写/锁定后 429 不新增/IP 层不写/链完整/成功清除 |
| test_tubes_routes.py | stock-in/out/batch/transfer 全链路：303/409/TOCTOU/CSRF 缺失 403/pipe_id 正则/手动 pipe_id 非法 → 422 重渲染（2026-08-01 新增） |
| test_stock_in_compliance.py | 合规门：通过持久化 True；越界 → ComplianceError 原子回滚；未知钢级；compliance_pass=True 跳过 |
| test_upload_limit.py | 中间件：超限 413、恰限值不 413 |
| test_used_count.py | 入库/出库/调拨后 used_count == COUNT(*)；refresh_used_count 修复漂移 |

> 已知测试盲区：~~viewer 越权（D1 修正后需补）~~ ✅ 已补 14 项、~~batch JSON CSRF（D2 修正后需补真实依赖测试）~~ ✅ require_csrf header 路径已实现（测试 stub 依赖的覆盖策略保留）、~~auth 路径线程池（D5）~~ ✅ 函数签名未变，测试无需改动。剩余盲区：无 viewer→403 的 standards 读路径断言（GET 仍 admin 内联检查，属既有行为）。

---

## 14. API 端点总表（新增，v0.3.0）

> 约定：除登录/登出外全部要求 `get_current_user`；写方法（POST/PUT/DELETE）要求 `require_csrf`；错误响应 JSON `{"detail": "..."}`（SSR 表单错误为 422 重渲染或 409/403 页）。角色列 = 实现现状（D1 已落地）。

### 认证

| 方法 | 路径 | 角色 | 请求 | 响应/错误 |
|---|---|---|---|---|
| GET | /login | 无 | — | 200 登录表单（set CSRF cookie） |
| POST | /login | 无 | form: username, password, csrf_token | 303 → /dashboard + auth cookies；401 凭证错；429 限流 |
| POST | /logout | 任意 | form: csrf_token | 303 → /login（POST + CSRF，`BEGIN IMMEDIATE` 清 last_refresh_jti——D3 已落地） |
| GET | / | /dashboard | 任意 | 200 仪表盘 |

### 仓库（warehouse）

| 方法 | 路径 | 角色（现状） | 请求 | 响应/错误 |
|---|---|---|---|---|
| GET | /tubes | 任意 | query: status, location, steel_grade, quality_mark, page | 200 tubes_list（分页 20，`deleted_at IS NULL` 已过滤——D6 已落地） |
| GET | /tubes/stock-out/{pipe_id} | 任意 | — | 200 出库确认表单 |
| POST | /tubes/stock-out | admin, operator | form: pipe_id, csrf_token | 303 → /tubes；409 ConflictError（竞争/报废/已出）；403 ForbiddenError（problematic 非 admin） |
| GET | /tubes/transfer/{pipe_id} | 任意 | — | 200 调拨表单；404 管不存在 |
| POST | /tubes/transfer | admin, operator | form: pipe_id, target_location, remark?, csrf_token | 303 → /tubes；409（同库位/非在库/竞争）；403（problematic 非 admin） |
| GET | /tubes/stock-in | 任意 | — | 200 入库表单（set CSRF cookie） |
| POST | /tubes/stock-in | admin, operator | form: type, outer_diameter_mm, wall_thickness_mm, length_m, steel_grade, location, manufacturer, pipe_id?, thread_type?, api_5ct_cert?, batch_no?, heat_no?, heat_treat_method?, quality_mark?, quality_note?, seamless_specs?, screen_specs?, csrf_token | 303 → /tubes；409（pipe_id 重复/库位停用）；422（合规失败或 pipe_id 格式非法，表单重渲染）；413（超限） |
| POST | /tubes/stock-in/batch | admin, operator | JSON: `{"tubes": [StockInRequest...]}`（1-500 条）+ csrf_token（body 或 X-CSRF-Token header——D2 已落地） | 200 `{ok, errors:[{index,error,pipe_id}]}`（部分成功） |
| GET | /tubes/import | 任意 | — | 200 导入页 |
| POST | /tubes/import | admin, operator | multipart: file(CSV), csrf_token | 200 结果页（逐行错误）；413 超 20MB；400 ImportFormatError / ImportRowLimitError |
| GET | /audit | 任意 | — | 200 最近 50 条审计日志 |
| GET | /audit/verify | 任意 | — | 200 验证页（空链也 200） |
| POST | /audit/verify | admin | form: csrf_token | 200 verify_result {ok, broken, verified_total} |
| GET | /standards | admin（内联检查，已知偏差：设计"全部"未落地） | query: grade, page | 200 规格列表 |
| POST | /standards | admin（require_admin 依赖） | form: steel_grade, outer_diameter_mm?, min_wall_thickness_mm?, max_wall_thickness_mm?, csrf_token | 303 → /standards；409 钢级已存在 |
| GET | /standards/{spec_id}/edit | admin | — | 200 编辑表单；404 |
| POST | /standards/{spec_id} | admin | form: 同上字段 | 303；409 重复钢级 |
| POST | /standards/{spec_id}/delete | admin | form: csrf_token | 303（**参考数据硬删**，有意为之） |

### 用户（users，全部 require_admin）

| 方法 | 路径 | 请求 | 响应/错误 |
|---|---|---|---|
| GET | /users | — | 200 用户列表 |
| GET | /users/create | — | 200 创建表单 |
| POST | /users/create | form: username, password, display_name, role, csrf_token | 303；409 UserConflictError；400 ValueError |
| GET | /users/edit/{user_id} | — | 200 编辑表单；404 |
| POST | /users/edit/{user_id} | form: display_name?, role?, password?(空=不变), csrf_token | 303；404 |
| POST | /users/revoke/{user_id} | form: csrf_token | 303（token_version+1, last_refresh_jti=NULL）；404 |
| POST | /users/toggle/{user_id} | form: csrf_token | 303；400 LastAdminError；404 |

---

## 15. 统一异常映射（新增）

| 异常 | HTTP | 触发场景 | 处理位置 |
|---|---|---|---|
| `ConflictError` | 409 | TOCTOU 竞争失败、pipe_id 重复、库位停用、同库位调拨 | main.py 全局 handler → JSON；表单路径 service 内捕获重渲染 |
| `ForbiddenError` | 403 | problematic 非 admin 出库/调拨 | 同上 |
| `ComplianceError` | 422 | 合规校验失败、无匹配钢级规格 | 同上（stock_in 表单 422 重渲染） |
| `UserConflictError` | 409 | 用户名已存在 | users 路由捕获 |
| `LastAdminError` | 400 | 停用最后一个 admin | users 路由捕获 |
| `ImportTooLargeError` | 413 | CSV > 20MB | import 路由捕获 |
| `ImportFormatError` | 400 | CSV 解析失败/缺必需列 | import 路由捕获 |
| `ImportRowLimitError` | 400 | CSV 行数 > 10,000（D7） | import 路由捕获 |
| HTTPException 401 | 401 | 未认证 | security.py 抛出 |
| HTTPException 403 | 403 | CSRF 失败 / require_roles 不通过 | security.py 抛出 |
| HTTPException 429 | 429 | 登录限流 | auth 路由抛出 |
| HTTPException 413 | 413 | 任意端点请求体超限 | 中间件（路由前） |

---

## 16. 运维脚本（新增，部署方配置）

### 16.1 used_count 每日对账 cron

```bash
#!/usr/bin/env bash
# /usr/local/bin/ayanami-reconcile.sh — 每日 02:00 执行
# 1) 找出漂移库位；2) 调用 refresh_used_count 修复；3) 记录偏差
DB="$(cd "$(dirname "$0")/.." && pwd)/data/inventory.db"
LOG=/var/log/ayanami/reconcile.log
DRIFT=$(sqlite3 -cmd ".timeout 5000" "$DB" \
  "SELECT l.code, l.used_count, COUNT(t.id)
   FROM locations l
   LEFT JOIN tubes t ON t.location = l.code AND t.status='in_stock' AND t.deleted_at IS NULL
   GROUP BY l.code
   HAVING l.used_count != COUNT(t.id);")
if [ -n "$DRIFT" ]; then
  echo "$(date -Is) DRIFT: $DRIFT" >> "$LOG"
  # 修复：对每个漂移库位调用 API（需 admin 会话）或直接用 python 调 refresh_used_count
  # 推荐：python -c "import asyncio, src.database, ...; asyncio.run(...)"
fi
# crontab: 0 2 * * * /usr/local/bin/ayanami-reconcile.sh
```

> 注意：`refresh_used_count` 在应用 service 层（需 Python 环境与 JWT_SECRET_KEY 配置），脚本应复用应用 venv 执行，或提供独立 CLI 入口（见 §17 待办建议）。

### 16.2 数据备份/恢复 cron

```bash
#!/usr/bin/env bash
# /usr/local/bin/ayanami-backup.sh — 每日备份 + 7 天保留 + 每周快照保留 4 周
DB="$(cd "$(dirname "$0")/.." && pwd)/data/inventory.db"
BK=/var/backups/ayanami
mkdir -p "$BK/daily" "$BK/weekly"
DAY=$(date +%F); DOW=$(date +%u)
# ⚠️ 必须用 .backup（禁止 cp）；busy_timeout=5000 防误判死锁
sqlite3 -cmd ".timeout 5000" "$DB" ".backup '$BK/daily/inventory_$DAY.db'"
if [ "$DOW" = "7" ]; then cp "$BK/daily/inventory_$DAY.db" "$BK/weekly/inventory_$DAY.db"; fi
# 保留策略：日备份 7 天、周快照 4 周
find "$BK/daily"   -name 'inventory_*.db' -mtime +7 -delete
find "$BK/weekly"  -name 'inventory_*.db' -mtime +28 -delete
# crontab: 30 1 * * * /usr/local/bin/ayanami-backup.sh
```

**每月恢复演练**：
```bash
RESTORE_DIR=$(mktemp -d)
sqlite3 -cmd ".timeout 5000" "$BK/daily/inventory_$(date -d '-1 day' +%F).db" \
  ".backup '$RESTORE_DIR/restore_test.db'"
sqlite3 "$RESTORE_DIR/restore_test.db" "PRAGMA integrity_check;"   # 期望: ok
# 验证后删除临时目录
```

---

## 17. 已知缺口与待办清单（新增）

### 缺陷修正（D1-D8 已全部落地 ✅，留档备查）

| ID | 缺陷 | 修正方案 | 落地位置/状态 |
|---|---|---|---|
| D1 | viewer 越权（warehouse 写路径无角色门禁） | `require_operator`/`require_admin` 补齐 + 测试矩阵 | ✅ tubes 5 写路由 + audit verify + standards 3 POST；14 项测试 |
| D2 | batch JSON 入库 CSRF 断裂（X-CSRF-Token header 被忽略） | require_csrf 支持 header（推荐）或前端改 body | ✅ header 优先 + dict 防护 + compare_digest |
| D3 | logout 不吊销 refresh | POST /logout + last_refresh_jti=NULL（BEGIN IMMEDIATE） | ✅ POST + CSRF + jti 清空；3 项测试 |
| D4 | refresh 无真轮换 | 新增 /refresh 端点（显式轮换） | ✅ POST /auth/refresh；4 项测试 |
| D5 | auth 路径 Rust 调用阻塞事件循环 | anyio.to_thread.run_sync 全覆盖 | ✅ security/auth/user_service/main 全覆盖 |
| D6 | list_tubes 显示软删管 | 补 `deleted_at IS NULL` 过滤 | ✅ tubes.py list_tubes；1 项测试 |
| D7 | CSV 导入无行数上限 | 设计上限（建议 10K 行/次，超限拒绝并提示分批） | ✅ MAX_IMPORT_ROWS=10_000 + ImportRowLimitError→400；2 项测试 |
| D8 | 手动 pipe_id 非法 → 500 | 单管入库路由捕获 `ValueError` → 422 表单重渲染 | ✅ tubes.py do_stock_in（2026-08-01）；1 项测试 |

### 功能缺口（规划中）

| 缺口 | 说明 |
|---|---|
| 盘点 /tubes/check（含盘盈走合规） | §5.3 流程设计就绪 |
| 质量标注 /tubes/quality | 独立页面 |
| 批量筛选出库 | §5.5 |
| 管子详情 /tubes/{pipe_id} | 含操作历史 |
| 库位管理 /locations*（4 页） | 数据模型就绪 |
| 批次查询 /batches | batch_no/heat_no 追溯 |
| Excel 导入 | openpyxl 路径（选型已定） |
| 强制入库审批（forced_intake 审计） | §11 |
| 出库不足临近库位提示 | §5.4 |
| used_count 对账 CLI 入口 | 独立命令（免 HTTP 会话） |
| audit_logs.ts 回填 + NOT NULL 收紧迁移 | §4.7 漂移修复 |
| 审计哈希链月度快照 | 定期落库 |

### 已知实现取舍（有意为之，勿改）

- `standards` 删除为**硬删**（审计 + 删前快照，v0.5.0 起）。
- `transaction_items.pipe_id` 无 FK（防级联删管）。
- Cookie 无 `secure=True`（LAN 部署）；部署方若上 HTTPS 应开启。
- CSV 导入行级部分成功（非整体回滚）——设计已对齐。
- `verify_token` Rust API 弃用保留（兼容外部调用，Python 不引用）。

---

## 附录 A：参考文档

- 本设计文档（v0.6.0，2026-08-01；D1-D10 已实现，代码与文档全量核对同步）
- `AGENTS.md`（项目约定速查，v0.6.0 同步更新：角色门禁现状、logout 改 POST、/auth/refresh、登录锁定审计、115 项测试基线）
- 迁移脚本：`migrations/versions/`
- 测试：`tests/`
