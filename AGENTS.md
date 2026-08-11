# ERP v2 — Project Status (P0 + P1 + P2 完成)

> 重写项目。详细拆分见同目录 `PRD.md` / `detailed-design.md` / `frontend-design.md` / `tasks.md`。

## 启动

```bash
# Backend (Rust Axum :3000)
cd backend && cp .env.example .env && cargo run

# Frontend (Vue 3 + Element Plus :5173)
cd frontend && bun install && bun run dev

# 登录: admin / admin123
```

## 当前实现状态

### 后端 (`backend/`)

| 模块 | 状态 | 测试 |
| --- | --- | --- |
| Auth+RBAC | ✅ P0 完成 | 4 |
| Catalog | ✅ P0 完成 | 8 |
| Parties | ✅ P0 完成 | 9 |
| Locations+Inbound+Outbound+Stock | ✅ P0 完成 | 22 (12 inv + 10) |
| 采购订单 (含 Decimal 金额) | ✅ P0 完成 | 13 |
| 销售订单 + ATP 预留 | ✅ P0 完成 | 14 |
| 审批流 (data-driven ERPNext-style) | ✅ P0 完成 | 13 |
| 收货 + 发货 (端到端联动) | ✅ P0 完成 | 9 |
| **E2E** PO+SO 端到端 | ✅ P0 完成 | 2 |
| Finance (会计科目/日记账/发票/付款/试算平衡) | ✅ P1 完成 | (含 lib) |
| Inventory Check 盘点 | ✅ P1 完成 | 5 |
| ATP `available_qty` 端点 + service | ✅ P1 完成 | 1 |
| Reports (inventory_summary/inbound_outbound/sales_trend/finance_summary + CSV) | ✅ P1 完成 | — |
| **P1 E2E** (finance + check + atp 释放) | ✅ P1 完成 | 4 |
| CSV 商品导入 (`POST /items/import` multipart) | ✅ P2 完成 | 2 |
| 审批流多级/条件 (`amount_threshold` + `transition_with_amount`) | ✅ P2 完成 | 2 |
| finance.threshold `012_workflow_threshold.sql` | ✅ P2 完成 | 1 migrations 校验 |

**总计**: 121 测试全绿 (`cargo test`)

### 前端 (`frontend/`)

| 模块 | 状态 |
| --- | --- |
| Vue 3 + Pinia + Element Plus + TanStack Vue Query 骨架 | ✅ |
| Auth (Login + MainLayout + RBAC permission 守卫) | ✅ |
| 商品/供应商/客户/仓库/库位/库存/入库/出库 | ✅ |
| 采购订单/销售订单 (含审批按钮) | ✅ |
| Workflow 流程实例/待办 | ✅ |
| Finance 5 页 (AccountList/JournalEntryList/InvoiceList/PaymentList/TrialBalance) | ✅ |
| Inventory Check + Inventory Logs + ATP 可用量 | ✅ |
| Reports 4 页 + ECharts 可视化 (line+bar) | ✅ |
| CSV 导入按钮 + 报告弹窗 | ✅ |
| 操作日志页面 | ✅ |
| 404 + 深灰侧边栏 + skeleton + 路由过渡动画 + 统一错误提示 | ✅ |

**总计**: `bunx vue-tsc --noEmit` + `bun run build` 全绿

### CI

`.github/workflows/ci.yml`: backend `cargo check --all-targets` + `cargo test --all`；frontend `bunx tsc --noEmit` + `bun run build`。

## 关键设计决策

