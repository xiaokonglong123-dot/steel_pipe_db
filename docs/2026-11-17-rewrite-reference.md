# SteelPipe 推倒重写参考文档（greenfield）

> 版本：v1.0（2026-11-17）
> 前提：业务语义唯一来源 = `docs/2026-07-13-api5ct-inventory-design.md`（v0.8.0）。
> 决策依据见 §0，全部来自你本轮的四个选择。
>
> **v1.1 修订（2026-11-17，当日）**：数据库由 PostgreSQL 16 改为 **SQLite（WAL）**。全文已按此改齐：
> 事务串行化回到设计 §12.1 的 `BEGIN IMMEDIATE`（sqlx 的 `Pool::begin_with` 直接支持），
> 代价是放弃多实例部署（见 §3.3、§8）。迁移、PRAGMA 与链尾表已在 M0 实测通过。

---

## 0. 已固化的决策

| 决策 | 结论 |
| --- | --- |
| 重构形态 | 推倒重写（greenfield），不做增量迁移 |
| 动机（四条全选） | ① 摆脱双语言构建/部署 ② 结构腐烂要干净分层 ③ 为 qc/采购/销售与 PG 铺路 ④ 换技术栈本身 |
| 主栈 | Rust 单体（axum + sqlx），**不再有 Python 层** |
| 前端 | 前后端分离：后端只出 JSON，前端 SPA |
| 兼容约束 | 完全自由：URL 路径、表单字段、DB schema 均可重来，现有数据可弃 |

由此推出的硬结论：

1. 双语言必须彻底消失。不是"Rust 为主、Python 为辅"，而是**Python 代码零保留**。
2. 前后端分离意味着 20 个 Jinja2 模板全部废弃；`src/static` 只能移植设计 token，不能移植 Bootstrap 覆盖层。
3. 数据可弃 → 不做数据迁移，不做 schema 兼容层，`alembic` 与 5 个 Python 迁移文件作废，改用 `sqlx::migrate!` 的纯 SQL。
4. 170 个 Python 测试不可执行，但**其 docstring 是业务规则的最后一份成文记录**（大量用例只在这里写清：TOCTOU 409、CSRF 403、批量原子性、登录锁定审计、413 上传上限）。必须先把它们抽成规格表再写代码。

---

## 1. 现状盘点：哪些能带走

| 资产 | 体量 | 可复用度 | 处理方式 |
| --- | --- | --- | --- |
| `rust-core/src/audit.rs` | 167 行 | **~93%** | 只删 11 行 pyo3 胶水，核心链计算原样搬入 `crates/audit` |
| `rust-core/src/auth.rs` | 230 行 | **~94%** | 删 14 行 pymodule 注册，`hash_password`/`verify_password`/`issue_*`/`decode_claims` 直接用 |
| `rust-core/src/compliance.rs` | 342 行 | **~91%** | 删 32 行 pyo3；`ComplianceResult`/`StandardSpec` 结构体同时变成 API DTO |
| `rust-core/Cargo.toml` | — | 高 | `argon2` / `jsonwebtoken` / `sha2` / `uuid` / `serde` 版本可直接沿用 |
| 设计文档 | 89 KB / 17 章 | **100%** | 唯一业务真源；§3/§4/§12/§17 是重写的验收基线 |
| 170 个 test 函数 | 5116 行 | **规格价值，无执行价值** | 抽成规格表（§7），翻译为 Rust 集成测试 + Playwright e2e |
| `src/static/app.css` 设计 token | 46 行变量 + 网格/徽章/表单组件约 250 行 | **部分** | token 层与组件语义可移植；`--bs-*` Bootstrap 覆盖层（12 处 + 大量选择器）丢弃 |
| 20 个 Jinja2 模板 | 2014 行 | **0** | 全废。页面信息结构（表格列、表单字段、流程步骤）可作为 SPA 页面设计输入 |
| `src/**` Python | 5635 行 | **0** | 全废，仅作行为参考 |
| `migrations/*.py`、`alembic.ini` | — | **0** | 作废，改写为 SQL |
| `data/inventory.db` | 1 管 / 2 交易 / 3 审计 / 1 用户 | **0** | 按你的选择弃用；保留一次 `sqlite3 .dump` 归档到 `docs/legacy/` 以防万一 |

