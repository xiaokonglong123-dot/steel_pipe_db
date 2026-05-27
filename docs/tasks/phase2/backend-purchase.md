# Phase 2 — Backend: Purchase Management Module (P1)

> Based on: `docs/requirements.en.md` §3.4; `docs/detailed-design.en.md` §4.5, §5.3.7-10, §6.5

## Tasks

### 1.1 DB Migration
- [ ] Create `suppliers` table migration
- [ ] Create `purchase_orders` table migration
- [ ] Create `purchase_order_items` table migration

### 1.2 Domain Layer
- [ ] Define `Supplier`, `PurchaseOrder`, `PurchaseOrderItem` structs
- [ ] Define DTOs: `CreateSupplierDto`, `UpdateSupplierDto`, `CreatePurchaseOrderDto` (with items array)
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

### 1.4 Service Layer
- [ ] Implement `PurchaseService`:
  - Supplier CRUD
  - `create_purchase_order(dto)`: create order + generate order number (format: PO-20260519-XXXX)
  - `approve_purchase_order(id)`: approve → status flow draft → pending → approved
  - `reject_purchase_order(id, reason)`: reject → status flow back to draft
  - `link_inbound_to_po(inbound_id, po_id)`: link inbound to PO (update received_quantity, auto-mark completed when fully received)
  - `get_purchase_order(id)`: view order detail with linked inbound records

### 1.5 Handler Layer
- [ ] Supplier endpoints:
  - `GET /api/v1/suppliers` — list suppliers
  - `POST /api/v1/suppliers` — create supplier
  - `GET /api/v1/suppliers/{id}` — supplier detail
  - `PUT /api/v1/suppliers/{id}` — update supplier
  - `DELETE /api/v1/suppliers/{id}` — delete supplier
- [ ] Purchase order endpoints:
  - `GET /api/v1/purchase-orders` — list purchase orders
  - `POST /api/v1/purchase-orders` — create purchase order
  - `GET /api/v1/purchase-orders/{id}` — order detail (with items + linked inbound records)
  - `PUT /api/v1/purchase-orders/{id}` — update purchase order
  - `POST /api/v1/purchase-orders/{id}/approve` — approve
  - `POST /api/v1/purchase-orders/{id}/reject` — reject
  - `POST /api/v1/purchase-orders/{id}/link-inbound` — link inbound order

### 1.6 Tests
- [ ] Test supplier CRUD
- [ ] Test purchase order creation + approval flow
- [ ] Test PO-inbound linkage (auto-update received_quantity, auto-complete)

> **Deps**: Pipe management module (references pipe types/specs), inventory management module (inbound linkage)
> **Shared**: `OrderStatus` enum shared with sales module
