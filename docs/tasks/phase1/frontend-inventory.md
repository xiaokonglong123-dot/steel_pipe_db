# Phase 1 — 前端：库存管理模块 (P0 MVP)

> 基于：`docs/前端设计文档.md` §4.1, §4.2, §6, §8

## 任务清单

### 1.1 共享类型与 API
- [ ] 定义 `features/inventory/types.ts`：InboundRecord, OutboundRecord, InventoryItem, InventoryLog, Location, CheckRecord, CheckItem 等类型
- [ ] 定义 `features/inventory/api/inventoryApi.ts`：
  - `getInboundRecords(...)` / `getInboundRecord(id)` / `createInboundRecord(data)` / `deleteInboundRecord(id)`
  - `approveInbound(id)` / `rejectInbound(id, reason)` — 审批/驳回非采购入库单
  - `getOutboundRecords(...)` / `getOutboundRecord(...)` / `createOutboundRecord(...)` / `deleteOutboundRecord(...)`
  - `approveOutbound(id)` / `rejectOutbound(id, reason)` — 审批/驳回非销售出库单
  - `getInventory(filter)` — 实时库存
  - `getInventoryLogs(filter)` / `getPipeLifecycle(pipeType, pipeId)`
  - `createCheck(data)` / `submitCheckItem(checkId, itemId, found)` / `getCheckReport(id)` / `getCheckList(filter)`
  - `getLocations(filter)` / `createLocation(data)` / `assignPipe(pipeId, locationId)` / `transferPipe(pipeId, newLocationId)`
- [ ] 实现 `hooks/useInventory.ts`（React Query hooks 封装）

### 1.2 库存共享组件
- [ ] 实现 `StockSummaryCards`：库存总览 KPI 卡片（总库存、各类型数量、在库/出库占比）
- [ ] 实现 `InventoryTable`：实时库存表格（按钢级/规格分组聚合展示）
- [ ] 实现 `LocationTree`：库位树形组件（库区→货架→层位）

### 1.3 库存查询页面
- [ ] 实现 `InventoryPage`（实时库存页面）：
  - 顶部 StockSummaryCards
  - 筛选条件（管材类型、钢级、库位）
  - InventoryTable 展示
  - 点击行可查看该规格下的管材明细列表

### 1.4 入库管理页面
- [ ] 实现 `InboundListPage`：
  - 入库记录表格（入库单号、日期、类型、供应商、状态）
  - 筛选（日期范围、入库类型）
  - 操作：查看详情、新增入库
- [ ] 实现 `InboundFormPage`（新增入库）：
  - 选择入库类型（采购入库/生产入库/退货入库）
  - 类型联动规则：
    - 采购入库：order_id 为必填，弹出采购订单选择器筛选已审核订单；自动校验所选管材与 PO 明细一致
    - 生产入库/退货入库：无需 order_id，创建后需仓库主管审批，页面提示"需主管审批后生效"
  - 从管材列表选择要入库的管材（支持批量选择 + 批量新建管材）
  - 提交 → 创建入库单
- [ ] 实现 `InboundApprovalPanel`（入库审批面板）：
  - 展示待审批入库单列表（approval_status=pending, inbound_type=production/return）
  - 点击查看详情 + 审批/驳回按钮
  - 审批通过后自动刷新库存；驳回需填写原因

### 1.5 出库管理页面
- [ ] 实现 `OutboundListPage`（结构同入库列表）：
  - 出库记录表格 + 筛选 + 操作
- [ ] 实现 `OutboundFormPage`（新增出库）：
  - 选择出库类型（销售出库/调拨出库/报废）
  - 类型联动规则：
    - 销售出库：order_id 为必填，弹出销售订单选择器筛选已审核订单；自动校验所选管材与 SO 明细一致
    - 调拨出库/报废出库：无需 order_id，创建后需仓库主管审批，页面提示"需主管审批后生效"
  - 从在库管材列表选择要出库的管材
  - 提交 → 校验库存 → 创建出库单
- [ ] 实现 `OutboundApprovalPanel`（出库审批面板）：
  - 展示待审批出库单列表（approval_status=pending, outbound_type=transfer/scrapped）
  - 点击查看详情 + 审批/驳回按钮
  - 审批通过后自动扣减库存；驳回需填写原因

### 1.6 库存流水页面
- [ ] 实现 `InventoryLogPage`：
  - 流水表格（时间、管材编号、变动类型、关联单据、操作人）
  - 按管材编号/日期范围/变动类型筛选
  - 支持点击管材编号跳转详情

### 1.7 盘点管理页面
- [ ] 实现 `InventoryCheckPage`：
  - 盘点记录列表（盘点单号、日期、状态、盘点人）
  - 操作：新建盘点
- [ ] 创建盘点：选择库位 → 系统自动生成待盘点列表 → 确认创建
- [ ] 执行盘点：逐根核对管材（已找到/缺失）→ 提交结果
- [ ] 查看盘点差异报告（列表对比 + 差异数统计）

### 1.8 库位管理页面
- [ ] 实现 `LocationManagePage`：
  - LocationTree 展示所有库位
  - 新增/编辑/删除库位
  - 点击库位查看该库位内管材列表
  - 支持拖拽或选择管材进行库位调拨

### 1.9 国际化
- [ ] 创建 `src/i18n/resources/zh/inventory.json` 和 `en/inventory.json`

> **依赖**: 管材管理前端模块（引用了管材类型/选择组件）
