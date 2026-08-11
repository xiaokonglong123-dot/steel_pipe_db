# ERP v2 — 精简核心版 PRD

> **版本**: v2.0-alpha
> **日期**: 2026-08-09
> **状态**: Draft（待 Review）
> **定位**: 全栈重写——精简核心 ERP，8 业务模块 + Auth 底座
> **关联文档**:
>   - [UBIQUITOUS_LANGUAGE_LATEST.md](../../specs/UBIQUITOUS_LANGUAGE_LATEST.md) — 术语规范
>   - [rewrite-tech-stack-comparison.md](../../docs/rewrite-tech-stack-comparison.md) — 技术栈对比（参考）
>   - 后续：`detailed-design.md` / `frontend-design.md` / `tasks.md`

---

## 1. 背景与目标

### 1.1 为什么重写（不是重构）

ERP v1 系统（编号 `Ikari_Shinji`）经历了钢管行业系统→通用 ERP 的多轮演变，累积了不可逆的历史包袱：

| 问题 | v1 现状 | v2 解决 |
|------|--------|---------|
| **历史演化** | 钢管→通用 ERP，PG→SQLite 回迁补丁，37 个迁移里有 5 个"空占位"，020 跳号 | 全新 schema，从零迁移，无演化遗留 |
| **模块膨胀** | 15+ 模块（HR/制造/项目/资产/通知/门户/BI/合同等），大量未启用 | 聚焦 8 核心模块 |
| **金额精度** | f64 落库 + Decimal DTO 层（妥协方案）；借贷平衡用 f64 等值比较有 bug | 全链路 Decimal |
| **架构双轨** | 两套分层（legacy 扁平目录 vs 新模块目录），auth_handler + auth/handlers 并存 | 统一分层 |
| **前端技术** | React+AntD——功能完备但本重写换新栈 | Vue 3 + Element Plus |
| **文档漂移** | AGENTS.md 多处不实（Axios→fetch、共享组件数量错、i18n key 死链） | 设计先行，文档与代码同步 |

**结论**: 不是继续打补丁，而是**推倒重来**——保留 v1 验证过的核心模式（Extension DI、JWT fail-closed、查库 RBAC、审批流数据驱动、TOCTOU 事务），扔掉所有历史包袱。

### 1.2 重写范围

- **新建目录**: `erp-v2/`（仓库内新目录，旧代码留在 git 历史可随时回退）
- **数据策略**: 空库 + 种子起步，**不迁移旧数据**
- **设计先行**（项目规则 #185）: PRD → 详细设计 → 前端设计 → 任务拆分 → 实现
- **功能参照**: 现有实现仅作功能清单、领域逻辑、测试语义的参照，不搬运代码

---

## 2. 产品定位

### 2.1 一句话定位

**精简核心 ERP**：8 个核心业务模块做深做透，支撑中小企业的采购、销售、库存、财务、审批、报表。

### 2.2 部署形态

| 维度 | 决策 |
|------|------|
| 架构 | 单厂、单实例、单租户 |
| 数据库 | SQLite3 单文件（`sqlite://data/erp.db?mode=rwc`，WAL 模式） |
| 语言 | zh-CN 优先（v2.0 单语言交付；i18n 框架保留，英文后续按需） |
| 并发 | ≥ 20 用户同时在线 |
| 数据量 | 商品 1-10 万 SKU，日志百万级 |

---

## 3. 用户角色

| 角色 | 职责 | RBAC 权限组 |
|------|------|------------|
| **管理员 (admin)** | 系统配置、用户管理、主数据维护 | 全部权限 |
| **经理 (manager)** | 审批、查阅报表、业务监督 | 全部读 + 审批 |
| **仓库 (warehouse)** | 入库、出库、盘点、库存查询 | 库存/商品 读写 |
| 采购 (purchaser) | 采购订单、采购收货 | 采购/供应商 读写 |
| 销售 (sales) | 销售订单、发货 | 销售/客户 读写 |
| 财务 (finance) | 会计科目、日记账、发票、付款、试算平衡 | 财务 读写 |

> RBAC 模式：roles + permissions 为唯一权限事实源，middleware 查库实时校验，（权限变更即时生效）。

---

## 4. 模块划分总览

### 4.1 平台底座（非业务模块）

| 底座模块 | 包含 |
|---------|------|
| **Auth & RBAC** | 用户 CRUD、角色/权限管理（查库实时校验）、JWT 登录/刷新/登出、refresh token rotation、操作日志 |

