# Phase 2 — Frontend: Purchase Management Module (P1)

> Based on: `docs/frontend-design.en.md` §4.1, §6; `docs/detailed-design.en.md` §6.5

## Tasks

### 1.1 Shared Types & API
- [ ] Define `features/purchases/types.ts`: Supplier, PurchaseOrder, OrderItem etc.
- [ ] Define `features/purchases/api/supplierApi.ts`: supplier CRUD API
- [ ] Define `features/purchases/api/purchaseApi.ts`:
  - Purchase order CRUD
  - Approve / reject
  - Link inbound order
- [ ] Implement React Query hooks wrapper

### 1.2 Supplier Management Pages
- [ ] Implement `SupplierListPage`: supplier table + add/edit/delete
- [ ] Implement `SupplierFormPage`: supplier form (name, contact, phone, email, address, certs)

### 1.3 Purchase Order Pages
- [ ] Implement `PurchaseOrderListPage`:
  - Filters: order number, supplier, status, date range
  - Order table (order number, supplier, date, status, total amount, actions)
  - Actions: view detail, approve, delete
  - +New purchase order button
- [ ] Implement `PurchaseOrderFormPage`:
  - Select supplier (SupplierSelect component)
  - Add order items (select pipe spec: grade + OD + WT + qty + unit price)
  - Auto-calculate total amount
  - Notes field
- [ ] Implement `PurchaseOrderDetailPage`:
  - Order basic info display
  - Items table (grade, spec, qty, received qty, unit price, subtotal)
  - Linked inbound record list (click to navigate to inbound detail)
  - Approve / cancel action buttons

### 1.4 Shared Components
- [ ] Implement `OrderStatusTag`: order status badge (draft 🟡 / pending 🔵 / approved 🟢 / completed ⚪ / cancelled 🔴)
- [ ] Implement `SupplierSelect`: supplier selector (search + dropdown)

### 1.5 i18n
- [ ] Create `src/i18n/resources/zh/purchase.json` and `en/purchase.json`

> **Deps**: Pipe management frontend module (pipe spec selection)
> **Shared**: `OrderStatusTag` component shared with sales module
