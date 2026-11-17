# Ayanami_Rei / SteelPipe 重构参考文档

> **已作废（2026-11-17）**：本文件是「渐进重构」方案，方向已被否决。现行方案见 `docs/2026-11-17-rewrite-reference.md`（推倒重写 + Rust 单体 + 前后端分离）。保留本文件仅作决策留档。
>
> 版本：v1.0（2026-11-17）
> 性质：重构前置分析，不含代码改动。业务语义仍以 `docs/2026-07-13-api5ct-inventory-design.md`（v0.8.0）为唯一权威。
> 用法：§1 决定读什么，§3 是重构的硬边界，§4 是重构要解决的问题，§5 是语言选型，§6 是路线，§7 是待拍板项。

---

## 1. 参考文档清单与权威性分级

| 层级 | 文档 | 体量 | 用法 |
| --- | --- | --- | --- |
| A 权威 | `docs/2026-07-13-api5ct-inventory-design.md` | 1110 行 / 89 KB / 17 章 | 业务与架构的唯一真源。重构时逐条对照 §3、§4、§6、§7、§12、§17 |
| A 权威 | `tests/`（16 文件 / 170 个 test 函数 / 5116 行） | — | 行为契约。24 个测试走 HTTP 黑盒，天然是重构的验收网 |
| A 权威 | `migrations/versions/`（5 个版本） | — | Schema 真源。设计文档 §4 的表结构以迁移为准 |
| B 约定 | `AGENTS.md`（项目内，14 KB） | — | 命令入口、ANTI-PATTERNS、未来模块目录约定。与设计文档冲突时以设计文档为准 |
| B 约定 | `pyproject.toml` | — | ruff 规则集、pytest marker（`slow` / `requires_rust`）、依赖清单 |
| C 历史 | `CHANGELOG.md` | 2.4 KB | 只用来判断"某个特性是不是本轮新增" |
| D 专项记录 | `docs/superpowers/specs/`（3 篇：前端重写、audit-gap、login-lockout、check） | 37 KB | 单个缺陷（D1–D18）的落地证据。已并入设计文档 §0.x 与 §17，重构时只查细节 |
| D 专项记录 | `docs/superpowers/plans/`（3 篇） | 141 KB | 执行计划，非设计。**不要**当规范读，会读到已被推翻的中间态 |
| E 噪声 | `.superpowers/sdd/**/baseline/`、`**/review.diff` | 大量 | 冻结快照，ruff 已 `extend-exclude`。重构前建议整体归档或删除，别 grep |
| E 噪声 | `.cortexkit/magic-context/historian/*.xml` | — | 会话历史，同上 |

阅读顺序（约 90 分钟）：

1. 设计文档 §1.2、§2.2、§6、§7 —— 范围、原则、结构、安全现状
2. 设计文档 §3（Rust 核心）、§4（数据模型）、§12（并发与一致性）、§17（缺口与取舍）
3. 项目 `AGENTS.md` 的 ANTI-PATTERNS + COMMANDS
4. 挑 `src/modules/warehouse/routers/tubes.py`（1238 行）通读一遍，这是重构的主战场
5. `tests/test_routes_protection.py` + `tests/test_tubes_routes.py` —— 看清门禁和路由契约是怎么被钉住的

---

## 2. 现状量化基线（2026-11-17 实测）

| 项 | 数值 |
| --- | --- |
| Python 源码 | 26 文件 / 5635 行 |
| 最大单文件 | `warehouse/routers/tubes.py` 1238 行；`warehouse/services/tube_service.py` 832 行 |
| 路由端点数 | 50（`@router.get/post/put/delete`） |
| Jinja2 模板 | 20 个 / 2014 行（含 `base.html`、`_macros.html`） |
| 前端静态资产 | 932 行（`app.css` 791 + `app.js` 141）+ vendor Bootstrap 5.3.3 / icons 1.11.3，共 736 KB |
| Rust 核心 | 4 文件 / 776 行（`auth` 230、`compliance` 342、`audit` 167、`lib` 37） |
| 测试 | 16 文件 / 170 test 函数 / 5116 行 |
| 运行期依赖 | 9 个（fastapi, uvicorn, sqlalchemy, alembic, aiosqlite, pydantic, pydantic-settings, jinja2, python-multipart） |
| 数据表 | 11 个 + `check_sessions` / `check_items` |
| DB | SQLite WAL，`data/inventory.db` 124 KB（生产库极小，20 个测试库散落在 `data/`） |
| 构建 | 两段式：`maturin develop --release` 建 Rust，再 `uvicorn --workers 1` |