### 4.2 8 个核心业务模块

| # | 模块 | 核心实体 | 一句话范围 |
|---|------|---------|-----------|
| 1 | **商品主数据 (Catalog)** | Item/SKU、分类、单位、规格 | 商品的唯一业务实体管理 |
| 2 | **往来单位 (Parties)** | 供应商、客户 | 供应商与客户主数据 |
| 3 | **库存 (Inventory)** | 仓库库位、库存余额、入库、出库、盘点、ATP 预留、流水追溯 | 从入库到出库的完整物流闭环 |
| 4 | **采购 (Purchasing)** | 采购订单、采购收货 | 采购订单→到货入库的全链条 |
| 5 | **销售 (Sales)** | 销售订单、发货 | 销售订单→发货出库的全链条 |
| 6 | **财务 (Finance)** | 会计科目、日记账、发票、付款、试算平衡 | 从记账到报表的财务闭环 |
| 7 | **审批流 (Workflow)** | 审批定义、实例、任务 | 数据驱动的业务审批引擎 |
| 8 | **报表 (Reports)** | 库存汇总、出入库明细、销售趋势、财务摘要 | 经营决策所需的聚合分析 |

---

## 5. 功能需求（FR）——按模块详列

### 5.0 Auth & RBAC（平台底座）

| 编号 | 功能 | 验收标准 |
|------|------|---------|
| FR-AUTH-001 | 用户登录 | POST `/auth/login`，Argon2id 验证，返回 access token（httpOnly cookie） + refresh token |
| FR-AUTH-002 | Token 刷新 | POST `/auth/refresh`，refresh token 轮换（SHA-256 hash 存储），旧 token 吊销 |
| FR-AUTH-003 | 用户登出 | POST `/auth/logout`，吊销该用户所有 refresh token |
| FR-AUTH-004 | 用户管理 | CRUD，管理员可增删改查用户，含角色分配 |
| FR-AUTH-005 | RBAC 权限校验 | middleware 级查库校验（user→roles→permissions），权限变更即时生效，token 不带 permissions |
| FR-AUTH-006 | 操作日志 | 记录登录、数据修改等关键操作（user_id / action / target / timestamp） |

### 5.1 商品主数据 (Catalog)

| 编号 | 功能 | 验收标准 |
|------|------|---------|
| FR-CAT-001 | 商品 CRUD | 创建/更新/删除/查询商品，SKU 全局唯一，必填字段：SKU、名称 |
| FR-CAT-002 | 分类与单位 | 分类预置（原材料/半成品/成品/备件），单位自由文本（kg/m/pc/件等） |
| FR-CAT-003 | 规格 | 自由文本规格字段，不承载行业强制字段 |
| FR-CAT-004 | 商品状态 | draft → active → disabled，仅 active 可交易 |
| FR-CAT-005 | 搜索与分页 | 按 SKU/名称/分类/单位/规格/状态组合筛选，模糊搜索，分页返回 |
| FR-CAT-006 | 软删除 | `deleted_at` 非空即已删除，列表默认过滤 |

### 5.2 往来单位 (Parties)

| 编号 | 功能 | 验收标准 |
|------|------|---------|
| FR-PTY-001 | 供应商 CRUD | 编码（code）唯一；字段：编码、名称、联系人、电话、邮箱、地址、状态(active/inactive)；软删除 |
| FR-PTY-002 | 客户 CRUD | 同上（编码唯一），无客户信用模块（v2 不做客户信用） |
| FR-PTY-003 | 搜索筛选 | 按编码/名称/状态搜索，分页 |

### 5.3 库存 (Inventory)

| 编号 | 功能 | 验收标准 |
|------|------|---------|
| FR-INV-001 | 仓库库位 | 仓库/库位层级（location 表，code 唯一），状态 active/inactive |
| FR-INV-002 | 入库单 | 头+行，类型：采购收货/生产完工/退货/其他；创建后待过账 |
| FR-INV-003 | 入库过账 | 过账→单事务更新物化库存余额 + 写 inventory_logs |
| FR-INV-004 | 出库单 | 头+行，类型：销售发货/内部领用/其他 |
| FR-INV-005 | 出库过账 | 过账→单事务扣减库存余额 + 写 inventory_logs |
| FR-INV-006 | 库存查询 | 按商品/库位/分类实时查询库存余额 |
| FR-INV-007 | 库存流水追溯 | 按商品 SKU 查看完整 inventory_logs 移动轨迹（入库/出库/盘点调整） |
| FR-INV-008 | 盘点 | 生成盘点单→录入实盘数→差异报告→调整过账 |
| FR-INV-009 | ATP 预留 | 销售订单可预留库存；可用量 = 库存余额 - 已预留量；防超卖（v2 统一口径，仅此一套） |

