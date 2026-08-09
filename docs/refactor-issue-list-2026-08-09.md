# ERP 全栈重构问题清单

> 日期：2026-08-09 ｜ 范围：整体架构 + 后端 Rust + 前端 React + 数据库/迁移层
> 数据库方向：**已定 SQLite3**（`sqlx` 已只启用 `sqlite` feature），所有 PG 残留均为清理对象。
> 基线：当前工作区（dirty，管道业务清理进行中）**无法编译**——`cargo check` 4 个硬错误。以下清单按"先恢复基线，再谈架构"排序。
> 严重度：P0=当前阻断 ｜ P1=清理/同步 ｜ P2=架构决策 ｜ P3=质量增强。

> **2026-08-09 阶段 0 执行记录**（已落地，见下方对应条目打勾）：
> - ✅ auth 栈 4 文件 PG→SQLite（`user_repo`/`refresh_token_repo`/`auth_service`/`auth_handler`），cargo check 全绿
> - ✅ 两处 `0.0.0` 非法字面量 + `sales_trend` 绑定数不匹配
> - ✅ RBAC 查库实时校验（middleware/auth.rs 每次请求查 `users` + `user_roles`，权限变更即时生效）
> - ✅ **新增 P0-4**：SQLite 动态类型 + sqlx 0.8 严格解码——`NUMERIC` affinity 把整数值存为 INTEGER，f64 解码崩溃。9 个迁移文件 40 处 NUMERIC→REAL + `DEFAULT 0/1`→`0.0/1.0`；全库 ~25 处 f64 聚合 `COALESCE(SUM...)` 补 `CAST(... AS REAL)`（含测试辅助）。**cargo test 全绿（25 目标 ~290 测试，此前因编译失败从未跑过）**
> - ✅ 订单 zod schema 对齐后端 item_id + 前端 purchases/sales/contracts 3 模块 types/Form/Detail 重写为商品(SKU)选择（新共享组件 ItemPicker）；前端 tsc + vite build 全绿

---

## 一、P0 — 当前阻断（不修，一切重构无从谈起）

### 1. 编译失败：PG→SQLite 迁移未完成（auth 栈）
- `backend/src/repositories/user_repo.rs:1`、`refresh_token_repo.rs:1`、`services/auth_service.rs:7`、`handlers/auth_handler.rs:8` — 4 处 `use sqlx::PgPool;`
- `auth_handler.rs` 共 12 处 `Extension<PgPool>`；`auth_service.rs` 全程 `&PgPool` 签名
- SQL 仍为 PG 方言：`$1` 占位符、`NOW()`、`RETURNING`、`TRUE/FALSE`
- `Cargo.toml:16` sqlx features 只有 `["runtime-tokio-rustls","sqlite","chrono","uuid","rust_decimal"]` → `PgPool` 被 feature 门控掉，**cargo check 报 4 个 E0432**（实测）
- 连带后果：`main.rs:135` bootstrap_admin 用 SqlitePool 调 `UserRepo::create(&PgPool)`，类型不匹配；**全部 23 个测试文件（~8600 行）无法运行**
- 修复方向：4 个文件全部改为 `SqlitePool` + `?` 占位符 + `datetime('now')`（约 40+ 条 SQL 改写）

### 2. 必然运行时崩溃（两处非法 SQL / 绑定数不匹配）
- `backend/src/repositories/inventory_repo.rs:74` 和 `backend/src/inventory_atp/repos.rs:58` — `COALESCE(SUM(...), 0.0.0)` 的 `0.0.0` 是**非法 SQLite 字面量**（sqlite3 实测 Parse error near ".0"）。空结果集触发 COALESCE 兜底分支时，库存查询 / ATP 预留必炸
- `backend/src/bi/services.rs:13-19` `sales_trend` — SQL 只有 1 个 `?`，却 `.bind(tenant_id).bind(months)` 绑定 2 个值，且 `tenant_id` 未用于 WHERE（无租户过滤）→ BI 销售趋势端点必然报错
- 修复方向：`0.0.0` → `0`；`sales_trend` 改为 1 个 bind（或加租户过滤），并补测试