Rust 侧可搬总量：**739 / 776 行（95%）**，剩余 37 行是 PyO3 边界。这是整件事最大的利好：安全核心不用重写，只需从"被 FFI 调用的扩展模块"改成"被 workspace 直接依赖的 crate"。

---

## 2. 目标架构

```text
浏览器 (SPA)  ──HTTPS/JSON──▶  单二进制 (axum)  ──sqlx──▶  SQLite (WAL)
                                     │
                     ┌───────────────┴────────────────┐
              领域规则 (domain)                  审计链 (audit)
              API 5CT 合规 (compliance)          argon2/JWT (auth)
                     └── 全在进程内，无 FFI，无跨语言 ──┘
```

对比原架构的三处结构性变化：

| 维度 | 原 | 新 |
| --- | --- | --- |
| 语言边界 | Python ↔ PyO3 ↔ Rust，24 个 `anyio.to_thread` 调用点 | 无边界，全部 Rust 同步/异步直接调用 |
| 并发模型 | SQLite 单写者 + 单 worker（Python 侧串行） | SQLite 单写者，但由 Rust 单进程直接持有；读并发走连接池 |
| 审计链串行化 | `BEGIN IMMEDIATE` 独占写锁 | 同语义，收敛为 `infra::WRITE_TXN_BEGIN` 单一常量 |
| 部署 | maturin + uvicorn + Python 环境 | 一个二进制 + 一个 .db 文件 |
| 渲染 | Jinja2 服务端渲染 + 表单 POST | SPA 调 JSON API，浏览器不做整页刷新 |

---

## 3. 技术选型

### 3.1 后端

| 层 | 选择 | 理由 | 备选与否决理由 |
| --- | --- | --- | --- |
| 运行时 | tokio 1.x（multi-thread） | 事实标准 | — |
| Web | axum 0.8 | tower 中间件生态、类型安全 extractor、与 sqlx/serde 同族 | actix-web：性能相近但 extractor/中间件模型分裂；poem：生态小 |
| 数据库 | **SQLite（WAL）** + sqlx 0.8（async、`sqlite` feature） | 零运维、单文件、备份只需 `.backup`；项目负载（局域网、单机、万级管道）远未触达写瓶颈 | PostgreSQL：需要额外服务进程与运维，收益在当前规模兑现不了；Diesel：同步 + 编译重，sqlx 的 SQL 直写更适合手写事务与审计链 |
| 事务/串行化 | `Pool::begin_with("BEGIN IMMEDIATE")`（`infra::WRITE_TXN_BEGIN`） | 直接复刻设计 §12.1 的防 TOCTOU 语义：先取写锁再读链尾 | 全局 mutex：与 SQLite 自带写锁重复；PG SERIALIZABLE：引入不存在的并发需求 |
| 序列化/校验 | serde + `validator` 0.18 | 请求校验与领域不变量分离 | `garde` 亦可，二选一 |
| 错误 | `thiserror` + axum `IntoResponse` → RFC 9457 Problem Details | 对应设计 §15 统一异常映射 | 自定义 JSON 错误体：少一份规范依据 |
| 认证 | `jsonwebtoken` + `argon2`（沿用现有 crate 版本） | 现有实现直接搬 | 换 OIDC：LAN 内部无收益 |
| 观测 | `tracing` + `tracing-subscriber`（json 层）+ `tower-http` TraceLayer | 替代原 Python logging | — |
| 限流 | PG 表 `login_attempts` + `tower-governor`（IP 层） | 原实现是进程内 dict，多实例下失效 | 保留内存态：单实例可接受，但与本轮动机冲突 |
| OpenAPI | `utoipa` 注解 → `/api/openapi.json` | 前后端类型同步的唯一可靠管道 | 手写 TS 类型：一定会漂移 |
| 静态资源 | `rust-embed` 内嵌 `web/dist` | 真单二进制，直接满足动机① | 外部静态目录：多一个部署项 |
| CSV 导入 | `csv` crate（流式） | pandas 只为容错 BOM；`csv` + 手工 BOM 剥离足够 | `calamine` 留作后续 xlsx |
| 审计 CLI | 同一个二进制加子命令（clap）：`spi audit verify --batch`、`spi reconcile used-count` | 补上设计 §12.2.1 与 §17 的 CLI 缺口 | 单独二进制：多一份构建 |

