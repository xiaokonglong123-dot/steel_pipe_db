# Phase 3 — Backend: Contracts Module (P2)

> Based on: `docs/requirements.en.md` §3.4, `docs/detailed-design.en.md` §5.3.13

## Task List

### 1.1 Database Migration
- [ ] Create `contracts` table migration (main contract table)
- [ ] Create `contract_items` table migration (contract line items)
- [ ] Create `contract_payments` table migration (payment schedules)

### 1.2 Domain Layer
- [ ] Define `Contract`, `ContractItem`, `ContractPayment` structs
- [ ] Define enums: `ContractStatus` (Draft / Active / Completed / Terminated), `ContractType` (Sales / Purchase)
- [ ] Define DTOs: `CreateContractDto`, `UpdateContractDto`

### 1.3 Repository Layer
- [ ] Implement `ContractRepo`: CRUD + list with filtering (type, status, date range, linked customer/supplier)
- [ ] Implement `ContractItemRepo`: batch create/update, query by contract_id
- [ ] Implement `ContractPaymentRepo`: CRUD, query by contract_id

### 1.4 Service Layer
- [ ] Implement `ContractService`:
  - `create(dto)`: Create contract with auto-generated contract number
  - `update(id, dto)`: Update basic contract info
  - `update_status(id, status)`: State machine (draft → active → completed / terminated)
  - `get_with_items(id)`: Contract + line items + payment schedule
  - `list(filter)`: Paginated filtered listing
  - `add_payment(id, dto)` / `update_payment(id, dto)` / `delete_payment(id)`

### 1.5 Handler Layer
- [ ] `GET /api/v1/contracts` — Contract list
- [ ] `POST /api/v1/contracts` — Create contract
- [ ] `GET /api/v1/contracts/{id}` — Contract detail (with items + payments)
- [ ] `PUT /api/v1/contracts/{id}` — Update contract
- [ ] `PUT /api/v1/contracts/{id}/status` — Update status
- [ ] `DELETE /api/v1/contracts/{id}` — Delete contract
- [ ] `POST /api/v1/contracts/{id}/payments` — Add payment schedule
- [ ] `PUT /api/v1/contracts/{id}/payments/{payment_id}` — Update payment schedule
- [ ] `DELETE /api/v1/contracts/{id}/payments/{payment_id}` — Delete payment schedule

### 1.6 Testing
- [ ] Test contract CRUD
- [ ] Test status transition logic
- [ ] Test contract number generation

> **Dependencies**: Purchase / Sales modules (references customer & supplier data)
