# Phase 2 — Backend: Sales Management Module (P1)

> Based on: `docs/requirements.en.md` §3.4; `docs/detailed-design.en.md` §4.6, §5.3.11-12, §6.6
> Architecture: backend crate `erp-server` (Rust + Axum + SQLx 0.8 **sqlite** feature)；DB = SQLite3. 销售订单 (Sales Order) / 销售报价 (Customer Quote) / 发货 (Shipment) / 客户信用 (Customer Credit) 全部按通用 ERP 语义；订单行项目以「商品 (Item) + SKU」为对象维度，无管材专属字段. ATP 库存可用 = 在库 − 库存预留. 错误码 140xx (订单) / 170xx (客户). 术语见 `specs/UBIQUITOUS_LANGUAGE_LATEST.md`。

## Tasks

### 1.1 DB Migration

- [ ] Create `customers` table migration
- [ ] Create `sales_orders` table migration
- [ ] Create `sales_order_items` table migration (links to item via sku)
- [ ] Create `customer_quotes` table migration (销售报价)
- [ ] Create `customer_credits` table migration (客户信用额度)
- [ ] Create `shipments` table migration (发货)

### 1.2 Domain Layer

- [ ] Define `Customer`, `SalesOrder`, `SalesOrderItem`, `CustomerQuote`, `CustomerCredit`, `Shipment` structs
- [ ] Define DTOs: `CreateCustomerDto`, `UpdateCustomerDto`, `CreateSalesOrderDto` (with items array, items reference items.sku)
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
- [ ] Implement `CustomerQuoteRepo`:
  - `create(dto)` / `update(id, dto)` / `find_by_id(id)` / `list(filter)` (by customer / by RFQ)
- [ ] Implement `CustomerCreditRepo`:
  - `find_by_customer(customer_id)` / `update_credit(customer_id, dto)` / `get_available_credit(customer_id)` (额度 - 已用)
- [ ] Implement `ShipmentRepo`:
  - `create(dto)` / `find_by_id(id)` / `list_by_sales_order(so_id)`

### 1.4 Service Layer

- [ ] Implement `SalesService`:
  - Customer CRUD
  - `create_sales_order(dto)`: create order + generate order number (format: SO-20260519-XXXX)
  - `approve_sales_order(id)`: approve → status flow draft → pending → approved
  - `reject_sales_order(id, reason)`: reject → status flow back to draft
  - `link_outbound_to_so(outbound_id, so_id)`: link outbound to SO (update delivered_quantity, auto-mark completed when fully delivered)
  - `get_sales_order(id)`: view order detail with linked outbound records
  - `get_atp(sku, qty)`: query available-to-promise (在库 − 库存预留)
  - `create_customer_quote(dto)`: create customer-facing sales quote
  - `check_customer_credit(customer_id, order_amount)`: validate customer credit before order approval
  - `create_shipment(dto)`: create shipment (links to sales order + outbound records)
  - `get_shipment(id)` / `list_shipments(filter)`: query shipments

### 1.5 Handler Layer

- [ ] Customer endpoints:
  - `GET /api/v1/customers` — list customers
  - `POST /api/v1/customers` — create customer
  - `GET /api/v1/customers/{id}` — customer detail
  - `PUT /api/v1/customers/{id}` — update customer
  - `DELETE /api/v1/customers/{id}` — delete customer
  - `GET /api/v1/customers/{id}/credit` — customer credit status
  - `PUT /api/v1/customers/{id}/credit` — update customer credit
- [ ] Sales order endpoints:
  - `GET /api/v1/sales-orders` — list sales orders
  - `POST /api/v1/sales-orders` — create sales order
  - `GET /api/v1/sales-orders/{id}` — order detail (with items + linked outbound records)
  - `PUT /api/v1/sales-orders/{id}` — update sales order
  - `POST /api/v1/sales-orders/{id}/approve` — approve
  - `POST /api/v1/sales-orders/{id}/reject` — reject
  - `GET /api/v1/atp` — ATP query (query params: sku, qty)
  - `POST /api/v1/sales-orders/{id}/link-outbound` — link outbound order
- [ ] Customer quote endpoints:
  - `GET /api/v1/customer-quotes` — list customer quotes
  - `POST /api/v1/customer-quotes` — create customer quote
  - `GET /api/v1/customer-quotes/{id}` — quote detail
- [ ] Shipment endpoints:
  - `GET /api/v1/shipments` — list shipments
  - `POST /api/v1/shipments` — create shipment
  - `GET /api/v1/shipments/{id}` — shipment detail

### 1.6 Tests

- [ ] Test customer CRUD
- [ ] Test sales order creation + approval flow
- [ ] Test SO-outbound linkage (auto-update delivered_quantity, auto-complete)
- [ ] Test ATP calculation logic (在库 − 库存预留)
- [ ] Test customer credit check (insufficient credit → approval blocked)
- [ ] Test shipment creation + SO linkage

> **Deps**: 商品主数据 (item) 模块 (订单行项目通过 SKU 引用)、库存管理模块 (outbound 联动 + ATP 查询 + 库存预留)
> **Shared**: `OrderStatus` enum shared with purchase module
