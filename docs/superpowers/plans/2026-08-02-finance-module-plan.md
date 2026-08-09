# Finance Module Implementation Plan

**Goal:** Full accounting module: chart of accounts, GL journal entries, AR/AP, invoices, payments, multi-currency, financial reports.

**Architecture:** New module `backend/src/finance/` with same pattern as others. Tables use `finance_` prefix (SQLite, no schema).

---

### Task 1: Create finance schema tables

```sql
CREATE TABLE IF NOT EXISTS finance_chart_of_accounts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    account_code TEXT UNIQUE, account_name TEXT, parent_id INTEGER, ...
);
CREATE TABLE IF NOT EXISTS finance_journal_entries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    entry_no TEXT, date TEXT, ...
);
CREATE TABLE IF NOT EXISTS finance_journal_entry_details (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    account_id INTEGER, amount_debit REAL, amount_credit REAL
);
CREATE TABLE IF NOT EXISTS finance_invoices (...);
CREATE TABLE IF NOT EXISTS finance_payments (...);
```

### Task 2: Core business logic — create journal entry (transaction)

- Recording: 3-way check (数据库系统)

### Task 3: AR/AP flow — invoice → journal entry → payment

```rust
FinanceService::record_invoice(type="AR", source="sales_order", journal: credit=AR account, debit=Revenue)
FinanceService::record_payment(source="AR invoice", credit=Bank, debit=AR account)
// Simpler pattern for AP
```

### Task 4: Multi-currency exchange rates

### Task 5: Reconciliation (debit = credit equality check)

### Task 6: 前端页面

`ChartOfAccountsPage`, `JournalEntryPage`, `InvoicePage`, `PaymentPage`

---