### 3.2 前端

| 项 | 选择 | 理由 |
| --- | --- | --- |
| 构建 | Vite 7 + TypeScript 5.x（strict） | 无框架绑定，开发期 proxy 到 axum |
| 框架 | React 19 | 生态最全，表格/表单库成熟；单人维护看文档密度 |
| 路由 | TanStack Router（文件式，类型安全） | 路由参数与查询参数类型化，配合 TanStack Query |
| 数据层 | TanStack Query | 缓存/失效/重试语义现成，避免手写 fetch 状态机 |
| 表格 | TanStack Table（虚拟化 + 列固定） | 管子列表、盘点明细、审计日志都是宽表长表；原设计方向 B 就是"工业数据网格" |
| 表单 | react-hook-form + zod | zod schema 由 OpenAPI 生成，前后端校验同源 |
| 样式 | Tailwind CSS 4 + 移植原 token（`--bg-base/#0a0e14`、`--accent/#22d3ee`、`--r/3px` 等 46 个变量） | 原 app.css 的价值在 token 与密度规范，不在 Bootstrap 类名 |
| 组件 | shadcn/ui（Radix 底座，源码进仓库） | 无运行时版本锁定，暗色工业风易调 |
| 类型生成 | `openapi-typescript` → `web/src/api/schema.d.ts` | 契约漂移在 CI 里挂掉 |
| e2e | Playwright | 关键路径：登录、入库合规拦截、出库 TOCTOU、批量原子性、盘点、审计验证 |

否决：SvelteKit/Vue 无技术劣势，但 React 在你将来找资料/样例时的边际收益更高；Next.js 否决——不需要 SSR，且引入 node 服务端会破坏"单二进制"。

### 3.3 关键取舍（明确记下来）

- **放弃服务端渲染**：LAN 内网、单页数据网格，首屏体验不依赖 SSR；代价是丢掉 2014 行模板，收益是彻底的 API/UI 解耦。
- **Cookie 与 CSRF**：原设计必须自实现 CSRF（因为表单 POST + cookie 会话）。SPA 下改成 **access token 放内存 + refresh token 放 httpOnly/SameSite=Strict cookie**，CSRF 只需覆盖 refresh/logout；攻击面比原方案小。
- **SQLite 意味着放弃多实例**：单写者是文件级约束，横向扩展需先换库。本项目的负载场景（单机、局域网、万级管道）不构成限制，但 qc/采购/销售同时上量后需重新评估。这是有意识的取舍，不是遗漏。
- **单写者带来的好处被保留**：审计链的"读链尾 + 写审计行"不再需要 SERIALIZABLE 重试逻辑，`BEGIN IMMEDIATE` 天然串行，比 PG 方案少一层失败模式。

---

## 4. 数据模型与不变量的新实现

11 表 88 列照设计文档 §4 重建为 SQL 迁移。原设计的"有意为之"取舍按你的自由授权重新评估：

