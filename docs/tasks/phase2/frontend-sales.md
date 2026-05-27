# Phase 2 — Frontend: Sales Management Module (P1)

> Based on: `docs/frontend-design.en.md` §4.1, §6; `docs/detailed-design.en.md` §6.6

## Tasks

### 1.1 Shared Types & API
- [ ] Define `features/sales/types.ts`: Customer, SalesOrder, OrderItem etc.
- [ ] Define `features/sales/api/customerApi.ts`: customer CRUD API
- [ ] Define `features/sales/api/salesApi.ts`:
  - Sales order CRUD
  - Approve / reject
  - ATP available-to-promise query
  - Link outbound order
- [ ] Implement React Query hooks wrapper

### 1.2 Customer Management Pages
- [ ] Implement `CustomerListPage`: customer table + add/edit/delete
- [ ] Implement `CustomerFormPage`: customer form

### 1.3 Sales Order Pages
- [ ] Implement `SalesOrderListPage`:
  - Filters: order number, customer, status, date range
  - Order table (order number, customer, date, status, total amount, ATP summary, actions)
  - Actions: view detail, approve, delete
  - +New sales order button
- [ ] Implement `SalesOrderFormPage`:
  - Select customer (CustomerSelect component)
  - Add order items (select pipe spec + qty, show ATP available quantity)
  - Auto-calculate total amount
- [ ] Implement `SalesOrderDetailPage`:
  - Order basic info
  - Items table + delivered qty tracking
  - Linked outbound record list
  - Approve / cancel actions

### 1.4 Shared Components
- [ ] Implement `OrderStatusTag`: order status badge (shared with purchase module, same definition)
- [ ] Implement `CustomerSelect`: customer selector (search + dropdown)
- [ ] Implement `AtpBadge`: stock availability badge (green sufficient / yellow low / red out-of-stock)

### 1.5 i18n
- [ ] Create `src/i18n/resources/zh/sales.json` and `en/sales.json`

> **Deps**: Pipe management frontend module (pipe spec selection), inventory management frontend module (ATP query)
> **Shared**: `OrderStatusTag` component shared with purchase module
