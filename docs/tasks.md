# ERP v2 — 实施任务拆分

> **日期**: 2026-08-09
> **依赖**: [PRD.md](./PRD.md) / [detailed-design.md](./detailed-design.md) / [frontend-design.md](./frontend-design.md)
> **规则 #185**: 设计文档先行——全部设计文档 review 通过后才进入实现。
> **验证基线**: 每阶段结束 `cargo test` 全绿 + `bunx tsc --noEmit` 全绿 + `bun run build` 成功。

---

## 阶段概览

| Phase | 名称 | 任务数 | 预计后端 .rs 行 | 预计前端 .vue/.ts 行 | 里程碑 |
|-------|------|--------|----------------|---------------------|--------|
| **P0** | 核心交易闭环 | 12 | ~3000 | ~4000 | 登录 → 主数据 → 采购订单 → 入库 → 库存查询 → 销售订单 → 发货 → 待办审批 → 库存流水，全链路可走通 |
| **P1** | 财务闭环 | 8 | ~2000 | ~2000 | 科目 → 日记账 → 发票 → 付款 → 试算平衡 → 盘点 → ATP，财务稽核闭环 |
| **P2** | 增强与打磨 | 6 | ~800 | ~1500 | 报表增强 + CSV 导入 + 审批多级 |

---

## P0 — 核心交易闭环（MVP）

### Task 0.1: 项目骨架 + 基础设施

| 项 | 说明 |
|----|------|
| **目标** | 搭建 erp-v2/backend 和 erp-v2/frontend 的基础骨架，跑通 CI |
| **后端范围** | Cargo.toml（依赖清单）、`main.rs`/`lib.rs`/`config.rs`/`db.rs`（池 + 迁移）、`error.rs`（error_codes! 宏 + IntoResponse）、`response.rs`（ApiResponse/PaginatedResponse/Meta）、`.env.example`、`rust-toolchain.toml` |
| **前端范围** | `package.json`、`vite.config.ts`、`tsconfig.json`、`main.ts`（Vue app + Pinia + Router + VueQuery）、`App.vue`、`api/client.ts`、`api/queryClient.ts`、`router/index.ts`、`styles/` |
| **验证** | `cargo check` 绿 + `bun run build` 成功（空页面） |

### Task 0.2: 迁移文件（001-009）

| 项 | 说明 |
|----|------|
| **目标** | 按 detailed-design §4 写出全部 9 个 SQLite 迁移文件，跑通 migration |
| **产出** | `migrations/001_auth_rbac.sql` 到 `009_seed.sql`（9 个文件，全表 ~40 张） |
| **验证** | `cargo test` 中 `test_pool()` 能跑通全量迁移 + bootstrap admin 成功 |

### Task 0.3: Auth 模块（认证 + RBAC + 操作日志）

| 项 | 说明 |
|----|------|
| **后端范围** | `auth.rs`（JWT 签发/校验/refresh rotation、Argon2id 密码哈希）、`middleware/auth.rs`（AuthUser 提取器）、`middleware/rbac.rs`（查库实时权限校验）、`repos/auth_repo.rs`（users/roles/user_roles/refresh_tokens CRUD）、`services/auth_service.rs`（登录/刷新/登出/用户管理）、`http/auth.rs`（login/refresh/logout/me/users CRUD）、`http/mod.rs`（组装认证路由链） |
| **前端范围** | `features/auth/LoginPage.vue`、`features/auth/api.ts`、`features/auth/queryKeys.ts`、`stores/authStore.ts`（token/auth_user/permissions）、`layouts/AppLayout.vue`（侧边栏骨架 + 用户信息 + 退出按钮） |
| **验证** | 手动 E2E：浏览器打开 → 登录（admin/admin123）→ 获取 token → GET /auth/me 返回用户信息 → 退出 → 刷新无法访问。单元测试：`auth_integration.rs` 覆盖登录/刷新/登出/用户 CRUD。 |

### Task 0.4: 商品主数据 (Catalog)

