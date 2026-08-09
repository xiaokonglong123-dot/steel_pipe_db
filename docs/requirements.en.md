# ERP (Enterprise Resource Planning) System — PRD

> **Version**: v1.1 (general-purpose ERP rewrite)
> **Date**: 2026-08
> **Stack**: Rust (Axum, backend crate `erp-server`) + React
> **Type**: Web App (Frontend/Backend split)
> **History**: This system was refactored from a steel-pipe industry system; all legacy pipe terminology is deprecated.

---

## Revision History

| Version | Date | Changes | Author |
|---------|------|---------|--------|
| v1.1 | 2026-08 | General-purpose ERP rewrite: Item/SKU master data, removed pipe/threading/labels/quality-cert modules, migrated to SQLite3, added workflow/HR/finance/manufacturing/project/assets/notification/portal/BI modules | — |
| v1.0 | 2026-05-19 | Initial version (legacy steel-pipe system era) | — |

---

## 1. Project Background & Objectives

### 1.1 Background

Procurement, sales, inventory, manufacturing, finance, and HR all run as separate silos, with documents scattered across spreadsheets and paper forms. There is no integrated system tying them together — from purchase order, to arrival and inbound, to stock management, to sales outbound, to accounting and management reporting. Everything depends on experience and manual reconciliation, which produces errors and inconsistent numbers.

### 1.2 Objective

Build a Rust-based general-purpose ERP system (backend crate `erp-server`) that handles **Item (Item+SKU) master data, inventory, procurement, sales, manufacturing inspection, finance, HR, projects, fixed assets, and workflow approvals** in one place. One system to run the business — from order to ledger, from warehouse to dashboard.

---

## 2. Target Users & Roles

| Role | What They Do | Key Concerns |
| ------ | ------------- | -------------- |
| **Warehouse Operator** | Receiving, outbound, transfer, stock count, inventory query | Stock levels, bin locations, speed |
| **Procurement / Sales Staff** | Purchase orders, sales orders, supplier/customer management | ATP stock, document status, contract info |
| **Finance / HR Staff** | Accounting, invoicing, payments, payroll | Accounts, journal entries, invoices, attendance and salaries |
| **Manufacturing / QC Staff** | Work order execution, inspection records, NCR handling | BOM, work order status, inspection results, NCR closure |
| **Management** | Dashboards, BI analytics, decisions | Inventory turns, sales trends, finance summary, supplier performance |

System is **multi-user**, role-based access (RBAC).

---

## 3. Functional Requirements

### 3.1 Item Master Data (P0 — Must Have)

**FR-ITEM-001: Item Master Data Management**

- **Description**: CRUD for the Item entity — the single business object of the whole system
- **Fields**:
  - SKU (globally unique; system-assigned or manual)
  - Name (item name)
  - Category (raw material / semi-finished / finished goods / spare parts, etc.)
  - Unit (kg / m / pc / piece, etc.)
  - Spec (descriptive attribute, optional, carries no industry-mandated fields)
  - Status (draft / active / disabled)
  - Notes, attachments (images, manuals, etc.)
- **Acceptance Criteria**:
  - Full CRUD
  - SKU globally unique — no collisions
  - Search by any combination of fields
  - Single item table — no per-industry-type table splitting

### 3.2 Inventory Management (P0 — Must Have)

**FR-INV-001: Inbound Management**

- Multiple inbound types: purchase receipt, production completion, return, etc.
- Each record: inbound number, date, item details (qty/batch), supplier/source, operator
- Auto-update stock on inbound

**FR-INV-002: Outbound Management**

- Sales outbound, internal requisition, transfer outbound, etc.
- Each record: outbound number, date, customer/destination, item details, operator
- Batch or per-SKU outbound

**FR-INV-003: Stock Query & Count**

- Real-time stock by item, category, location, warehouse
- Movement ledger — full inbound/outbound history per item
- Stock count: generate count sheets, enter counts, produce variance reports

**FR-INV-004: Location Management**

- Multi-level locations: warehouse → location
- Bind items to locations, support moves

**FR-INV-005: Reservation / ATP**

- Reserve available stock for sales orders (or work orders)
- Available-to-Promise (ATP) visibility

### 3.3 Manufacturing & Inspection (P1 — Should Have)

**FR-MFG-001: BOM & Work Orders**

- Maintain item BOMs (bill of materials)
- Release work orders from BOMs, track progress; work orders consume items per BOM

**FR-MFG-002: Inspection & NCR**

- Work orders link to inspection records: test items, results, date, inspector
- Create NCR (Non-Conformance Report) when inspection fails; track corrective closure

