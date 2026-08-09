# Procurement Deepen Implementation Plan

**Goal:** Enhance existing purchase order module with requisitions, quotes, 3-way matching, delivery tracking.

**Architecture:** Extend existing `backend/src/procurement/` module with new tables `purchase_requisitions`, `supplier_quotes`, `po_receipts`. Minimal new handlers.

**Database:** SQLite3 (`sqlite://data/erp.db?mode=rwc`), 无 schema 前缀

---

### Task 1: Add database columns/tables

Extend `suppliers` with new columns, create `purchase_requisitions`, `supplier_quotes`, `po_receipts` table migration.

### Task 2: Create purchase requisition service

```rust
PurchaseService::create_requisition(dto) → return req_id
PurchaseService::approve_requisition(req_id) → generate PO
```

### Task 3: Quote management service + API

### Task 4: Goods receipt matching service

When a receipt is created: reconcile PO → receipt → invoice flow pattern.

### Task 5: Frontend — supplier scorecard + requisition page
