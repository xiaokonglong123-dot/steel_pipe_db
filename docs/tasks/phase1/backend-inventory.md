# Phase 1 — 后端：库存管理模块 (P0 MVP)

> 基于：`docs/需求文档.md` §3.2；`docs/详细设计文档.md` §4.3, §5.3.3-5a, §5.3.19-20, §6.3

## 任务清单

### 1.1 数据库迁移
- [ ] 创建 `locations` 表迁移（库区/货架/层位分级，full_code 唯一索引）
- [ ] 创建 `inbound_records` 表迁移（入库单表头）
- [ ] 创建 `inbound_items` 表迁移（入库明细，每根管材一条记录）
- [ ] 创建 `outbound_records` 表迁移（出库单表头）
- [ ] 创建 `outbound_items` 表迁移（出库明细）
- [ ] 创建 `inventory_logs` 表迁移（库存变动日志）
- [ ] 创建 `inventory_check_records` 表迁移（盘点记录）
- [ ] 创建 `inventory_check_items` 表迁移（盘点明细，逐根核对）

### 1.2 领域层 (Domain)
- [ ] 定义 `Location` 结构体（含 zone_code/shelf_code/level_code/full_code）
- [ ] 定义 `InboundRecord` + `InboundItem` 结构体
- [ ] 定义 `OutboundRecord` + `OutboundItem` 结构体
- [ ] 定义 `InventoryLog` 结构体
- [ ] 定义 `InventoryCheckRecord` + `InventoryCheckItem` 结构体
- [ ] 定义 DTO：`CreateInboundDto`（含 pipes 数组；inbound_type='purchase' 时 order_id 必填）、`CreateOutboundDto`（outbound_type='sales' 时 order_id 必填）、`ApproveDto`（reason 可选）、`RejectDto`（reason 必填）、`CreateLocationDto`、`CreateCheckDto`
- [ ] 定义筛选参数：`InboundFilter`、`OutboundFilter`、`InventoryFilter`、`CheckFilter`
- [ ] 定义枚举：`InboundType` (Purchase/Production/Return), `OutboundType` (Sales/Transfer/Scrapped), `ChangeType` (Inbound/Outbound/Transfer/CheckAdjust), `CheckStatus`, `ApprovalStatus` (AutoApproved/Pending/Approved/Rejected)

### 1.3 仓库层 (Repository)
- [ ] 实现 `LocationRepo`：
  - `create(dto)` / `update(id, dto)` / `delete(id)` / `find_by_id(id)` / `list(filter)`
  - `find_by_full_code(code)`（唯一性校验）
- [ ] 实现 `InboundRepo`：
  - `create(record + items)`（事务：插入表头 + 批量插入明细）
  - `find_by_id(id)`（含 items JOIN）
  - `list(filter)`（分页查询表头列表）
  - `delete(id)`
- [ ] 实现 `OutboundRepo`（对标 InboundRepo）
- [ ] 实现 `InventoryLogRepo`：
  - `create(log)` / `list(filter)` / `find_by_pipe(pipe_type, pipe_id)`
- [ ] 实现 `CheckRepo`：
  - `create_check(dto)` / `submit_item(check_id, pipe_id, found)` / `list(filter)` / `get_check_result(id)`

### 1.4 服务层 (Service)
- [ ] 实现 `InventoryService`：
  - `create_inbound(dto)`：事务——创建入库单 + 更新管材状态为 in_stock + 写入 inventory_log + 更新库位使用量。
    **约束**：inbound_type='purchase' 时校验 order_id 非空且对应采购订单状态为 approved；
    入库后自动更新采购订单的 received_quantity。
    production/return 类型创建后 approval_status=pending，暂不更新库存。
  - `approve_inbound(id)`：审批非采购入库单 → approval_status=approved → 执行库存更新（管材状态、日志、库位）
  - `reject_inbound(id, reason)`：驳回非采购入库单 → approval_status=rejected
  - `create_outbound(dto)`：事务——创建出库单 + 校验管材在库 + 更新状态为 outbound + 写入 inventory_log + 更新库位。
    **约束**：outbound_type='sales' 时校验 order_id 非空且对应销售订单状态为 approved；
    出库后自动更新销售订单的 delivered_quantity。
    transfer/scrapped 类型创建后 approval_status=pending，暂不扣减库存。
  - `approve_outbound(id)`：审批非销售出库单 → approval_status=approved → 执行库存扣减
  - `reject_outbound(id, reason)`：驳回非销售出库单 → approval_status=rejected
  - `get_stock_status(pipe_type, pipe_id)`：查看单根管材状态
  - `list_inventory(filter)`：实时库存查询（支持按钢级/规格/库位/类型分组聚合）
  - `list_inventory_logs(filter)`：库存变动流水
  - `create_check(dto)`：创建盘点单，自动填充该库位所有在库管材
  - `submit_check_item(check_id, pipe_id, found)`：逐根提交盘点结果
  - `get_check_report(check_id)`：生成盘点差异报告
  - `create_location(dto)` / `assign_pipe_to_location(pipe_id, location_id)` / `transfer_location(pipe_id, new_location_id)`