### 5.4 采购 (Purchasing)

| 编号 | 功能 | 验收标准 |
|------|------|---------|
| FR-PUR-001 | 采购订单 CRUD | 头（供应商、日期、状态、金额）+ 行（商品、数量、单价）；状态：draft→submitted→approved→ordered→partially_received→received→cancelled |
| FR-PUR-002 | 审批接入 | 采购订单提交后自动创建审批流实例，审批通过后状态变更为 approved |
| FR-PUR-003 | 采购收货 | 关联采购订单，录入收货数量（部分收货），过账即入库（写 inventory_logs + 更新库存余额），订单状态联动 |
| FR-PUR-004 | 订单列表/详情/搜索 | 按供应商/状态/日期范围/订单号搜索 |

### 5.5 销售 (Sales)

| 编号 | 功能 | 验收标准 |
|------|------|---------|
| FR-SAL-001 | 销售订单 CRUD | 头（客户、日期、状态、金额）+ 行（商品、数量、单价）；创建时自动 ATP 检查（库存可用量≥订单量） |
| FR-SAL-002 | 审批接入 | 提交→创建审批流，审批通过后进入待发货状态 |
| FR-SAL-003 | 发货 | 关联销售订单，填写发货数量，过账即出库（自动扣减库存 + 写 inventory_logs），订单状态联动 |
| FR-SAL-004 | 订单列表/详情/搜索 | 按客户/状态/日期范围/订单号搜索 |
| FR-SAL-005 | ATP 检查 | 创建/编辑时实时计算可用库存 = 库存余额 - 预留量；库存不足不允许提交 |

### 5.6 财务 (Finance)

| 编号 | 功能 | 验收标准 |
|------|------|---------|
| FR-FIN-001 | 会计科目 | 预置标准科目树（资产/负债/权益/收入/费用），支持自定义子科目 |
| FR-FIN-002 | 日记账 | 分录记录（日期/科目/摘要/借方/贷方），**借贷平衡用 rust_decimal 累计 + round_dp(4) 校验**，不平衡拒绝 |
| FR-FIN-003 | 发票 | 关联采购/销售订单，记录发票号/金额/日期 |
| FR-FIN-004 | 付款 | 记录对外付款（供应商/金额/日期），关联发票 |
| FR-FIN-005 | 试算平衡 | 按科目汇总借方/贷方发生额，输出试算平衡表 |

> **金额精度**：全链路 rust_decimal，DB 存储层在详细设计中确定（候选：TEXT 十进制字符串 / 整数分）；禁止 f64 存金额。

### 5.7 审批流 (Workflow)

| 编号 | 功能 | 验收标准 |
|------|------|---------|
| FR-WF-001 | 审批流定义 | 数据驱动（workflows/states/transitions 表），定义项包含：适用单据类型、节点、迁移规则；加节点改数据不改代码 |
| FR-WF-002 | 审批流实例 | 单据提交→创建实例；状态随审批进展推进 |
| FR-WF-003 | 审批任务 | 每个审批节点生成待办任务（assignee/status/comment），审批人有待办列表 |
| FR-WF-004 | 审批动作 | approve/reject，动作触发状态迁移 |
| FR-WF-005 | 接入单据 | 采购订单、销售订单 的提交/审批/驳回接入审批流 |

### 5.8 报表 (Reports)

| 编号 | 功能 | 验收标准 |
|------|------|---------|
| FR-RPT-001 | 库存汇总 | 按商品/分类维度汇总库存余额 |
| FR-RPT-002 | 出入库明细 | 按时间范围/商品筛选出入库记录 |
| FR-RPT-003 | 销售分析 | 销售额趋势（按月/季），Top N 商品/客户 |
| FR-RPT-004 | 财务摘要 | 本日/本月/本季收入、支出、应收、应付汇总 |
| FR-RPT-005 | 导出 | 报表结果导出 CSV |

---

## 6. 明确不做（砍掉的模块）

