# Phase 1 — Backend: Inventory Management Module (P0 MVP)

> Based on: `docs/requirements.en.md` §3.2; `docs/detailed-design.en.md` §4.3, §5.3.3-5a, §5.3.19-20, §6.3

## Tasks

### 1.1 DB Migration
- [ ] Create `locations` table migration (zone/shelf/level hierarchy, full_code unique index)
- [ ] Create `inbound_records` table migration (inbound order header)
- [ ] Create `inbound_items` table migration (inbound line items, one per pipe)
- [ ] Create `outbound_records` table migration (outbound order header)
- [ ] Create `outbound_items` table migration (outbound line items)
- [ ] Create `inventory_logs` table migration (stock change log)
- [ ] Create `inventory_check_records` table migration (count/check records)
- [ ] Create `inventory_check_items` table migration (check line items, pipe-by-pipe)

### 1.2 Domain Layer
- [ ] Define `Location` struct (zone_code/shelf_code/level_code/full_code)
- [ ] Define `InboundRecord` + `InboundItem` structs
- [ ] Define `OutboundRecord` + `OutboundItem` structs
- [ ] Define `InventoryLog` struct
- [ ] Define `InventoryCheckRecord` + `InventoryCheckItem` structs
- [ ] Define DTOs: `CreateInboundDto` (with pipes array; order_id required when inbound_type='purchase'), `CreateOutboundDto` (order_id required when outbound_type='sales'), `ApproveDto` (reason optional), `RejectDto` (reason required), `CreateLocationDto`, `CreateCheckDto`
- [ ] Define filter params: `InboundFilter`, `OutboundFilter`, `InventoryFilter`, `CheckFilter`
- [ ] Define enums: `InboundType` (Purchase/Production/Return), `OutboundType` (Sales/Transfer/Scrapped), `ChangeType` (Inbound/Outbound/Transfer/CheckAdjust), `CheckStatus`, `ApprovalStatus` (AutoApproved/Pending/Approved/Rejected)

### 1.3 Repository Layer
- [ ] Implement `LocationRepo`:
  - `create(dto)` / `update(id, dto)` / `delete(id)` / `find_by_id(id)` / `list(filter)`
  - `find_by_full_code(code)` (uniqueness check)
- [ ] Implement `InboundRepo`:
  - `create(record + items)` (tx: insert header + batch insert items)
  - `find_by_id(id)` (with items JOIN)
  - `list(filter)` (paginated header list)
  - `delete(id)`
- [ ] Implement `OutboundRepo` (mirrors InboundRepo)
- [ ] Implement `InventoryLogRepo`:
  - `create(log)` / `list(filter)` / `find_by_pipe(pipe_type, pipe_id)`
- [ ] Implement `CheckRepo`:
  - `create_check(dto)` / `submit_item(check_id, pipe_id, found)` / `list(filter)` / `get_check_result(id)`

### 1.4 Service Layer
- [ ] Implement `InventoryService`:
  - `create_inbound(dto)`: tx — create inbound order + update pipe status to in_stock + write inventory_log + update location usage.
    **Constraints**: inbound_type='purchase' requires non-empty order_id linked to an approved PO;
    auto-updates PO's received_quantity on completion.
    production/return types start with approval_status=pending, don't touch stock yet.
  - `approve_inbound(id)`: approve non-purchase inbound → approval_status=approved → execute stock update (pipe status, log, location)
  - `reject_inbound(id, reason)`: reject non-purchase inbound → approval_status=rejected
  - `create_outbound(dto)`: tx — create outbound order + verify pipes in stock + update status to outbound + write inventory_log + update location.
    **Constraints**: outbound_type='sales' requires non-empty order_id linked to an approved SO;
    auto-updates SO's delivered_quantity on completion.
    transfer/scrapped types start with approval_status=pending, don't deduct stock yet.
  - `approve_outbound(id)`: approve non-sales outbound → approval_status=approved → execute stock deduction
  - `reject_outbound(id, reason)`: reject non-sales outbound → approval_status=rejected
  - `get_stock_status(pipe_type, pipe_id)`: check a single pipe's status
  - `list_inventory(filter)`: real-time stock query (supports grouping by grade/spec/location/type)
  - `list_inventory_logs(filter)`: stock change history
  - `create_check(dto)`: create check order, auto-fill with all in-stock pipes at that location
  - `submit_check_item(check_id, pipe_id, found)`: submit check results pipe by pipe
  - `get_check_report(check_id)`: generate check variance report
  - `create_location(dto)` / `assign_pipe_to_location(pipe_id, location_id)` / `transfer_location(pipe_id, new_location_id)`
