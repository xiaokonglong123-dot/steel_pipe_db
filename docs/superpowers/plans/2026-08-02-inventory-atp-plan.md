# Inventory ATP Implementation Plan

**Goal:** ATP queries, stock reservations, expected arrivals, internal transfers.

**Architecture:** Extend inventory crate with atp_service. New tables: `atp_slots`, `internal_transfers`, `reservations`.

---

### Task 1: ATP table + service

`select stock(from inventory) + expected(从采购) - reserved(SO委托) = atp_qty`

### Task 2: Batch reservation service (for sales order creation)

### Task 3: Internal transfer (between warehouses) API

### Task 4: Frontend addition — ATP query page

---