| 砍掉的模块/功能 | v1 对应 | 砍掉理由 | 可恢复性 |
|----------------|---------|---------|---------|
| **HR** | 员工/考勤/薪资/劳动合同 | 超出核心进销存+财务闭环 | P3 独立模块 |
| **制造** | BOM/工单/质检/NCR | 非核心业务 | P3 独立模块 |
| **项目** | 项目/WBS/预算 | 非核心业务 | P3 |
| **固定资产** | 登记/折旧/处置 | 非核心业务 | P3 |
| **通知平台** | 收件箱/模板/偏好 | 审批待办即通知；独立通知中心不建 | 审批流内邮件通知可 P2 |
| **门户 Portal** | 外部 Party 账户/PO 接受/SO 确认 | 精简核心不考虑外部门户 | P3 |
| **BI 独立模块** | `bi/` 命名空间、sqlx 大查询 | 并入报表的精简聚合分析（RPT） | — |
| **合同独立模块** | `contracts/` 表+页面 | 不在 8 模块内；订单可带可选备注/关联编号 | P2 可选补回 |
| **采购申请/报价/评分** | Requisition/Supplier Quote/Scorecard | 精简采购：只做订单+收货闭环 | P2 可选 |
| **销售报价/客户信用** | Customer Quote/Credit | 精简销售：只做订单+发货闭环 | P2 可选 |
| **数据导入 Excel/XLSX** | calamine + rust_xlsxwriter 导入 | 精简依赖；CSV 导入商品 P2 可选 | P2 |
| **多租户** | 部分表有 tenant_id 残留 | 明确单租户，移除所有 tenant_id | 如需转 SaaS 再做 |
| **i18n 双语言** | 33 个 zh/en locale 文件，大量死 key | v2.0 仅 zh-CN；前端保留轻量 i18n 架构但内容只做中文 | 不再主动做 en |

---

## 7. 非功能需求

### 7.1 性能

| 指标 | 目标 |
|------|------|
| 单页查询响应 | ≤ 2 秒（10 万条） |
| 并发用户 | ≥ 20 同时在线 |
| 可用性 | 99.5%（年停机 ≤ 44 小时） |
| 数据量 | 商品 1-10 万 SKU，日志百万级，SQLite WAL |

### 7.2 安全

- 密码：Argon2id（`m=19456, t=2, p=1`）
- 认证：Stateless JWT (HS256)，access token httpOnly cookie，refresh token 服务器端 SHA-256 hash 存储 + 轮换
- RBAC：middleware 查库实时校验 user→roles→permissions，权限变更即时生效
- 限流：auth endpoint IP-based 限流
- 软删除：所有业务实体 `deleted_at` 不为空即删除
- 操作日志：关键操作（创建/修改/删除/登录）全记录

### 7.3 金额精度

- **原则**：全链路 rust_decimal（禁止 f64 存金额）
- **业务约定**：金额计算、比较、借贷平衡一律用 `rust_decimal::Decimal`
- **存储方案**（详细设计确认）：候选方案包括 TEXT 十进制字符串或 INTEGER 最小单位（分/微单位）；禁止 REAL/f64 存储
- **SQL 聚合**：金额汇总优先在应用层用 Decimal 累计；SQL 视图中如需 SUM 则用 CAST 后 Decimal 回读校验

### 7.4 API 契约（与 v1 对齐）

**成功响应**（继承 v1 已定型的形状）：

```json
// 单条
{ "success": true, "request_id": "req_...", "data": { ... } }
// 分页
{ "success": true, "request_id": "req_...", "data": { "items": [...] }, "meta": { "total": N, "page": P, "page_size": S, "total_pages": N } }
```

**错误响应**：

```json
{ "success": false, "code": 11001, "request_id": "req_...", "message": "...", "details": null }
```

**HTTP 语义**：
- 创建 → 201 `ApiResponse::created()`
- 删除 → 204 空 body
- 错误码分域（继承 v1 码表）：

| 域 | 范围 | 示例 |
|----|------|------|
| 通用 | 100xx | Validation(10002)、NotFound(10003) |
| Auth | 110xx | Unauthorized(11001)、TokenExpired(11002)、Forbidden(11003) |
| 商品 | 120xx | ItemNotFound(12001) |
| 库存 | 130xx | InsufficientStock(13001)、LocationNotFound(13002) |
| 订单 | 140xx | OrderNotFound(14002)、CannotModify(14001) |
| 往来单位 | 150xx | SupplierNotFound(15001)、CustomerNotFound(15002) |
| 财务 | 160xx | AccountNotFound(16001)、UnbalancedJournal(16002) |
| 审批流 | 170xx | WorkflowNotFound(17001) |
| 报表 | 180xx | — |
| 数据库 | 50001 | Database |

