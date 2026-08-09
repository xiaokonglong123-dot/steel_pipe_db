# Sales CRM Implementation Plan

**Goal:** Deepen sales orders with shipment tracking, credit control, quote management.

**Architecture:** Extend existing `backend/src/sales_crm/` module. Add tables: `sales_quotes`, `shipments`. Update `customers` with credit fields.

**Database:** SQLite3 (`sqlite://data/erp.db?mode=rwc`), 无 schema 前缀

---

### Task 1: Make table migrations

```sql
CREATE TABLE IF NOT EXISTS sales_quotes (...);
CREATE TABLE IF NOT EXISTS shipments (...);
ALTER TABLE customers ADD COLUMN credit_limit REAL DEFAULT 0;
```

### Task 2: Create shipment service

### Task 3: Credit control API

`GET /api/customers/:id/credit` returns credit_available = credit_limit - current_open_invoice_amount

### Task 4: Quote to order conversion service

### Task 5: Frontend additions
