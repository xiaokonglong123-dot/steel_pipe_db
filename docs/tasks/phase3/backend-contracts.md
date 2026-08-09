# Phase 3 — Backend: Contracts Module (P2)

> Based on: `docs/requirements.en.md` §3.4, `docs/detailed-design.en.md` §5.3.13
> Architecture: backend crate `erp-server` (Rust + Axum + SQLx 0.8 **sqlite** feature)；DB = SQLite3 at `sqlite://data/erp.db?mode=rwc`. 合同 (Contract) 包括 **销售合同** / **采购合同** / **劳动合同** 三种实体（与 `specs/UBIQUITOUS_LANGUAGE_LATEST.md` 一致），按 `ContractType` 区分；本模块聚焦 销售合同 / 采购合同，劳动合同由 HR 模块管理. 合同行项目按「商品 (Item) + SKU」为对象维度，无管材专属字段. 错误码 140xx (订单/合同). 术语见 `specs/UBIQUITOUS_LANGUAGE_LATEST.md`.

## Task List

### 1.1 Database Migration

- [ ] Create `contracts` table migration (main contract table, with `contract_type` enum: Sales / Purchase)
- [ ] Create `contract_items` table migration (contract line items, links to item via sku)
- [ ] Create `contract_payments` table migration (payment schedules / 收付款计划)

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
  - `create(dto)`: create contract with auto-generated contract number (format: SC- / PC- + date + seq for sales / purchase)
  - `update(id, dto)`: update basic contract info
  - `update_status(id, status)`: state machine (draft → active → completed / terminated)
  - `get_with_items(id)`: contract + line items + payment schedule
  - `list(filter)`: paginated filtered listing
  - `add_payment(id, dto)` / `update_payment(id, dto)` / `delete_payment(id)`

### 1.5 Handler Layer

- [ ] `GET /api/v1/contracts` — contract list (with type/status filter)
- [ ] `POST /api/v1/contracts` — create contract
- [ ] `GET /api/v1/contracts/{id}` — contract detail (with items + payments)
- [ ] `PUT /api/v1/contracts/{id}` — update contract
- [ ] `PUT /api/v1/contracts/{id}/status` — update status
- [ ] `DELETE /api/v1/contracts/{id}` — delete contract
- [ ] `POST /api/v1/contracts/{id}/payments` — add payment schedule
- [ ] `PUT /api/v1/contracts/{id}/payments/{payment_id}` — update payment schedule
- [ ] `DELETE /api/v1/contracts/{id}/payments/{payment_id}` — delete payment schedule

### 1.6 Testing

- [ ] Test contract CRUD
- [ ] Test status transition logic
- [ ] Test contract number generation (SC- / PC- prefix)
- [ ] Test payment schedule CRUD

> **Dependencies**: 商品主数据 (item) 模块 (合同行项目通过 SKU 引用)、采购模块 (采购合同引用供应商)、销售模块 (销售合同引用客户)
