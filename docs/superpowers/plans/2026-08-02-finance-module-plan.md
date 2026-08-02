# Finance Module Implementation Plan

**Goal:** Full accounting module: chart of accounts, GL journal entries, AR/AP, invoices, payments, multi-currency, financial reports.

**Architecture:** New crate `finance` inside modules with same pattern as others. All tables in schema `finance`.

---

### Task 1: Create finance schema tables

```sql
CREATE TABLE finance.chart_of_accounts (id BIGSERIAL, account_code VARCHAR(50) UNIQUE, account_name VARCHAR(200), parent_id BIGINT, ...);
CREATE TABLE finance.journal_entries (id BIGSERIAL, entry_no VARCHAR(100), date DATE, ...);
CREATE TABLE finance.journal_entry_details (id, account_id, amount_debit NUMERIC, amount_credit NUMERIC);
CREATE TABLE finance.invoices, payments, payment_requests...
```

### Task 2: Core business logic — create journal entry (transaction)

- Recording: Delivery to: 3-way check (数据库系统)

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