### 3. 前后端契约断裂（拉订单必炸 + 前端调 404 API）
- **zod schema 过期**：`frontend/src/zod-schemas/orders.ts:12-15,47-50` 的订单行 schema 严格要求 `pipe_type/grade/od/wt` 且**无 `.passthrough()`**；后端 `models/purchase_order.rs:61-77`、`sales_order.rs` 订单行只有 `item_id/quantity/unit_price/...` → `purchaseApi.ts:20,28` / `salesApi.ts:21,29` 每次拉订单列表/详情都会 `validateResponse` throw
- **前端调用后端无路由**（对照 `router.rs` 全部 171 条路径）：`/seamless-pipes`、`/screen-pipes`、`/welded-pipes`（pipes/api/pipeApi.ts）、`/pipes/search`（inventoryApi.ts:398、search/api/index.ts:20）、`/quality/certs|grades|attachments`、`/labels/batch|shipping`、`/threading/calc`、`/casing/design-check` — 这些页面仍挂在 `routes/index.tsx` 上，全部 404

---

## 二、P1 — 清理与同步（低风险，可批量做）

### 4. 前端模块残留未下线
- `features/pipes/` 整套（8 页 + pipeApi + 3 套 hooks + pipeQueryKeys + 2 个 json）仍在路由/菜单；`features/README.md` 已声明"管材专属模块已由通用商品/SKU 取代"
- 4 个纯占位页仍挂路由：`sales/pages/AtpPage.tsx`、`reports/pages/TrendsPage.tsx`、`inventory/pages/LogsPage.tsx`、`quality/pages/Api5ctRefPage.tsx`（EmptyState 占位）
- 4 个占位页引用**不存在的 i18n key**（访问会渲染原始 key 串）：`t('title')`/`t('log_empty_description')`（inventory.json 无）、`t('api5ct_empty')`（quality.json 无）、`t('trends_empty')` + `en/common.json` 缺 `menu.trends`、命名空间 `'atp'`（无 atp.json）
- 冗余双轨：`reports/pages/DashboardPage.tsx`（硬编码中文列名）vs `bi/pages/DashboardPage.tsx`（i18n，真实现）；`/sales/atp` 占位 vs `/inventory/atp` 真实现
- `threading/` 模块（独立计算工具页）与 ERP 方向无关，挂在 manufacturing 菜单下

### 5. 死代码
- 后端：`src/cache.rs` + `cache_invalidator.rs` 基础设施零调用（`CacheInvalidator`/`CacheInvalidationRegistry`/`init_default_invalidation_rules` 无 handler/service 使用；`CacheManager` 仅 location_handler 一处用，ItemsCache/DashboardCache 从不读）
- 后端：`error.rs:73-78,91-93` 死错误码 `PipeNotFound(12001)/PipeNumberDuplicate(12002)/PipeStatusConflict(12003)/QualityCertNotFound(15001)/AttachmentNotFound(15002)` 零引用；`response.rs:102` 的 `no_content()` 零调用
- 后端：`benches/api_bench.rs:1,40` 标题仍是 "Steel Pipe DB"、`MockPaginatedItem.pipe_number` 残留
- 前端：根 `src/queryKeys.ts`（含假 key `['feature', params]`）零引用；`features/auth/store/authStore.ts` 无引用 re-export；`src/api/__tests__/client.test.ts` 与 `src/__tests__/api/client.test.ts` 两份并存
- 共享组件 12 个中 6 个零引用：`FormField`、`ActionButton`、`ListPageTemplate`、`StatusBadge`、`StatusTag`、`Can`；其中 StatusBadge 与 StatusTag 功能重叠、状态映射不统一
- `reports/api/reportApi.ts:14-37` 留 3 处 `[#4]`/`[#6]` 审计协商注释