| 原取舍 | 新决定 | 理由 |
| --- | --- | --- |
| 单 worker（Python 侧串行） | **保留单进程**，但串行点收敛为 `BEGIN IMMEDIATE` 单一常量 | 单写者是 SQLite 的文件级约束，与语言无关；Rust 单进程不再需要"单 worker 参数"这种口头约定 |
| `transaction_items.pipe_id` 无 FK | 改为 `ON DELETE RESTRICT` 的 FK | 软删 + RESTRICT 同样防级联删管，还多一层完整性（SQLite 需逐连接 `foreign_keys=ON`，已在连接池配置并测试） |
| 管子/库位不物理删除 | **保留**（软删 + `is_active`） | 审计可追溯的前提 |
| `used_count` 反规范化 | **保留**，用 `UPDATE ... SET used_count = used_count + 1` 原子增减 + 每日对账 CLI | 避免全表 COUNT，且 PG 下原子增减天然安全 |
| 审计链 tip 存 DB | **保留**，单行表 `CHECK (id = 1)` + 写锁 | 崩溃可恢复；单写者下无需额外锁协议 |
| 审计 `verify_chain` 必须分批 | **保留**，默认 1000 行/批，流式返回 | 20K 审计突发不 OOM 的原设计约束 |
| Cookie 无 `secure`（LAN） | 随部署形态定；HTTPS 部署时开 `secure` | 前后端分离后 token 更敏感 |
| `standards` 硬删 | 保留硬删 + 删前快照进审计 | 独立标准表无审计链依赖 |
| 登录限流进程内 | 改 PG 表（IP 5/60s、用户名 10/300s → 锁 300s，锁定时写 `login_lockout` 审计） | 多实例一致 |

审计链协议的落地 SQL（`crates/infra/migrations/0001_init.sql`，M0 已实测）：

```sql
CREATE TABLE audit_chain_tip (
    id         INTEGER PRIMARY KEY CHECK (id = 1),
    last_seq   INTEGER NOT NULL CHECK (last_seq >= 0),
    last_hash  TEXT    NOT NULL CHECK (length(last_hash) = 64),
    updated_at TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);
```

一个业务写 + 一条审计行，同一事务：

```rust
// BEGIN IMMEDIATE：先拿写锁，再读链尾 —— 两个写者不可能读到同一 tip
let mut tx = pool.begin_with(steelpipe_infra::WRITE_TXN_BEGIN).await?;

// 1) 业务写（含 TOCTOU 守卫：UPDATE tubes SET ... WHERE pipe_id = ? AND status = 'in_stock'）
// 2) 读链尾（此时已被写锁独占，无需 FOR UPDATE）
let (seq, prev): (i64, String) =
    sqlx::query_as("SELECT last_seq, last_hash FROM audit_chain_tip WHERE id = 1")
        .fetch_one(&mut *tx).await?;
// 3) hash = sha256(prev_hex ‖ action ‖ detail ‖ timestamp)，插入审计行
// 4) UPDATE audit_chain_tip SET last_seq = ?, last_hash = ?, updated_at = ? WHERE id = 1
tx.commit().await?;                 // 无 40001：写锁已串行化，失败只可能是 SQLITE_BUSY（busy_timeout=5s 兜住）
```

不变量：**没有 `unit_of_work` 包装的写路径不允许编译通过**。实现方式：`infra::uow` 只暴露 `async fn write<F>(&self, op: F) -> Result<T>`，仓储的写方法接收 `&mut Tx` 而非 `&Pool`；`domain` 与 `api` 不得直接持有 `Pool`（§5 的 CI 断言）。

---

## 5. 工程结构与分层约束

```text
steelpipe/
├── Cargo.toml                  # [workspace] resolver = "3"
├── rust-toolchain.toml         # stable，pin 版本
├── crates/
│   ├── domain/                 # 纯领域：实体、状态机、不变量。禁止 axum/sqlx/tokio 依赖
│   ├── compliance/             # API 5CT 五项校验（搬自 rust-core/compliance.rs）
│   ├── audit/                  # 哈希链计算 + canonical 序列化（搬自 rust-core/audit.rs）
│   ├── auth/                   # argon2 + JWT claims（搬自 rust-core/auth.rs）
│   ├── infra/                  # sqlx 仓储、迁移、uow、审计写入、对账 SQL。禁止 axum
│   ├── api/                    # axum 路由、DTO、中间件、错误映射、utoipa
│   └── server/                 # main：配置、tracing、静态资源内嵌、clap 子命令
├── crates/infra/migrations/    # sqlx 纯 SQL，0001_init.sql 起（SQLite 方言）
├── web/                        # Vite + React + TS
│   ├── src/routes/             # 12 个页面路由
│   ├── src/api/schema.d.ts     # openapi-typescript 生成物（不入库手改）
│   └── dist/                   # rust-embed 内嵌目标
├── tests/                      # 集成测试（#[sqlx::test]）
├── e2e/                        # Playwright
└── scripts/                    # 备份、初始化、seed 标准数据
```

