# Phase 2 — Backend: Sales Management Module (P1)

> Based on: `docs/requirements.en.md` §3.4; `docs/detailed-design.en.md` §4.6, §5.3.11-12, §6.6

## Tasks

### 1.1 DB Migration
- [ ] Create `customers` table migration
- [ ] Create `sales_orders` table migration
- [ ] Create `sales_order_items` table migration

### 1.2 Domain Layer
- [ ] Define `Customer`, `SalesOrder`, `SalesOrderItem` structs
- [ ] Define DTOs: `CreateCustomerDto`, `UpdateCustomerDto`, `CreateSalesOrderDto` (with items array)
- [ ] Define enum: `OrderStatus` (Draft / Pending / Approved / Completed / Cancelled) — shared with purchase module
- [ ] Define filter params: `CustomerFilter`, `SalesOrderFilter`
- [ ] Define ATP query params and result DTO

### 1.3 Repository Layer
- [ ] Implement `CustomerRepo`:
  - `create(dto)` / `update(id, dto)` / `delete(id)` / `find_by_id(id)` / `list(filter)`
- [ ] Implement `SalesOrderRepo`:
  - `create(order + items)` tx
  - `update_status(id, status)` (approval flow)
  - `find_by_id(id)` with items JOIN
  - `list(filter)` with customer name JOIN
  - `update_delivered_quantity(order_id, quantity)` (outbound-linked update)
- [ ] Implement `SalesOrderItemRepo`:
  - `batch_create(order_id, items)` / `find_by_order_id(order_id)`

### 1.4 Service Layer
- [ ] Implement `SalesService`:
  - Customer CRUD
  - `create_sales_order(dto)`: create order + generate order number (format: SO-20260519-XXXX)
  - `approve_sales_order(id)`: approve → status flow draft → pending → approved
  - `reject_sales_order(id, reason)`: reject → status flow back to draft
  - `link_outbound_to_so(outbound_id, so_id)`: link outbound to SO (update delivered_quantity, auto-mark completed when fully delivered)
  - `get_sales_order(id)`: view order detail with linked outbound records
  - `get_atp(pipe_type, grade, od, wt)`: query available-to-promise (in-stock − locked SO quantities)

### 1.5 Handler Layer
- [ ] Customer endpoints:
  - `GET /api/v1/customers` — list customers
  - `POST /api/v1/customers` — create customer
  - `GET /api/v1/customers/{id}` — customer detail
  - `PUT /api/v1/customers/{id}` — update customer
  - `DELETE /api/v1/customers/{id}` — delete customer
- [ ] Sales order endpoints:
  - `GET /api/v1/sales-orders` — list sales orders
  - `POST /api/v1/sales-orders` — create sales order
  - `GET /api/v1/sales-orders/{id}` — order detail (with items + linked outbound records)
  - `PUT /api/v1/sales-orders/{id}` — update sales order
  - `POST /api/v1/sales-orders/{id}/approve` — approve
  - `POST /api/v1/sales-orders/{id}/reject` — reject
  - `GET /api/v1/atp` — ATP query (query params: pipe_type, grade, od, wt)
  - `POST /api/v1/sales-orders/{id}/link-outbound` — link outbound order

### 1.6 Tests
- [ ] Test customer CRUD
- [ ] Test sales order creation + approval flow
- [ ] Test SO-outbound linkage (auto-update delivered_quantity, auto-complete)
- [ ] Test ATP calculation logic (in-stock − locked)

> **Deps**: Pipe management module (references pipe types/specs), inventory management module (outbound linkage + ATP query)
> **Shared**: `OrderStatus` enum shared with purchase module
