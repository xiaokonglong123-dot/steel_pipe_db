# ERP 重写方案 (Rewrite Plan)

> 版本: v1.1 (用户已确认，7 条 ADR 全部定为「已定」)
> 关联: 本文档后续将展开为 PRD → detailed-design → frontend-design → tasks
> 决策前提（用户已确认）: 技术栈保留 (Rust + Vue3)；模块范围按“合理”增删；痛点由本方案诊断收敛

---

## 1. 为什么重写 v2

上一次重构（v2，从钢管 ERP 推倒）把**业务边界划对了**，但**实现没有做成产品**。重写的原因是三个根本问题，均有代码证据：

### 1.1 前端是 demo 脚手架，不是可用产品
- 万能 `CrudList.vue` 套所有列表页，列表显示原始外键 ID（supplier_id / customer_id 是数字），不显示名称。
- 订单详情页（PurchaseOrderDetail / SalesOrderDetail）只用 `el-descriptions` 渲染头字段 4 个，**无任何行项目明细**。
- 详情页按钮硬编码 `['submit','approve','reject','cancel']`，无视当前状态；draft 也显示“审批/收货”。
- 无订单录入表单（PurchaseOrderForm 15 行 / SalesOrderForm 4 行），**核心业务（录带明细订单）根本做不到**。
- 库存查询页要用户手输 item_id 数字，无下拉。
- 类型为 `FormRow = Record<string, string>`，近似 any。

### 1.2 后端领域模型不干净 + repo 臃肿
- `domain/order.rs` 把采购(status Ordered/Received)和销售(AwaitingShipment/Shipped)**塞进同一个 enum，靠注释区分**，语义混乱。
- `inventory_repo.rs` 770 行、`workflow_repo.rs` 623 行、`sales_repo.rs` 536 行 —— 上帝文件。

### 1.3 判定标准错位
- v2 验收的是“API 跑得通”，而非“产品能用”。导致录订单(带明细)这种核心闭环在前端层面不存在。

---

## 2. 这次重写的成功标准 = “可用产品”

重写不是“代码写得更好看”，而是**交付一个真实可操作的 ERP**：

| 层 | v2 的“完成” | 本重写的“完成” |
|---|---|---|
| 前端 | 页面存在、能调 API | 能真录订单(带明细行)、外键显示名称、按钮由状态机驱动 |
| 后端 | 分层存在、测试绿 | 领域模型干净 + repo 精简 + API 契约稳定 |
| 流程 | cargo test 绿 | **端到端人工可走完：建商品→供应商→录采购单→提交→审批→收货→入库存→录销售单→审批→发货→出库→财务→看报表** |

**Definition of Done 全层校验**：每个任务完成都必须同时满足「后端测试通过 + 前端页面可真实操作完成对应业务动作」，缺一不可。

---

## 3. 范围与技术栈（已确认）

| 项 | 决策 | 理由 |
|---|---|---|
| 技术栈 | **保留** Rust(Axum 0.8 + SQLx + SQLite WAL) + 前端 Vue3 + Element Plus + Pinia + TanStack Query | 痛点不在选型，在“做成什么样”；换栈解决不了“demo 前端”的问题 |
| 数据库 | **保留** SQLite 单文件 + WAL（单厂单实例） | 已验证，足够 |
| 部署形态 | 单厂单实例单租户 | 沿用 |
| 语言 | zh-CN 单一 | 沿用 |
| 旧数据 | **不迁移**（v2 也是空库起步；脚本重新生成） | 沿用 |

---

## 4. 模块范围：合理增删后的结论

**不新增业务域**（v1 的教训就是模块膨胀；本次核心是“做深”，不是“做多”）。将现有 9 块重新定位：

### 4.1 平台底座（1）
- **Auth & RBAC**：用户/角色/权限、JWT access+refresh 轮换、操作日志

### 4.2 核心业务域（6）— 本重写主体
| # | 域 | 聚合/实体 | 一句话 |
|---|---|---|---|
| 1 | 商品 Catalog | Item(SKU) | 商品主数据，active 才可交易 |
| 2 | 往来单位 Parties | Supplier, Customer | 供应商/客户 |
| 3 | 库存 Inventory | Location/Inventory/InventoryLog/Inbound/Outbound/Check/Reservation | 入库-出库-盘点-预留-余额双轨 |
| 4 | 采购 Purchasing | PurchaseOrder(+items) | 采购单→收货入库 |
| 5 | 销售 Sales | SalesOrder(+items) | 销售单→发货出库 |
| 6 | 财务 Finance | Account/JournalEntry/JournalLine/Invoice/Payment | 借贷记账→试算平衡 |

### 4.3 横切平台能力（2，非独立业务域）
- **Workflow 审批引擎**：数据驱动(定义/状态/迁移/实例/任务)，被采购、销售引用
- **Reports 报表**：库存汇总/出入库明细/销售趋势/财务摘要 + CSV 导出

> 说明：Workflow 和 Reports 确实是“功能”，但定位是横切能力而非独立业务域（它们不拥有自己的核心交易实体，而是服务/读取其他域）。

---

## 5. 目标架构