### 6. 文档全面漂移（AGENTS.md/README/docs 与代码严重脱节）
- **Axios 已不存在**：AGENTS.md:108,263、README.md:86,229、frontend/AGENTS.md 声称用 Axios；实际 `frontend/src/api/client.ts` 是原生 fetch（注释明写 "replaces Axios"），package.json 无 axios
- **共享组件数量不符**：多处声称 "9 个共享组件"且列出 `ConfirmModal/FileUploader/LoadingSpin/PageContainer/PageHeader`——**都不存在**；实际 12 个，名单也不同
- **endpoints 数全错**：AGENTS.md/README/CHANGELOG 声称 "~70"，backend/AGENTS.md 声称 "~200"；实测 `router.rs` 192 个 `.route()` 调用、171 个唯一路径
- **`zod.response()` 不存在**：AGENTS.md:113 声称 `validateResponse.ts` wraps `zod.response()`；zod v3 无此方法，实际是 `schema.safeParse(data)`
- **rust_decimal 声明反了**：backend/AGENTS.md:31 声称 "No rust_decimal…Don't go looking"，Cargo.toml:13-14 已引入且 `domain/money.rs`、contract/purchase_order/sales_order 模型在用；docs/tech-debt.md:20 仍把 f64 精度列为"待修复/阻塞"
- **toolchain**：声称 "Rust nightly 2024-02-08"；实际 `rust-toolchain.toml` 是 stable，CI 用 dtolnay stable
- **middleware 数量**：声称 3 个（auth/rbac/rate_limit），实际 4 个（漏 security_headers）
- **unitStore 不存在**：AGENTS.md:77 声称 stores 有 authStore/appStore/unitStore；`frontend/src/stores/` 只有前两个
- **指向不存在的设施**：AGENTS.md:35 "no Makefile despite README"——README 全文无 Makefile 字样；AGENTS.md:99 "no build.rs"——全仓 AGENTS*.md 无 build.rs 字样；AGENTS.md:32 "npx vite build --analyze"——实际是 `vite build --config vite.config.analyze.ts`
- **store 命名**：package.json name 仍是 `steel-pipe-db-frontend`；`types/index.ts:1-3` 注释仍写 "Steel Pipe DB frontend"；zod-schemas/core.ts:5 注释仍写"钢管、筛管"

### 7. PG / 旧库脚本残留（与 SQLite3 决策直接冲突）
- `scripts/pg-dev.sh`、`scripts/pg-install.sh` — 完整 PostgreSQL 18.4 用户级实例管理脚本（~/.local/pgsql:5432），无任何引用，纯残留
- `scripts/backup.sh`、`scripts/restore.sh` — 默认 `DB_PATH=./data/steel_pipe.db`、备份命名 `steel_pipe_*.db.gz`；实际 DB 是 `data/erp.db`，对当前系统无效
- `backend/seed_data.py`(22KB)、`seed_data_enhanced.py`(39KB) — 旧 Python seed，面向 `steel_pipe.db` + API 5CT 数据，与迁移体系重复
- `backend/data/steel_pipe.db` — 残留开发库；其 `_sqlx_migrations` 记录的是**旧迁移内容**，任何指向它的部署会因 checksum 校验失败 boot 崩（sqlx migrator 对已应用迁移做 checksum 校验）
- `.gitignore` 残留 `pipes.db`、`steel_pipe.ico`；`.env.example` 里 `HTTP_PORT=80` 无代码消费（config.rs 只有 SERVER_PORT）、CORS 注释示例 `https://pipe.example.com`
- 注释残留：`domain/date_utils.rs:6`、`migrations/022:24`、`015:10`、`024:2`、`tests/common/mod.rs:31-33`（"PostgreSQL → SQLite 进行中"）

### 8. 迁移层
- **020 跳号**：migrations/ 版本 001–037，中间 020 缺失（原文件含 `ALTER COLUMN ... SET DEFAULT` 的 PG 语法被删）。sqlx 不要求连续，但应补 `020_` 占位或重排，避免历史陷阱
- **空占位迁移**：010/019/021/024 正文只有注释（`intentionally contains no statements`）——合法但应知悉
- **FK 策略两套**：005/006/009 的订单/出入库/合同表全部无 `REFERENCES`（头注释明写 "No FK constraints — integrity enforced at application layer"）；022 之后的新模块表全部声明 REFERENCES。在 sqlx 默认 `foreign_keys=ON` 下形成无约束表群
- **死列**：022:181-184 给 users 加的 `password_changed_at/locked_until/login_failures`，全库零读写
- **缺索引**：journal_entries（028）无 status/entry_date 索引；role_permissions 的 permission_id 侧无索引；workflow_delegations 无 original_user_id 索引；portal_accounts/portal_events 无 tenant_id 索引
- **seed 逻辑三处重复**：022:165-171 与 025_backfill_user_roles.sql:8-14 逐字重复同一段 user_roles 回填，main.rs:146-154 又绑一次