---

## 3. 重构的硬边界（设计文档抽取，改动需显式决策）

这些是设计文档的明确选择，重构不能"顺手优化"掉：

| # | 不变量 | 出处 | 后果 |
| --- | --- | --- | --- |
| 1 | 单进程、单 worker；SQLite 单写者 | §2.2、§9.1、AGENTS.md | 任何多实例/多 worker 方案都要求先换 DB |
| 2 | 密码 / JWT / 合规判定 / 审计哈希**必须**经 Rust | §2.2、§7 | Python 侧引入 `PyJWT`/`argon2-cffi` 等于否决该原则 |
| 3 | 业务写操作与审计行同事务，且用 `BEGIN IMMEDIATE` | §12.1、§12.3、D18 | 不能改成"先写业务再发审计事件"的最终一致模型 |
| 4 | 审计链 tip 存 DB，不驻留内存；`verify_chain` 必须分批 | §3.2、§4.7 | 不能把链状态挪进 actor/缓存 |
| 5 | 管子、库位不物理删除 | §4.1、AGENTS.md | 软删 + 停用是数据模型前提 |
| 6 | `used_count` 为反规范化派生列，配每日对账 | §12.2 | 不能改成每次 `COUNT(*)` |
| 7 | 无前端构建（Jinja2 + 本地 CSS，无 Node） | §2.2、§8 | 引入 Vite/React 需要重写全部 20 个模板 |
| 8 | `pandas` 导入上限 10000 行/次，行级部分成功 | D7、§17 | 不是整体回滚事务 |
| 9 | Cookie 无 `secure`（LAN）；`standards` 硬删；`transaction_items.pipe_id` 无 FK | §17「有意为之」 | 重构不要"修正"这三项 |

---

## 4. 痛点定位（重构要解决的问题，按证据排序）

| # | 痛点 | 证据 | 影响 |
| --- | --- | --- | --- |
| P1 | 单文件承载 5 个领域 | `warehouse/routers/tubes.py` 1238 行，同时挂 `/tubes`、`/tubes/{pipe_id}`、`/tubes/quality`、`/locations*`、全部盘点端点（设计文档 §6 自己标注"尚未拆分"） | 改盘点要读管子路由；合并冲突高发 |
| P2 | 事务 + 审计样板代码重复 | `BEGIN IMMEDIATE` 出现 14 处、跨 5 个文件（tube_service 6、audit_service 3、auth 3、check_service 1、tubes router 1） | D18 就是这么漏的：库位写操作裸 `commit()` 没进审计链 |
| P3 | 角色门禁靠逐路由手写 `Depends` | D1 一次漏了 5 处写路由；`test_routes_protection.py` 758 行专门兜这个 | 每加一个写端点就多一处漏风险 |
| P4 | Rust 边界要求逐调用点手写 `anyio.to_thread.run_sync` | 24 处调用点；D5 就是漏了一批导致阻塞事件循环 | 漏一个就是隐性性能故障，靠 code review 兜 |
| P5 | `to_thread` 与 Rust 模组无类型契约 | `rust_core` 无 `.pyi`，`from rust_core import auth` 全靠 PyO3 动态 | `pyright strict` 无法覆盖安全核心，IDE 无补全 |
| P6 | service 层规则与路由耦合 | `tube_service.py` 832 行 + `check_service.py` 570 行，出库/调拨/盘点的 TOCTOU 守卫散在路由与服务之间 | 单测只能走 HTTP，无法按用例单测 |
| P7 | Pydantic schema 与 ORM 模型未分层 | `schemas.py` 279 行 + `models.py` 248 行，请求/响应/内部结构混用 | 表单路径与 JSON 路径校验不一致 |
| P8 | 测试基础设施缺失 | 16 个测试各自 `sys.path.insert` + 各自设 `DATABASE_URL` + 各自建库，`data/` 积了 20 个 `.db` | 无法并行跑、无法定向跑、CI 慢 |
| P9 | 未来模块扩展是硬编码 | `src/modules/__init__.py` 手写 5 个 router；设计文档要求 qc/sales/procurement 各加一行 | 模块增加时改核心文件 |
| P10 | 前端无抽象 | 20 模板 2014 行，宏只集中在 `_macros.html` | 加页面要复制网格结构 |