### 3.4 Procurement & Sales (P1 — Should Have)

**FR-PUR-001: Procurement Management**

- PO creation, approval flow, tracking
- Supplier info management (name, contact, qualifications, etc.)
- Purchase receipt links to inbound

**FR-PUR-002: Requisitions & Supplier Quotes**

- Internal purchase requisitions flowing through the system
- Supplier quote records and comparison

**FR-SALE-001: Sales Management**

- SO creation, approval flow, tracking
- Customer info management
- Sales outbound deducts inventory
- Available-to-promise (ATP) stock visibility

**FR-SALE-002: Customer Quotes & Customer Credit**

- Customer quotes issued to clients
- Customer credit limit management

**FR-CONTRACT-001: Contract Management**

- Purchase / sales contract basic info
- Link contracts to orders

### 3.5 Workflow Approvals (P1 — Should Have)

**FR-WF-001: Workflow Engine**

- Workflow definitions (node/condition configuration)
- Workflow instances and workflow tasks (todos)
- Purchase orders, sales orders, requisitions, etc. run through workflows

### 3.6 Data Import/Export (P1 — Should Have)

**FR-IO-001: Data Import**

- Excel/CSV batch import for item data
- Validate format and required fields during import
- Import result report (success/fail counts + reasons)

**FR-IO-002: Data Export**

- Export query results to Excel/CSV
- Standard reports: inventory summary, inbound/outbound details, etc.

### 3.7 Search & Filter (P0 — Must Have)

**FR-SEARCH-001: Multi-dimensional Search**

- Combined queries: SKU, name, category, unit, spec, status, location, etc.
- Fuzzy search (partial SKU, name, etc.)
- Paginated results

### 3.8 History Traceability (P0 — Must Have)

**FR-TRACE-001: Full Lifecycle Traceability**

- Every operation on every item logged — from inbound to outbound
- Every change tracked: who, when, what fields
- View full lifecycle by SKU

### 3.9 HR Management (P1 — Should Have)

**FR-HR-001: Employees & Departments**

- Employee profiles, department structure

**FR-HR-002: Attendance / Salary / Labor Contracts**

- Attendance records, salary payments, labor contract management

### 3.10 Finance Management (P1 — Should Have)

**FR-FIN-001: Accounts & Journal Entries**

- Chart of accounts, journal entry records

**FR-FIN-002: Invoices & Payments**

- Invoice (issued/received) records, outgoing payment records

**FR-FIN-003: Trial Balance**

- Financial statement validating debit/credit balance

### 3.11 Projects & Fixed Assets (P2 — Could Have)

**FR-PROJ-001: Projects & WBS**

- Project maintenance, WBS breakdown, budget management

**FR-ASSET-001: Fixed Assets**

- Fixed asset registration, straight-line depreciation, disposal

### 3.12 Notifications & Portal (P2 — Could Have)

**FR-NOTIF-001: Notifications**

- Notification inbox, templates, preferences

**FR-PORTAL-001: Portal**

- Portal account (Party) management: customer/supplier identities in the portal

### 3.13 BI Analytics & System Management (P1 — Should Have)

**FR-BI-001: BI Analytics**

- Aggregated reports: sales trend, inventory value, finance summary, supplier performance

**FR-SYS-001: User & Permission Management**

- User management, RBAC (roles + permissions + departments, optional tenants)
- Menus and buttons adapt to role

**FR-SYS-002: Operation Logs**

- Log key user actions: login, data changes, etc.
- Queryable and exportable

---

## 4. Non-Functional Requirements

### 4.1 Performance

| Metric | Target |
| -------- | -------- |
| Single page query response | ≤ 2s (within 100K records) |
| Data import | ≤ 60s for 100K records |
| Concurrent users | ≥ 20 simultaneous |
| Availability | 99.5% (≤ 44 hrs/year downtime) |

### 4.2 Data Scale

- Item master data: 100K+ SKUs
- Inventory movement logs: millions of records
- Storage: SQLite3 (connection string `sqlite://data/erp.db?mode=rwc`, WAL mode, well-indexed)

> **Note**: SQLite3 handles single-machine / small-scale concurrency fine. WAL mode + a properly configured connection pool are key for a web app. The connection string is fixed at `sqlite://data/erp.db?mode=rwc`; the DB file lives at `data/erp.db`.

### 4.3 Internationalization & Units

- **UI Language**: Chinese + English, switchable at runtime
- **Unit System**: items carry a `unit` field (kg / m / pc / etc.); unit conversion is handled by the frontend unitStore
- Storage stays consistent with the item's unit field

