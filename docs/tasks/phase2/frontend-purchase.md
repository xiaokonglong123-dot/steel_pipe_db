# Phase 2 — Frontend: Purchase Management Module (P1)

> Based on: `docs/frontend-design.en.md` §4.1, §6; `docs/detailed-design.en.md` §6.5
> Architecture: Vite + React 19 + Ant Design 5 + TanStack Query 5. 采购订单 (Purchase Order) / 采购报价 (Supplier Quote) / 供应商评分 (Scorecard) UI；订单行项目按「商品 (Item) + SKU」+ 规格 + 数量 + 单价 选择；无管材专属字段. 后端 crate `erp-server`，DB = SQLite3. 术语见 `specs/UBIQUITOUS_LANGUAGE_LATEST.md`。

## Tasks

### 1.1 Shared Types & API

- [ ] Define `features/purchases/types.ts`: Supplier, PurchaseOrder, OrderItem, SupplierQuote, SupplierScorecard etc.
- [ ] Define `features/purchases/queryKeys.ts` (TanStack Query key factory)
- [ ] Define `features/purchases/api/supplierApi.ts`: supplier CRUD API + scorecard
- [ ] Define `features/purchases/api/purchaseApi.ts`:
  - Purchase order CRUD
  - Approve / reject
  - Link inbound order
- [ ] Define `features/purchases/api/supplierQuoteApi.ts`: supplier quote CRUD
- [ ] Implement React Query hooks wrapper

### 1.2 Supplier Management Pages

- [ ] Implement `SupplierListPage`: supplier table + add/edit/delete
- [ ] Implement `SupplierFormPage`: supplier form (name, contact, phone, email, address, certs)
- [ ] Implement `SupplierScorecardPage`: supplier delivery on-time rate + quality score aggregated view

### 1.3 Purchase Order Pages

- [ ] Implement `PurchaseOrderListPage`:
  - Filters: order number, supplier, status, date range
  - Order table (order number, supplier, date, status, total amount, actions)
  - Actions: view detail, approve, delete
  - +New purchase order button
- [ ] Implement `PurchaseOrderFormPage`:
  - Select supplier (SupplierSelect component)
  - Add order items (select item: SKU + name + spec + qty + unit price)
  - Auto-calculate total amount
  - Notes field
- [ ] Implement `PurchaseOrderDetailPage`:
  - Order basic info display
  - Items table (SKU, name, spec, qty, received qty, unit price, subtotal)
  - Linked inbound record list (click to navigate to inbound detail)
  - Approve / cancel action buttons

### 1.4 Supplier Quote Pages

- [ ] Implement `SupplierQuoteListPage`:
  - Filters: supplier, status, date range
  - Quote table (quote number, supplier, related RFQ, total amount, status, actions)
- [ ] Implement `SupplierQuoteFormPage`:
  - Link to supplier + RFQ + items
  - Price terms (unit price, tax rate, total amount)
  - Validity period

### 1.5 Shared Components

- [ ] Implement `OrderStatusTag`: order status badge (draft 🟡 / pending 🔵 / approved 🟢 / completed ⚪ / cancelled 🔴)
- [ ] Implement `SupplierSelect`: supplier selector (search + dropdown)
- [ ] Implement `ItemPicker`: 商品选择器 (SKU + name search, displays item spec, supports batch select)

### 1.6 i18n

- [ ] Create `src/i18n/resources/zh/purchase.json` and `en/purchase.json`

> **Deps**: 商品主数据 (item) 前端模块 (ItemPicker 复用)
> **Shared**: `OrderStatusTag` / `ItemPicker` components shared with sales module
