# Phase 1 — Backend: Traceability Module (P0 MVP)

> Based on: `docs/requirements.en.md` §3.9; `docs/detailed-design.en.md` §5.3.6 (inventory_logs), §5.3.16 (operation_logs)
> Architecture: backend crate `erp-server` (Rust + Axum + SQLx 0.8 **sqlite** feature)；DB = SQLite3. 唯一对象维度为「商品 (Item) + SKU」；无管材专属字段. 术语见 `specs/UBIQUITOUS_LANGUAGE_LATEST.md`。

## Tasks

### 1.1 What This Module Is

Traceability isn't a standalone module — it's a **cross-cutting concern** that lives in every other module. Two layers:

- **Inventory traceability** (`inventory_logs`): every item's inbound/outbound history (by SKU)
- **Operation audit trail** (`operation_logs`): user action audit logs

### 1.2 Logging Infrastructure

- [ ] Call `OperationLogRepo::create()` in `AuthService` on login/logout
- [ ] Log operation on item create/update/delete in `ItemService` (record before/after field JSON)
- [ ] Log inventory_log + operation_log in `InventoryService` on inbound/outbound/check/transfer
- [ ] Implement `OperationLogger` trait or helper function, unified logging interface
- [ ] Log entry includes: user_id, username, action, target_type, target_id, target_summary, detail (JSON diff), ip_address

### 1.3 Traceability Query Endpoints

- [ ] `GET /api/v1/trace/item/{sku}` — single item full lifecycle trace
  - Returns all inventory_logs for that item (inbound → in-stock → outbound → scrapped etc.)
  - Show linked inbound/outbound order numbers
- [ ] `GET /api/v1/trace/lot/{lot_number}` — trace by lot number
  - Find all items with the same lot number + each item's current status
- [ ] `GET /api/v1/trace/order/{order_type}/{order_id}` — trace by order
  - Query all items linked to a purchase/sales order + their status

### 1.4 Tests

- [ ] Verify operation_log is correctly written on item create/update/delete
- [ ] Verify inventory_log is correctly written on inbound/outbound
- [ ] Verify trace endpoints return complete lifecycle data

> **Deps**: 商品主数据 (item) 模块、库存管理模块、系统管理模块 (users + log tables)
