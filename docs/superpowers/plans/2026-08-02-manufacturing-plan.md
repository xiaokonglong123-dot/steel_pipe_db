# Manufacturing Management Implementation Plan

**Goal:** BOM, work orders, routing, quality inspections, NCR, equipment register.

**Architecture:** New `manufacturing` crate. All tables in `manufacturing` schema.

---

### Task 1: Schema — 6 new tables: bom, bom_items, work_orders, routing_ops, quality_inspections, ncr, equipment

### Task 2: BOM service — tree-based CRUD, recursive explode

`BomService::explode_bom(id) → flat material requirements list`

### Task 3: Work order creation + lifecycle

- create from BOM → stream generation steps
- WorkOrder lifecycle management: `start` → `in_progress` (containers), `quality_check`, `done`

### Task 4: Quality inspection flow, NCR creation

### Task 5: Equipment maintenance scheduling

### Task 6: Frontend

BomListPage, BomDetailPage (tree), WorkOrderDetailPage (progress)