| 项 | 说明 |
|----|------|
| **后端范围** | `repos/catalog_repo.rs`（items CRUD + 搜索分页）、`services/catalog_service.rs`、`http/catalog.rs`（5 个端点） |
| **前端范围** | `features/catalog/ItemListPage.vue`、`ItemFormPage.vue`、`ItemDetailPage.vue`、`api.ts`、`queryKeys.ts`；共享组件 `SearchBar.vue`、`DataTable.vue`、`PageHeader.vue` |
| **验证** | `cargo test` 商品 CRUD 测试绿；前端 → 新建商品 → 列表看到 → 编辑 → 删除（软删除） → 列表消失。搜索/分页正常工作。 |

### Task 0.5: 往来单位 (Parties)

| 项 | 说明 |
|----|------|
| **后端范围** | `repos/parties_repo.rs`（suppliers/customers CRUD 共用泛型或两个文件）、`services/parties_service.rs`、`http/parties.rs`（8 个端点） |
| **前端范围** | `features/parties/SupplierListPage.vue`、`SupplierFormPage.vue`、`CustomerListPage.vue`、`CustomerFormPage.vue`、`api.ts`、`queryKeys.ts`；共享组件 `PartyPicker.vue` |
| **验证** | 供应商/客户 CRUD 测试绿；前端创建供应商 → 采购订单可选供应商。 |

### Task 0.6: 库存 — 库位 + 入库

| 项 | 说明 |
|----|------|
| **后端范围** | `repos/inventory_repo.rs`（locations CRUD, inbound_records/items CRUD, inventory upsert, inventory_logs insert）、`services/inventory_service.rs`（inbound create/post——过账单事务更新库存余额 + 写日志）、`http/inventory.rs`（locations 3 端点 + inbound 4 端点） |
| **前端范围** | `features/inventory/LocationListPage.vue`、`InboundListPage.vue`、`InboundFormPage.vue`（创建入库单：选商品 + 库位 + 数量）、`api.ts`、`queryKeys.ts` |
| **验证** | 创建入库单 → 过账 → `inventory` 表余额增加 → `inventory_logs` 表有记录。单元测试覆盖过账事务。 |

### Task 0.7: 库存 — 出库 + 库存查询 + 流水

| 项 | 说明 |
|----|------|
| **后端范围** | 补全 `services/inventory_service.rs`（outbound create/post——过账扣减，检查余额≥0）、`repos/inventory_repo.rs`（stock query: inventory JOIN items JOIN locations, logs query 筛选+分页）、`http/inventory.rs`（outbound 4 端点 + stock 1 端点 + logs 1 端点） |
| **前端范围** | `features/inventory/OutboundListPage.vue`、`OutboundFormPage.vue`、`StockQueryPage.vue`（实时库存余额）、`InventoryLogsPage.vue`（流水追溯） |
| **验证** | 入库 → 出库 → 库存余额正确扣减。`inventory_logs` 按商品 SKU 展示完整移动轨迹。余额为负时过账拒绝。 |

### Task 0.8: 采购 — 订单 CRUD

| 项 | 说明 |
|----|------|
| **后端范围** | `repos/purchasing_repo.rs`（PO head + lines CRUD，金额 Decimal TEXT）、`services/purchasing_service.rs`（创建时计算行 total_price + 汇总 total_amount to_string 入库）、`http/purchasing.rs`（list/create/get/update/delete 5 端点） |
| **前端范围** | `features/purchasing/PurchaseOrderListPage.vue`、`PurchaseOrderFormPage.vue`（头：选供应商+日期；行：选商品+数量+单价→自动算小计→汇总总金额）、`PurchaseOrderDetailPage.vue`、`api.ts`、`queryKeys.ts`；共享组件 `ItemPicker.vue`、`StatusTag.vue` |
| **验证** | 创建采购订单 → 明细行金额正确 → 总金额汇总正确（Decimal 字符串） → 列表可见。仅 draft 状态可编辑。 |

### Task 0.9: 销售 — 订单 CRUD + ATP