> **严禁**：向客户端暴露原始 SQL 错误字符串；From<sqlx::Error> 一律转 50001 Database。

### 7.5 事务边界

- 库存过账（入库/出库）：**单事务**（更新库存余额 + 插入 inventory_log）
- 采购收货/销售发货：**单事务**（更新订单状态 + 库存过账）
- 财务日记账：借贷方插入单事务

---

## 8. 数据模型（第一版）

> 详细 schema 在 `detailed-design.md` 中展开，此处仅列实体清单和关键关系。

### 8.1 Auth 底座

| 表 | 说明 |
|----|------|
| `users` | 用户（username/display_name/password_hash, active, deleted_at） |
| `roles` | 角色（name, is_system） |
| `permissions` | 权限点（key, name），预置 item.read/write, stock.read/write, order.read/write/approve, finance.read/write, report.read, user.manage |
| `role_permissions` | 角色-权限关联 |
| `user_roles` | 用户-角色关联（多对多，查库实时校验的权威源） |
| `refresh_tokens` | token_hash/expires_at/revoked_at（轮换 + 吊销） |
| `operation_logs` | 操作审计（user_id/action/target/ip/created_at） |

### 8.2 Catalog — 商品主数据

| 表 | 说明 |
|----|------|
| `items` | SKU 唯一、名称、分类、单位、规格、状态 |

### 8.3 Parties — 往来单位

| 表 | 说明 |
|----|------|
| `suppliers` | 供应商主数据（code 唯一） |
| `customers` | 客户主数据（code 唯一） |

### 8.4 Inventory — 库存

| 表 | 说明 |
|----|------|
| `locations` | 仓库/库位层级（code 唯一） |
| `inventory` | **物化余额表**：item_id × location_id × quantity |
| `inventory_logs` | **事件日志**：change_type(inbound/outbound/transfer_in/transfer_out/check_adjust)、quantity、ref_type、ref_id、created_at（完整审计轨迹） |
| `inbound_records` | 入库单头（order_id 关联 PO、supplier_id、status、created_at） |
| `inbound_items` | 入库单行（item_id、quantity） |
| `outbound_records` | 出库单头 |
| `outbound_items` | 出库单行 |
| `check_records` | 盘点单头（location_id、status） |
| `check_items` | 盘点单行（item_id、system_qty、actual_qty、diff） |
| `reservations` | ATP 预留（item_id、quantity、order_type、order_id、status） |

### 8.5 Purchasing — 采购

| 表 | 说明 |
|----|------|
| `purchase_orders` | 采购订单头（order_no、supplier_id、status、doc_status、total_amount） |
| `purchase_order_items` | 采购订单行（item_id、quantity、unit_price、total_price） |

### 8.6 Sales — 销售

| 表 | 说明 |
|----|------|
| `sales_orders` | 销售订单头 |
| `sales_order_items` | 销售订单行 |

### 8.7 Finance — 财务

| 表 | 说明 |
|----|------|
| `accounts` | 会计科目（code、name、parent_id、type） |
| `journal_entries` | 日记账分录（account_id、debit/credit/decimal、date、description、ref_type、ref_id） |
| `invoices` | 发票（关联 PO/SO、金额、日期） |
| `payments` | 付款（供应商、金额、日期、关联发票） |

### 8.8 Workflow — 审批流

| 表 | 说明 |
|----|------|
| `workflows` | 审批流定义（name、applies_to(purchase_order/sales_order)、is_active） |
| `workflow_states` | 状态（workflow_id、state_key、doc_status） |
| `workflow_transitions` | 状态迁移规则（from_state→to_state、action、required_role） |
| `workflow_instances` | 审批流实例（workflow_id、business_type、business_id、current_state、status） |
| `workflow_tasks` | 审批任务（instance_id、assignee_id、action 可选、status、comment） |

---

## 9. 技术栈决策