分层约束用 CI 断言，不靠自觉：

```bash
# domain/compliance/audit/auth 不得依赖 IO 或 Web
cargo tree -p steelpipe-domain -e normal | grep -qE 'axum|sqlx|tokio' && exit 1
cargo tree -p steelpipe-audit  -e normal | grep -qE 'axum|sqlx' && exit 1
# api 不得直接拿 Pool 做写操作
! grep -rn 'pool.begin()' crates/api/src
```

对应你要解决的六个原痛点：

| 原痛点 | 新结构里的解法 |
| --- | --- |
| `tubes.py` 1238 行承载 5 领域 | `api/src/routes/{tubes,tube_detail,quality,locations,check,inbound,outbound,transfer}.rs`，每个 < 300 行；规则全在 `domain` |
| 事务+审计样板重复 14 处 | `infra::uow::write()` 单一入口，仓储只接 `&mut Tx` |
| 角色门禁逐路由手写（D1 漏 5 处） | 路由分层挂载：`Router::new().merge(operator_routes()).merge(admin_routes())`，门禁在层上不在端点上 |
| Rust 边界 24 处手写 `to_thread`（D5 漏一批） | 不存在边界；`audit`/`auth`/`compliance` 是直接 import 的 crate |
| service 832 行 + TOCTOU 守卫散落 | `domain` 的显式状态机（`Tube::stock_out()` 返回 `Result<_, DomainError>`），可纯单测 |
| 测试各自建库（data/ 积 21 个 .db） | `#[sqlx::test]` 每测试独立库，自动清理；PG 用 testcontainers |
| 模块注册硬编码 5 个 router | `crates/api/src/routes/mod.rs` 里一个 `Vec<Router>` + 领域目录约定；新增 qc 只需加目录 + 一行 |

---

## 6. 里程碑与验收

每阶段独立可跑、可演示、可回退。顺序刻意让"最难验证的东西"（合规、审计链、TOCTOU）先落地。

| # | 阶段 | 交付物 | 验收标准 |
| --- | --- | --- | --- |
| M0 | 骨架（2–3 天） | workspace + CI + `/api/health` + `/api/openapi.json` + `sqlx::migrate!` 空迁移 + rust-embed 冒烟 | `cargo build --release` 产出单二进制 ≥ 40 MB 以内；`cargo test` 绿；前端 `openapi-typescript` 生成成功 |
| M1 | 认证与用户（4–5 天） | argon2 哈希、JWT 签发/校验、refresh 轮换（`last_refresh_jti`）、登录限流（PG 表）、`login_lockout` 审计、RBAC 中间件层、用户 CRUD | 规格 §7 的 A1–A8 全绿；无 `JWT_SECRET_KEY` 时启动 fail-fast（保留原语义） |
| M2 | 领域 + 合规（4–5 天） | 11 表 SQL 迁移、domain 实体与状态机、compliance crate 接入、`standard_specs` seed、黄金用例单测 | 设计 §11 五项检查逐项通过/不通过 + 原因；黄金向量（固定输入 → 固定输出）比对 rust-core 现实现，逐字节一致 |
| M3 | 库存核心（6–8 天） | 入库、出库、批量出库（原子）、调拨、质量标注、库位管理、单据与明细、`used_count` 原子增减、审计链写入 | TOCTOU 409 用例全绿；批量出库中途失败整体回滚；并发 50 路出库不超卖；每笔写操作 1 条审计行 |
| M4 | 盘点（3–4 天） | `check_sessions`/`check_items`、差异计算、报告 | 设计 §5.3 流程可跑通；盘盈管强制走合规校验 |
| M5 | 导入/标准/审计（4–5 天） | CSV 流式导入（10k 行上限、行级部分成功、BOM 容错）、标准规格 CRUD、审计查询与分批 verify、`spi audit verify` / `spi reconcile` 子命令 | 10k 行导入 < 30s；超限 400；verify 20k 行审计内存平稳 |
| M6 | 前端（8–10 天） | 12 路由：登录、总览、管子列表（数据网格+筛选+批量）、管子详情、入库、出库、调拨、质量、库位、盘点列表/详情/报告、标准、用户、审计、导入 | Playwright 关键路径 10 条全绿；暗色 token 与密度规范与原设计系统一致 |
| M7 | 硬化交付（3–4 天） | 上传/请求体上限（413）、Problem Details 全量、备份/恢复脚本、systemd + compose、部署文档、`docs/` 重写 | 冷启动到可登录 < 2s；`pg_dump` 备份恢复演练通过；一条命令完成部署演练 |