---

## 三、P2 — 架构统一（需要决策后实施）

### 9. 两套分层架构并存
- legacy 扁平目录：顶层 `handlers/`(18 文件) + `services/`(17) + `repositories/`(18) + `models/` + `dto/` + `domain/`
- 新模块目录：auth/workflow/hr/finance/procurement/sales_crm/inventory_atp/manufacturing/project/assets/notification/portal/bi（各含 handlers+repos+services）
- `router.rs:59-74` 同时 use 两套
- **auth 域双份**：`handlers/auth_handler.rs`（登录/用户 CRUD，legacy）与 `auth/handlers.rs`（RBAC roles/permissions/departments，新模块）并存
- 新模块反向依赖 legacy：11 个新模块 handler `use crate::handlers::auth_handler::AuthenticatedUser`；`inventory_atp/services.rs:15-16` 直接用 legacy `inventory_repo`/`inventory_log_repo`
- 拆分标准不透明：hr/finance/procurement 有独立模块，orders/contracts/suppliers/customers/reports/data_io 留在 legacy

### 10. 错误码域混乱 + 响应语义双轨
- 死错误码零引用（见 P1-5）；实际 Top 使用：`Validation(10002)`×201、`NotFound(10003)`×111、`OrderNotFound(14002)`×49、`OrderCannotModify(14001)`×37
- **120xx 声称是 Item 域但 Item 域错误码根本不存在**（item_service 只能用通用 Validation/NotFound）
- 文档表与实际矛盾：`docs/api/README.md:256-290` 错误码表与 `error.rs` 实际定义不一致（11002=登录失败 vs 实际 11002=TokenExpired；14001/14002 顺序颠倒；11004/11005 文档有定义无）
- **删除语义双轨**：15 处 handler 直接返回裸 `(StatusCode::NO_CONTENT, ()).into_response()`，4 处返回 `ApiResponse::ok(())`——同是 delete，前端可能拿 204 空 body 或 `{success,...}`
- **创建语义双轨**：16 处 `ApiResponse::created()`(201)，新模块一律 `ApiResponse::ok()`(200)
- `PaginatedResponse` 把 total/page/page_size/total_pages 在 `meta` 和 `data` 里重复两份（response.rs:33-47）
- **错误信息泄露**：`AppError::IntoResponse` 把 `self.to_string()` 直接序列化进响应（error.rs:121,130）；`From<sqlx::Error>` 原样塞入 SQL 错误字符串 → DB 错误（含 SQL 语句细节）以 500 暴露给客户端

### 11. 金额：f64 vs rust_decimal（已引入未落地）
- f64 存金额遍布：`models/finance.rs:40-95`（debit/credit/amount/tax）、`contract.rs:29-95`、`sales_order.rs`、`purchase_order.rs`、`hr.rs:76-82`（工资）、`assets.rs:15-33`（折旧）、`project.rs:18-62`
- **`finance/services.rs:93` 借贷平衡用 f64 等值比较 `debit_total != credit_total`**——浮点误差可拒绝合法凭证或放过非法凭证
- rust_decimal 只到 DTO 输入层 + `domain/money.rs` 转换器，`from_decimal` 又转回 f64 落库/计算
- 迁移层金额类型也不一致：早期表 `REAL`（002:15、006:74-75/109-110、009:20/39-40/53），后期模块 `NUMERIC`（028/027/034/035/029/026）——两类都被 sqlx 解码成 f64
- `models/contract.rs:43-63` 的 `total_amount_decimal()` 等访问器零调用（装饰性精确化）

