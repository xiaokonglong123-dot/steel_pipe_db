# Phase 1 — Backend: Traceability Module (P0 MVP)

> Based on: `docs/requirements.en.md` §3.9; `docs/detailed-design.en.md` §5.3.6 (inventory_logs), §5.3.16 (operation_logs)

## Tasks

### 1.1 What This Module Is
Traceability isn't a standalone module — it's a **cross-cutting concern** that lives in every other module. Two layers:
- **Inventory traceability** (`inventory_logs`): every pipe's inbound/outbound history
- **Operation audit trail** (`operation_logs`): user action audit logs

### 1.2 Logging Infrastructure
- [ ] Call `OperationLogRepo::create()` in `AuthService` on login/logout
- [ ] Log operation on pipe create/update/delete in `PipeService` (record before/after field JSON)
- [ ] Log inventory_log + operation_log in `InventoryService` on inbound/outbound/check/transfer
- [ ] Implement `OperationLogger` trait or helper function, unified logging interface
- [ ] Log entry includes: user_id, username, action, target_type, target_id, target_summary, detail (JSON diff), ip_address

### 1.3 Traceability Query Endpoints
- [ ] `GET /api/v1/trace/pipe/{pipe_type}/{pipe_id}` — single pipe full lifecycle trace
  - Returns all inventory_logs for that pipe (inbound → in-stock → outbound → scrapped etc.)
  - Show linked inbound/outbound order numbers
- [ ] `GET /api/v1/trace/heat-number/{heat_number}` — trace by heat number
  - Find all pipes with the same heat number + each pipe's current status
- [ ] `GET /api/v1/trace/order/{order_type}/{order_id}` — trace by order
  - Query all pipes linked to a purchase/sales order + their status

### 1.4 Tests
- [ ] Verify operation_log is correctly written on pipe create/update/delete
- [ ] Verify inventory_log is correctly written on inbound/outbound
- [ ] Verify trace endpoints return complete lifecycle data

> **Deps**: Pipe management module, inventory management module, system management module (users + log tables)