P7–P10 是次级，P1–P6 是主战场。

---

## 5. 语言与技术栈选型

### 5.1 决策权重

| 权重 | 因素 | 现状判断 |
| --- | --- | --- |
| 高 | 维护人数 = 1，业务规则密度高（API 5CT 五项校验、哈希链、盘点差异、TOCTOU 守卫） | 读代码速度 > 运行速度 |
| 高 | 已有可复用资产 | Python 5635 行 + Rust 776 行 + 模板 2014 行 + 测试 5116 行 |
| 中 | 性能 | LAN 单机、生产库 124 KB、导入上限 1 万行。**不构成选型压力** |
| 中 | 部署运维 | 现在要 Rust 工具链 + maturin + Python 环境；目标是单条命令起来 |
| 中 | 并发天花板 | SQLite 单写者，扩展模块上线后需要换 DB |
| 低 | 生态/招人 | 内部系统，无 |

### 5.2 候选方案

| 方案 | 栈 | 工作量 | 收益 | 代价 |
| --- | --- | --- | --- | --- |
| **A. 原地渐进重构（推荐）** | Python 3.12 + uv + FastAPI + SQLAlchemy 2.0 async + Jinja2 + 保留 Rust/PyO3 | 中（8 阶段，可分批交付） | 170 个测试保留可用；P1–P10 全可解；风险可控、每阶段可回退 | 双语言构建仍在；PyO3 类型边界仍需手写 stub |
| **A′. A 再去 Rust** | Python + `argon2-cffi` + `PyJWT`，合规与哈希链用 Python 实现 | 中 + 删 Rust | 构建从双语言降到单语言；调试同一栈 | **违背设计 §2.2 强制约定**；Argon2 性能下降但 LAN 场景无感；等于否决 v0.3.0 的战略决策，需你明确拍板 |
| **B. 全 Rust 重写** | axum + sqlx + minijinja + tokio；`auth`/`compliance`/`audit` 776 行近乎直接复用 | 大（5600 行 Python + 2000 行模板 + 170 测试全部重写） | 单二进制部署；无双语言边界；并发模型自由；为换 PostgreSQL 铺路 | 交付周期以月计；模板层与表单校验（`python-multipart`、CSRF）要自建；170 个测试是唯一行为契约，重写期没有真实回归网 |
| **C. Rust 核心 + 薄 Python 网关** | 领域逻辑全 Rust，Python 只做表单渲染与迁移脚本 | 大 | 逻辑集中 | 边界从 24 处变成整层跨进程/跨 FFI，P4 恶化而非缓解 |
| **D. Node/TypeScript** | Fastify + Drizzle + EJS | 大 | 前端可升级 | 现有资产零复用；无收益来源 |

### 5.3 推荐：方案 A，并保留 B 的迁移窗口

理由（按权重）：

1. **测试是唯一的行为契约**。170 个 test 函数里 24 个走 HTTP 黑盒，A 方案下它们是免费回归网；B 方案下它们要先重写才能用，重写期等于没有网。
2. **P1–P6 全是"结构问题"，不是"语言问题"**。拆路由、抽 `unit_of_work`、统一 `rust_call()` 包装、门禁前置到路由组 —— Python 侧同样能解，且每步可独立提交。
3. **性能不构成理由**。生产库 124 KB，导入上限 1 万行，单机 LAN。换语言的收益无法在这套负载上兑现。
4. **Rust 该留的地方已经留对了**。`auth`/`compliance`/`audit` 776 行没有一条业务查询，正是"纯函数 + 无状态"该待的位置（设计 §2.2 ANTI-PATTERNS 明确反对 Rust 持有链 tip / 会话状态）。
5. **A 不堵住 B**。P3/P5 把 Rust 边界收敛成单一模块 + 生成 `.pyi` 后，如果将来真要迁 Rust，那层已经是清晰的接口，直接翻译成 crate 而非从散落的 24 个调用点里考古。

### 5.4 切换到方案 B 的量化触发条件

出现任一条时，重新评估并切 B（或先切 PostgreSQL + 多实例）：

- 需要多终端并发写（多库位同时盘点、多人同时导入），SQLite 单写者成为可观测瓶颈
- 需要多实例/容器横向扩展，即必须放弃单 worker 约束
- 单次导入规模超过 10 万行，或 CSV 解析耗时进入分钟级
- PyO3 双语言构建成为主要故障源（跨平台分发、CI 构建时间失控）
- 上线 qc + procurement + sales 三个模块后，Python 侧超过 1.5 万行