### 12. RBAC 双轨 + JWT 内嵌权限
- `users.role` 列（001 CHECK admin/warehouse/qc/sales）与 `user_roles` 表（022）双轨并存；`middleware/rbac.rs:27` 只读 `ctx.role`
- `auth_service.rs:306-314` 把 role + permissions 嵌进 JWT；`middleware/auth.rs:108-114` 直接信任 token 内 claim 构造 AuthContext → **权限变更在 token 过期（默认 24h）前不生效**，被降权/删角色用户仍持旧权限
- 022 建的锁定字段（locked_until/login_failures/password_changed_at）零使用——无失败锁定、无强制改密
- 门户 JWT 与内部 JWT 共用同一 HMAC 密钥（`portal/services.rs:68-72` vs `middleware/auth.rs:104`），门户 token 固定 24h、无 refresh/撤销；portal_login 无任何限流

### 13. 多租户半成品
- 核心表（sales_orders/purchase_orders/inventory_logs/inbound/outbound/contracts/customers/suppliers）无 `tenant_id`；新表（auth/finance/procurement 等）有
- `bi/handlers.rs:25,39,46`、`portal/handlers.rs:26,84,98,110,123,133` **硬编码 tenant_id=1**

### 14. ATP 三套口径并存
- `handlers/atp_handler.rs` + `services/inventory_query_service.rs`（读 inventory_logs）
- `inventory_atp/handlers.rs` + `services.rs`（读 atp_slots）
- `services/sales_service.rs:315-332` 的 `available_quantity`（第三套，每条 3 个子查询）
- 三者可对同一商品给出不同数值

### 15. 限流可绕过
- `rate_limit.rs:85-99` 直接信任客户端可控的 `X-Forwarded-For`/`X-Real-IP` 首值，无信任代理配置 → 登录 5/min 暴力破解防护形同虚设
- `rate_limit.rs:45` 全局单 `Mutex<HashMap>` 串行化所有限流检查

### 16. 启动期 panic / 运行期吞错
- `main.rs:56,63,69,95,101,103` 多个 `.expect()`，95 行 env 驱动 panic
- `services/trace_service.rs:120,157` — `Err(_) => 0.0` 吞掉库存查询错误并伪装成 0 库存返回（会误导缺货判断）

---

## 四、P3 — 质量增强

### 17. 性能
- `services/trace_service.rs:111-127,149-165` — 双重循环 N+1（每条 record 查 items，每 item 再查 stock_on_hand）
- `services/sales_service.rs:384-400` — 审批逐 item 调 `available_quantity`
- `services/inbound_service.rs:16-38`、`outbound_service.rs:46-76` — 逐 item `SELECT EXISTS` 验证（批量导入 O(n) 往返）
- 一批列表接口无分页、硬 LIMIT 500/200：finance/repos.rs:160,309,369、assets/repos.rs:41、hr/repos.rs:71,255、inventory_atp/repos.rs:169,249、portal/repos.rs:81
- `handlers/location_handler.rs:42-48` — **缓存 key 不含 page/page_size**：翻页请求会命中第 1 页缓存返回错页（正确性 bug）
- 单据号生成用 `MAX(id)+1`（finance/services.rs:305、inventory_atp/services.rs:198）— 并发下单据号冲突

### 18. 测试与 CI
- 23 个 service 测试文件 ~8600 行 + tempfile 隔离基建（tests/common/mod.rs）——**但 CI 不跑 cargo test**（ci.yml 只有 cargo check + tsc + npm run build）；且当前编译失败，全部跑不了
- 无 handler/router/middleware 测试（鉴权、RBAC、限流、CORS 零覆盖）
- `tests/bi_service_test.rs:27` 调 `sales_trend(&pool, 1, 12)` 会撞上生产代码同名 bug——说明测试从未在改动后执行过
- `.gitignore` 忽略 `/backend/Cargo.lock`——binary crate 应提交 lockfile 保证可复现构建

