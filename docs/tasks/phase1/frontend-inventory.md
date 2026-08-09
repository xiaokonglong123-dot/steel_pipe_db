# Phase 1 — Frontend: Inventory Management Module (P0 MVP)

> Based on: `docs/frontend-design.en.md` §4.1, §4.2, §6, §8
> Architecture: Vite + React 19 + Ant Design 5 + TanStack Query 5 + Zustand 5. 路由 `createBrowserRouter` + `ProtectedRoute`；server state 走 TanStack Query (key factory 集中在 `queryKeys.ts`)。对象维度统一为「商品 (Item) + SKU」；无管材专属字段. 后端 crate `erp-server`，DB = SQLite3. 术语见 `specs/UBIQUITOUS_LANGUAGE_LATEST.md`.

## Tasks

### 1.1 Shared Types & API

- [ ] Define `features/inventory/types.ts`: Item, InboundRecord, OutboundRecord, InventoryItem, InventoryLog, Location, CheckRecord, CheckItem, InventoryReservation etc.
- [ ] Define `features/inventory/api/inventoryApi.ts`:
  - `getInboundRecords(...)` / `getInboundRecord(id)` / `createInboundRecord(data)` / `deleteInboundRecord(id)`
  - `approveInbound(id)` / `rejectInbound(id, reason)` — approve/reject non-purchase inbound
  - `getOutboundRecords(...)` / `getOutboundRecord(...)` / `createOutboundRecord(...)` / `deleteOutboundRecord(...)`
  - `approveOutbound(id)` / `rejectOutbound(id, reason)` — approve/reject non-sales outbound
  - `getInventory(filter)` — real-time stock
  - `getInventoryLogs(filter)` / `getItemLifecycle(sku)` (按 SKU 查全生命周期)
  - `createCheck(data)` / `submitCheckItem(checkId, itemId, found)` / `getCheckReport(id)` / `getCheckList(filter)`
  - `getLocations(filter)` / `createLocation(data)` / `assignItem(sku, locationId)` / `transferItem(sku, newLocationId)`
  - `createReservation(data)` / `releaseReservation(id)`
- [ ] Define `features/inventory/queryKeys.ts` (TanStack Query key factory)
- [ ] Implement `hooks/useInventory.ts` (React Query hooks wrapper)

### 1.2 Inventory Shared Components

- [ ] Implement `StockSummaryCards`: inventory overview KPI cards (total stock, counts by category, in-stock/outbound ratio)
- [ ] Implement `InventoryTable`: real-time stock table (grouped/aggregated by category/spec)
- [ ] Implement `LocationTree`: location tree component (zone → shelf → level)

### 1.3 Stock Query Page

- [ ] Implement `InventoryPage` (real-time stock page):
  - Top StockSummaryCards
  - Filters (category, spec, location)
  - InventoryTable display
  - Click row to view item detail list under that spec

### 1.4 Inbound Management Pages

- [ ] Implement `InboundListPage`:
  - Inbound record table (order number, date, type, supplier, status)
  - Filters (date range, inbound type)
  - Actions: view detail, create inbound
- [ ] Implement `InboundFormPage` (create inbound):
  - Select inbound type (purchase / production / return)
  - Type-based rules:
    - Purchase: order_id required, show PO selector filtered to approved orders; auto-validate selected items match PO items
    - Production/Return: no order_id needed, requires warehouse supervisor approval after creation, page shows "Needs supervisor approval to take effect"
  - Select items for inbound from item list (batch select + batch create new items)
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
    - Sales: order_id required, show SO selector filtered to approved orders; auto-validate selected items match SO items
    - Transfer/Scrapped: no order_id needed, requires warehouse supervisor approval after creation, page shows "Needs supervisor approval to take effect"
  - Select items from in-stock list
  - Submit → validate stock → create outbound order
- [ ] Implement `OutboundApprovalPanel` (outbound approval panel):
  - Show pending outbound orders (approval_status=pending, outbound_type=transfer/scrapped)
  - Click to view detail + approve/reject buttons
  - Auto-deduct stock on approval; reject requires reason

### 1.6 Inventory Log Page

- [ ] Implement `InventoryLogPage`:
  - Log table (time, SKU, change type, linked document, operator)
  - Filter by SKU / date range / change type
  - Click SKU to navigate to item detail

### 1.7 Inventory Check Pages

- [ ] Implement `InventoryCheckPage`:
  - Check record list (check order number, date, status, checker)
  - Actions: create new check
- [ ] Create check: select location → system auto-generates pending check list → confirm creation
- [ ] Execute check: verify items one by one (found / missing) → submit results
- [ ] View check variance report (list comparison + variance count stats)

### 1.8 Location Management Page

- [ ] Implement `LocationManagePage`:
  - LocationTree showing all locations
  - Add/edit/delete location
  - Click location to view items at that location
  - Support drag or select items for location transfer

### 1.9 Inventory Reservation (ATP) Page

- [ ] Implement `ReservationPage`:
  - List active reservations (by order / by SKU)
  - Create reservation (select sales order / work order + SKU + qty)
  - Release reservation (manual release + auto-release on outbound completion)
  - Show available stock vs reserved stock per SKU

### 1.10 i18n

- [ ] Create `src/i18n/resources/zh/inventory.json` and `en/inventory.json`

> **Deps**: 商品主数据 (item) 前端模块 — 通过 SKU 关联；与库存 API 共享 `queryKeys.ts`