另外：**换 PostgreSQL 与换语言解耦**。SQLAlchemy 2.0 async 抽象层已经隔离了方言，A 方案可在 Python 内先完成 DB 迁移（P7），再看是否需要 B。

### 5.5 环境与工具链建议（A 方案）

```bash
# Python：3.12（pyproject 现写 py311，先验证 py312 通过再提；pandas 需配合）
uv python pin 3.12
uv sync --all-extras

# Rust：仅构建期需要
rustup toolchain install stable
uv run maturin develop --release   # 或 maturin build --release 后装 wheel

# 质量门（重构期间逐阶段收紧）
uv run ruff check                      # 现状：select 已含 E/F/I/UP/B/SIM/RET/RUF
uv run pyright --outputjson src        # 新增，先 basic，P5 完成后 strict 覆盖 src/safety.py
uv run pytest -q                       # 基线：170 个 test 函数
uv run pytest -q -m "not slow and not requires_rust"
```

选型细节：

| 组件 | 建议 | 理由 |
| --- | --- | --- |
| 包管理 | 继续 `uv`（已有 `uv.lock`） | 不改工具链 |
| 类型检查 | 引入 `pyright`，分阶段：`basic` → 目录级 `strict` | P5 需要 Rust 边界有类型；`mypy` 亦可，只要一次性选定 |
| 数据库 | 短期 SQLite 不变；P7 阶段用 SQLAlchemy 方言抽象 + alembic 双跑验证 PostgreSQL | 换 DB 是独立决策，不与语言绑定 |
| 模板 | 保持 Jinja2；局部可用 `jinja2.ext.do` 之外不引新扩展 | 维持"无前端构建" |
| 测试 | 引 `conftest.py` 集中 fixture，DB 用 `tmp_path`，消除 `data/` 下的测试库 | P8 |
| Rust 类型 | `pyo3-stub-gen` 或手写 `rust_core.pyi` | P5 |
| 格式化 | ruff format（与 lint 分开开关） | 一次性统一，避免重构 diff 噪声 |

不引入：Node/Vite、ORM 替换（Django ORM / Tortoise）、Pydantic v1 兼容层、asyncio 之外的并发框架、`celery`/`redis`。

---

## 6. 重构路线

原则：每阶段可独立提交、可回退；测试数只增不减；阶段内不动业务语义。

### P0 — 基线锁定（0.5 天）

- `git tag pre-refactor-2026-11`；`sqlite3 data/inventory.db ".backup data/backup/pre_refactor.db"`
- 跑全量测试记录基线（预期 170 个 test、157 通过 + 3 slow，见 AGENTS.md §NOTES）
- 归档或删除 `.superpowers/`、`.cortexkit/`、`data/test_*.db`、`ayanami_rei.egg-info/`
- **验收**：`git status` 干净；测试基线数字写进本文件附录

### P1 — 工程环境（1–2 天）

- `conftest.py`：单例 app fixture、`tmp_path` DB、`DATABASE_URL`/`JWT_SECRET_KEY` 集中注入、干掉 16 份重复的 `sys.path.insert`
- 引入 pyright（`basic`）+ pre-commit（ruff check/format + pyright + pytest 冒烟）
- `data/` 只留生产库；`.gitignore` 补 `data/test_*.db`
- **验收**：测试数与 P0 一致；`data/` 无新增 .db；CI 或本地钩子可一键跑

### P2 — 事务与审计统一（3–5 天，最高价值）

- 新增 `src/db/txn.py`：`unit_of_work()` —— 包 `BEGIN IMMEDIATE` + commit/rollback + 统一异常映射（对应设计 §15）
- 新增 `@audited(action=..., snapshot=...)` 或 `write_with_audit()`：同事务写审计行，强制审计参数
- 迁移 14 处 `BEGIN IMMEDIATE` 到统一入口，逐处对照 `test_admin_audit.py` / `test_audit_route.py` / `test_panic_recovery.py`
- **验收**：`grep -c 'BEGIN IMMEDIATE' src` 只在 `txn.py` 命中；审计相关测试全绿；新增一条"裸 commit 写路径"的守卫测试

### P3 — 路由拆分与门禁前置（5–7 天）

