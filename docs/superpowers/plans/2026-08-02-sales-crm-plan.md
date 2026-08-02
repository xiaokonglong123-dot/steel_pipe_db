# Sales CRM Implementation Plan

**Goal:** Deepen sales orders with shipment tracking, credit control, quote management.

**Architecture:** Extend existing orders crate. Add tables: `sales_quotes`, `shipments`. Update `customers` with credit fields.

---

### Task 1: Make table migrations

```sql
CREATE TABLE orders.sales_quotes (...);
CREATE TABLE orders.shipments (...);
ALTER TABLE orders.customers ADD COLUMN credit_limit ...
```

### Task 2: Create shipment service

### Task 3: Credit control API

`GET /api/customers/:id/credit` returns credit_available = credit_limit - current_open_invoice_amount

### Task 4: Quote to order conversion service

### Task 5: Frontend additions