### 5.1 后端分层（保留骨架，重排内部）
```
http/       — 路由 + 请求/响应 DTO + 参数校验 (thin)
services/   — 业务规则 + 事务边界 + 领域编排（唯一的业务逻辑所在地）
repos/      — 纯 SQL 数据访问，每聚合一个文件，单文件 ≤ 300 行（超出即拆）
domain/     — 真正的领域模型：聚合根、值对象(Money/Quantity)、各聚合自己的状态机
middleware/ — auth(JWT 注入 AuthUser) + rbac(查库权限)
error.rs    — AppError + 错误码(不泄露 SQL)
response.rs — 统一响应包装
db.rs       — 连接池 + 迁移
```
要点修正：
- **状态机拆分**：`PurchaseOrderStatus`、`SalesOrderStatus` 各自独立 enum + 自己的合法迁移表，不再共用一个 OrderStatus。
- **repo 拆分**：inventory_repo 按子域拆 `inventory_balance_repo` / `inventory_log_repo` / `check_repo` / 等，workflow_repo 同理拆 definition/instance/task。

### 5.2 前端架构（本次重写的核心修复区）
原则：**为“录入与审批”这两个核心交互把产品做真，不是把列表页做全。**

- **类型层**：`types/` 每域一个文件，严格类型（无 `Record<string,string>`）；金额/数量在 TS 侧用 string(十进制) 或 number 必须与后端 JSON 序列化契约一致（后端 Decimal 序列化为 string 保精度）。
- **API 层**：`api/client.ts` 统一封装(错误/401 处理)；每域一个 `api/<domain>.ts`，返回强类型。
- **组件层**：
  - 通用：`PageHeader` / `DataTable` / `PaginationBar` / `FormField`
  - 领域通用：`EntitySelect`(带搜索远程下拉，用于选商品/供应商/客户/库位，显示名称但提交 id)、`MoneyDisplay`、`OrderStatusTag`、`OrderLineEditor`(订单行项目表格编辑器：行可增删、选商品带名称、数量、单价、自动算行小计与总额)
  - 废弃“万能 CrudList”：仅对“纯主数据”(商品/供应商/客户/科目/用户)提供轻量 `MasterDataCrud`；单据(采购/销售)用专用页面。
- **状态机驱动 UI**：详情页根据后端返回的 `status` 和当前用户权限，**只暴露合法动作按钮**(由后端额外返回 `allowed_actions` 防前后端状态逻辑漂移)。
- **外键显示名称**：列表/详情所有关联字段显示可读名称（后端 join 返回 `supplier_name` 等，或前端批量查询映射）。

### 5.3 数据库 schema
- 保留 v2 的表结构与迁移基线（001–013 已验证），**新增迁移继续编号 014+**，且一律不改已执行迁移（规则保留）。
- 新增必要字段：订单表可能需要 `supplier_name/...` 冗余？不——由后端 join 返回，不回写库。
- 保留：物化库存余额 + inventory_logs 双轨；Decimal 全链(TEXT 存储)；审批流数据驱动。

---

## 6. 关键设计决策（ADR）

| ADR | 决策 | 状态 |
|---|---|---|
| ADR-R1 | 状态机按聚合拆分：PurchaseOrderStatus / SalesOrderStatus 独立 | ✅ 已定 (2026-09-03) |
| ADR-R2 | repo 单文件 ≤ 300 行，超出即按子域拆分 | ✅ 已定 (2026-09-03) |
| ADR-R3 | 前端单据(采购/销售)使用专用页面 + OrderLineEditor；万能 CrudList 仅服务纯主数据 | ✅ 已定 (2026-09-03) |
| ADR-R4 | 后端详情接口返回 `allowed_actions`，前端按钮完全由该数组驱动，消除前端硬编码状态逻辑 | ✅ 已定 (2026-09-03) |
| ADR-R5 | 所有外键在 API 响应中同时返回 id + name（如 supplier_id + supplier_name） | ✅ 已定 (2026-09-03) |
| ADR-R6 | 金额/数量: DB TEXT(Decimal) 不变；API JSON 序列化为 **string**，前端按 string 处理保精度（数量也去掉 to_f64 折损） | ✅ 已定 |
| ADR-R7 | “完成”定义全层化：cargo test + 前端真实完成对应业务动作 | ✅ 已定 (2026-09-03) |

---

## 7. 路线图

| 阶段 | 内容 | 交付 | 验收 |
|---|---|---|---|
| M0 设计 | 本方案确认 → PRD / detailed-design / frontend-design / tasks | 4 份文档 | 用户确认 |
| M1 后端核心 | 领域模型重构 + repo 拆分 + API 契约稳态(含 allowed_actions、外键名称) | 后端 + 测试 | cargo test 绿 |
| M2 前端产品 | 核心页重做（商品/供应商/客户 + 采购/销售订单录单+详情+审批 + 库存） | 可操作前端 | 人工跑通端到端闭环 |
| M3 存量域收尾 | 盘点/财务/审批/报表/操作日志 前端 + 报表可视化 | 全量 | 全模块可操作 |
| M4 打磨 | 错误处理/空态/加载/权限细化/性能 | 体验 | 回归 |

---

## 8. 风险

| 风险 | 缓解 |
|---|---|
| 范围蔓延（重蹈 v1 膨胀） | 严格不做新增业务域；砍功能列“明确不做”表 |
| 重写中再次做成 demo | DoD 全层化 + M2 强制人工走通闭环后再进 M3 |
| 后端大重构破坏现有 124 测试 | 先改领域模型保住测试语义，repo 拆分期间保持测试全绿 |

---

## 9. 下一步

本文档是锚点。用户确认后，按顺序产出：
1. `docs/PRD.md`（重写版：动机/目标/范围/NFR/数据模型/路线图）
2. `docs/detailed-design.md`（后端：领域模型、schema、API 契约、错误码、事务边界）
3. `docs/frontend-design.md`（信息架构、路由、组件清单、交互规范、状态设计）
4. `docs/tasks.md`（里程碑任务拆分 + 每任务验收）