### 4.4 Security

- Passwords hashed with **Argon2id** (not bcrypt — we use the `argon2` crate with OWASP-recommended params: m=19456, t=2, p=1)
- Sensitive operations (delete/modify critical data) require confirmation
- API auth via **JWT** (jsonwebtoken crate, configurable expiry, refresh token rotation)
- HTTPS in production (obviously)

### 4.5 Maintainability

- Rust's type system catches a whole class of bugs at compile time
- Modular architecture — handler → service → repository, clear layer boundaries
- Backend REST API, frontend just calls HTTP
- API docs via OpenAPI (utoipa)

### 4.6 System Architecture & Technology Stack

| Layer | Technology | Why |
| ------- | ----------- | ----- |
| **Backend** | Rust + Axum 0.8 + SQLx 0.8 (crate `erp-server`) | Axum is the most ergonomic async web framework in Rust right now. SQLx gives compile-time checked SQL with native SQLite support via its `sqlite` feature. No ORM overhead. |
| **Database** | SQLite3 (`sqlite://data/erp.db?mode=rwc`, WAL) | Zero config, file-level, perfect for this scale. The 37 legacy migration files will be rewritten to SQLite syntax minus the pipe tables. |
| **Frontend** | React 19 + TypeScript (strict) + Vite + Ant Design 5 + TanStack Query 5 + Zustand 5 | React 19 is the latest stable. Vite is insanely fast for dev. TypeScript strict catches nulls and bad types. |
| **API** | JSON REST | Standard RESTful — easy to integrate, debug with curl, works with any frontend. |

---

## 5. Item Master Data Reference

> The Item is the single business entity of the system, and the SKU is its unique business code. This section defines the generic master-data conventions, replacing the industry-specific fields of the legacy pipe era.

### 5.1 SKU Numbering Rules

| Rule | Description | Example |
| ------ | ------------- | --------- |
| Auto-generated | System generates `{category-code}-{yyyymm}-{seq}` | `FG-202608-0001` |
| Manual | Manual entry allowed, but must be globally unique | Company-specific codes |
| Uniqueness | Globally unique, no collisions | — |

### 5.2 Categories

| Category | Description |
| ---------- | ------------- |
| Raw material | Inbound raw materials for production |
| Semi-finished | Intermediate products in manufacturing |
| Finished goods | Final items ready for sale |
| Spare parts | Maintenance and consumable items |

### 5.3 Units

`kg` / `m` / `pc` / `piece` / `box` / `L` — units are maintained per category; cross-unit conversion is handled by the frontend unitStore.

### 5.4 Spec

Spec is a descriptive free-text attribute (e.g. dimensions, material, model) and **carries no industry-mandated fields**.

### 5.5 Item Status

| Status | Description |
| -------- | ------------- |
| draft | Draft, not tradable |
| active | Enabled, available for procurement/sales |
| disabled | Disabled, no longer traded |

---

## 6. Preliminary Data Model

### 6.1 Core Entity Relationships

```
                    ┌──────────────────────┐
                    │     Item (商品/SKU)    │  ← Single business entity, one table
                    └──────────┬───────────┘
                               │
Supplier ──→ PurchaseOrder ──→ InboundRecord ──→ Item ──→ OutboundRecord ──→ Customer
                     ↑                                              │
                Contract(purchase)                              SalesOrder
                     │                                              ↑
                     └──────────────────────────────────────────────┘

Item ──N:1── Location (warehouse → location)
Item ──N:1── InventoryLog (movement audit)
WorkOrder ──N:1── Item (BOM consumption) ──1:N── Inspection ── NCR
Supplier ──1:N── PortalAccount
```

### 6.2 Main Data Entities

