# Phase 2 — 前端：销售管理模块 (P1)

> 基于：`docs/前端设计文档.md` §4.1, §6；`docs/详细设计文档.md` §6.6

## 任务清单

### 1.1 共享类型与 API
- [ ] 定义 `features/sales/types.ts`：Customer, SalesOrder, OrderItem 等类型
- [ ] 定义 `features/sales/api/customerApi.ts`：客户 CRUD API
- [ ] 定义 `features/sales/api/salesApi.ts`：
  - 销售订单 CRUD
  - 审核 / 驳回
  - ATP 可售量查询
  - 关联出库单
- [ ] 实现 React Query hooks 封装

### 1.2 客户管理页面
- [ ] 实现 `CustomerListPage`：客户表格 + 新增/编辑/删除
- [ ] 实现 `CustomerFormPage`：客户表单

### 1.3 销售订单页面
- [ ] 实现 `SalesOrderListPage`：
  - 筛选：订单号、客户、状态、日期范围
  - 订单表格（订单号、客户、日期、状态、总金额、ATP 摘要、操作）
  - 操作：查看详情、审核、删除
  - +新增销售订单按钮
- [ ] 实现 `SalesOrderFormPage`：
  - 选择客户（CustomerSelect 组件）
  - 添加订单明细（选择管材规格 + 数量，显示 ATP 可售量）
  - 自动计算总金额
- [ ] 实现 `SalesOrderDetailPage`：
  - 订单基本信息
  - 明细表格 + 已出库数量追踪
  - 关联出库记录列表
  - 审核/取消操作

### 1.4 共享组件
- [ ] 实现 `OrderStatusTag`：订单状态标签（与采购模块共享同一定义，可共用）
- [ ] 实现 `CustomerSelect`：客户选择器（搜索+下拉）
- [ ] 实现 `AtpBadge`：库存可售量显示徽标（绿色充足 / 黄色紧张 / 红色缺货）

### 1.5 国际化
- [ ] 创建 `src/i18n/resources/zh/sales.json` 和 `en/sales.json`

> **依赖**：管材管理前端模块（管材规格选择）、库存管理前端模块（ATP 可售量查询）
> **独立模块**：与采购管理模块共享 `OrderStatusTag` 组件