| 项 | 说明 |
|----|------|
| **后端范围** | `repos/sales_repo.rs`（SO head + lines CRUD）、`services/sales_service.rs`（创建时 ATP 检查：available_qty = inventory_qty - SUM(reservations.qty)，库存不足拒绝）、`http/sales.rs`（list/create/get/update/delete 5 端点） |
| **前端范围** | `features/sales/SalesOrderListPage.vue`、`SalesOrderFormPage.vue`、`SalesOrderDetailPage.vue`、`api.ts`、`queryKeys.ts` |
| **验证** | 先入库 100 个商品 A → 创建销售订单 80 个 → 成功。再创建 30 个 → ATP 拒绝。手动 E2E：入库→销售→ATP 检查流水线。 |

### Task 0.10: 审批流引擎

| 项 | 说明 |
|----|------|
| **后端范围** | `repos/workflow_repo.rs`（workflows/states/transitions CRUD + instances/tasks CRUD）、`services/workflow_service.rs`（单据提交→创建 instance+tasks，审批动作→推进 instance.current_state + 更新 task.status，状态迁移联动→回调 order service 更新 PO/SO status）、`http/workflow.rs`（workflows CRUD + tasks list + approve/reject 端点） |
| **前端范围** | `features/workflow/TaskListPage.vue`（我的待办列表）、`api.ts`、`queryKeys.ts`；共享 hooks `useApprove.ts` |
| **验证** | 采购订单提交 → workflow_tasks 有审批待办 → 经理 approve → PO status='approved' → workflow_instance 完成。驳回 → PO status='rejected'。手动 E2E 审批闭环。 |

### Task 0.11: 采购收货 + 销售发货

| 项 | 说明 |
|----|------|
| **后端范围** | 扩展 `services/purchasing_service.rs`（receive: 创建 inbound_record 并自动 post → 更新 PO items.received_qty → 联动 PO status partially_received/received）、扩展 `services/sales_service.rs`（ship: 创建 outbound_record 并自动 post → 释放 reservations → 联动 SO status） |
| **前端范围** | 采购详情页 "收货" 弹窗（输入各商品本次收货数量，≤ 订单行 quantity - received_qty）；销售详情页 "发货" 弹窗。 |
| **验证** | PO approved → 收货 50%（状态 partially_received） → 库存余额增加 → 收货 100%（状态 received）。SO approved → 发货 100% → 库存扣减 → reservation released。 |

### Task 0.12: 端到端集成测试 + CI

| 项 | 说明 |
|----|------|
| **目标** | 跑通 P0 全链路的集成测试 + CI 配置 |
| **后端范围** | `tests/common/mod.rs`（test_pool 辅助）、`tests/p0_e2e.rs`（完整链路：admin login → create item → create supplier → create PO → submit → approve → receive → create customer → create SO → submit → approve → ship → verify stock → verify logs） |
| **前端范围** | CI 配置（`.github/workflows/ci.yml`：`cargo check` + `cargo test` + `cd frontend && bun install && bunx tsc --noEmit && bun run build`） |
| **验证** | `cargo test` 全绿（含 P0 E2E 测试）；`bunx tsc --noEmit` 全绿；`bun run build` 成功；CI pipeline 全绿。 |

---

## P1 — 财务闭环

### Task 1.1: 财务 — 会计科目 + 日记账

| 项 | 说明 |
|----|------|
| **后端范围** | `repos/finance_repo.rs`（accounts CRUD、journal_entries + lines CRUD）、`services/finance_service.rs`（journal_entry create：逐行解析 Decimal，借方合计=`debits.sum()`，贷方合计=`credits.sum()`，`debits.round_dp(4) != credits.round_dp(4)` → UnbalancedJournal(16002)）、`http/finance.rs`（accounts 2 端点 + journal_entries 3 端点） |
| **前端范围** | `features/finance/AccountListPage.vue`、`JournalEntryListPage.vue`、`JournalEntryFormPage.vue`（分录行：选科目 + 借方金额 or 贷方金额 → 提交时前端计算借贷方 → 显示差额提示） |
| **验证** | 借方≠贷方时创建拒绝；借贷平衡时创建成功；`cargo test` 覆盖 Decimal round_dp 精度校验（如 0.1+0.2=0.3）。 |

### Task 1.2: 财务 — 发票 + 付款