- `warehouse/routers/tubes.py` → `tubes.py`（管子 CRUD/出入库）、`tube_detail.py`、`quality.py`、`locations.py`、`check.py`
- 路由挂载改为 `APIRouter(prefix=..., dependencies=[Depends(require_operator)])`，写路由组级门禁；读路由显式声明
- 同步拆 `templates/warehouse/*` 分组（仅目录移动，不改模板内容）
- **验收**：单文件 < 400 行；`test_routes_protection.py` 758 行不改一行仍全绿（这是门禁契约的试金石）；`/openapi.json` 端点集合与拆分前逐一比对无差异

### P4 — 领域分层（5–7 天）

- service 层收敛：路由只做"解析 + 调 service + 渲染/序列化"，禁止在路由里开事务
- `schemas.py` 按模块拆分为 `schemas/`（输入/输出分离），表单校验与 JSON 校验共用同一 Pydantic 模型
- TOCTOU 守卫下沉到 service，配单测（不经 HTTP）
- **验收**：路由文件无 `session.execute`；新增 service 级单测 ≥ 20 条

### P5 — Rust 边界收敛（2–3 天）

- `src/safety.py`：单一出口，`async def call(fn, *args)` 内部 `anyio.to_thread.run_sync`，禁止其他模块直接 `import rust_core`
- 生成/手写 `rust_core.pyi`；pyright 对 `safety.py` 开 `strict`
- 24 处调用点改为 `await safety.call(...)`
- **验收**：`grep -rn 'from rust_core' src` 只命中 `safety.py`；pyright strict 通过；`requires_rust` 标记测试全绿

### P6 — 前端收敛（3–5 天，可与 P4 并行）

- `_macros.html` 扩为组件库（表格、徽章、分页、表单行、批量操作坞），20 模板改用宏
- 单页局部交互（条件编辑器、批量选择）保持 `app.js` 原生实现，不引框架；如需提升，只考虑 htmx（无构建）
- **验收**：模板总行数下降；页面视觉与交互无回归（按 `docs/superpowers/specs/2026-07-31-frontend-redesign-design.md` 逐页对照）

### P7 — 数据层（可选，3–5 天 + 观察期）

- alembic 双跑（SQLite + PostgreSQL）验证方言无差异；并发路径补 `SELECT ... FOR UPDATE` 分支
- 决策点：是否放弃"单 worker"约束（见 §5.4 触发条件）
- **验收**：同一套测试在两种方言下全绿；`used_count` 对账脚本 CLI 化（§12.2.1 缺口）

### P8 — 扩展模块骨架（1–2 天，等 qc/sales/procurement 立项再做）

- `src/modules/__init__.py` 改为 `MODULES = [...]` 注册表 + 约定式发现
- 每个模块强制 `routers/`、`services/`、`schemas/` 三件套
- **验收**：新增一个空模块只需加目录与一行注册

顺序与依赖：P0 → P1 → P2 → P3 →（P4、P6 并行）→ P5 → P7/P8。P2 先行，因为它把"漏审计"这类缺陷从"靠 review"变成"靠类型/接口"。

---

## 7. 待拍板的决策

| # | 决策 | 选项 | 建议 |
| --- | --- | --- | --- |
| 1 | 是否保留 Rust 安全核心 | 保留（A）/ 去 Rust（A′）/ 全 Rust（B） | 保留（A）。去掉等于否决设计 §2.2，需先改设计文档再动手 |
| 2 | 是否本轮就换 PostgreSQL | 继续 SQLite / P7 阶段迁 PG | 继续 SQLite。先做完 P2–P5，让迁移成本可测 |
| 3 | 前端是否引入构建 | 保持 Jinja2 / 局部 htmx / 全套 Vite | 保持 Jinja2。前端重写刚落地（2026-10-24），没有二次重写的理由 |
| 4 | 重构粒度 | 一次性大改 / 8 阶段渐进 | 8 阶段渐进，每阶段一个 PR + 测试基线对比 |

---

## 附录：本文件引用的设计文档章节

§0.9–§0.11（实现状态与修订）、§1.2（范围）、§2.2（架构原则）、§3（Rust 核心）、§4（数据模型）、§6（项目结构）、§7（安全）、§8（技术选型）、§9.1（启动约束）、§12（并发一致性）、§13（测试与迁移）、§15（异常映射）、§17（缺口与有意取舍）。