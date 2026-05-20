# Phase 2 — 前端：采购管理模块 (P1)

> 基于：`docs/前端设计文档.md` §4.1, §6；`docs/详细设计文档.md` §6.5

## 任务清单

### 1.1 共享类型与 API
- [ ] 定义 `features/purchases/types.ts`：Supplier, PurchaseOrder, OrderItem 等类型
- [ ] 定义 `features/purchases/api/supplierApi.ts`：供应商 CRUD API
- [ ] 定义 `features/purchases/api/purchaseApi.ts`：
  - 采购订单 CRUD
  - 审核 / 驳回
  - 关联入库单
- [ ] 实现 React Query hooks 封装

### 1.2 供应商管理页面
- [ ] 实现 `SupplierListPage`：供应商表格 + 新增/编辑/删除
- [ ] 实现 `SupplierFormPage`：供应商表单（名称、联系人、电话、邮箱、地址、资质证书）

### 1.3 采购订单页面
- [ ] 实现 `PurchaseOrderListPage`：
  - 筛选：订单号、供应商、状态、日期范围
  - 订单表格（订单号、供应商、日期、状态、总金额、操作）
  - 操作：查看详情、审核、删除
  - +新增采购订单按钮
- [ ] 实现 `PurchaseOrderFormPage`：
  - 选择供应商（SupplierSelect 组件）
  - 添加订单明细（选择管材规格：钢级 + 外径 + 壁厚 + 数量 + 单价）
  - 自动计算总金额
  - 备注
- [ ] 实现 `PurchaseOrderDetailPage`：
  - 订单基本信息展示
  - 明细表格（钢级、规格、数量、已入库数量、单价、小计）
  - 关联入库记录列表（点击跳转入库详情）
  - 审核/取消操作按钮

### 1.4 共享组件
- [ ] 实现 `OrderStatusTag`：订单状态标签（draft🟡 / pending🔵 / approved🟢 / completed⚪ / cancelled🔴）
- [ ] 实现 `SupplierSelect`：供应商选择器（搜索+下拉）

### 1.5 国际化
- [ ] 创建 `src/i18n/resources/zh/purchase.json` 和 `en/purchase.json`

> **依赖**：管材管理前端模块（管材规格选择）
> **独立模块**：与销售管理模块共享 `OrderStatusTag` 组件
