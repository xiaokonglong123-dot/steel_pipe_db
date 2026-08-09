# Phase 2 — Frontend: Sales Management Module (P1)

> Based on: `docs/frontend-design.en.md` §4.1, §6; `docs/detailed-design.en.md` §6.6
> Architecture: Vite + React 19 + Ant Design 5 + TanStack Query 5. 销售订单 (Sales Order) / 销售报价 (Customer Quote) / 发货 (Shipment) / 客户信用 (Customer Credit) UI；订单行项目按「商品 (Item) + SKU」+ 规格 + 数量 + 单价 选择，并显示 ATP 库存可用. 后端 crate `erp-server`，DB = SQLite3. 术语见 `specs/UBIQUITOUS_LANGUAGE_LATEST.md`。

## Tasks

### 1.1 Shared Types & API

- [ ] Define `features/sales/types.ts`: Customer, SalesOrder, OrderItem, CustomerQuote, CustomerCredit, Shipment etc.
- [ ] Define `features/sales/queryKeys.ts` (TanStack Query key factory)
- [ ] Define `features/sales/api/customerApi.ts`: customer CRUD API + credit
- [ ] Define `features/sales/api/salesApi.ts`:
  - Sales order CRUD
  - Approve / reject
  - ATP available-to-promise query
  - Link outbound order
- [ ] Define `features/sales/api/customerQuoteApi.ts`: customer quote CRUD
- [ ] Define `features/sales/api/shipmentApi.ts`: shipment CRUD
- [ ] Implement React Query hooks wrapper

### 1.2 Customer Management Pages

- [ ] Implement `CustomerListPage`: customer table + add/edit/delete
- [ ] Implement `CustomerFormPage`: customer form
- [ ] Implement `CustomerCreditPage`: customer credit management (额度 + 已用 + 可用)

### 1.3 Sales Order Pages

- [ ] Implement `SalesOrderListPage`:
  - Filters: order number, customer, status, date range
  - Order table (order number, customer, date, status, total amount, ATP summary, actions)
  - Actions: view detail, approve, delete
  - +New sales order button
- [ ] Implement `SalesOrderFormPage`:
  - Select customer (CustomerSelect component)
  - Add order items (select item + qty, show ATP available quantity)
  - Auto-calculate total amount
  - Real-time customer credit check (insufficient → warning banner)
- [ ] Implement `SalesOrderDetailPage`:
  - Order basic info
  - Items table + delivered qty tracking
  - Linked outbound record list
  - Linked shipment list
  - Approve / cancel actions

### 1.4 Customer Quote Pages

- [ ] Implement `CustomerQuoteListPage`:
  - Filters: customer, status, date range
  - Quote table (quote number, customer, total amount, status, actions)
- [ ] Implement `CustomerQuoteFormPage`:
  - Link to customer + items
  - Price terms + validity period

### 1.5 Shipment Pages

- [ ] Implement `ShipmentListPage`:
  - Filters: shipment number, SO number, customer, date range, status
  - Shipment table (shipment number, SO number, customer, item count, shipped date, status, actions)
- [ ] Implement `ShipmentFormPage`:
  - Select sales order (auto-fill customer + items + remaining qty)
  - Pick outbound records to include in this shipment
  - Carrier + tracking number + shipping address

### 1.6 Shared Components

- [ ] Implement `OrderStatusTag`: order status badge (shared with purchase module, same definition)
- [ ] Implement `CustomerSelect`: customer selector (search + dropdown)
- [ ] Implement `AtpBadge`: 库存可用徽章 (green sufficient / yellow low / red out-of-stock)
- [ ] Implement `ItemPicker` (shared with purchase module): 商品选择器 (SKU + name search, displays item spec, supports batch select)

### 1.7 i18n

- [ ] Create `src/i18n/resources/zh/sales.json` and `en/sales.json`

> **Deps**: 商品主数据 (item) 前端模块 (ItemPicker 复用)、库存管理前端模块 (ATP 查询 + 在库数量)
> **Shared**: `OrderStatusTag` / `ItemPicker` components shared with purchase module
