# Phase 3 — Frontend: Contracts Module (P2)

> Based on: `docs/frontend-design.en.md` §4.1

## Task List

### 1.1 Shared Types & API
- [ ] Define `features/contracts/types.ts`: Contract, ContractItem, ContractPayment, etc.
- [ ] Define `features/contracts/api/contractApi.ts`:
  - Contract CRUD + status changes
  - Payment schedule CRUD

### 1.2 Contract List Page
- [ ] Implement `ContractListPage`:
  - Filters: contract number, type (sales/purchase), status, date range, linked customer/supplier
  - Table columns: number, type, customer/supplier, total amount, status, signing date, expiry date, actions
  - Row actions: view detail, edit, delete, change status

### 1.3 Contract Form Page
- [ ] Implement `ContractFormPage`:
  - Basic info: contract number (auto / manual), type selector, customer/supplier selector
  - Line items: dynamic rows (product name, spec, qty, unit price, amount, delivery date)
  - Auto-calculate total amount
  - Payment schedule: dynamic rows (milestone, amount, due date, notes)
  - File attachment upload
  - Date pickers: signing date, effective date, expiry date

### 1.4 Contract Detail Page
- [ ] Implement `ContractDetailPage`:
  - Card-based layout showing basic info
  - Line items table
  - Payment schedule table (with paid/unpaid status)
  - Status change dropdown
  - Linked orders list (purchase/sales order cross-reference)

> **Dependencies**: Purchase / Sales modules (supplier & customer pickers)