- **Money 全链 Decimal**: `rust_decimal::Decimal`，DB 存 `TEXT`，`Decimal::to_string()`。**不允许 SQL SUM over TEXT money 列**——报表金额聚合在应用层做。日期分组 REAL 聚合允许 (ADR-002 例外)
- **Inventory 双轨**: 物化 `inventory` 表（balance）+ `inventory_logs` 事件日志（audit trail）。Inbound 写 +OUT 检查余额
- **审批流 data-driven**: `workflows` / `workflow_states` / `workflow_transitions`。新增节点不出代码——只需 INSERT 行
- **金额阈值条件**: `workflow_transitions.amount_threshold TEXT`，`workflow_service::transition_with_amount(pool, inst_id, action, user, comment, business_amount: Option<Decimal>)` 会优先选 threshold 满足的 transition；不满足则 fallback 走无阈值的 transition
- **RBAC 实时查库**: JWT 携带 `user_id` + `permissions` 数组，每个请求 auth_middleware 查库注入 AuthUser 扩展
- **Spec Drift 处理**: 010_warehouses / 011_seed_workflows / 012_workflow_threshold 三条迁移均不修改已执行迁移，按规则 #234 + touch main.rs 触发重编译
- **设计文档已同步**: detailed-design.md 反映 010_warehouses 父表层级；detailed-design + tasks 同步 inventory API 短路径

## Spec Drift 全记录（避免未来读者困惑）

1. `010_warehouses.sql`: 先 `warehouses` + `locations.warehouse_id/deleted_at`，原 spec 没有 `warehouses` 表 (Locations 子代理正确 ALTER 而不是改 main migration)
2. `011_seed_workflows.sql`: 注入 PO/SO active workflow 数据，含 4 states + 3 transitions 各。这条新迁移通过 `UPDATE workflows SET is_active = 0` 发挥 (workflow_threshold_test 的方式) — 测试时主动 deactivate 011 注入以使用 demo seed
3. `012_workflow_threshold.sql`: `ALTER TABLE workflow_transitions ADD COLUMN amount_threshold TEXT`。**触发条件**：存在 business_amount ≥ threshold 的 transition，否则 fallback 走 NULL threshold 的 transition
4. `WorkflowTransitionRow` 增 `amount_threshold: Option<String>`，影响 SELECT 列出 4 处 SQL 的列名：`list_outgoing_transitions` / `find_transition` / `list_transitions_by_action` / `INSERT` 重载 — 已统一对齐
5. `WorkflowStateRow.doc_status` 是 **INTEGER** 而非 TEXT (008 schema)；任何 INSERT workflow_states 必须传 integer（seed_total 0=draft/1=submitted/2=senior_review/3=approved/4=rejected 各自映射）
6. 2 个 stale workflow 测试因 011 seed 失效已修：`start_instance_without_active_workflow` 用 `"nonexistent_workflow_xyz"`；`delete_workflow_with_running_instance` 用 011 注入的 wf_id（保证 `find_active_workflow_by_type` 返回带 instance 的那条）
7. Frontend 修正了 permission 名字对齐后端实际：`item.read`/`stock.read`/`order.read`/`order.approve`/`finance.read`/`report.read`/`user.manage`

## 已完成的迁移文件

```
001_auth_rbac.sql          — users / roles / role_permissions / operation_logs / refresh_tokens
002_catalog.sql            — items (SKU master, 含 draft/active/disabled)
003_parties.sql            — suppliers / customers
004_inventory.sql          — warehouses / locations / inventory / inventory_logs
005_purchasing.sql         — purchase_orders / purchase_order_items (+ DOC_DRAFT 0 自增)
006_sales.sql              — sales_orders / sales_order_items / reservations
007_finance.sql            — accounts / journal_entries / journal_lines / invoices / payments
008_workflow.sql           — workflows / workflow_states / workflow_transitions / workflow_instances / workflow_tasks
009_seed.sql               — admin/manager/finance 角色 + roles 11 个权限 (不是 12)
010_warehouses.sql         — ALTER locations.warehouse_id + deleted_at (在 child 迁移之后的 ALTER)
011_seed_workflows.sql     — PO/SO 种子 workflows + states + transitions
012_workflow_threshold.sql — ALTER workflow_transitions.amount_threshold TEXT
```

## P0/P1/P2 完成度

- P0 (核心交易闭环): ✅ 全部 12 任务完成
- P1 (财务 + 报表 + ATP): ✅ 全部 9 任务完成
- P2 (增强 + Excel 导入 + UI 打磨): ✅ 全部 6 任务完成

**项目重写 MVP 全部完成交付。**
