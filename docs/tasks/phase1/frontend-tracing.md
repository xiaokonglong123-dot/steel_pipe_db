# Phase 1 — Frontend: Traceability Module (P0 MVP)

> Based on: `docs/requirements.en.md` §3.9; `docs/frontend-design.en.md` §4.1
> Architecture: Vite + React 19 + Ant Design 5 + TanStack Query 5. 唯一对象维度为「商品 (Item) + SKU」；无管材专属字段. 后端 crate `erp-server`，DB = SQLite3. 术语见 `specs/UBIQUITOUS_LANGUAGE_LATEST.md`.

## Tasks

### 1.1 What This Is

Traceability is a cross-cutting concern. On the frontend, it shows up as timeline views in item detail pages and inventory log pages.

### 1.2 Traceability in Item Detail Pages

- [ ] Add tabs to `ItemDetailPage` (通用商品详情):
  - **Inbound/Outbound History** Tab: show that item's inventory_logs as a timeline
  - **Operation Logs** Tab: show operation_logs related to that item
- [ ] Implement `TraceTimeline` component (Ant Design Timeline):
  - Display each stock change chronologically
  - Different change types get different icons/colors (inbound green, outbound blue, transfer orange, check purple)
  - Each entry shows: time, change type, linked document number, operator
  - Click linked document number to navigate to the corresponding inbound/outbound detail page

### 1.3 Lot Number Traceability

- [ ] Add lot number traceability entry point in manufacturing module or item search page
- [ ] Input lot number → show all items under that lot number + each item's current status

> **Deps**: 商品主数据 (item) 前端模块、库存管理前端模块