| Entity | Description | Core Fields |
| -------- | ------------- | ------------- |
| **Item** | Item master data | id, sku(unique), name, category, unit, spec, status, notes, attachments, created_at, updated_at, deleted_at |
| **Location / Warehouse** | Storage spot / warehouse | id, warehouse_id, code, description, max_capacity, current_usage |
| **Supplier** | Supplier | id, code, name, contact_person, phone, email, address, qualification, score |
| **Customer** | Customer | id, code, name, contact_person, phone, email, address, credit_limit |
| **PurchaseOrder** | PO header | id, order_no, supplier_id, order_date, status, total_amount, currency, contract_id |
| **PurchaseOrderItem** | PO line item | id, order_id, item_id, quantity, received_quantity, unit_price |
| **SalesOrder** | SO header | id, order_no, customer_id, order_date, status, total_amount, currency, contract_id |
| **SalesOrderItem** | SO line item | id, order_id, item_id, quantity, delivered_quantity, unit_price |
| **InboundRecord / Item** | Inbound header/items | id, record_no, inbound_type(purchase/production/return), order_id, supplier_id, approval_status, items[] |
| **OutboundRecord / Item** | Outbound header/items | id, record_no, outbound_type(sales/transfer/scrapped), order_id, customer_id, approval_status, items[] |
| **InventoryLog** | Movement log | id, item_id, change_type, reference_id, operator_id, operator_name, remark, created_at |
| **Reservation** | ATP reservation | id, item_id, sales_order_id / work_order_id, quantity, status |
| **WorkOrder** | Work order | id, work_order_no, item_id (product), bom_id, quantity, status, start_date, due_date |
| **Inspection** | Inspection record | id, work_order_id, inspect_date, inspector, result(pass/fail/pending), test_items, file_url |
| **NCR** | Non-conformance report | id, inspection_id, description, corrective_action, status |
| **WorkflowDefinition** | Workflow template | id, name, nodes(JSON), conditions(JSON), is_active |
| **WorkflowInstance** | Workflow instance | id, definition_id, business_type, business_id, status |
| **WorkflowTask** | Approval task | id, instance_id, node_key, assignee_id, status, comment |
| **Employee** | Employee | id, employee_no, name, department_id, position, phone, email, hire_date |
| **Attendance / Salary / LaborContract** | Attendance / salary / labor contract | separate tables, linked via employee_id |
| **Account / JournalEntry / Invoice / Payment** | Finance core | separate tables |
| **Project / WBS / BudgetItem** | Project / WBS / budget | separate tables, linked via project_id |
| **FixedAsset / Depreciation / Disposal** | Fixed assets | separate tables, linked via asset_id |
| **Notification** | Notification | id, user_id, title, content, read_at, template_id |
| **PortalAccount** | Portal identity | id, party_type(customer/supplier), party_id, username, password_hash |
| **User / Role / Permission** | RBAC | separate tables |
| **OperationLog** | Audit log | id, user_id, username, action, target_type, target_id, detail(JSON), ip_address, created_at |

---

## 7. Priority Roadmap

| Phase | Scope | Priority |
| ------- | ------- | ---------- |
| **Phase 1 (MVP)** | Item master data, Inventory (inbound/outbound/query/count), Multi-dimensional search, Auth & RBAC, History traceability | P0 |
| **Phase 2** | Procurement/Sales/Contracts, Workflow approvals, Manufacturing & Inspection (BOM/work orders/Inspection/NCR), Data import/export | P1 |
| **Phase 3** | Finance, HR, Projects & Fixed Assets, Notifications & Portal, BI analytics, i18n & unit switching | P2 |

---

## 8. Appendix

### 8.1 Glossary

| Term | English | What It Is |
| ------ | --------- | ------------ |
| 商品 | Item | The tradable business object, the single entity of the system |
| SKU | SKU | Unique business code of an item |
| 规格 | Spec | Descriptive attribute of an item (optional) |
| 库存 | Inventory | Stock records of items at locations |
| 入库 | Inbound | Business action of items entering inventory |
| 出库 | Outbound | Business action of items leaving inventory |
| 盘点 | Count Session | Periodic reconciliation of book vs. physical inventory |
| 采购订单 | Purchase Order | Formal purchase document issued to a supplier |
| 销售订单 | Sales Order | Formal sales document from a customer |
| 采购收货 | Receipt | Confirmation of supplier arrival and inbound |
| 发货 | Shipment | Confirmation of sales outbound to a customer |
| 审批流 | Workflow | Approval engine for business documents |
| 工单 | Work Order | Execution document for production/processing tasks |
| 质检 | Inspection | Quality inspection record in manufacturing (not the legacy "quality certificate") |
| 不合格品单 | NCR | Correction record when inspection fails |
| 固定资产 | Fixed Asset | Long-held depreciable asset |
| 门户账户 | Party | Customer/supplier identity in the portal |
| BI 分析 | Analytics | Decision-oriented aggregated reporting |

### 8.2 Related Documents

- Terminology canon: `specs/UBIQUITOUS_LANGUAGE_LATEST.md` (authoritative term list; all docs must follow it)
- Detailed design: `docs/detailed-design.en.md` (DB schema, API endpoints, architecture)
- Frontend design: `docs/frontend-design.en.md` (component tree, routing, state management)
