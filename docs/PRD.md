# ERP v3 — 核心版 PRD

> **版本**: v3.0-draft
> **日期**: 2026-09-03
> **状态**: Draft
> **定位**: 在 v2 正确业务边界之上的一次“面向产品可用性”的重写——把 6 核心业务域做深做真
> **关联**: `rewrite-plan.md`(动机与决策) → 本文档 → `detailed-design.md` → `frontend-design.md` → `tasks.md`

---

## 1. 背景与目标

### 1.1 为什么在 v2 之上再重写
ERP v2 已从旧系统划清了模块边界，但产品做了一半：前端是 API 演示脚手架，核心业务闭环（录带明细的订单→审批→收发→财务）不可用；后端领域模型混乱（PO/SO 状态机混杂）、repo 臃肿。v3 不是改边界，而是**把既有边界内的东西实现成可用产品**（详见 `rewrite-plan.md` §1 的代码级证据）。

### 1.2 本次目标（一句话）
**交付一个真实可进销存+财务闭环的 ERP**，满足：录单带行明细、外键显示名称、按钮由状态机驱动、后端领域模型干净。

---

## 2. 产品定位与范围

- 架构：单厂、单实例、单租户
- 数据库：SQLite3 单文件 WAL
- 语言：zh-CN
- 并发：≥ 20 在线

**模块构成（合理增删结论，不新增业务域）**：
- 底座: Auth & RBAC
- 6 业务域: 商品 / 往来单位 / 库存 / 采购 / 销售 / 财务
- 2 横切能力: 审批流(Workflow) / 报表(Reports)

---

## 3. 用户角色

| 角色 | 职责 | 权限组 |
|---|---|---|
| admin | 系统/主数据/全权限 | 全 |
| manager | 审批+查报表 | 全读+order.approve |
| warehouse | 入库/出库/盘点/库存 | 库存,商品读写 |
| purchaser | 采购/收货 | 采购,供应商读写 |
| sales | 销售/发货 | 销售,客户读写 |
| finance | 科目/日记账/发票/付款/试算平衡 | 财务读写 |

RBAC：查库实时校验，权限即时生效。

---

## 4. 功能需求（FR）—— 增量为主（v2 已有 FR 继承不再复述）

通用增量（决定“可用性”）:
- FR-UX-001 外键显示：所有 API 响应对外键同时返回 `xxx_id` 与 `xxx_name`，前端一律显示名称。
- FR-UX-002 状态机驱动：单据详情/列表按后端返回 `allowed_actions` 渲染可执行动作按钮。
- FR-UX-003 订单行编辑：采购/销售单支持“行项目编辑器”（选商品(名称+搜索)/数量/单价/自动行小计与总额）。

### 4.1 库存 Inventory
- FR-INV-001 仓库/库位
- FR-INV-002/003 入库单 + 过账（事务）
- FR-INV-004/005 出库单 + 过账（事务）
- FR-INV-006 库存查询（按商品/库位/分类）
- FR-INV-007 库存流水追溯（按 SKU）
- FR-INV-008 盘点（生成→录实盘→差异→调整过账）
- FR-INV-009 ATP 预留与可用量

### 4.2 采购 Purchasing
- FR-PUR-001 采购订单 CRUD（含行明细），状态 draft→submitted→approved→ordered→partially_received→received / cancelled / rejected
- FR-PUR-002 提交触发审批流，审批通过后进入可收货
- FR-PUR-003 收货（部分收货），过账即入库（事务），订单状态联动
- FR-PUR-004 搜索/分页

### 4.3 销售 Sales
- FR-SAL-001 销售订单 CRUD（含行明细），创建自动 ATP 检查
- FR-SAL-002 提交触发审批流
- FR-SAL-003 发货（部分发货），过账扣库存（事务），订单状态联动
- FR-SAL-004 搜索/分页
- FR-SAL-005 ATP=库存余额-预留，不足不可提交

### 4.4 财务 Finance
- FR-FIN-001 科目树
- FR-FIN-002 日记账（借贷平衡 Decimal 校验）
- FR-FIN-003 发票
- FR-FIN-004 付款
- FR-FIN-005 试算平衡

### 4.5 审批流 Workflow
- FR-WF-001 数据驱动定义（workflows/states/transitions/amount_threshold）
- FR-WF-002 实例随单据创建
- FR-WF-003 待办任务
- FR-WF-004 approve/reject
- FR-WF-005 接入采购/销售

### 4.6 报表 Reports
- FR-RPT-001 库存汇总
- FR-RPT-002 出入库明细
- FR-RPT-003 销售趋势 + TopN
- FR-RPT-004 财务摘要
- FR-RPT-005 CSV 导出

### 4.7 商品/往来单位/Auth（保持 v2 能力）
- 商品: CRUD(分类/单位/规格/状态 draft→active→disabled) / 搜索分页 / 软删除
- 供应商/客户: CRUD(code 唯一) / 软删除
- Auth: 登录/JWT/refresh 轮换/登出/用户 CRUD/RBAC 校验/操作日志

---

## 5. 明确不做
沿用 v2 裁剪结论（HR/制造/项目/资产/门户/通知中心/多租户/Excel导入等不进入 v3 范围；CSV 商品导入仅保留 v2 已有能力，不强化）。

---

## 6. 非功能需求

| 指标 | 目标 |
|---|---|
| 金额/数量精度 | Decimal 全链；DB TEXT 存；**API JSON 统一用 string**（数量不再走 f64 折损） |
| API 契约 | `{success,request_id,data}` 统一；分页带 meta；错误码分域；不泄露 SQL |
| 事务 | 库存过账/收发联动/财务分录 单事务 |
| 前端可用性 | 外键名称渲染、状态机按钮、订单行编辑器 |
| 响应 | 单页 ≤ 2s(10万级)；20 并发 |
| 安全 | Argon2id、JWT+refresh rotation、RBAC 查库、限流 auth |

---

## 7. 数据模型（第一版实体清单，同 v2 基线；新增要求二：响应含名称字段、单据状态机独立）

表清单沿用 v2（users/roles/role_permissions/user_roles/permissions/refresh_tokens/operation_logs、items、suppliers、customers、warehouses/locations、inventory、inventory_logs、inbound_records/items、outbound_records/items、check_records/items、reservations、purchase_orders/items、sales_orders/items、accounts、journal_entries/lines、invoices、payments、workflows/states/transitions/instances/tasks）。**schema 不变更主结构**；若有新列通过 014+ 迁移追加。

---

## 8. 验收（Definition of Done）
1. `cargo test` 全绿、`bunx vue-tsc --noEmit` 与 `bun run build` 全绿；
2. **人工端到端可走完**：建商品→供应商/客户→录采购单(带多条明细)→提交→审批→收货入库(查库存增加)→录销售单→ATP→审批→发货→库存减少→记日记账→看试算平衡→看报表；
3. 列表/详情均显示实体名称而非 ID；按钮仅展示当前状态+权限下合法动作。

---

## 9. 路线图
见 `rewrite-plan.md` §7（M0 设计 / M1 后端 / M2 前端核心 / M3 收尾 / M4 打磨）。