工期（单人）：**全职 6–8 周**；业余 15 h/周 **约 3 个月**。M1–M3 是风险集中段（占 60%），M6 是体力活。

---

## 7. 规格迁移：170 个测试 → 规格表

这些测试不可直接执行，但每条 docstring 就是一个验收条件。做法：先产出 `docs/spec/` 下按模块拆的规格表，格式固定为「输入 → 期望状态码/状态变迁 → 审计副作用」，然后才写代码。已从测试文件头读出的高价值规格（开工前必须成表）：

| 来源文件 | 关键规格 | 目标落地 |
| --- | --- | --- |
| `test_routes_protection.py`（758 行） | 三角色 × 端点 × CSRF 的访问矩阵 | `api/tests/rbac_matrix.rs`，表驱动 |
| `test_tubes_routes.py` | 入库 303 / 冲突 409 / 出库 303 / 重复出库 409 / 批量 JSON 200 / 缺 CSRF 403 / 调拨同库位 409 | `domain` 单测 + `api` 集成测试 |
| `test_stock_in_compliance.py` | 合规不过 → 拒绝；`forced_intake` 需 admin + 审计 | `compliance` 单测 + `api` 集成测试 |
| `test_batch_stockout.py` / `test_concurrent_stockout.py` | 批量原子；并发不超卖 | `#[sqlx::test]` + `tokio::join!` 50 路压测 |
| `test_used_count.py` / `test_check.py` | used_count 一致性；盘点差异归属 | `infra` 集成测试 |
| `test_audit_chain_20k.py` / `test_audit_route.py` | 20k 行分批 verify；链断裂检出 | `audit` 单测（黄金向量）+ CLI 测试 |
| `test_login_lockout_audit.py` | 第 10 次失败锁 300s 且只写 1 条 `login_lockout` 审计 | `infra` 集成测试 |
| `test_upload_limit.py` | 超 `MAX_UPLOAD_SIZE` → 413，chunked 也拦 | `api` 中间件测试 |
| `test_panic_recovery.py` | 进程内 panic 后审计链与库存仍一致 | 事务回滚测试（PG 下改测 40001 重试） |
| `test_import_http.py` | 超 10k 行 400；行级部分成功 | 导入集成测试 |
| `test_security.py` | JWT 过期/篡改/吊销；token_version | `auth` 单测 |

配套产物（M0 就要建好，否则后面补不上）：

- `docs/spec/index.md`：规格条目编号（A1–A8、B1–B12…），每个里程碑的验收引用编号
- `docs/spec/golden/`：合规与审计链的固定输入/输出向量，直接从现有 Rust 实现导出
- CI 三道闸：`cargo test`、`cargo clippy -D warnings`、`openapi generate --fail-on-drift`