- [ ] Number generation: `generate_inbound_no()` / `generate_outbound_no()` / `generate_check_no()`

### 1.5 Handler Layer
- [ ] Inbound management endpoints:
  - `GET /api/v1/inbound-records` — list inbound records
  - `POST /api/v1/inbound-records` — create inbound (header + items in one shot; order_id required for purchase type)
  - `GET /api/v1/inbound-records/{id}` — inbound detail (with line items)
  - `POST /api/v1/inbound-records/{id}/approve` — approve non-purchase inbound (production/return), requires warehouse/admin role
  - `POST /api/v1/inbound-records/{id}/reject` — reject non-purchase inbound
  - `DELETE /api/v1/inbound-records/{id}` — delete (only auto_approved or rejected status)
- [ ] Outbound management endpoints:
  - `GET /api/v1/outbound-records` — list outbound records
  - `POST /api/v1/outbound-records` — create outbound (order_id required for sales type)
  - `GET /api/v1/outbound-records/{id}` — outbound detail
  - `POST /api/v1/outbound-records/{id}/approve` — approve non-sales outbound (transfer/scrapped), requires warehouse/admin role
  - `POST /api/v1/outbound-records/{id}/reject` — reject non-sales outbound
  - `DELETE /api/v1/outbound-records/{id}` — delete (only auto_approved or rejected status)
- [ ] Stock query endpoints:
  - `GET /api/v1/inventory` — real-time stock (aggregated, supports multiple grouping dimensions)
  - `GET /api/v1/inventory/logs` — stock change history
  - `GET /api/v1/inventory/logs/{pipe_type}/{pipe_id}` — single pipe's full lifecycle
- [ ] Check management endpoints:
  - `POST /api/v1/inventory/checks` — create check order
  - `PUT /api/v1/inventory/checks/{id}/items/{item_id}` — submit check result
  - `GET /api/v1/inventory/checks/{id}` — check detail + variance report
  - `GET /api/v1/inventory/checks` — check record list
- [ ] Location management endpoints:
  - `GET /api/v1/locations` — list locations
  - `POST /api/v1/locations` — create location
  - `PUT /api/v1/locations/{id}` — update location
  - `PUT /api/v1/locations/{id}/assign` — bind pipe to location
  - `PUT /api/v1/locations/{id}/transfer` — transfer location

### 1.6 Tests
- [ ] Test purchase inbound: order_id validation (missing → error, invalid order_id → error)
- [ ] Test purchase inbound: auto-update received_quantity on linked approved PO
- [ ] Test production inbound: approval_status is pending after creation, stock unchanged
- [ ] Test production inbound: stock increases after approval
- [ ] Test production inbound: status becomes rejected on reject
- [ ] Test sales outbound: order_id validation (missing → error, invalid order_id → error)
- [ ] Test sales outbound: auto-update delivered_quantity on linked approved SO
- [ ] Test scrapped outbound: approval_status is pending after creation, stock unchanged
- [ ] Test scrapped outbound: stock deducts after approval
- [ ] Test inbound tx integrity (create inbound → stock increase → log written)
- [ ] Test outbound stock deduction (reject when insufficient stock)
- [ ] Test check variance report generation
- [ ] Test location capacity validation

> **Deps**: Pipe management module (references pipe types/IDs)
