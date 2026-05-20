# Phase 2 — 后端：销售管理模块 (P1)

> 基于：`docs/需求文档.md` §3.4；`docs/详细设计文档.md` §4.6, §5.3.11-12, §6.6

## 任务清单

### 1.1 数据库迁移
- [ ] 创建 `customers` 表迁移
- [ ] 创建 `sales_orders` 表迁移
- [ ] 创建 `sales_order_items` 表迁移

### 1.2 领域层
- [ ] 定义 `Customer`、`SalesOrder`、`SalesOrderItem` 结构体
- [ ] 定义 DTO：`CreateCustomerDto`、`UpdateCustomerDto`、`CreateSalesOrderDto`（含 items 数组）
- [ ] 定义枚举：`OrderStatus` (Draft / Pending / Approved / Completed / Cancelled) — 与采购管理模块共享同一定义
- [ ] 定义筛选参数：`CustomerFilter`、`SalesOrderFilter`
- [ ] 定义 ATP 查询参数与结果 DTO

### 1.3 仓库层
- [ ] 实现 `CustomerRepo`：
  - `create(dto)` / `update(id, dto)` / `delete(id)` / `find_by_id(id)` / `list(filter)`
- [ ] 实现 `SalesOrderRepo`：
  - `create(order + items)` 事务
  - `update_status(id, status)`（审核流转）
  - `find_by_id(id)` 含 items JOIN
  - `list(filter)` 含关联客户名称
  - `update_delivered_quantity(order_id, quantity)`（出库联动更新）
- [ ] 实现 `SalesOrderItemRepo`：
  - `batch_create(order_id, items)` / `find_by_order_id(order_id)`

### 1.4 服务层
- [ ] 实现 `SalesService`：
  - 客户 CRUD
  - `create_sales_order(dto)`：创建销售订单 + 生成订单号（格式：SO-20260519-XXXX）
  - `approve_sales_order(id)`：审核 → 状态流转 draft → pending → approved
  - `reject_sales_order(id, reason)`：驳回 → 状态流转 back to draft
  - `link_outbound_to_so(outbound_id, so_id)`：出库关联销售订单（更新 delivered_quantity，全部出库后自动标记 completed）
  - `get_sales_order(id)`：查看订单详情含关联出库记录
  - `get_atp(pipe_type, grade, od, wt)`：查询可售库存量（在库数量 − 已锁定的销售订单数量）

### 1.5 处理器层
- [ ] 客户端点：
  - `GET /api/v1/customers` — 客户列表
  - `POST /api/v1/customers` — 创建客户
  - `GET /api/v1/customers/{id}` — 客户详情
  - `PUT /api/v1/customers/{id}` — 更新客户
  - `DELETE /api/v1/customers/{id}` — 删除客户
- [ ] 销售订单端点：
  - `GET /api/v1/sales-orders` — 销售订单列表
  - `POST /api/v1/sales-orders` — 创建销售订单
  - `GET /api/v1/sales-orders/{id}` — 销售订单详情（含明细 + 关联出库记录）
  - `PUT /api/v1/sales-orders/{id}` — 更新销售订单
  - `POST /api/v1/sales-orders/{id}/approve` — 审核通过
  - `POST /api/v1/sales-orders/{id}/reject` — 驳回
  - `GET /api/v1/atp` — 可售量查询（query params: pipe_type, grade, od, wt）
  - `POST /api/v1/sales-orders/{id}/link-outbound` — 关联出库单

### 1.6 测试
- [ ] 测试客户 CRUD
- [ ] 测试销售订单创建 + 审核流程
- [ ] 测试销售订单与出库联动（delivered_quantity 自动更新，自动完成）
- [ ] 测试 ATP 计算逻辑（在库 − 已锁定）

> **依赖**：管材管理模块（引用 pipe 类型/规格）、库存管理模块（出库关联 + ATP 查询）
> **独立模块**：与采购管理模块共享 `OrderStatus` 枚举定义