---

## 8. 风险与缓解

| 风险 | 触发条件 | 缓解 |
| --- | --- | --- |
| 工期失控（第一大风险） | M3 结束仍无端到端可演示流程 | 里程碑切片 + 每阶段可演示；M3 前不做任何前端工作 |
| 领域规则丢失 | 只读设计文档不读测试 docstring | §7 规格表先行；Rust 侧黄金向量比对现实现 |
| 审计链语义漂移 | 重写链计算时改了 canonical 序列化 | 用现有 `audit.rs` 导出向量做逐字节比对；不同则视为 bug |
| 前后端分离新增攻击面 | token 存储/CSRF/CORS 设计草率 | 同源部署（rust-embed）；access 内存 + refresh httpOnly/SameSite=Strict；CSRF 只护 cookie 端点 |
| SQLite 单写者成为瓶颈 | 多终端同时大批量导入/盘点 | M3 用 50 路并发出库压测定上限；超限则按 §5.4 触发条件评估换库（连接池已用 `DbPool` 别名隔离，换驱动只改 infra 一行） |
| 前端资产重做低估 | 932 行 CSS 里 Bootstrap 覆盖层不可搬 | 只移植 token 层（46 变量）+ 密度规范；组件用 shadcn 重搭，不逐行照搬 |
| 单人决策疲劳 | 12 个页面 × 网格/表单组合 | 先做 1 个参考页面（管子列表）定规范，其余复制 |

---

## 9. 明确不做

- 不做数据迁移、不做旧 URL/schema 兼容层（你已授权）
- 不保留 Python 任何一行（含 `pandas`、`alembic`、`uvicorn`、`pytest`）
- 不引 SSR（Next/SvelteKit）、不引 GraphQL、不引微服务拆分
- 不引 Node 服务端（Vite 仅构建期）
- 不在本轮做 WebSocket/实时推送（LAN 场景手动刷新足够）
- 不在本轮做 xlsx 导入（`calamine` 留接口）

---

## 10. 遗留待确认（3 条，均可默认执行）

| # | 问题 | 默认取值 |
| --- | --- | --- |
| 1 | 数据库文件放哪、备份策略 | 默认 `data/steelpipe.db`（已在 .gitignore）；备份沿用 `sqlite3 .backup` + 7 天日备 + 4 周周快照（设计 §16.2） |
| 2 | 是否需要保留局域网 HTTP 明文（原设计 cookie 无 secure） | 默认反代 HTTPS + `secure` cookie；内网例外可配置关闭 |
| 3 | 旧数据是否需要一次性导出归档 | 默认 `sqlite3 data/inventory.db .dump > docs/legacy/2026-11-17-inventory.sql`，仅归档不导入 |

---

## 附录：与设计文档的对照索引

| 设计文档章节 | 新架构中的位置 |
| --- | --- |
| §2.2 架构原则（分层隔离 / Rust 安全核心） | 原则保留，形式从 FFI 变 workspace crate；"强制约定"变成"cargo 依赖图 + CI 断言" |
| §3 Rust 模块划分 | `crates/{auth,compliance,audit}` |
| §4 数据模型（11 表 88 列） | `migrations/0001_init.sql` |
| §5 页面流程 | `web/src/routes/*`（信息结构沿用，交互重做） |
| §6 项目结构 | §5 本节 workspace 布局 |
| §7 安全设计 | §3.3 token/CSRF 重设计 + M1 |
| §8 技术选型 | §3 本文档 |
| §9 部署 | M7（单二进制 + SQLite 文件） |
| §12 并发与一致性 | §4 审计链协议（`BEGIN IMMEDIATE`）+ 连接池 PRAGMA |
| §13 测试与迁移 | §7 规格迁移 + `sqlx::migrate!` |
| §15 统一异常映射 | RFC 9457 Problem Details |
| §17 缺口与取舍 | §4 取舍表（部分废除，逐条注明） |
