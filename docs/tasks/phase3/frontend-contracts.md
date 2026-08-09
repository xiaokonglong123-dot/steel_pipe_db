# Phase 3 — Frontend: Contracts Module (P2)

> Based on: `docs/frontend-design.en.md` §4.1
> Architecture: Vite + React 19 + Ant Design 5 + TanStack Query 5. 合同 (Contract) 包括 **销售合同** / **采购合同** 两种（劳动合同由 HR 模块管理）；合同行项目按「商品 (Item) + SKU」+ 规格 + 数量 + 单价 + 交期 选择. 后端 crate `erp-server`，DB = SQLite3. 术语见 `specs/UBIQUITOUS_LANGUAGE_LATEST.md`.

## Task List

### 1.1 Shared Types & API

- [ ] Define `features/contracts/types.ts`: Contract, ContractItem, ContractPayment, ContractType, ContractStatus etc.
- [ ] Define `features/contracts/queryKeys.ts` (TanStack Query key factory)
- [ ] Define `features/contracts/api/contractApi.ts`:
  - Contract CRUD + status changes
  - Payment schedule CRUD

### 1.2 Contract List Page

- [ ] Implement `ContractListPage`:
  - Filters: contract number, type (sales / purchase), status, date range, linked customer/supplier
  - Table columns: number, type, customer/supplier, total amount, status, signing date, expiry date, actions
  - Row actions: view detail, edit, delete, change status

### 1.3 Contract Form Page

- [ ] Implement `ContractFormPage`:
  - Basic info: contract number (auto SC-/PC- prefix / manual), type selector, customer/supplier selector
  - Line items: dynamic rows (item: SKU + name + spec, qty, unit price, amount, delivery date)
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
  - Linked orders list (purchase / sales order cross-reference)

> **Dependencies**: 商品主数据 (item) 前端模块 (ItemPicker 复用)、采购前端模块 (SupplierSelect)、销售前端模块 (CustomerSelect)
