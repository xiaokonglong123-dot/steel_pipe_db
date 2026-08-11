# Phase 1 — Frontend: Inventory Management Module (P0 MVP)

> Based on: `docs/frontend-design.en.md` §4.1, §4.2, §6, §8

## Tasks

### 1.1 Shared Types & API
- [ ] Define `features/inventory/types.ts`: InboundRecord, OutboundRecord, InventoryItem, InventoryLog, Location, CheckRecord, CheckItem etc.
- [ ] Define `features/inventory/api/inventoryApi.ts`:
  - `getInboundRecords(...)` / `getInboundRecord(id)` / `createInboundRecord(data)` / `deleteInboundRecord(id)`
  - `approveInbound(id)` / `rejectInbound(id, reason)` — approve/reject non-purchase inbound
  - `getOutboundRecords(...)` / `getOutboundRecord(...)` / `createOutboundRecord(...)` / `deleteOutboundRecord(...)`
  - `approveOutbound(id)` / `rejectOutbound(id, reason)` — approve/reject non-sales outbound
  - `getInventory(filter)` — real-time stock
  - `getInventoryLogs(filter)` / `getPipeLifecycle(pipeType, pipeId)`
  - `createCheck(data)` / `submitCheckItem(checkId, itemId, found)` / `getCheckReport(id)` / `getCheckList(filter)`
  - `getLocations(filter)` / `createLocation(data)` / `assignPipe(pipeId, locationId)` / `transferPipe(pipeId, newLocationId)`
- [ ] Implement `hooks/useInventory.ts` (React Query hooks wrapper)

### 1.2 Inventory Shared Components
- [ ] Implement `StockSummaryCards`: inventory overview KPI cards (total stock, counts by type, in-stock/outbound ratio)
- [ ] Implement `InventoryTable`: real-time stock table (grouped/aggregated by grade/spec)
- [ ] Implement `LocationTree`: location tree component (zone → shelf → level)

### 1.3 Stock Query Page
- [ ] Implement `InventoryPage` (real-time stock page):
  - Top StockSummaryCards
  - Filters (pipe type, grade, location)
  - InventoryTable display
  - Click row to view pipe detail list under that spec

### 1.4 Inbound Management Pages
- [ ] Implement `InboundListPage`:
  - Inbound record table (order number, date, type, supplier, status)
  - Filters (date range, inbound type)
  - Actions: view detail, create inbound
- [ ] Implement `InboundFormPage` (create inbound):
  - Select inbound type (purchase / production / return)
  - Type-based rules:
    - Purchase: order_id required, show PO selector filtered to approved orders; auto-validate selected pipes match PO items
    - Production/Return: no order_id needed, requires warehouse supervisor approval after creation, page shows "Needs supervisor approval to take effect"
  - Select pipes for inbound from pipe list (batch select + batch create new pipes)
  - Submit → create inbound order
- [ ] Implement `InboundApprovalPanel` (inbound approval panel):
  - Show pending inbound orders (approval_status=pending, inbound_type=production/return)
  - Click to view detail + approve/reject buttons
  - Auto-refresh stock on approval; reject requires reason

### 1.5 Outbound Management Pages
- [ ] Implement `OutboundListPage` (same structure as inbound list):
  - Outbound record table + filters + actions
- [ ] Implement `OutboundFormPage` (create outbound):
  - Select outbound type (sales / transfer / scrapped)
  - Type-based rules:
    - Sales: order_id required, show SO selector filtered to approved orders; auto-validate selected pipes match SO items
    - Transfer/Scrapped: no order_id needed, requires warehouse supervisor approval after creation, page shows "Needs supervisor approval to take effect"
  - Select pipes from in-stock list
  - Submit → validate stock → create outbound order
- [ ] Implement `OutboundApprovalPanel` (outbound approval panel):
  - Show pending outbound orders (approval_status=pending, outbound_type=transfer/scrapped)
  - Click to view detail + approve/reject buttons
  - Auto-deduct stock on approval; reject requires reason

### 1.6 Inventory Log Page
- [ ] Implement `InventoryLogPage`:
  - Log table (time, pipe number, change type, linked document, operator)
  - Filter by pipe number / date range / change type
  - Click pipe number to navigate to detail

### 1.7 Inventory Check Pages
- [ ] Implement `InventoryCheckPage`:
  - Check record list (check order number, date, status, checker)
  - Actions: create new check
- [ ] Create check: select location → system auto-generates pending check list → confirm creation
- [ ] Execute check: verify pipes one by one (found / missing) → submit results
- [ ] View check variance report (list comparison + variance count stats)

### 1.8 Location Management Page
- [ ] Implement `LocationManagePage`:
  - LocationTree showing all locations
  - Add/edit/delete location
  - Click location to view pipes at that location
  - Support drag or select pipes for location transfer

### 1.9 i18n
- [ ] Create `src/i18n/resources/zh/inventory.json` and `en/inventory.json`

> **Deps**: Pipe management frontend module (references pipe types / selection components)