### 19. 前端模式不一致
- **15/30 feature 无 `queryKeys.ts` 工厂**，页面散落内联 `queryKey: [...]`（bi/finance/assets/hr/notification/procurement/project/sales_crm/workflow/manufacturing/inventory_atp/auth 管理页）——`features/AGENTS.md:128` 明确禁止
- 新模块用 `invalidate = () => [...keys].forEach(...)` 手动枚举失效（字符串 key 拼写易碎），与老模块 mutation onSuccess 失效工厂 key 模式不一致
- **Form 校验全库无一处用 zod**（统一 antd rules），与"zod 统一"约定不符，且无复用 schema
- 硬编码中文 vs i18n 死 key 并存（见 P1-4）；`data-io` 三页硬编码中文而 data_io.json 全套 key 未用
- 巨大页面：InboundListPage 505 行、InboundFormPage 478 行（与 ListPage 重复实现批量管材 Modal）、OutboundFormPage 394、OutboundListPage 389、ContractFormPage 347、UserManagementPage 331、InventoryCheckListPage 321、PurchaseOrderFormPage 300——均未拆组件
- 12 个新模块各内联 `arrayOf/singleOf + .passthrough()`（48 处 passthrough），应抽到 lib/validateResponse.ts；`atpApi.ts:48-51,61-64` 的 reserve/createCountTemplate 直接返回 `res.data` 绕过校验；purchaseApi/salesApi 用 `as {...}` 强转掩盖 schema 未定义形状
- `reports/types.ts`(19 interface) 与 `zod-schemas/reports.ts` 逐字段重复定义，双向漂移
- `vite.config.ts:45-79` manualChunks bug：`id.includes('react')` 会命中 react-router/react-i18next/@tanstack/react-query，chunk 归属与注释不符
- `ProtectedRoute.tsx:11-32` 声明 `roles` prop 但从未使用；`/data-io/export` 路由无 `handle.roles`，可直接 URL 绕过菜单级 RBAC
- mutation 错误处理：全库仅 3 处 onError，全局只 console.error，用户看不到失败原因

### 20. 其它运行期风险
- `services/trace_service.rs` 与 `sales_service.rs` 大量金额在 f64 上运算（发票合计 finance/services.rs:168-178、合同金额、折旧 assets/services.rs:67）
- `create_transfer` 依赖的 `stock_on_hand_at_location` 命中 P0-2 的坏 SQL → 转库操作无法执行

---

## 五、值得保留（重构时勿破坏）

- **Extension DI + JwtSecret newtype**（无 AppState 全局态，router.rs:1264-1268）
- `error_codes!` 宏 + 统一 `IntoResponse for AppError`（error.rs:24-54）——方向正确，只是定义与使用脱节
- JWT fail-closed（config.rs:87-131）、bootstrap_admin 从 env 建初始管理员（main.rs:107）
- refresh token rotation + httpOnly cookie + SHA-256 哈希存储
- 前端 `api/client.ts` fetch 封装：401 single-flight refresh、超时、FormData/Blob 支持
- `validateResponse` + zod-schemas 集中化机制本身正确（问题在 schema 内容过期 + 覆盖不全）
- 迁移文件 SQLite port 注释质量高（022:11-26 逐条记录 BIGSERIAL→INTEGER 等）；统一 `INTEGER PRIMARY KEY AUTOINCREMENT`；017/022/025 种子幂等
- `tests/common/mod.rs` 的 tempfile 隔离测试基建
- i18n 33 文件 zh/en 对等、按模块命名空间；TS strict 全绿（无 any/ts-ignore）——类型纪律是最好的资产
- 事务与 TOCTOU 处理认真（sales_service approve/reject 状态守卫 WHERE；inbound/outbound 头+明细+日志单事务）
- `domain/order.rs` 状态机集中清晰；`domain/money.rs` 可作 Decimal 迁移切入点
- `vite.config.analyze.ts` chunk 分析脚本完整可用（只需把命令名改对）

---

## 六、架构方向决策（默认方案，待确认，可单项推翻）

> 2026-08-09 记录。以下为基于代码现状的推荐默认值；若某项与实际产品路线不符，改文档后即可调整对应阶段的实施细节（每项均可在阶段 2 内独立推进，不互相阻塞）。

