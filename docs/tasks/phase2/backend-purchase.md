# Phase 2 — 后端：采购管理模块 (P1)

> 基于：`docs/需求文档.md` §3.4；`docs/详细设计文档.md` §4.5, §5.3.7-10, §6.5

## 任务清单

### 1.1 数据库迁移
- [ ] 创建 `suppliers` 表迁移
- [ ] 创建 `purchase_orders` 表迁移
- [ ] 创建 `purchase_order_items` 表迁移

### 1.2 领域层
- [ ] 定义 `Supplier`、`PurchaseOrder`、`PurchaseOrderItem` 结构体
- [ ] 定义 DTO：`CreateSupplierDto`、`UpdateSupplierDto`、`CreatePurchaseOrderDto`（含 items 数组）
- [ ] 定义枚举：`OrderStatus` (Draft / Pending / Approved / Completed / Cancelled)
- [ ] 定义筛选参数：`SupplierFilter`、`PurchaseOrderFilter`

### 1.3 仓库层
- [ ] 实现 `SupplierRepo`：
  - `create(dto)` / `update(id, dto)` / `delete(id)` / `find_by_id(id)` / `list(filter)`
- [ ] 实现 `PurchaseOrderRepo`：
  - `create(order + items)` 事务
  - `update_status(id, status)`（审核流转）
  - `find_by_id(id)` 含 items JOIN
  - `list(filter)` 含关联供应商名称
  - `update_received_quantity(order_id, quantity)`（入库联动更新）
- [ ] 实现 `PurchaseOrderItemRepo`：
  - `batch_create(order_id, items)` / `find_by_order_id(order_id)`

### 1.4 服务层
- [ ] 实现 `PurchaseService`：
  - 供应商 CRUD
  - `create_purchase_order(dto)`：创建订单 + 生成订单号（格式：PO-20260519-XXXX）
  - `approve_purchase_order(id)`：审核 → 状态流转 draft → pending → approved
  - `reject_purchase_order(id, reason)`：驳回 → 状态流转 back to draft
  - `link_inbound_to_po(inbound_id, po_id)`：入库关联采购订单（更新 received_quantity，全部入库后自动标记 completed）
  - `get_purchase_order(id)`：查看订单详情含关联入库记录

### 1.5 处理器层
- [ ] 供应商端点：
  - `GET /api/v1/suppliers` — 供应商列表
  - `POST /api/v1/suppliers` — 创建供应商
  - `GET /api/v1/suppliers/{id}` — 供应商详情
  - `PUT /api/v1/suppliers/{id}` — 更新供应商
  - `DELETE /api/v1/suppliers/{id}` — 删除供应商
- [ ] 采购订单端点：
  - `GET /api/v1/purchase-orders` — 采购订单列表
  - `POST /api/v1/purchase-orders` — 创建采购订单
  - `GET /api/v1/purchase-orders/{id}` — 采购订单详情（含明细 + 关联入库记录）
  - `PUT /api/v1/purchase-orders/{id}` — 更新采购订单
  - `POST /api/v1/purchase-orders/{id}/approve` — 审核通过
  - `POST /api/v1/purchase-orders/{id}/reject` — 驳回
  - `POST /api/v1/purchase-orders/{id}/link-inbound` — 关联入库单

### 1.6 测试
- [ ] 测试供应商 CRUD
- [ ] 测试采购订单创建 + 审核流程
- [ ] 测试采购订单与入库联动（received_quantity 自动更新，自动完成）

> **依赖**：管材管理模块（引用 pipe 类型/规格）、库存管理模块（入库关联）
> **独立模块**：与销售管理模块共享 `OrderStatus` 枚举定义