- [ ] 编号生成：`generate_inbound_no()` / `generate_outbound_no()` / `generate_check_no()`

### 1.5 处理器层 (Handler)
- [ ] 入库管理端点：
  - `GET /api/v1/inbound-records` — 入库记录列表
  - `POST /api/v1/inbound-records` — 创建入库（表头 + 明细一次性提交；purchase 类型 order_id 必填）
  - `GET /api/v1/inbound-records/{id}` — 入库详情（含明细）
  - `POST /api/v1/inbound-records/{id}/approve` — 审批非采购入库单（production/return），需 warehouse/admin 角色
  - `POST /api/v1/inbound-records/{id}/reject` — 驳回非采购入库单
  - `DELETE /api/v1/inbound-records/{id}` — 删除（仅可删除 auto_approved 或 rejected 状态的记录）
- [ ] 出库管理端点：
  - `GET /api/v1/outbound-records` — 出库记录列表
  - `POST /api/v1/outbound-records` — 创建出库（sales 类型 order_id 必填）
  - `GET /api/v1/outbound-records/{id}` — 出库详情
  - `POST /api/v1/outbound-records/{id}/approve` — 审批非销售出库单（transfer/scrapped），需 warehouse/admin 角色
  - `POST /api/v1/outbound-records/{id}/reject` — 驳回非销售出库单
  - `DELETE /api/v1/outbound-records/{id}` — 删除（仅可删除 auto_approved 或 rejected 状态的记录）
- [ ] 库存查询端点：
  - `GET /api/v1/inventory` — 实时库存（聚合查询，支持多种分组维度）
  - `GET /api/v1/inventory/logs` — 库存变动流水
  - `GET /api/v1/inventory/logs/{pipe_type}/{pipe_id}` — 单根管材的完整生命周期
- [ ] 盘点管理端点：
  - `POST /api/v1/inventory/checks` — 创建盘点单
  - `PUT /api/v1/inventory/checks/{id}/items/{item_id}` — 提交盘点结果
  - `GET /api/v1/inventory/checks/{id}` — 盘点详情 + 差异报告
  - `GET /api/v1/inventory/checks` — 盘点记录列表
- [ ] 库位管理端点：
  - `GET /api/v1/locations` — 库位列表
  - `POST /api/v1/locations` — 创建库位
  - `PUT /api/v1/locations/{id}` — 更新库位
  - `PUT /api/v1/locations/{id}/assign` — 管材绑定库位
  - `PUT /api/v1/locations/{id}/transfer` — 库位调拨

### 1.6 测试
- [ ] 测试采购入库：order_id 校验（缺省时报错，无效 order_id 时报错）
- [ ] 测试采购入库：关联已审核采购订单后自动更新 received_quantity
- [ ] 测试生产入库：创建后 approval_status 为 pending，库存不变化
- [ ] 测试生产入库：审批通过后库存增加
- [ ] 测试生产入库：驳回后审批状态为 rejected
- [ ] 测试销售出库：order_id 校验（缺省时报错，无效 order_id 时报错）
- [ ] 测试销售出库：关联已审核销售订单后自动更新 delivered_quantity
- [ ] 测试报废出库：创建后 approval_status 为 pending，库存不变化
- [ ] 测试报废出库：审批通过后库存扣减
- [ ] 测试入库事务完整性（创建入库→库存增加→日志写入）
- [ ] 测试出库库存扣减（库存不足时拒绝）
- [ ] 测试盘点差异报告生成逻辑
- [ ] 测试库位容量校验

> **依赖**: 管材管理模块（引用 pipe 类型/ID）
