# Phase 1 — Frontend: Traceability Module (P0 MVP)

> Based on: `docs/requirements.en.md` §3.9; `docs/frontend-design.en.md` §4.1

## Tasks

### 1.1 What This Is
Traceability is a cross-cutting concern. On the frontend, it shows up as timeline views in pipe detail pages and inventory log pages.

### 1.2 Traceability in Pipe Detail Pages
- [ ] Add tabs to `SeamlessPipeDetailPage` and `ScreenPipeDetailPage`:
  - **Inbound/Outbound History** Tab: show that pipe's inventory_logs as a timeline
  - **Operation Logs** Tab: show operation_logs related to that pipe
- [ ] Implement `TraceTimeline` component (Ant Design Timeline):
  - Display each stock change chronologically
  - Different change types get different icons/colors (inbound green, outbound blue, transfer orange, check purple)
  - Each entry shows: time, change type, linked document number, operator
  - Click linked document number to navigate to the corresponding inbound/outbound detail page

### 1.3 Heat Number Traceability
- [ ] Add heat number traceability entry point in quality module or pipe search page
- [ ] Input heat number → show all pipes under that heat number + each pipe's current status

> **Deps**: Pipe management frontend module, inventory management frontend module