| 决策项 | 默认方案 | 依据 / 代价 |
|--------|----------|-------------|
| **模块边界** | 全模块化并轨：legacy 顶层 `handlers/ services/ repositories/` 按资源域拆入新模块目录（items/orders/contracts/parties/inventory/reports/data_io + auth 并入），`dto/ models/ domain/` 保留为共享层 | 13 个新模块已落地此模式，AGENTS.md 已承诺；拆 49 文件工作量大但每步可验证；**共享解耦已完成**（f0871f0：AuthenticatedUser→middleware/auth.rs、party 宏→src/macros.rs），搬迁主体见文档末尾「阶段 2f 搬迁指引」 |
| **多租户** | 明确单租户：移除新模块中 `tenant_id` 硬编码与 tenants 表依赖（保留表但固定=1，或迁移期一并删），核心表不再补 tenant_id | 系统是单厂 ERP，无 SaaS 迹象；半成品（bi/portal 硬编码=1 掩盖问题）比没有更糟 |
| **金额类型** | 财务运算/比较域用 rust_decimal（借贷平衡已改 Decimal 累计+round_dp(4)，提交 a8b3f40）；**落库保持 f64**——sqlx-sqlite 0.8 明确不实现 rust_decimal 编解码（SQLite NUMERIC affinity 仅保留 15 位有效数字，官方文档称实现它只是陷阱） | Cargo.toml 已引入；REAL→NUMERIC 重建表方案因 sqlx 不支持 Decimal 而放弃 |
| **RBAC 校验** | 查库实时校验：`user_roles`/`role_permissions` 为唯一事实源，`users.role` 降级为展示列（或删除）；token 只带 user_id，请求时查库（可加短 TTL 缓存） | 权限变更即时生效；借 P0-1 的 auth 重写窗口一并做，避免二次重写 |
| **响应语义**（补充） | 统一：成功 `{success, request_id, data}`；创建一律 201 `ApiResponse::created()`；删除一律 204 空 body；错误一律 `{success:false, code, request_id, message, details}` 且**不向客户端暴露 SQL 细节** | 修双轨：15 处裸 204 vs 4 处 ok(())；16 处 201 vs 新模块 200 |
| **错误码**（补充） | 按 AGENTS.md 现有域表补齐 Item 域错误码（120xx 替换死 Pipe 码），废弃 `PipeNotFound` 等 5 个死码；修正 docs/api/README.md 错误码表 | 120xx 声称是 Item 域但 Item 码根本不存在；文档表与实际定义相反 |

> **阶段 2 执行记录（2026-08-09）**：
> - ✅ 2a 响应语义统一（f5ee26a）：创建 25 处→201、删除 4 处→204
> - ✅ 2b 单租户收口（f5ee26a）：bi/portal 有 AuthContext 处改取 tenant_id（portal-api 门户 JWT 保留 1）
> - ✅ 2c 错误码文档对齐（f5ee26a）：docs/api/README 110xx/140xx/150xx 修正
> - ✅ 2d 金额（a8b3f40）：借贷平衡 Decimal 累计 + round_dp(4)；**决策修订**：落库保持 f64（sqlx-sqlite 不支持 Decimal）
> - ✅ 2e ATP 口径 + 限流（91a4b44）：sales 可用量改标准口径（补 check_adjust/transfer）；限流改 TCP 对端 IP（ConnectInfo）
> - ✅ 2f 共享解耦（f0871f0）：AuthenticatedUser 移 middleware、party 宏移 src/macros.rs
> - ✅ 2f 搬迁主体（c57dfd1）：49 文件 git mv 入 items/orders/contracts/parties/inventory/reports/data_io + auth 并入，删顶层三兄弟，291 测试全绿。搬迁指引保留供参考（已完成）

> **阶段 3 执行记录（2026-08-09）**：
> - ✅ 3a CI（07d24c4）：backend job 增加 cargo test（report-then-fail）；提交 Cargo.lock（binary 可复现构建）
> - ✅ 3b 缓存正确性（07d24c4）：location 缓存 key 加入 page/page_size（原翻页命中第 1 页缓存）
> - ✅ 3c N+1（07d24c4）：trace_service 双循环→批量 stock_on_hand_map；sales approve 逐 item 保留（TOCTOU 刻意）
> - ✅ 3d queryKeys 工厂（91ae039）：11 个 feature 新建工厂 + auth 补 role/department 工厂，页面内联 queryKey 全部消除
> - ✅ 3e mutation onError（91ae039）：全库 27 个 useMutation 文件补失败提示
> - ⚠️ 环境注记：测试期间 /tmp（tmpfs）被 tempfile 残留库撑满致 disk I/O error，非代码问题；已清理并全量复绿

