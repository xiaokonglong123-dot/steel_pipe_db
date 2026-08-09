# Supplier/Customer Portal Implementation Plan

**Goal:** External login for suppliers and customers to view PO, accept, submit.

**Architecture:** New `backend/src/portal/` module.

**Database:** SQLite3 (`sqlite://data/erp.db?mode=rwc`), 无 schema 前缀

---

### Task 1: Portal table — portal_accounts tied to tenants

### Task 2: Supplier view endpoints

### Task 3: Customer view endpoints

### Task 4: Frontend — PortalLoginPage, separate routes