| 层 | 选型 | 说明 |
|----|------|------|
| **后端语言** | Rust (edition 2021, stable) | 保留（v1 已验证） |
| **Web 框架** | Axum 0.8 | `Extension<SqlitePool>` + `Extension<JwtSecret>`（v1 模式） |
| **ORM** | SQLx 0.8 (`sqlite` feature) | 编译期 SQL 检查 |
| **数据库** | SQLite3（WAL，单文件 `data/erp.db`） | 零配置、单实例足够 |
| **认证** | jsonwebtoken 9 + argon2 0.5 | JWT + refresh rotation + httpOnly cookie |
| **金额** | rust_decimal 1 + 存储策略（TEXT/整数分） | 全链路 Decimal |
| **前端框架** | Vue 3 + TypeScript + Vite | 从 React 换栈 |
| **UI 组件** | Element Plus | 表格/表单/日期/对话框全 |
| **状态管理** | Pinia | Vue 生态标准 |
| **服务端状态** | `@tanstack/vue-query` | 缓存/去重/重取（v1 TanStack Query 的 Vue 对应） |
| **路由** | Vue Router 4 | — |
| **验证** | zod（前后端共用 schema？） | 前端响应校验 |
| **构建** | Vite | — |
| **日志** | tracing + tracing-subscriber | — |

---

## 10. 优先级路线图

| Phase | 范围 | 优先级 | 预计产出 | 验证方式 |
|-------|------|--------|---------|---------|
| **P0 (MVP)** | Auth+RBAC、商品主数据、往来单位、库存（入库/出库/查询/流水）、采购订单+收货+审批、销售订单+发货+审批、审批流引擎、操作日志 | 🔴 Must Have | 核心交易闭环：采购→入库→库存→销售→出库 | cargo test 全绿 + 手动 E2E |
| **P1 (Finance)** | 财务（科目/日记账/发票/付款/试算平衡）、盘点、ATP 预留、报表基础（库存汇总/出入库明细/CSV 导出） | 🟡 Should Have | 财务闭环 | cargo test + 试算平衡校验 |
| **P2 (Polish)** | 报表增强（销售趋势/财务摘要）、商品 CSV 导入、审批流条件/多级增强 | 🟢 Nice to Have | 完整 8 模块 | cargo test + tsc + vite build |

---

## 11. 术语

本 PRD 所有术语遵守项目术语表 `specs/UBIQUITOUS_LANGUAGE_LATEST.md`：

- **商品 (Item)** — 唯一业务实体，由 **SKU** 标识
- **供应商 / 客户** — 往来单位
- **采购订单** — 向供应商采购的正式单据
- **销售订单** — 客户下达的销售单据
- **入库 (Inbound)** — 商品进入库存
- **出库 (Outbound)** — 商品离开库存
- **库存 (Inventory)** — 库位上的商品存量
- **盘点 (Count Session)** — 账面与实物核对
- **审批流 (Workflow)** — 单据的审批流转
- **日记账 (Journal Entry)** — 财务分录记录
- **试算平衡 (Trial Balance)** — 借贷平衡报表

已废弃的旧术语（禁止使用）：钢管、管号、质证书、API 5CT、标签打印、螺纹加工。

---

## 12. 决策记录（ADR）

| 决策 | 内容 | 日期 | 状态 |
|------|------|------|------|
| **ADR-001** | 全栈重写（非重构），新技术栈前端 Vue 3 + 后端保留 Rust | 2026-08-09 | ✅ 已定 |
| **ADR-002** | ⚠️ 金额存储方案：全链路 Decimal，DB 候选为 TEXT 字符串或整数分（禁止 f64）→ 详细设计中定 | 2026-08-09 | ⚠️ 待定 |
| **ADR-003** | 库存模型：物化余额表 + inventory_logs 审计日志双轨（继承 v1 已验证模式） | 2026-08-09 | ✅ 已定 |
| **ADR-004** | 审批流：数据驱动（workflows/states/transitions），加节点改数据不改代码 | 2026-08-09 | ✅ 已定 |
| **ADR-005** | 响应语义：成功统一 {success,request_id,data}，201 创建，204 删除，错误码分域 | 2026-08-09 | ✅ 已定 |
| **ADR-006** | 单语言 zh-CN（前端保留轻量 i18n 结构但内容仅中文；英文 P2 可选） | 2026-08-09 | ✅ 已定 |

---

> **下一步**: 待此 PRD Review 通过后，产出 `detailed-design.md`（后端架构 + DB schema + API 契约）和 `frontend-design.md`（Vue 3 组件树 + 路由 + 状态设计）。