| 项 | 说明 |
|----|------|
| **后端范围** | 扩展 `repos/finance_repo.rs`（invoices CRUD, payments CRUD）、`services/finance_service.rs`（payment 创建 → 可选关联 invoice → 更新 invoice.status）、`http/finance.rs`（invoices 3 端点 + payments 2 端点） |
| **前端范围** | `features/finance/InvoiceListPage.vue`（关联 PO/SO）、`PaymentListPage.vue`（关联 supplier + invoice） |
| **验证** | 创建发票 → 关联采购订单 → 创建付款 → 发票状态联动（paid）。 |

### Task 1.3: 财务 — 试算平衡

| 项 | 说明 |
|----|------|
| **后端范围** | `repos/reports_repo.rs` 或 `services/finance_service.rs`（trial_balance: GROUP BY account_id 汇总 journal_entry_lines.debit/credit，按 account 层级输出）、`http/finance.rs`（trial-balance 1 端点） |
| **前端范围** | `features/finance/TrialBalancePage.vue`（科目层级树 + 借方/贷方列，展开明细） |
| **验证** | 有日记账数据 → 试算平衡表显示。借方合计 = 贷方合计（汇总级验证）。 |

### Task 1.4: 盘点 (Check)

| 项 | 说明 |
|----|------|
| **后端范围** | 扩展 `services/inventory_service.rs`（check create: 生成盘点单 + 快照 system_qty；check count: 录入 actual_qty + 计算 diff；check post: 差异 non-zero 行 → 生成 check_adjust 类型的 inventory_logs + 更新 inventory） |
| **前端范围** | `features/inventory/CheckListPage.vue`、`CheckFormPage.vue`（选库位 → 自动列出该库位所有商品+库存余额 → 录入实盘数 → 差异高亮） |
| **验证** | 库存余额 100 → 盘点实盘 95 → 过账差异 adjust → inventory 变 95。差异日志可见。 |

### Task 1.5: ATP 预留

| 项 | 说明 |
|----|------|
| **后端范围** | 扩展 `services/sales_service.rs`（approved → 自动创建 reservations: item_id × order qty, order_type='sales'）、`services/inventory_service.rs`（available_qty 查询: inventory.quantity - COALESCE(SUM(active reservations.quantity), 0)） |
| **前端范围** | `StockQueryPage.vue` 加"可用量"列（平衡 = 库存 - 预留）。销售订单创建时前端展示 ATP 检查结果。 |
| **验证** | 库存 100 → 销售订单 1 预留 40 → 可用量 60 → 超卖拒绝。发货后 reservation released → 可用量恢复。 |

### Task 1.6: 库存流水追溯页面

| 项 | 说明 |
|----|------|
| **前端范围** | `features/inventory/InventoryLogsPage.vue`：按商品 SKU + 日期范围筛选流水记录，表格显示 change_type/quantity/location/ref_type/ref_no/created_at，支持跳转到来源单据 |
| **验证** | 前端 → 搜索某商品 SKU → 显示完整入库→出库→盘点调整轨迹。 |

### Task 1.7: 报表 — 基础汇总

| 项 | 说明 |
|----|------|
| **后端范围** | `repos/reports_repo.rs`（inventory_summary: GROUP BY item.category + item.id；inbound_outbound: 出入库明细 JOIN items；sales_trend: GROUP BY strftime('%Y-%m', order_date)；finance_summary: journal_entry_lines 汇总）、`http/reports.rs`（4 个 endpoint）+ CSV 导出（`content-type: text/csv`） |
| **前端范围** | `features/reports/InventorySummaryPage.vue`、`InboundOutboundPage.vue`、`SalesTrendPage.vue`、`FinanceSummaryPage.vue`，每个带"导出 CSV"按钮 |
| **验证** | 有数据 → 各报表页面有数据 → 导出 CSV 内容正确。 |

### Task 1.8: P1 集成测试

| 项 | 说明 |
|----|------|
| **后端范围** | `tests/p1_e2e.rs`（完整财务链路：科目→日记账→试算平衡→发票→付款；盘点创建→过账→调整验证；ATP 预留→发货→释放验证） |
| **验证** | `cargo test` 全绿。 |

