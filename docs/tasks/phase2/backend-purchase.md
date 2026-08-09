# Phase 2 — Backend: Purchase Management Module (P1)

> Based on: `docs/requirements.en.md` §3.4; `docs/detailed-design.en.md` §4.5, §5.3.7-10, §6.5
> Architecture: backend crate `erp-server` (Rust + Axum + SQLx 0.8 **sqlite** feature)；DB = SQLite3. 采购订单 (Purchase Order) / 采购报价 (Supplier Quote) / 采购收货 (Receipt) / 供应商评分 (Scorecard) 全部按通用 ERP 语义；订单行项目以「商品 (Item) + SKU」为对象维度，无管材专属字段. 错误码 140xx (订单) / 160xx (供应商). 术语见 `specs/UBIQUITOUS_LANGUAGE_LATEST.md`。

## Tasks

### 1.1 DB Migration

- [ ] Create `suppliers` table migration
- [ ] Create `purchase_orders` table migration
- [ ] Create `purchase_order_items` table migration (links to item via sku)
- [ ] Create `supplier_quotes` table migration (采购报价)
- [ ] Create `supplier_scorecards` table migration (供应商评分)

### 1.2 Domain Layer

- [ ] Define `Supplier`, `PurchaseOrder`, `PurchaseOrderItem`, `SupplierQuote`, `SupplierScorecard` structs
- [ ] Define DTOs: `CreateSupplierDto`, `UpdateSupplierDto`, `CreatePurchaseOrderDto` (with items array, items reference items.sku)
- [ ] Define enum: `OrderStatus` (Draft / Pending / Approved / Completed / Cancelled)
- [ ] Define filter params: `SupplierFilter`, `PurchaseOrderFilter`

### 1.3 Repository Layer

- [ ] Implement `SupplierRepo`:
  - `create(dto)` / `update(id, dto)` / `delete(id)` / `find_by_id(id)` / `list(filter)`
- [ ] Implement `PurchaseOrderRepo`:
  - `create(order + items)` tx
  - `update_status(id, status)` (approval flow)
  - `find_by_id(id)` with items JOIN
  - `list(filter)` with supplier name JOIN
  - `update_received_quantity(order_id, quantity)` (inbound-linked update)
- [ ] Implement `PurchaseOrderItemRepo`:
  - `batch_create(order_id, items)` / `find_by_order_id(order_id)`
- [ ] Implement `SupplierQuoteRepo`:
  - `create(dto)` / `update(id, dto)` / `find_by_id(id)` / `list(filter)` (by supplier / by RFQ)
- [ ] Implement `ScorecardRepo`:
  - `create(dto)` / `list_by_supplier(supplier_id, filter)` / `get_aggregated(supplier_id)` (avg score + on-time rate)

### 1.4 Service Layer

- [ ] Implement `PurchaseService`:
  - Supplier CRUD
  - `create_purchase_order(dto)`: create order + generate order number (format: PO-20260519-XXXX)
  - `approve_purchase_order(id)`: approve → status flow draft → pending → approved
  - `reject_purchase_order(id, reason)`: reject → status flow back to draft
  - `link_inbound_to_po(inbound_id, po_id)`: link inbound to PO (update received_quantity, auto-mark completed when fully received)
  - `get_purchase_order(id)`: view order detail with linked inbound records
  - `create_supplier_quote(dto)`: create supplier quote response
  - `record_supplier_scorecard(dto)`: record supplier delivery evaluation
  - `get_supplier_scorecard(supplier_id)`: aggregated supplier scorecard (delivery on-time rate + quality score)

### 1.5 Handler Layer

- [ ] Supplier endpoints:
  - `GET /api/v1/suppliers` — list suppliers
  - `POST /api/v1/suppliers` — create supplier
  - `GET /api/v1/suppliers/{id}` — supplier detail
  - `PUT /api/v1/suppliers/{id}` — update supplier
  - `DELETE /api/v1/suppliers/{id}` — delete supplier
  - `GET /api/v1/suppliers/{id}/scorecard` — supplier aggregated scorecard
- [ ] Purchase order endpoints:
  - `GET /api/v1/purchase-orders` — list purchase orders
  - `POST /api/v1/purchase-orders` — create purchase order
  - `GET /api/v1/purchase-orders/{id}` — order detail (with items + linked inbound records)
  - `PUT /api/v1/purchase-orders/{id}` — update purchase order
  - `POST /api/v1/purchase-orders/{id}/approve` — approve
  - `POST /api/v1/purchase-orders/{id}/reject` — reject
  - `POST /api/v1/purchase-orders/{id}/link-inbound` — link inbound order
- [ ] Supplier quote endpoints:
  - `GET /api/v1/supplier-quotes` — list supplier quotes
  - `POST /api/v1/supplier-quotes` — create supplier quote
  - `GET /api/v1/supplier-quotes/{id}` — quote detail

### 1.6 Tests

- [ ] Test supplier CRUD
- [ ] Test purchase order creation + approval flow
- [ ] Test PO-inbound linkage (auto-update received_quantity, auto-complete)
- [ ] Test supplier scorecard aggregation (avg score + on-time rate)

> **Deps**: 商品主数据 (item) 模块 (订单行项目通过 SKU 引用)、库存管理模块 (inbound 联动)
> **Shared**: `OrderStatus` enum shared with sales module