## 六·附：阶段 2f 搬迁指引（专门任务用）

目标：删除顶层 `handlers/ services/ repositories/` 三兄弟，legacy 资源域文件按如下映射搬入新模块（每模块 `mod.rs` 声明 handlers/repos/services），`dto/ models/ domain/` 保留共享层。

| 目标模块 | handlers 来源 | services 来源 | repos 来源 |
|----------|--------------|--------------|-----------|
| `items/` | item_handler | item_service | item_repo |
| `orders/` | purchase_handler, sales_handler | purchase_service, sales_service | purchase_order_repo, sales_order_repo |
| `contracts/` | contract_handler | contract_service | contract_repo |
| `parties/` | customer_handler, supplier_handler | customer_service, supplier_service | customer_repo, supplier_repo |
| `inventory/` | atp_handler, check_handler, inbound_handler, inventory_handler, location_handler, outbound_handler | check_service, inbound_service, inventory_query_service, location_service, outbound_service, trace_service | check_repo, inbound_repo, inventory_repo, inventory_log_repo, location_repo, outbound_repo |
| `reports/` | report_handler | report_service | report_repo |
| `data_io/` | data_io_handler | data_io_service | data_io_repo |
| `auth/`（并入现有） | auth_handler→handlers_legacy | auth_service→services_legacy | user_repo→repos_legacy, refresh_token_repo |
| 顶层单文件 | health_handler→`health.rs` | utils→`utils.rs` | operation_log_repo→`operation_log.rs` |

搬迁步骤（**一次一步、每步 cargo check**，避免覆盖）：
1. **文件移动用 `git mv`**（不用 rename——本任务初版曾因 rename 覆盖丢失 4 个文件，靠 git 恢复），每步先 `git status` 确认无同名冲突。
2. 移动后改每个文件内的 `crate::handlers::X` → `crate::<mod>::handlers`（handler 文件名统一 `handlers.rs`、service→`services.rs`、repo→`repos.rs`，与 13 个新模块一致）。
3. `src/handlers/mod.rs`、`services/mod.rs`、`repositories/mod.rs` 删对应行；`lib.rs`/`main.rs` 加新模块声明（`pub mod items;` 等）；`auth/mod.rs` 追加 `handlers_legacy/services_legacy/repos_legacy` 声明。
4. router.rs 所有 `crate::handlers::X` 引用改新路径（约 100 处，建议按模块分批）。
5. 每完成一个模块 `cargo check`，最后 `cargo test` + 全量回归。
6. 完成删空目录 + 更新 AGENTS.md 架构树。

## 七、建议执行顺序

| 阶段 | 内容 | 验证方式 |
|------|------|----------|
| **阶段 0：止血** | 修复 P0-1（auth 栈 PG→SQLite）、P0-2（两处坏 SQL）、P0-3（订单 zod schema + 下线 404 页面/API） | `cargo check` 全绿 + `cargo test` 恢复运行 + 前端拉订单不 throw |
| **阶段 1：清理** | P1 全部：前端残留下线、死代码删除、脚本/残留库清理、文档同步（AGENTS/README 重写对齐现实）、迁移占位补齐 | `cargo check` + `tsc --noEmit` + `npm run build` 三绿 |
| **阶段 2：架构统一** | 按「六、架构方向决策」逐项实施：模块边界并轨、单租户收口、金额 Decimal 落地（REAL→NUMERIC 重建表）、RBAC 查库实时校验、响应/错误码统一 | 每项独立 PR + 对应测试 |
| **阶段 3：质量** | P3：N+1 修整、分页补全、缓存 key 修复、queryKeys 工厂推广、CI 加 `cargo test`、提交 Cargo.lock、前端 Form zod 化、组件拆分 | CI 全绿 + 测试覆盖率提升 |

> **说明**：阶段 0 完成后即恢复"可编译、可测试"基线。阶段 0/1 不依赖架构决策（六），可立即执行；阶段 2 依赖决策确认。若部分决策被推翻，仅调整阶段 2 的对应子项。