---

## P2 — 增强与打磨

### Task 2.1: 报表 — 图表可视化

| 项 | 说明 |
|----|------|
| **前端范围** | SalesTrendPage + FinanceSummaryPage 加图表（推荐 ECharts 或 @antv/g2，柱状图+折线图），库存汇总加饼图（按分类占比） |
| **验证** | 前端图表正确渲染、可交互。 |

### Task 2.2: 商品 CSV 批量导入

| 项 | 说明 |
|----|------|
| **后端范围** | `http/catalog.rs` 加 POST `/items/import`（接受 multipart CSV，validator 校验行 + 报告成功/失败行数） |
| **前端范围** | ItemListPage 加"导入"按钮 → 上传 CSV 弹窗 → 显示导入报告 |
| **验证** | 上传合法 CSV → 全部成功导入；含坏行的 CSV → 报告失败行数+原因。 |

### Task 2.3: 审批流多级/条件增强

| 项 | 说明 |
|----|------|
| **后端范围** | `services/workflow_service.rs` 支持多节点（按 required_role 串行 asignee），支持金额阈值条件（`total_amount > X` → 追加高级审批节点） |
| **前端范围** | WorkflowListPage 加节点配置 UI |
| **验证** | 多级审批走通（提交→一级 approve→二级 approve→单据 approved）。 |

### Task 2.4: 操作日志页面

| 项 | 说明 |
|----|------|
| **前端范围** | `features/auth/OperationLogPage.vue`（管理员查看：按用户/时间/操作类型筛选，表格展示 + 分页） |
| **验证** | 创建/修改/删除操作 → 操作日志可见。 |

### Task 2.5: UI/UX 打磨

| 项 | 说明 |
|----|------|
| **前端范围** | 深灰侧边栏配色（Element Plus 主题覆盖）；Responsive 适配（移动端最小宽度 1024px）；loading skeletons；404 页面；ElMessage 统一错误提示。 |
| **验证** | 视觉 QA：全页面配色一致、间距合理、操作按钮位置统一。 |

### Task 2.6: 文档同步 + AGENTS.md

| 项 | 说明 |
|----|------|
| **范围** | 更新 `erp-v2/docs/AGENTS.md`（项目规则 + 架构事实，继承根 AGENTS.md 的 `DESIGN_DOCS_FIRST = true`）；清理 v1 时代残留的过期引用 |
| **验证** | 文档内容与代码一致（共享组件数量、endpoint 数、模块目录）——吸取 v1 文档漂移的教训（refactor-issue-list P1-6）。 |

---

## 基础设施备忘

### 环境约束

| 项 | v1 问题 | v2 处理 |
|----|---------|---------|
| npm 不可用 | `npm install` 失败 | 用 `bun install` / `bun run` |
| 无 Docker/Sudo | v1 的 PG 脚本残留 | v2 纯 SQLite，零外部依赖 |
| 无 Makefile | 已告知 | cargo build / bun run 直接跑 |
| Cargo.lock | v1 被 .gitignore 忽略 | v2 **必须提交** Cargo.lock（binary crate 可复现构建，模块声明规约） |

### 金额约束

- 所有金额列 SQLite `TEXT` 存储 `Decimal::to_string()`
- SQL 层**不**做 SUM/AVG 于金额 TEXT 列（SQLite 会 CAST AS REAL 丢精度）
- 金额聚合一律在 Rust service 层用 `Decimal` 累计
- DTO 序列化 Decimal → JSON number（rust_decimal serde feature）

### 测试约束

- 每个 `test_pool()` 创建独立 tempfile SQLite ，跑全量 migration
- 测试库残留由 tempfile Drop 自动清理
- `tests/common/mod.rs` 是唯一测试基建（单一事实源）

---

> **实现顺序强制**: P0 全部完成后才进入 P1；P1 完成后才 P2。每阶段结束时 `cargo test` + `bunx tsc --noEmit` + `bun run build` 必须三绿。
>
> **文档先行**（规则 #185）: 全部设计文档 review 通过后，按此任务拆解进入实现。
