# ERP (Enterprise Resource Planning) System — Detailed Design

> **Version**: v2.0 (general-purpose ERP rewrite)
> **Date**: 2026-08
> **Based on**: docs/requirements.en.md v1.1
> **Stack**: Rust (crate `erp-server`) + Axum + SQLx + SQLite3 (WAL) | React 19 + Ant Design 5
> **History**: This system was refactored from a steel-pipe industry system; the legacy pipe module is replaced by the generic Item/SKU module.

---

## Revision History

| Version | Date | Changes | Author |
| --------- | ------ | --------- | -------- |
| v2.0 | 2026-08 | General-purpose ERP rewrite: Item (Item+SKU) master data replaces the pipe master data; removed quality-cert/labels/threading modules; migrated to SQLite3 (the 37 legacy migrations rewritten to SQLite syntax minus the pipe tables); added workflow/HR/finance/manufacturing/project/asset/notification/portal/BI module designs | — |
| v1.0 | 2026-05-19 | Initial version (legacy steel-pipe system era) | - |
| v1.1 | 2026-05-19 | Inbound/outbound changed to Header+Items structure; inventory check changed to per-item verification; added attachments table; removed PATCH endpoints; fixed sort_by injection vulnerability | - |

---

## Table of Contents

1. [System Overview](#1-system-overview)
2. [Tech Stack Decisions](#2-tech-stack-decisions)
3. [System Architecture](#3-system-architecture)
4. [Module Design](#4-module-design)
5. [Database Detailed Design](#5-database-detailed-design)
6. [REST API Design](#6-rest-api-design)
7. [Project Directory Structure](#7-project-directory-structure)
8. [Error Handling & Response Spec](#8-error-handling--response-specification)
9. [Non-Functional Design](#9-non-functional-design)
10. [Security Design](#10-security-design)
11. [i18n & Unit Switching Design](#11-internationalization--unit-switching-design)

---

## 1. System Overview

### 1.1 What This Thing Is

A general-purpose ERP (Enterprise Resource Planning) system covering Item (Item+SKU) master data, inventory, procurement, sales, manufacturing inspection, finance, HR, projects, fixed assets, workflow approvals, notifications and portal. The backend crate is `erp-server`; the database is SQLite3 at `sqlite://data/erp.db?mode=rwc`.

### 1.2 Core Capabilities

| Capability | Description |
| ------------ | ------------- |
| Item lifecycle tracking | Full traceability from purchase receipt to sales dispatch |
| Integrated inventory | Procurement, stock, and sales all linked together |
| Manufacturing & inspection | BOMs, work orders, inspections, NCRs in one loop |
| Workflow approvals | Definition/instance/task engine for business documents |
| Multi-user RBAC | Roles/permissions/departments/tenants (4 sample roles: admin/warehouse/sales/procurement) |
| HR & finance | Employees/attendance/salaries/labor contracts; accounts/journal/invoices/payments/trial balance |
| Projects & fixed assets | Projects/WBS/budget; registration/straight-line depreciation/disposal |
| Notifications & portal | Inbox/templates/preferences; portal accounts (Party) |
| i18n | Chinese/English UI + item unit conversion |

---

## 2. Tech Stack Decisions

### 2.1 Backend

| Layer | Choice | Version | Why |
| ------- | -------- | --------- | ----- |
| **Web Framework** | Axum | 0.8+ | Mainstream Rust async framework, tower middleware, great ecosystem |
| **SQL Layer** | SQLx | 0.8+ | Compile-time checked SQL, no ORM overhead, native SQLite support via the `sqlite` feature |
| **Database** | SQLite3 (WAL) | 3.46+ | Zero config, file-level; connection string `sqlite://data/erp.db?mode=rwc` |
| **Serialization** | Serde + serde_json | 1.x | The Rust standard |
| **Auth** | JWT (jsonwebtoken) | — | Stateless, works well with SPA |
| **Password Hashing** | Argon2 | — | OWASP recommended. `m=19456, t=2, p=1` |
| **Async Runtime** | Tokio | 1.x | Standard Rust async |
| **Validation** | Validator | 0.19+ | Derive-macro based struct validation |
| **Logging** | Tracing + tracing-subscriber | — | Structured logging, JSON output |
| **File Upload** | Axum multipart | — | For attachments (item images, document scans, etc.) |
| **Excel** | calamine (read) + rust_xlsxwriter (write) | — | Excel import/export |
| **API Docs** | utoipa + utoipa-swagger-ui | — | OpenAPI 3.0 auto-generated docs |

### 2.2 Frontend

| Layer | Choice | Version | Why |
| ------- | -------- | --------- | ----- |
| **UI Framework** | React | 19.x | Latest stable, mature ecosystem |
| **Build Tool** | Vite | 6.x | Fast dev server, ESBuild |
| **Component Lib** | Ant Design | 5.x | Enterprise-grade, great tables/forms |
| **Routing** | React Router | 7.x | Nested routes, lazy loading |
| **Server State** | TanStack Query | 5.x | Caching, stale-while-revalidate, optimistic updates |
| **Client State** | Zustand | 5.x | Lightweight, no boilerplate, localStorage persistence |
| **HTTP Client** | Axios | 1.x | Interceptors for auth + refresh |
| **i18n** | react-i18next | 15.x | Namespace support, lazy loading |
| **Type Safety** | TypeScript 5 (strict) + Zod 3 | — | Runtime response validation |

**Feature modules**: auth, items, inventory, suppliers, customers, purchases, sales, workflow, hr, finance, procurement, manufacturing, projects, assets, notifications, portal, contracts, reports, search, profile. Each feature has `api/`, `pages/`, `types/` subdirectories plus `queryKeys.ts`.

### 2.3 Architecture Style

**RESTful frontend-backend separation**: Backend serves JSON over HTTP, frontend is a SPA.

**Monolithic backend** (not microservices): Given the project's size and team, a modular monolith organized by domain makes way more sense than splitting into a dozen microservices.

---

## 3. System Architecture

### 3.1 C4 Container Diagram

```
┌────────────────────────────────────────────────────────────┐
│                       User Client                            │
│  ┌─────────────────────────────────────────────────────┐   │
│  │                 React SPA (Browser)                    │   │
│  │  Routing: React Router  │  State: Zustand + TanStack    │   │
│  │  UI: Ant Design 5    │  HTTP: Axios (baseURL=/api)  │   │
│  └────────────────────┬────────────────────────────────┘   │
└───────────────────────┼────────────────────────────────────┘
                        │ HTTP/JSON (REST)
                        │ JWT Bearer Token
                        ▼
┌────────────────────────────────────────────────────────────┐
│            Rust Backend Service (Axum, erp-server)           │
│                                                             │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐          │
│  │Middleware│ │ Handler │ │ Service │ │  Data   │          │
│  │ Layer    │ │ Layer   │ │ Layer   │ │ Layer   │          │
│  │ Auth     │ │(Routing)│ │(Business│ │(SQLx)   │          │
│  │ Logging  │ │         │ │ Logic)  │ │         │          │
│  │ CORS     │ │         │ │         │ │         │          │
│  └─────────┘ └─────────┘ └─────────┘ └────┬────┘          │
│                                           │                │
│                                  ┌────────▼────────┐       │
│                                  │  SQLite3 (WAL)   │       │
│                                  │  data/erp.db     │       │
│                                  └─────────────────┘       │
└────────────────────────────────────────────────────────────┘
```

### 3.2 Layered Architecture

```
┌──────────────────────────────────────────────────────┐
│                    Handler Layer                       │
│  (HTTP Routing + Request Parsing + Response JSON + Auth Checks) │
│  Job: Parse params, call service, return JSON                    │
├──────────────────────────────────────────────────────┤
│                    Service Layer                       │
│  (Business Logic, unit struct + static methods)        │
│  Job: CRUD orchestration, transactions, permission checks, logging │
├──────────────────────────────────────────────────────┤
│                    Repository Layer                    │
│  (Data Access + SQL, static methods)                   │
│  Job: Execute SQLx queries, map rows, paginate, sort   │
├──────────────────────────────────────────────────────┤
│                    Domain Layer                        │
│  (Data Models + Types)                                 │
│  Job: Struct definitions, enums, business constants    │
└──────────────────────────────────────────────────────┘
```

> **Dependency injection**: `Extension<SqlitePool>` / `Extension<JwtSecret>` — no global state (no `State<AppState>`).

### 3.3 Module Dependencies

```
                    ┌──────────────┐
                    │ System Admin │
                    │ (Users/Roles/│
                    │ Departments) │
                    └──────┬───────┘
                           │ depends on
        ┌──────────────────┼──────────────────┐
        ▼                  ▼                  ▼
  ┌──────────┐      ┌──────────┐      ┌──────────────┐
  │ Item     │      │Purchase/ │      │ Manufacturing│
  │ Mgmt     │      │ Sales    │      │ (BOM/工单/   │
  │ Module   │      │ Module   │      │  质检/NCR)   │
  └─────┬────┘      └─────┬────┘      └──────┬───────┘
        │                 │                  │
        ▼                 ▼                  ▼
  ┌──────────────────────────────────────────────┐
  │          Inventory Mgmt Module                │
  │  (Inbound/Outbound/Check/Location/Reserve/    │
  │   Real-time Stock Query)                      │
  └──────────┬───────────────────────────────────┘
             │
             ▼
  ┌──────────────────────────────────────────────┐
  │  Workflow Module + History Traceability       │
  │  (Operation Logs)                             │
  └──────────────────────────────────────────────┘
```

---

## 4. Module Design

### 4.1 Module Overview

| Module | Priority | Core Stuff | Testable Independently? |
| -------- | ---------- | ------------ | ------------------------ |
| **Item Management** | P0 | Item+SKU CRUD, search, archive | Yes |
| **Inventory Management** | P0 | Inbound/outbound/check/location/reservation/stock query | Yes |
| **Manufacturing & Inspection** | P1 | BOMs, work orders, inspections, NCRs | Yes |
| **Procurement Management** | P1 | Suppliers, POs, receipts, supplier quotes, scorecards | Yes |
| **Sales Management** | P1 | Customers, SOs, shipments, customer quotes, credit, ATP | Yes |
| **Contract Management** | P2 | Purchase/sales contracts | Yes |
| **Workflow** | P1 | Definitions/instances/tasks | Yes |
| **HR** | P1 | Employees/attendance/salaries/labor contracts | Yes |
| **Finance** | P1 | Accounts/journal/invoices/payments/trial balance | Yes |
| **Projects & Assets** | P2 | Projects/WBS/budget; registration/depreciation/disposal | Yes |
| **Notifications & Portal** | P2 | Inbox/templates/preferences; portal accounts | Yes |
| **Data Import/Export** | P1 | Excel/CSV import/export | Yes |
| **Search & Filtering** | P0 | Multi-dimensional item search (shared) | -- |
| **BI Analytics** | P2 | Sales trend/inventory value/finance summary/supplier performance | Yes |
| **History Traceability** | P0 | Full lifecycle operation logs | Yes |
| **System Management** | P1 | Users, RBAC, audit logs | Yes |

### 4.2 Item Management Module

**Item+SKU management (generic item master data)**

```
┌─────────────────────────────────────────────────────┐
│               Item Management Module (ItemModule)     │
│                                                      │
│  ┌────────────────────┐                              │
│  │    ItemHandler      │                              │
│  │  (Item CRUD)        │                              │
│  └────────┬───────────┘                              │
│           │                                          │
│  ┌────────▼──────────────────┐                       │
│  │         ItemService        │                       │
│  │  (generic queries, SKU    │                       │
│  │   uniqueness)              │                       │
│  └────────┬──────────────────┘                       │
│           │                                          │
│  ┌────────▼──────────┐                               │
│  │    ItemRepo        │                               │
│  │  (Item data access)│                               │
│  └───────────────────┘                               │
└─────────────────────────────────────────────────────┘
```

**Core Interfaces:**

| Interface | Description |
| ----------- | ------------- |
| `create_item(dto)` | Create an item record |
| `update_item(id, dto)` | Update item info |
| `delete_item(id)` | Soft delete (checks inventory references first) |
| `get_item(id)` | Get single item details |
| `list_items(filters)` | Query items with filters + pagination |
| `generate_sku(category)` | Auto-generate SKU (`{category-code}-{yyyymm}-{seq}`) |
| `validate_sku_unique(sku)` | Check SKU global uniqueness |
| `search_items(filters)` | Multi-dimensional combined search |

### 4.3 Inventory Management Module

```
┌──────────────────────────────────────────────────────┐
│              Inventory Management Module              │
│              (InventoryModule)                        │
│                                                        │
│  ┌──────────┐ ┌──────────┐ ┌────────┐ ┌──────────┐   │
│  │ Inbound  │ │ Outbound │ │ Stock  │ │ Location │   │
│  │ Handler  │ │ Handler  │ │ Query  │ │ Handler  │   │
│  └─────┬────┘ └─────┬────┘ └───┬────┘ └─────┬────┘   │
│        │            │         │           │           │
│  ┌─────▼────────────▼─────────▼───────────▼─────┐   │
│  │           InventoryService                    │   │
│  │    (Inventory changes + audit trail +         │   │
│  │     validation + ATP reservations)            │   │
│  └─────┬────────────┬────────────────┬──────────┘   │
│        │            │                │               │
│  ┌─────▼────┐ ┌─────▼──────┐ ┌──────▼───────┐      │
│  │ Inbound  │ │ Outbound   │ │ Reservation  │      │
│  │ Repo     │ │ Repo       │ │ / Location   │      │
│  └──────────┘ └────────────┘ └──────────────┘      │
└──────────────────────────────────────────────────────┘
```

**Core Interfaces:**

| Interface | Description |
| ----------- | ------------- |
| `create_inbound(dto)` | Create inbound, auto-update stock. **Constraint**: when type='purchase', order_id required + PO must be approved. Production/return types start as `pending` and need supervisor approval. |
| `approve_inbound(id)` | Approve non-purchase inbound, applies stock changes |
| `reject_inbound(id, reason)` | Reject non-purchase inbound |
| `create_outbound(dto)` | Create outbound, deduct stock. Sales type auto-approved. Transfer/scrapped need approval. |
| `approve_outbound(id)` | Approve non-sales outbound, deducts stock |
| `reject_outbound(id, reason)` | Reject non-sales outbound |
| `get_stock_status(item_id)` | Check stock for a single item |
| `list_inventory(filters)` | Real-time stock query (aggregated) |
| `list_inventory_logs(filters)` | Transaction log |
| `create_inventory_check(dto)` | Create a stock check |
| `submit_check_result(dto)` | Submit results, generate variance report |
| `create_reservation(dto)` | Reserve stock for a sales order / work order |
| `get_atp(item_id)` | Available-to-promise: in stock − reserved − in-transit |
| `create_location(dto)` | Create storage location |
| `assign_item_to_location(item_id, location_id)` | Bind item to location |
| `transfer_location(item_id, new_location_id)` | Move item between locations |

### 4.4 Manufacturing & Inspection Module

**Core Interfaces:**

| Interface | Description |
| ----------- | ------------- |
| `create_bom(dto)` | Create a BOM (product item + component list) |
| `update_bom(id, dto)` | Update a BOM |
| `create_work_order(dto)` | Release a work order from a BOM |
| `complete_work_order(id)` | Complete a work order (consumes items per BOM) |
| `create_inspection(dto)` | Create an inspection record (linked to a work order) |
| `update_inspection(id, dto)` | Update an inspection record |
| `list_inspections(filters)` | List inspections with filters |
| `create_ncr(dto)` | Create an NCR when inspection fails |
| `close_ncr(id)` | Close an NCR (corrective loop) |

### 4.5 Procurement Management Module (ProcurementModule)

**Dependencies**: Inventory (receipt linkage), Item Management (spec references)
**Depended by**: Inventory references POs when creating purchase inbound

```
┌─────────────────────────────────────────────┐
│        Procurement Management Module         │
│           (ProcurementModule)                │
│                                              │
│  ┌────────────────┐ ┌─────────────────────┐ │
│  │ SupplierHandler │ │ PurchaseOrderHandler │ │
│  │ (Supplier CRUD) │ │ (PO CRUD + Approval) │ │
│  └───────┬────────┘ └──────────┬──────────┘ │
│          │                     │            │
│  ┌───────▼─────────────────────▼──────────┐ │
│  │          PurchaseService                │ │
│  │  (Supplier mgmt + PO + receipt linkage) │ │
│  └───────┬─────────────────────┬──────────┘ │
│          │                     │            │
│  ┌───────▼────────┐ ┌──────────▼──────────┐ │
│  │ SupplierRepo    │ │ PurchaseOrderRepo   │ │
│  └────────────────┘ └─────────────────────┘ │
└─────────────────────────────────────────────┘
```

**Core Interfaces:**

| Interface | Description |
| ----------- | ------------- |
| `create_supplier(dto)` | Create supplier |
| `update_supplier(id, dto)` | Update supplier |
| `delete_supplier(id)` | Delete supplier |
| `list_suppliers(filter)` | List suppliers |
| `create_purchase_order(dto)` | Create PO (with line items) |
| `approve_purchase_order(id)` | Approve PO (draft → pending → approved) |
| `reject_purchase_order(id, reason)` | Reject PO |
| `link_receipt_to_po(receipt_id, po_id)` | Link receipt to PO, update received qty |
| `create_requisition(dto)` | Create a purchase requisition |
| `create_supplier_quote(dto)` | Record a supplier quote |
| `create_scorecard(dto)` | Supplier scorecard |

### 4.6 Sales Management Module (SalesModule)

**Dependencies**: Inventory (outbound linkage + ATP), Item Management (specs)
**Depended by**: Inventory references SOs when creating sales outbound

```
┌──────────────────────────────────────────────┐
│          Sales Management Module              │
│              (SalesModule)                    │
│                                               │
│  ┌────────────────┐ ┌──────────────────────┐  │
│  │ CustomerHandler │ │ SalesOrderHandler     │  │
│  │ (Customer CRUD) │ │ (SO CRUD + Approval)  │  │
│  └───────┬────────┘ └──────────┬───────────┘  │
│          │                     │              │
│  ┌───────▼─────────────────────▼────────────┐  │
│  │           SalesService                    │  │
│  │  (Customer mgmt + SO + shipment linkage   │  │
│   │   + ATP)                                  │  │
│  └───────┬─────────────────────┬────────────┘  │
│          │                     │               │
│  ┌───────▼────────┐ ┌──────────▼───────────┐  │
│  │ CustomerRepo    │ │ SalesOrderRepo        │  │
│  └────────────────┘ └──────────────────────┘  │
└──────────────────────────────────────────────┘
```

**Core Interfaces:**

| Interface | Description |
| ----------- | ------------- |
| `create_customer(dto)` | Create customer |
| `update_customer(id, dto)` | Update customer |
| `delete_customer(id)` | Delete customer |
| `list_customers(filter)` | List customers |
| `create_sales_order(dto)` | Create SO (with line items) |
| `approve_sales_order(id)` | Approve SO |
| `reject_sales_order(id, reason)` | Reject SO |
| `link_shipment_to_so(shipment_id, so_id)` | Link shipment to SO, update delivered qty |
| `create_customer_quote(dto)` | Create a customer quote |
| `get_atp(item_id)` | Available-to-promise: in stock − reserved |
| `update_customer_credit(id, dto)` | Customer credit management |

### 4.7 Workflow Module

**Core Interfaces:**

| Interface | Description |
| ----------- | ------------- |
| `create_definition(dto)` | Create a workflow definition (nodes/conditions) |
| `list_definitions(filter)` | List workflow definitions |
| `start_instance(definition_id, business_type, business_id)` | Start a workflow instance |
| `list_instances(filter)` | List workflow instances |
| `get_pending_tasks(user_id)` | Current user's pending approval tasks |
| `approve_task(task_id, comment)` | Approve (advance to next node or complete) |
| `reject_task(task_id, comment)` | Reject |

### 4.8 HR Module

**Core Interfaces:**

| Interface | Description |
| ----------- | ------------- |
| `create_employee(dto)` | Create an employee profile |
| `list_employees(filter)` | List employees |
| `create_department(dto)` | Create a department |
| `create_attendance(dto)` | Record attendance |
| `create_salary(dto)` | Record a salary payment |
| `create_labor_contract(dto)` | Create a labor contract |

### 4.9 Finance Module

**Core Interfaces:**

| Interface | Description |
| ----------- | ------------- |
| `create_account(dto)` | Create a chart-of-accounts entry |
| `create_journal_entry(dto)` | Create a journal entry |
| `create_invoice(dto)` | Create an invoice (issued/received) |
| `create_payment(dto)` | Create a payment record |
| `get_trial_balance()` | Trial balance report |

### 4.10 Project & Asset Modules

**Core Interfaces (Project):**

| Interface | Description |
| ----------- | ------------- |
| `create_project(dto)` | Create a project |
| `create_wbs(project_id, dto)` | Create a WBS node |
| `create_budget_item(project_id, dto)` | Create a budget item |

**Core Interfaces (Asset):**

| Interface | Description |
| ----------- | ------------- |
| `register_asset(dto)` | Register a fixed asset |
| `compute_depreciation(asset_id)` | Straight-line depreciation calculation |
| `dispose_asset(asset_id, dto)` | Dispose of an asset |

### 4.11 Notification & Portal Modules

**Core Interfaces (Notification):**

| Interface | Description |
| ----------- | ------------- |
| `list_inbox(user_id)` | Notification inbox |
| `mark_read(notification_id)` | Mark as read |
| `create_template(dto)` | Create a notification template |
| `update_preference(user_id, dto)` | Update notification preferences |

**Core Interfaces (Portal):**

| Interface | Description |
|-----------|-------------|
| `create_portal_account(dto)` | Create a portal account (linked to a customer/supplier) |
| `party_login(credentials)` | Portal login (issues a party JWT) |

### 4.12 BI Module

**Core Interfaces:**

| Interface | Description |
| ----------- | ------------- |
| `sales_trend(period)` | Sales trend statistics |
| `inventory_value()` | Inventory value statistics |
| `finance_summary(period)` | Finance summary |
| `supplier_performance(period)` | Supplier performance statistics |

### 4.13 System Management Module

**Core Interfaces:**

| Interface | Description |
| ----------- | ------------- |
| `create_user(dto)` | Create user (admin only) |
| `update_user(id, dto)` | Update user |
| `list_users(filters)` | List users |
| `assign_role(user_id, role)` | Assign role |
| `create_role(dto)` / `create_permission(dto)` / `create_department(dto)` | RBAC management |
| `login(credentials)` | Login, returns JWT |
| `refresh_token(token)` | Refresh JWT |
| `get_current_user()` | Get current user info |
| `list_operation_logs(filters)` | Query operation logs |

---

## 5. Database Detailed Design

### 5.1 Design Principles

- **SQLite3 WAL mode**: `PRAGMA journal_mode=WAL;` — concurrent reads without blocking; connection string `sqlite://data/erp.db?mode=rwc`
- **No FK constraints**: SQLite FK enforcement has perf overhead; we enforce referential integrity in application code
- **Indexes**: Index frequently queried fields, composite indexes for combined queries
- **Timestamps**: ISO 8601 text format (`TEXT`) everywhere
- **Enums**: Stored as `TEXT` — readable and extensible
- **Soft deletes**: Key tables have `deleted_at` field; no physical deletion
- **Migration strategy**: the 37 legacy migration files are rewritten to SQLite syntax, dropping the pipe/threading/labels/quality-cert tables and adding the generic item table plus the new business module tables

### 5.2 Database Initialization Config

```sql
-- Enable WAL mode
PRAGMA journal_mode = WAL;
-- Enable foreign keys (application-controlled, but validation is on)
PRAGMA foreign_keys = ON;
-- Set busy timeout
PRAGMA busy_timeout = 5000;
-- Sync mode: NORMAL balances perf and safety
PRAGMA synchronous = NORMAL;
-- Cache size: 64MB
PRAGMA cache_size = -64000;
-- Temp storage: memory
PRAGMA temp_store = MEMORY;
```

### 5.3 Table Structures

---

#### 5.3.1 items — Item Table (Item + SKU)

**Table**: `items`

**Purpose**: The single business entity of the whole system. Generic item master data replacing the legacy pipe tables (no industry-specific fields).

| Field | Type | Constraint | Description |
|-------|------|------------|-------------|
| `id` | INTEGER | PK, AUTOINCREMENT | Primary key |
| `sku` | TEXT | NOT NULL, UNIQUE | Unique business code (system-assigned or manual) |
| `name` | TEXT | NOT NULL | Item name |
| `category` | TEXT | NOT NULL | Category (raw material / semi-finished / finished goods / spare parts, etc.) |
| `unit` | TEXT | NOT NULL, DEFAULT 'pc' | Unit (kg / m / pc / piece, etc.) |
| `spec` | TEXT | -- | Spec (descriptive free text, optional) |
| `status` | TEXT | NOT NULL, DEFAULT 'draft' | draft / active / disabled |
| `notes` | TEXT | -- | Notes |
| `created_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | Created |
| `updated_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | Updated |
| `deleted_at` | TEXT | -- | Soft delete timestamp |

**Indexes:**

```sql
CREATE UNIQUE INDEX idx_items_sku ON items(sku);
CREATE INDEX idx_items_category_status ON items(category, status);
CREATE INDEX idx_items_name ON items(name);
```

---

#### 5.3.2 warehouses / locations — Warehouse & Location Tables

**Table**: `warehouses`

| Field | Type | Constraint | Description |
|-------|------|------------|-------------|
| `id` | INTEGER | PK, AUTOINCREMENT | Primary key |
| `code` | TEXT | NOT NULL, UNIQUE | Warehouse code |
| `name` | TEXT | NOT NULL | Warehouse name |
| `is_active` | INTEGER | NOT NULL, DEFAULT 1 | Enabled |
| `created_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | -- |

**Table**: `locations`

**Purpose**: Locations are the smallest unit inventory belongs to.

| Field | Type | Constraint | Description |
|-------|------|------------|-------------|
| `id` | INTEGER | PK, AUTOINCREMENT | Primary key |
| `warehouse_id` | INTEGER | NOT NULL | Parent warehouse |
| `code` | TEXT | NOT NULL | Location code |
| `full_code` | TEXT | NOT NULL, UNIQUE | Full code (warehouse_code + '-' + code) |
| `description` | TEXT | -- | Description |
| `max_capacity` | INTEGER | -- | Max items this location can hold |
| `current_usage` | INTEGER | NOT NULL, DEFAULT 0 | Current count |
| `is_active` | INTEGER | NOT NULL, DEFAULT 1 | Enabled? |
| `created_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | -- |
| `updated_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | -- |

**Indexes:**

```sql
CREATE UNIQUE INDEX idx_locations_full_code ON locations(full_code);
CREATE INDEX idx_locations_warehouse ON locations(warehouse_id);
```

---

#### 5.3.3 inbound_records — Inbound Record Header

**Table**: `inbound_records`

**Purpose**: Header for inbound orders. One header + N line items (`inbound_items`).

| Field | Type | Constraint | Description |
|-------|------|------------|-------------|
| `id` | INTEGER | PK, AUTOINCREMENT | Primary key |
| `record_no` | TEXT | NOT NULL, UNIQUE | Inbound record number |
| `inbound_type` | TEXT | NOT NULL, CHECK IN ('purchase','production','return') | Type |
| `inbound_date` | TEXT | NOT NULL | Inbound date |
| `order_id` | INTEGER | -- | Associated PO ID (required for purchase type, PO must be approved) |
| `supplier_id` | INTEGER | -- | Supplier ID |
| `operator_id` | INTEGER | -- | Operator user ID |
| `approval_status` | TEXT | NOT NULL, DEFAULT 'auto_approved' | auto_approved / pending / approved / rejected |
| `remark` | TEXT | -- | Notes |
| `created_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | -- |

**Indexes:**

```sql
CREATE INDEX idx_inbound_records_no ON inbound_records(record_no);
CREATE INDEX idx_inbound_records_date ON inbound_records(inbound_date);
CREATE INDEX idx_inbound_records_order ON inbound_records(order_id);
CREATE INDEX idx_inbound_records_supplier ON inbound_records(supplier_id);
```

---

#### 5.3.3a inbound_items — Inbound Line Items

**Table**: `inbound_items`

**Purpose**: Line items for inbound. Supports batch inbound (N items of the same spec) and individual inbound.

| Field | Type | Constraint | Description |
|-------|------|------------|-------------|
| `id` | INTEGER | PK, AUTOINCREMENT | Primary key |
| `inbound_id` | INTEGER | NOT NULL | Associated header ID |
| `item_id` | INTEGER | NOT NULL | Item ID (auto-created or selected on inbound) |
| `quantity` | INTEGER | NOT NULL | Quantity |
| `created_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | -- |

**Indexes:**

```sql
CREATE INDEX idx_inbound_items_inbound ON inbound_items(inbound_id);
CREATE INDEX idx_inbound_items_item ON inbound_items(item_id);
```

---

#### 5.3.4 outbound_records — Outbound Record Header

**Table**: `outbound_records`

**Purpose**: Mirror of inbound_records for outbound.

| Field | Type | Constraint | Description |
|-------|------|------------|-------------|
| `id` | INTEGER | PK, AUTOINCREMENT | Primary key |
| `record_no` | TEXT | NOT NULL, UNIQUE | Outbound record number |
| `outbound_type` | TEXT | NOT NULL, CHECK IN ('sales','transfer','scrapped') | Type |
| `outbound_date` | TEXT | NOT NULL | Outbound date |
| `order_id` | INTEGER | -- | Associated SO ID (required for sales type, SO must be approved) |
| `customer_id` | INTEGER | -- | Customer ID |
| `operator_id` | INTEGER | -- | Operator user ID |
| `approval_status` | TEXT | NOT NULL, DEFAULT 'auto_approved' | auto_approved / pending / approved / rejected |
| `remark` | TEXT | -- | Notes |
| `created_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | -- |

**Indexes:**

```sql
CREATE INDEX idx_outbound_records_no ON outbound_records(record_no);
CREATE INDEX idx_outbound_records_date ON outbound_records(outbound_date);
CREATE INDEX idx_outbound_records_order ON outbound_records(order_id);
CREATE INDEX idx_outbound_records_customer ON outbound_records(customer_id);
```

---

#### 5.3.4a outbound_items — Outbound Line Items

**Table**: `outbound_items`

**Purpose**: Line items for outbound.

| Field | Type | Constraint | Description |
|-------|------|------------|-------------|
| `id` | INTEGER | PK, AUTOINCREMENT | Primary key |
| `outbound_id` | INTEGER | NOT NULL | Associated header ID |
| `item_id` | INTEGER | NOT NULL | Item ID |
| `quantity` | INTEGER | NOT NULL | Quantity |
| `created_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | -- |

**Indexes:**

```sql
CREATE INDEX idx_outbound_items_outbound ON outbound_items(outbound_id);
CREATE INDEX idx_outbound_items_item ON outbound_items(item_id);
```

---

#### 5.3.5 inventory_logs — Inventory Change Log

**Table**: `inventory_logs`

**Purpose**: Every inventory change for every item. The foundation of lifecycle traceability.

| Field | Type | Constraint | Description |
|-------|------|------------|-------------|
| `id` | INTEGER | PK, AUTOINCREMENT | Primary key |
| `item_id` | INTEGER | NOT NULL | Item ID |
| `change_type` | TEXT | NOT NULL | inbound / outbound / transfer / check_adjust |
| `reference_id` | INTEGER | -- | Associated doc ID |
| `reference_no` | TEXT | -- | Associated doc number |
| `operator_id` | INTEGER | -- | Operator ID |
| `operator_name` | TEXT | -- | Operator name (denormalized) |
| `remark` | TEXT | -- | Description |
| `created_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | Timestamp |

**Indexes:**

```sql
CREATE INDEX idx_inventory_logs_item ON inventory_logs(item_id);
CREATE INDEX idx_inventory_logs_type ON inventory_logs(change_type);
CREATE INDEX idx_inventory_logs_time ON inventory_logs(created_at);
CREATE INDEX idx_inventory_logs_operator ON inventory_logs(operator_id);
```

---

#### 5.3.6 reservations — Inventory Reservation Table

**Table**: `reservations`

**Purpose**: Reserve available stock for sales orders or work orders (ATP allocation view).

| Field | Type | Constraint | Description |
|-------|------|------------|-------------|
| `id` | INTEGER | PK, AUTOINCREMENT | Primary key |
| `item_id` | INTEGER | NOT NULL | Item ID |
| `source_type` | TEXT | NOT NULL | sales_order / work_order |
| `source_id` | INTEGER | NOT NULL | Source doc ID |
| `quantity` | INTEGER | NOT NULL | Reserved quantity |
| `status` | TEXT | NOT NULL, DEFAULT 'active' | active / released / consumed |
| `created_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | -- |

---

#### 5.3.7 suppliers — Supplier Table

**Table**: `suppliers`

| Field | Type | Constraint | Description |
|-------|------|------------|-------------|
| `id` | INTEGER | PK, AUTOINCREMENT | Primary key |
| `code` | TEXT | NOT NULL, UNIQUE | Supplier code |
| `name` | TEXT | NOT NULL | Name |
| `contact_person` | TEXT | -- | Contact |
| `phone` | TEXT | -- | Phone |
| `email` | TEXT | -- | Email |
| `address` | TEXT | -- | Address |
| `qualification` | TEXT | -- | Qualification info |
| `score` | REAL | -- | Latest supplier score |
| `is_active` | INTEGER | NOT NULL, DEFAULT 1 | Enabled |
| `created_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | -- |
| `updated_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | -- |

**Indexes:**

```sql
CREATE UNIQUE INDEX idx_suppliers_code ON suppliers(code);
CREATE INDEX idx_suppliers_name ON suppliers(name);
```

---

#### 5.3.8 customers — Customer Table

**Table**: `customers`

| Field | Type | Constraint | Description |
|-------|------|------------|-------------|
| `id` | INTEGER | PK, AUTOINCREMENT | Primary key |
| `code` | TEXT | NOT NULL, UNIQUE | Customer code |
| `name` | TEXT | NOT NULL | Name |
| `contact_person` | TEXT | -- | Contact |
| `phone` | TEXT | -- | Phone |
| `email` | TEXT | -- | Email |
| `address` | TEXT | -- | Address |
| `credit_limit` | REAL | -- | Customer credit limit |
| `is_active` | INTEGER | NOT NULL, DEFAULT 1 | Enabled |
| `created_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | -- |
| `updated_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | -- |

---

#### 5.3.9 purchase_orders — Purchase Order Table

**Table**: `purchase_orders`

| Field | Type | Constraint | Description |
|-------|------|------------|-------------|
| `id` | INTEGER | PK, AUTOINCREMENT | Primary key |
| `order_no` | TEXT | NOT NULL, UNIQUE | PO number |
| `supplier_id` | INTEGER | NOT NULL | Supplier |
| `order_date` | TEXT | NOT NULL | Order date |
| `expected_date` | TEXT | -- | Expected delivery |
| `status` | TEXT | NOT NULL, DEFAULT 'draft' | draft / pending / approved / completed / cancelled |
| `total_amount` | REAL | -- | Total |
| `currency` | TEXT | NOT NULL, DEFAULT 'CNY' | Currency |
| `contract_id` | INTEGER | -- | Associated purchase contract |
| `notes` | TEXT | -- | Notes |
| `created_by` | INTEGER | -- | Creator |
| `created_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | -- |
| `updated_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | -- |

**Indexes:**

```sql
CREATE INDEX idx_purchase_orders_supplier ON purchase_orders(supplier_id);
CREATE INDEX idx_purchase_orders_status ON purchase_orders(status);
CREATE INDEX idx_purchase_orders_date ON purchase_orders(order_date);
```

---

#### 5.3.10 purchase_order_items — PO Line Items

**Table**: `purchase_order_items`

| Field | Type | Constraint | Description |
|-------|------|------------|-------------|
| `id` | INTEGER | PK, AUTOINCREMENT | Primary key |
| `order_id` | INTEGER | NOT NULL | PO ID |
| `item_id` | INTEGER | NOT NULL | Item ID |
| `quantity` | INTEGER | NOT NULL | Ordered qty |
| `received_quantity` | INTEGER | NOT NULL, DEFAULT 0 | Received qty |
| `unit_price` | REAL | -- | Unit price |
| `notes` | TEXT | -- | -- |

**Indexes:**

```sql
CREATE INDEX idx_poi_order ON purchase_order_items(order_id);
```

---

#### 5.3.11 sales_orders — Sales Order Table

Symmetric to purchase_orders, but links to `customer_id` instead.

**Table**: `sales_orders`

| Field | Type | Constraint | Description |
|-------|------|------------|-------------|
| `id` | INTEGER | PK, AUTOINCREMENT | Primary key |
| `order_no` | TEXT | NOT NULL, UNIQUE | SO number |
| `customer_id` | INTEGER | NOT NULL | Customer |
| `order_date` | TEXT | NOT NULL | Order date |
| `status` | TEXT | NOT NULL, DEFAULT 'draft' | draft / pending / approved / completed / cancelled |
| `total_amount` | REAL | -- | Total |
| `currency` | TEXT | NOT NULL, DEFAULT 'CNY' | Currency |
| `contract_id` | INTEGER | -- | Associated sales contract |
| `notes` | TEXT | -- | Notes |
| `created_by` | INTEGER | -- | -- |
| `created_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | -- |
| `updated_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | -- |

**Indexes:**

```sql
CREATE INDEX idx_sales_orders_customer ON sales_orders(customer_id);
CREATE INDEX idx_sales_orders_status ON sales_orders(status);
CREATE INDEX idx_sales_orders_date ON sales_orders(order_date);
```

#### 5.3.12 sales_order_items — SO Line Items

Symmetric to `purchase_order_items`.

| Field | Type | Constraint | Description |
|-------|------|------------|-------------|
| `id` | INTEGER | PK, AUTOINCREMENT | Primary key |
| `order_id` | INTEGER | NOT NULL | SO ID |
| `item_id` | INTEGER | NOT NULL | Item ID |
| `quantity` | INTEGER | NOT NULL | Ordered qty |
| `delivered_quantity` | INTEGER | NOT NULL, DEFAULT 0 | Delivered qty |
| `unit_price` | REAL | -- | Unit price |
| `notes` | TEXT | -- | -- |

---

#### 5.3.13 contracts — Contract Table

**Table**: `contracts`

| Field | Type | Constraint | Description |
|-------|------|------------|-------------|
| `id` | INTEGER | PK, AUTOINCREMENT | Primary key |
| `contract_no` | TEXT | NOT NULL, UNIQUE | Contract number |
| `contract_type` | TEXT | NOT NULL | purchase / sales |
| `party_a_id` | INTEGER | -- | Party A (us) |
| `party_b_id` | INTEGER | -- | Party B (supplier or customer) |
| `sign_date` | TEXT | -- | Signing date |
| `total_amount` | REAL | -- | Amount |
| `status` | TEXT | NOT NULL, DEFAULT 'active' | active / completed / terminated |
| `file_url` | TEXT | -- | Contract scan file path |
| `notes` | TEXT | -- | -- |
| `created_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | -- |
| `updated_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | -- |

---

#### 5.3.14 inspections / ncrs — Inspection & NCR Tables

**Table**: `inspections`

**Purpose**: Manufacturing inspection records (Inspection), linked to work orders. Replaces the legacy quality-cert module.

| Field | Type | Constraint | Description |
|-------|------|------------|-------------|
| `id` | INTEGER | PK, AUTOINCREMENT | Primary key |
| `work_order_id` | INTEGER | NOT NULL | Associated work order |
| `inspection_no` | TEXT | NOT NULL, UNIQUE | Inspection number |
| `inspect_date` | TEXT | -- | Inspection date |
| `inspector` | TEXT | -- | Inspector |
| `agency` | TEXT | -- | Inspection agency |
| `result` | TEXT | NOT NULL | pass / fail / pending |
| `test_items` | TEXT | -- | Test items (JSON array) |
| `file_url` | TEXT | -- | Report file path |
| `notes` | TEXT | -- | -- |
| `created_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | -- |
| `updated_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | -- |

**Indexes:**

```sql
CREATE INDEX idx_inspections_work_order ON inspections(work_order_id);
CREATE INDEX idx_inspections_no ON inspections(inspection_no);
```

**Table**: `ncrs`

| Field | Type | Constraint | Description |
|-------|------|------------|-------------|
| `id` | INTEGER | PK, AUTOINCREMENT | Primary key |
| `inspection_id` | INTEGER | NOT NULL | Associated inspection |
| `ncr_no` | TEXT | NOT NULL, UNIQUE | NCR number |
| `description` | TEXT | NOT NULL | Problem description |
| `corrective_action` | TEXT | -- | Corrective action |
| `status` | TEXT | NOT NULL, DEFAULT 'open' | open / closed |
| `created_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | -- |
| `updated_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | -- |

---

#### 5.3.15 workflow_definitions / instances / tasks — Workflow Tables

**Table**: `workflow_definitions`

| Field | Type | Constraint | Description |
|-------|------|------------|-------------|
| `id` | INTEGER | PK, AUTOINCREMENT | Primary key |
| `name` | TEXT | NOT NULL | Definition name |
| `business_type` | TEXT | NOT NULL | Applicable doc type (purchase_order / sales_order / requisition, etc.) |
| `nodes` | TEXT | NOT NULL | Node config (JSON) |
| `conditions` | TEXT | -- | Condition config (JSON) |
| `is_active` | INTEGER | NOT NULL, DEFAULT 1 | Enabled |
| `created_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | -- |

**Table**: `workflow_instances`

| Field | Type | Constraint | Description |
|-------|------|------------|-------------|
| `id` | INTEGER | PK, AUTOINCREMENT | Primary key |
| `definition_id` | INTEGER | NOT NULL | Workflow definition |
| `business_type` | TEXT | NOT NULL | Doc type |
| `business_id` | INTEGER | NOT NULL | Doc ID |
| `status` | TEXT | NOT NULL, DEFAULT 'running' | running / approved / rejected / cancelled |
| `current_node` | TEXT | -- | Current node key |
| `created_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | -- |
| `updated_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | -- |

**Table**: `workflow_tasks`

| Field | Type | Constraint | Description |
|-------|------|------------|-------------|
| `id` | INTEGER | PK, AUTOINCREMENT | Primary key |
| `instance_id` | INTEGER | NOT NULL | Workflow instance |
| `node_key` | TEXT | NOT NULL | Node key |
| `assignee_id` | INTEGER | NOT NULL | Assignee (user ID) |
| `status` | TEXT | NOT NULL, DEFAULT 'pending' | pending / approved / rejected |
| `comment` | TEXT | -- | Approval comment |
| `created_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | -- |
| `handled_at` | TEXT | -- | Handled at |

**Indexes:**

```sql
CREATE INDEX idx_wf_tasks_assignee ON workflow_tasks(assignee_id, status);
CREATE INDEX idx_wf_instances_business ON workflow_instances(business_type, business_id);
```

---

#### 5.3.16 users — User Table

**Table**: `users`

| Field | Type | Constraint | Description |
|-------|------|------------|-------------|
| `id` | INTEGER | PK, AUTOINCREMENT | Primary key |
| `username` | TEXT | NOT NULL, UNIQUE | Login username |
| `password_hash` | TEXT | NOT NULL | Argon2 hash |
| `display_name` | TEXT | NOT NULL | Display name |
| `email` | TEXT | -- | Email |
| `role` | TEXT | NOT NULL | admin / warehouse / sales / procurement |
| `department_id` | INTEGER | -- | Department |
| `language_pref` | TEXT | NOT NULL, DEFAULT 'zh' | zh / en |
| `unit_pref` | TEXT | NOT NULL, DEFAULT 'standard' | Unit preference |
| `is_active` | INTEGER | NOT NULL, DEFAULT 1 | Enabled |
| `last_login_at` | TEXT | -- | Last login |
| `created_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | -- |
| `updated_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | -- |

**Indexes:**

```sql
CREATE UNIQUE INDEX idx_users_username ON users(username);
CREATE INDEX idx_users_role ON users(role);
```

---

#### 5.3.17 operation_logs — Operation Log Table

**Table**: `operation_logs`

**Purpose**: Audit trail for all critical data changes.

| Field | Type | Constraint | Description |
|-------|------|------------|-------------|
| `id` | INTEGER | PK, AUTOINCREMENT | Primary key |
| `user_id` | INTEGER | -- | User ID |
| `username` | TEXT | -- | Username (denormalized) |
| `action` | TEXT | NOT NULL | create / update / delete / login / export |
| `target_type` | TEXT | NOT NULL | item / inbound / outbound / order / user, etc. |
| `target_id` | INTEGER | -- | Target object ID |
| `target_summary` | TEXT | -- | Summary (e.g. SKU) |
| `detail` | TEXT | -- | Change details (JSON, before/after) |
| `ip_address` | TEXT | -- | Source IP |
| `created_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | Timestamp |

**Indexes:**

```sql
CREATE INDEX idx_operation_logs_user ON operation_logs(user_id);
CREATE INDEX idx_operation_logs_target ON operation_logs(target_type, target_id);
CREATE INDEX idx_operation_logs_action ON operation_logs(action);
CREATE INDEX idx_operation_logs_time ON operation_logs(created_at);
```

---

#### 5.3.18 item_attachments — Item Attachments Table

**Table**: `item_attachments`

**Purpose**: Files/photos linked to items (photos, manuals, inspection reports). One-to-many with items.

| Field | Type | Constraint | Description |
|-------|------|------------|-------------|
| `id` | INTEGER | PK, AUTOINCREMENT | Primary key |
| `item_id` | INTEGER | NOT NULL | Item ID |
| `file_name` | TEXT | NOT NULL | Original filename |
| `file_path` | TEXT | NOT NULL | Storage path |
| `file_type` | TEXT | -- | image / pdf / other |
| `file_size` | INTEGER | -- | Size in bytes |
| `uploaded_by` | INTEGER | -- | Uploader user ID |
| `created_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | -- |

**Indexes:**

```sql
CREATE INDEX idx_item_attachments_target ON item_attachments(item_id);
```

---

#### 5.3.19 inventory_check_records / items — Stock Check Tables

**Table**: `inventory_check_records`

| Field | Type | Constraint | Description |
|-------|------|------------|-------------|
| `id` | INTEGER | PK, AUTOINCREMENT | Primary key |
| `check_no` | TEXT | NOT NULL, UNIQUE | Check number |
| `check_date` | TEXT | NOT NULL | Check date |
| `status` | TEXT | NOT NULL, DEFAULT 'in_progress' | in_progress / completed |
| `operator_id` | INTEGER | -- | Checker |
| `notes` | TEXT | -- | -- |
| `created_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | -- |
| `updated_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | -- |

**Table**: `inventory_check_items`

**Purpose**: One row per item per check. System pre-populates expected items, checker confirms each one. Each item is either "found" or "missing" — no quantity math.

| Field | Type | Constraint | Description |
|-------|------|------------|-------------|
| `id` | INTEGER | PK, AUTOINCREMENT | Primary key |
| `check_id` | INTEGER | NOT NULL | Check record ID |
| `item_id` | INTEGER | NOT NULL | Item ID |
| `expected` | INTEGER | NOT NULL, DEFAULT 1 | Expected flag (1 = should be here) |
| `found` | INTEGER | -- | NULL=not checked, 1=found, 0=missing |
| `notes` | TEXT | -- | Discrepancy description |

**Indexes:**

```sql
CREATE INDEX idx_check_items_check ON inventory_check_items(check_id);
CREATE INDEX idx_check_items_status ON inventory_check_items(check_id, found);
```

---

#### 5.3.20 Extended Module Core Tables (HR / Finance / Manufacturing / Project / Asset / Notification / Portal)

| Module | Table | Core Fields |
|--------|-------|-------------|
| HR | `employees` | id, employee_no, name, department_id, position, phone, email, hire_date |
| HR | `departments` | id, name, parent_id |
| HR | `attendance` | id, employee_id, date, status |
| HR | `salaries` | id, employee_id, period, amount, paid_at |
| HR | `labor_contracts` | id, employee_id, contract_no, start_date, end_date, status |
| Finance | `accounts` | id, code, name, type(asset/liability/equity/revenue/expense) |
| Finance | `journal_entries` | id, entry_no, entry_date, account_id, debit, credit, ref_type, ref_id |
| Finance | `invoices` | id, invoice_no, invoice_type(in/out), party_id, amount, status |
| Finance | `payments` | id, payment_no, payment_type, party_id, amount, paid_at |
| Manufacturing | `boms` | id, product_item_id, components(JSON), quantity, unit |
| Manufacturing | `work_orders` | id, work_order_no, product_item_id, bom_id, quantity, status, start_date, due_date |
| Project | `projects` | id, project_code, name, owner_id, start_date, end_date, status |
| Project | `wbs_items` | id, project_id, parent_id, name, weight |
| Project | `budget_items` | id, project_id, wbs_id, category, planned_amount, actual_amount |
| Asset | `fixed_assets` | id, asset_code, name, category, purchase_date, original_value, useful_life, salvage_value |
| Asset | `depreciations` | id, asset_id, period, amount, cumulative_amount |
| Asset | `disposals` | id, asset_id, dispose_date, dispose_type, proceeds |
| Notification | `notifications` | id, user_id, title, content, read_at, template_id |
| Notification | `notification_templates` | id, code, title, content, variables(JSON) |
| Notification | `notification_preferences` | id, user_id, category, enabled |
| Portal | `portal_accounts` | id, party_type(customer/supplier), party_id, username, password_hash, is_active |
| Procurement | `requisitions` | id, requisition_no, requester_id, status, items(JSON) |
| Procurement | `receipts` | id, receipt_no, purchase_order_id, supplier_id, status, items(JSON) |
| Procurement | `supplier_quotes` | id, quote_no, supplier_id, item_id, price, valid_until, status |
| Procurement | `supplier_scorecards` | id, supplier_id, period, score, dimension(JSON) |
| Sales CRM | `customer_quotes` | id, quote_no, customer_id, item_id, price, valid_until, status |
| Sales CRM | `shipments` | id, shipment_no, sales_order_id, customer_id, status, items(JSON) |
| Sales CRM | `customer_credit` | id, customer_id, credit_limit, used_amount, updated_at |

---

### 5.4 Entity Relationship Diagram

```
┌──────────────┐       ┌──────────────────┐
│   suppliers   │──1:N──│  purchase_orders  │
└──────────────┘       └────────┬─────────┘
                                │ 1:N
                       ┌────────▼─────────┐
                       │ purchase_order_items│
                       └──────────────────┘

┌──────────────┐       ┌──────────────────┐
│   customers   │──1:N──│   sales_orders    │
└──────────────┘       └────────┬─────────┘
                                │ 1:N
                       ┌────────▼─────────┐
                       │ sales_order_items  │
                       └──────────────────┘

┌──────────┐      ┌──────────────────┐      ┌───────────────┐
│   items   │──N:1│    locations      │1:N──│  warehouses    │
└─────┬────┘      └──────────────────┘      └───────────────┘
      │
      │ 1:N
      ▼
┌─────────────────────────────────────────┐
│            inventory_logs                 │
│  (Records all inventory changes)          │
└─────────────────────────────────────────┘

┌────────────────┐     ┌──────────────────┐     ┌─────────┐
│ inbound_records │──1:N│  inbound_items   │──N:1│  items   │
│   (Header)      │     │   (Line Items)    │     └─────────┘
└────────────────┘     └──────────────────┘

┌────────────────┐     ┌──────────────────┐     ┌─────────┐
│ outbound_records│──1:N│ outbound_items   │──N:1│  items   │
│   (Header)      │     │   (Line Items)    │     └─────────┘
└────────────────┘     └──────────────────┘

┌────────────┐     ┌──────────────────┐
│ work_orders │──1:N│   inspections     │──1:N── ncrs
└────────────┘     └──────────────────┘

┌──────────────────────┐
│ workflow_definitions │──1:N── workflow_instances ──1:N── workflow_tasks
└──────────────────────┘

┌──────────┐       ┌──────────────────┐       ┌─────────────────┐
│   users   │──1:N──│  operation_logs   │       │    contracts     │
└──────────┘       └──────────────────┘       └────────┬────────┘
                                                        │ 1:N
                                              ┌─────────┴─────────┐
                                              ▼                   ▼
                                     ┌────────────────┐   ┌────────────────┐
                                     │  purchase_orders│   │  sales_orders   │
                                     └────────────────┘   └────────────────┘
```

---

## 6. REST API Design

### 6.1 API Base Spec

**Base path**: `/api/v1`

**Response format** (ApiResponse<T> / PaginatedResponse<T>):

```json
// Success
{
  "success": true,
  "data": { ... },
  "meta": {
    "page": 1,
    "page_size": 20,
    "total": 100,
    "total_pages": 5
  },
  "request_id": "req_xxxxx"
}

// Error
{
  "success": false,
  "code": 12001,
  "message": "Item not found",
  "details": { "item_id": 123 },
  "request_id": "req_xxxxx"
}
```

**Auth**: `Authorization: Bearer <jwt_token>`

**Pagination params**:

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `page` | integer | 1 | Page number |
| `page_size` | integer | 20 | Items per page (max 100) |

### 6.2 Item Management API

#### 6.2.1 Item CRUD

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/v1/items` | List with filters |
| `POST` | `/api/v1/items` | Create |
| `GET` | `/api/v1/items/{id}` | Get details |
| `PUT` | `/api/v1/items/{id}` | Update |
| `DELETE` | `/api/v1/items/{id}` | Soft delete (checks inventory references) |
| `GET` | `/api/v1/items/search` | Multi-dimensional search |

**GET /api/v1/items query params**:

| Parameter | Type | Description |
|-----------|------|-------------|
| `q` | string | Fuzzy search (sku / name / spec) |
| `category` | string | Exact category match |
| `unit` | string | Unit match |
| `status` | string | draft / active / disabled |
| `sort_by` | string | Sort field (whitelist: `created_at`, `sku`, `name`, `category`, `status`) |
| `sort_order` | string | asc / desc |

**POST /api/v1/items body**:

```json
{
  "sku": "FG-202608-0001",
  "name": "Finished Good A",
  "category": "finished_goods",
  "unit": "pc",
  "spec": "standard",
  "status": "active",
  "notes": ""
}
```

**GET /api/v1/items/{id} response**:

```json
{
  "success": true,
  "data": {
    "id": 1,
    "sku": "FG-202608-0001",
    "name": "Finished Good A",
    "category": "finished_goods",
    "unit": "pc",
    "spec": "standard",
    "status": "active",
    "location": { "id": 1, "full_code": "WH1-A-01" },
    "created_at": "2026-08-01T10:00:00Z",
    "updated_at": "2026-08-01T10:00:00Z"
  }
}
```

#### 6.2.2 Item Search

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/v1/items/search` | Generic multi-dimensional search (SKU/name/category/unit/spec/status/location) |

```json
// Response
{
  "success": true,
  "data": { "items": [ ... ] },
  "meta": { "total": 60, "page": 1, "page_size": 20, "total_pages": 3 }
}
```

### 6.3 Inventory Management API

#### 6.3.1 Inbound Management

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/v1/inbound-records` | List (headers) |
| `POST` | `/api/v1/inbound-records` | Create (header + items). Purchase type needs approved PO. |
| `GET` | `/api/v1/inbound-records/{id}` | Details (with line items) |
| `GET` | `/api/v1/inbound-records/{id}/items` | Line items |
| `POST` | `/api/v1/inbound-records/{id}/approve` | Approve non-purchase inbound |
| `POST` | `/api/v1/inbound-records/{id}/reject` | Reject |
| `DELETE` | `/api/v1/inbound-records/{id}` | Delete (only auto_approved or rejected) |

**POST /api/v1/inbound-records — Purchase Inbound**:

```json
{
  "inbound_type": "purchase",
  "inbound_date": "2026-08-01",
  "order_id": 1,
  "supplier_id": 1,
  "operator_id": 1,
  "remark": "Purchase inbound",
  "items": [
    { "item_id": 1, "quantity": 100 },
    { "item_id": 2, "quantity": 50 }
  ]
}
```

**POST /api/v1/inbound-records — Non-Purchase (needs approval)**:

```json
{
  "inbound_type": "production",
  "inbound_date": "2026-08-01",
  "operator_id": 1,
  "remark": "Production inbound",
  "items": [
    { "item_id": 100, "quantity": 10 }
  ]
}
```

> **Approval flow**: production/return inbound are created as `pending`, must be approved before stock is updated.
> **Batch inbound**: If `items` is empty + `batch_create` provided, system auto-creates N items of same spec.

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/api/v1/inbound-records/batch` | Batch create items + single-step inbound |

```json
{
  "inbound_type": "purchase",
  "inbound_date": "2026-08-01",
  "order_id": 1,
  "supplier_id": 1,
  "operator_id": 1,
  "remark": "Batch purchase inbound",
  "item_spec": {
    "name": "Raw Material B",
    "category": "raw_material",
    "unit": "kg",
    "spec": "Type A"
  },
  "quantity": 100
}
```

#### 6.3.2 Outbound Management

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/v1/outbound-records` | List (headers) |
| `POST` | `/api/v1/outbound-records` | Create (header + items). Sales type needs approved SO. |
| `GET` | `/api/v1/outbound-records/{id}` | Details |
| `GET` | `/api/v1/outbound-records/{id}/items` | Line items |
| `POST` | `/api/v1/outbound-records/{id}/approve` | Approve non-sales outbound |
| `POST` | `/api/v1/outbound-records/{id}/reject` | Reject |
| `DELETE` | `/api/v1/outbound-records/{id}` | Delete |

**POST /api/v1/outbound-records — Sales Outbound**:

```json
{
  "outbound_type": "sales",
  "outbound_date": "2026-08-01",
  "order_id": 1,
  "customer_id": 1,
  "operator_id": 1,
  "remark": "Sales outbound",
  "items": [
    { "item_id": 1, "quantity": 100 }
  ]
}
```

**POST /api/v1/outbound-records — Non-Sales (needs approval)**:

```json
{
  "outbound_type": "scrapped",
  "outbound_date": "2026-08-01",
  "operator_id": 1,
  "remark": "Scrapped outbound",
  "items": [
    { "item_id": 50, "quantity": 5 }
  ]
}
```

#### 6.3.3 Real-Time Inventory Query

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/v1/inventory` | Aggregated stock list |
| `GET` | `/api/v1/inventory/statistics` | Statistics summary |
| `GET` | `/api/v1/inventory/logs` | Transaction logs |
| `GET` | `/api/v1/atp` | ATP query (params: item_id) |

**GET /api/v1/inventory query params**:

| Parameter | Description |
|-----------|-------------|
| `item_id` | Item ID |
| `category` | Category |
| `status` | Status |
| `location_id` | Location |

**Response**:

```json
{
  "success": true,
  "data": [
    {
      "item_id": 1,
      "sku": "FG-202608-0001",
      "name": "Finished Good A",
      "total_quantity": 500,
      "in_stock": 480,
      "reserved": 20,
      "locations": ["WH1-A-01", "WH1-A-02"]
    }
  ],
  "meta": { "total": 25, "page": 1, "page_size": 20 }
}
```

#### 6.3.4 Inventory Check

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/v1/inventory-checks` | Check list |
| `POST` | `/api/v1/inventory-checks` | Create check |
| `GET` | `/api/v1/inventory-checks/{id}` | Details |
| `PUT` | `/api/v1/inventory-checks/{id}` | Update |
| `POST` | `/api/v1/inventory-checks/{id}/complete` | Complete, generate variance report |

#### 6.3.5 Location & Reservation Management

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/v1/warehouses` / `/api/v1/locations` | Warehouse/location list |
| `POST` | `/api/v1/locations` | Create location |
| `PUT` | `/api/v1/locations/{id}` | Update location |
| `DELETE` | `/api/v1/locations/{id}` | Delete location |
| `POST` | `/api/v1/locations/{id}/assign` | Bind item to location |
| `POST` | `/api/v1/items/{item_id}/transfer-location` | Transfer item location |
| `GET/POST` | `/api/v1/inventory/reservations` | Reservation list/create |
| `DELETE` | `/api/v1/inventory/reservations/{id}` | Release reservation |

### 6.4 Manufacturing & Inspection API

| Method | Path | Description |
|--------|------|-------------|
| `GET/POST` | `/api/v1/manufacturing/boms` | BOM list/create |
| `GET/POST` | `/api/v1/manufacturing/work-orders` | Work order list/create |
| `GET` | `/api/v1/manufacturing/work-orders/{id}` | Work order details |
| `POST` | `/api/v1/manufacturing/work-orders/{id}/complete` | Complete work order |
| `GET/POST` | `/api/v1/manufacturing/inspections` | Inspection list/create |
| `GET` | `/api/v1/manufacturing/inspections/{id}` | Inspection details |
| `PUT` | `/api/v1/manufacturing/inspections/{id}` | Update inspection |
| `GET/POST` | `/api/v1/manufacturing/ncrs` | NCR list/create |
| `POST` | `/api/v1/manufacturing/ncrs/{id}/close` | Close NCR |
| `GET` | `/api/v1/items/{id}/trace` | Full item lifecycle trace |

### 6.5 Procurement Management API

#### Suppliers

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/v1/suppliers` | List |
| `POST` | `/api/v1/suppliers` | Create |
| `GET` | `/api/v1/suppliers/{id}` | Details |
| `PUT` | `/api/v1/suppliers/{id}` | Update |
| `DELETE` | `/api/v1/suppliers/{id}` | Delete |

#### Purchase Orders

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/v1/purchase-orders` | List |
| `POST` | `/api/v1/purchase-orders` | Create |
| `GET` | `/api/v1/purchase-orders/{id}` | Details |
| `PUT` | `/api/v1/purchase-orders/{id}` | Update |
| `POST` | `/api/v1/purchase-orders/{id}/approve` | Approve (runs through workflow) |
| `POST` | `/api/v1/purchase-orders/{id}/reject` | Reject |
| `POST` | `/api/v1/purchase-orders/{id}/link-inbound` | Link to inbound |

#### Requisitions / Supplier Quotes / Receipts / Scorecards

| Method | Path | Description |
|--------|------|-------------|
| `GET/POST` | `/api/v1/requisitions` | Requisition list/create |
| `GET/POST` | `/api/v1/supplier-quotes` | Supplier quote list/create |
| `GET/POST` | `/api/v1/receipts` | Receipt list/create |
| `GET/POST` | `/api/v1/supplier-scorecards` | Scorecard list/create |

### 6.6 Sales Management API

#### Customers

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/v1/customers` | List |
| `POST` | `/api/v1/customers` | Create |
| `GET` | `/api/v1/customers/{id}` | Details |
| `PUT` | `/api/v1/customers/{id}` | Update |
| `DELETE` | `/api/v1/customers/{id}` | Delete |

#### Sales Orders

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/v1/sales-orders` | List |
| `POST` | `/api/v1/sales-orders` | Create |
| `GET` | `/api/v1/sales-orders/{id}` | Details |
| `PUT` | `/api/v1/sales-orders/{id}` | Update |
| `POST` | `/api/v1/sales-orders/{id}/approve` | Approve (runs through workflow) |
| `POST` | `/api/v1/sales-orders/{id}/reject` | Reject |
| `POST` | `/api/v1/sales-orders/{id}/link-outbound` | Link to outbound |
| `GET` | `/api/v1/atp` | ATP query (params: item_id) |

#### Customer Quotes / Shipments / Customer Credit

| Method | Path | Description |
|--------|------|-------------|
| `GET/POST` | `/api/v1/customer-quotes` | Customer quote list/create |
| `GET/POST` | `/api/v1/shipments` | Shipment list/create |
| `GET/PUT` | `/api/v1/customer-credit/{customer_id}` | Customer credit get/update |

### 6.7 Contract Management API

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/v1/contracts` | List |
| `POST` | `/api/v1/contracts` | Create |
| `GET` | `/api/v1/contracts/{id}` | Details |
| `PUT` | `/api/v1/contracts/{id}` | Update |

### 6.8 Data Import/Export API

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/api/v1/import/items` | Import items (Excel/CSV) |
| `POST` | `/api/v1/import/inventory` | Import inventory data |
| `POST` | `/api/v1/import/purchase-orders` | Import purchase orders |
| `GET` | `/api/v1/export/items` | Export items |
| `GET` | `/api/v1/export/inventory` | Export inventory report |
| `GET` | `/api/v1/export/inventory-logs` | Export inventory logs |

**Import**: `multipart/form-data` with file.

**Import response**:

```json
{
  "success": true,
  "data": {
    "total_rows": 1000,
    "success_rows": 985,
    "failed_rows": 15,
    "errors": [
      { "row": 23, "reason": "Duplicate SKU: FG-202608-0001" },
      { "row": 67, "reason": "Category 'unknown' is not in the allowed list" }
    ]
  }
}
```

### 6.9 System Management API

#### Users & Auth

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/api/v1/auth/login` | Login |
| `POST` | `/api/v1/auth/refresh` | Refresh token |
| `POST` | `/api/v1/auth/logout` | Logout |
| `GET` | `/api/v1/auth/me` | Current user |
| `GET` | `/api/v1/users` | List (admin) |
| `POST` | `/api/v1/users` | Create (admin) |
| `PUT` | `/api/v1/users/{id}` | Update |
| `PUT` | `/api/v1/users/{id}/role` | Change role (admin) |
| `DELETE` | `/api/v1/users/{id}` | Disable user |
| `GET/POST` | `/api/v1/roles` | Role list/create |
| `GET/POST` | `/api/v1/permissions` | Permission list/create |
| `GET/POST` | `/api/v1/departments` | Department list/create |

**POST /api/v1/auth/login**:

```json
{
  "username": "admin",
  "password": "********"
}
```

**Response**:

```json
{
  "success": true,
  "data": {
    "access_token": "eyJhbGciOiJIUzI1NiIs...",
    "token_type": "Bearer",
    "expires_in": 3600,
    "user": {
      "id": 1,
      "username": "admin",
      "display_name": "Administrator",
      "role": "admin",
      "language_pref": "zh",
      "unit_pref": "standard"
    }
  }
}
```

#### Operation Logs

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/v1/operation-logs` | List with filters |

**Query params**: `user_id`, `action`, `target_type`, `date_from`/`date_to`, `q`

### 6.10 Workflow API

| Method | Path | Description |
|--------|------|-------------|
| `GET/POST` | `/api/v1/workflow/definitions` | Definition list/create |
| `GET/POST` | `/api/v1/workflow/instances` | Instance list/start |
| `GET` | `/api/v1/workflow/tasks` | Task list (filter by assignee_id / status) |
| `POST` | `/api/v1/workflow/tasks/{id}/approve` | Approve |
| `POST` | `/api/v1/workflow/tasks/{id}/reject` | Reject |

### 6.11 Reports & BI Analytics API

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/v1/reports/inventory-summary` | Inventory summary |
| `GET` | `/api/v1/reports/inventory-monthly` | Monthly changes |
| `GET` | `/api/v1/reports/in-out-statistics` | In/out stats |
| `GET` | `/api/v1/reports/turnover-rate` | Turnover rate |
| `GET` | `/api/v1/bi/sales-trend` | Sales trend (BI) |
| `GET` | `/api/v1/bi/inventory-value` | Inventory value (BI) |
| `GET` | `/api/v1/bi/finance-summary` | Finance summary (BI) |
| `GET` | `/api/v1/bi/supplier-performance` | Supplier performance (BI) |

### 6.12 HR / Finance / Project / Asset / Notification / Portal API

| Module | Method | Path | Description |
|--------|--------|------|-------------|
| HR | `GET/POST` | `/api/v1/hr/employees` | Employee list/create |
| HR | `GET/POST` | `/api/v1/hr/attendance` | Attendance list/record |
| HR | `GET/POST` | `/api/v1/hr/salaries` | Salary list/record |
| HR | `GET/POST` | `/api/v1/hr/labor-contracts` | Labor contract list/create |
| Finance | `GET/POST` | `/api/v1/finance/accounts` | Account list/create |
| Finance | `GET/POST` | `/api/v1/finance/journal` | Journal list/create |
| Finance | `GET/POST` | `/api/v1/finance/invoices` | Invoice list/create |
| Finance | `GET/POST` | `/api/v1/finance/payments` | Payment list/create |
| Finance | `GET` | `/api/v1/finance/trial-balance` | Trial balance |
| Project | `GET/POST` | `/api/v1/projects` | Project list/create |
| Project | `GET/POST` | `/api/v1/projects/{id}/wbs` | WBS list/create |
| Project | `GET/POST` | `/api/v1/projects/{id}/budget` | Budget list/create |
| Asset | `GET/POST` | `/api/v1/assets` | Asset list/register |
| Asset | `GET` | `/api/v1/assets/{id}/depreciation` | Depreciation |
| Asset | `POST` | `/api/v1/assets/{id}/disposal` | Disposal |
| Notification | `GET` | `/api/v1/notifications` | Inbox |
| Notification | `POST` | `/api/v1/notifications/{id}/read` | Mark read |
| Notification | `GET/POST` | `/api/v1/notifications/templates` | Templates |
| Notification | `PUT` | `/api/v1/notifications/preferences` | Preferences |
| Portal | `GET/POST` | `/api/v1/portal/accounts` | Portal account list/create |
| Portal | `POST` | `/api/v1/portal/login` | Portal login (party JWT) |

---

## 7. Project Directory Structure

### 7.1 Backend (Rust, crate `erp-server`)

```
erp-server/
├── Cargo.toml                    # Dependencies (sqlx 0.8 sqlite feature)
├── Cargo.lock
├── .env                          # Env vars (DATABASE_URL=sqlite://data/erp.db?mode=rwc, JWT_SECRET, etc.)
├── .env.example                  # Template
├── sqlx-data.json                # SQLx offline check data
├── migrations/                   # 37 legacy migrations rewritten to SQLite syntax, pipe tables dropped
│   ├── 001_create_users.sql
│   ├── 002_create_items.sql      # Item table (replaces legacy industry-specific master tables)
│   ├── 003_create_locations.sql
│   ├── 004_create_inventory.sql
│   ├── 005_create_orders.sql
│   ├── 006_create_procurement.sql
│   ├── 007_create_sales_crm.sql
│   ├── 008_create_workflow.sql
│   ├── 009_create_hr.sql
│   ├── 010_create_finance.sql
│   ├── 011_create_manufacturing.sql
│   ├── 012_create_projects.sql
│   ├── 013_create_assets.sql
│   ├── 014_create_notifications.sql
│   ├── 015_create_portal.sql
│   ├── 016_create_bi_views.sql
│   └── 017_create_logs.sql
├── src/
│   ├── main.rs                   # Entry: server startup, config
│   ├── lib.rs                    # Library entry
│   ├── config.rs                 # Config (env → Config struct)
│   ├── router.rs                 # Route registration (~70 endpoints)
│   ├── error.rs                  # AppError enum + numeric error codes
│   ├── response.rs               # ApiResponse, PaginatedResponse
│   ├── cache.rs                  # TTL in-memory cache
│   ├── middleware/
│   │   ├── mod.rs
│   │   ├── auth.rs               # JWT auth middleware
│   │   ├── rbac.rs               # Role-based access control
│   │   ├── logging.rs            # Request logging
│   │   └── request_id.rs         # Request ID generation
│   ├── auth/                     # RBAC: roles/permissions/departments/tenants
│   │   ├── repos.rs
│   │   ├── services.rs           # IdentityService
│   │   └── handlers.rs
│   ├── workflow/                 # Approval engine
│   │   ├── repos.rs
│   │   ├── services.rs           # WorkflowService
│   │   └── handlers.rs
│   ├── hr/                       # HR
│   │   ├── repos.rs
│   │   ├── services.rs           # HrService
│   │   └── handlers.rs
│   ├── finance/                  # Finance
│   │   ├── repos.rs
│   │   ├── services.rs           # FinanceService
│   │   └── handlers.rs
│   ├── procurement/              # Procurement
│   │   ├── repos.rs
│   │   ├── services.rs
│   │   └── handlers.rs
│   ├── sales_crm/                # Sales CRM
│   │   ├── repos.rs
│   │   ├── services.rs
│   │   └── handlers.rs
│   ├── inventory_atp/            # Inventory (items/stock/reservations/transfers/count)
│   │   ├── repos.rs
│   │   ├── services.rs           # ItemService / InventoryService / ReservationService
│   │   └── handlers.rs           # item_handler / inventory_handler / atp_handler
│   ├── manufacturing/            # Manufacturing (BOMs/work orders/inspections/NCRs)
│   │   ├── repos.rs
│   │   ├── services.rs
│   │   └── handlers.rs
│   ├── project/                  # Projects (projects/WBS/budget)
│   │   ├── repos.rs
│   │   ├── services.rs
│   │   └── handlers.rs
│   ├── assets/                   # Fixed assets
│   │   ├── repos.rs
│   │   ├── services.rs
│   │   └── handlers.rs
│   ├── notification/             # Notifications
│   │   ├── repos.rs
│   │   ├── services.rs
│   │   └── handlers.rs
│   ├── portal/                   # Portal
│   │   ├── repos.rs
│   │   ├── services.rs
│   │   └── handlers.rs
│   ├── bi/                       # BI analytics
│   │   ├── services.rs
│   │   └── handlers.rs
│   ├── domain/
│   │   ├── mod.rs
│   │   ├── item.rs               # Item enums, constants
│   │   └── order.rs              # Order status enums
│   ├── models/
│   │   ├── mod.rs
│   │   ├── item.rs               # Item struct
│   │   ├── location.rs
│   │   ├── inbound.rs            # InboundRecord + InboundItem
│   │   ├── outbound.rs           # OutboundRecord + OutboundItem
│   │   ├── inventory_log.rs
│   │   ├── reservation.rs
│   │   ├── supplier.rs
│   │   ├── customer.rs
│   │   ├── purchase_order.rs
│   │   ├── sales_order.rs
│   │   ├── contract.rs
│   │   ├── inspection.rs         # Inspection + Ncr
│   │   ├── work_order.rs
│   │   ├── workflow.rs           # Definition + Instance + Task
│   │   ├── inventory_check.rs    # Check + CheckItem
│   │   ├── user.rs
│   │   └── operation_log.rs
│   ├── dto/
│   │   ├── mod.rs
│   │   ├── item_dto.rs           # Item create/update/query DTO
│   │   ├── inventory_dto.rs
│   │   ├── order_dto.rs
│   │   ├── auth_dto.rs
│   │   └── common.rs             # Pagination / sort / search params
│   ├── handlers/                 # Thin HTTP layer
│   │   ├── mod.rs
│   │   ├── item_handler.rs
│   │   ├── inventory_handler.rs
│   │   ├── manufacturing_handler.rs
│   │   ├── purchase_handler.rs
│   │   ├── sales_handler.rs
│   │   ├── supplier_handler.rs
│   │   ├── customer_handler.rs
│   │   ├── contract_handler.rs
│   │   ├── location_handler.rs
│   │   ├── data_io_handler.rs
│   │   ├── auth_handler.rs
│   │   ├── user_handler.rs
│   │   ├── workflow_handler.rs
│   │   ├── hr_handler.rs
│   │   ├── finance_handler.rs
│   │   ├── project_handler.rs
│   │   ├── asset_handler.rs
│   │   ├── notification_handler.rs
│   │   ├── portal_handler.rs
│   │   ├── bi_handler.rs
│   │   ├── log_handler.rs
│   │   ├── report_handler.rs
│   │   └── atp_handler.rs
│   ├── services/                 # Unit struct + static methods
│   │   ├── mod.rs
│   │   ├── item_service.rs
│   │   ├── inventory_service.rs
│   │   ├── location_service.rs
│   │   ├── manufacturing_service.rs
│   │   ├── purchase_service.rs
│   │   ├── sales_service.rs
│   │   ├── data_io_service.rs
│   │   ├── auth_service.rs
│   │   ├── user_service.rs
│   │   ├── report_service.rs
│   │   ├── contract_service.rs
│   │   ├── trace_service.rs
│   │   ├── customer_service.rs
│   │   └── supplier_service.rs
│   └── repositories/             # Static methods + soft-delete aware
│       ├── mod.rs
│       ├── item_repo.rs
│       ├── location_repo.rs
│       ├── warehouse_repo.rs
│       ├── inbound_repo.rs
│       ├── outbound_repo.rs
│       ├── inventory_log_repo.rs
│       ├── reservation_repo.rs
│       ├── supplier_repo.rs
│       ├── customer_repo.rs
│       ├── purchase_order_repo.rs
│       ├── sales_order_repo.rs
│       ├── contract_repo.rs
│       ├── inspection_repo.rs
│       ├── work_order_repo.rs
│       ├── workflow_repo.rs
│       ├── inventory_check_repo.rs
│       ├── user_repo.rs
│       ├── operation_log_repo.rs
│       ├── report_repo.rs
│       └── data_io_repo.rs
└── tests/
    ├── common/
    │   ├── mod.rs
    │   └── test_db.rs            # In-memory SQLite DB init for tests
    ├── item_tests.rs
    ├── inventory_tests.rs
    ├── order_tests.rs
    ├── workflow_tests.rs
    ├── auth_tests.rs
    └── api_integration_tests.rs
```

### 7.2 Frontend (React 19)

```
erp-frontend/
├── package.json
├── tsconfig.json
├── vite.config.ts
├── public/
│   └── favicon.ico
├── src/
│   ├── main.tsx                    # Entry
│   ├── App.tsx                     # Root: ConfigProvider + QueryClient + Router
│   ├── routes/
│   │   ├── index.tsx               # Route config (createBrowserRouter)
│   │   └── ProtectedRoute.tsx      # Auth guard + role check
│   ├── layouts/
│   │   └── MainLayout.tsx          # Sidebar + Header + Outlet
│   ├── features/                   # 20 feature modules
│   │   ├── auth/                   # Login, user management
│   │   ├── items/                  # Item/SKU CRUD, search
│   │   ├── inventory/              # Inbound/outbound/stock/check/locations/reservations
│   │   ├── suppliers/
│   │   ├── customers/
│   │   ├── purchases/              # POs
│   │   ├── sales/                  # SOs, ATP
│   │   ├── workflow/               # Definitions, instances, tasks
│   │   ├── hr/                     # Employees, attendance, salaries, labor contracts
│   │   ├── finance/                # Accounts, journal, invoices, payments, trial balance
│   │   ├── procurement/            # Requisitions, supplier quotes, receipts, scorecards
│   │   ├── manufacturing/          # BOMs, work orders, inspections, NCRs
│   │   ├── projects/               # Projects, WBS, budget
│   │   ├── assets/                 # Fixed assets
│   │   ├── notifications/          # Inbox
│   │   ├── portal/                 # Portal accounts
│   │   ├── contracts/
│   │   ├── reports/                # Dashboard, reports, BI
│   │   ├── search/                 # Global search
│   │   └── profile/                # Profile settings
│   ├── shared/
│   │   ├── components/             # 9 shared components
│   │   ├── hooks/
│   │   └── utils/
│   ├── api/
│   │   ├── client.ts               # Axios + interceptors (auth, refresh)
│   │   └── queryClient.ts          # TanStack Query config
│   ├── stores/                     # authStore, appStore, unitStore
│   ├── i18n/                       # react-i18next (zh + en, per-module)
│   ├── zod-schemas/                # Runtime Zod validation schemas
│   ├── lib/
│   │   └── validateResponse.ts     # Zod response validation wrapper
│   └── styles/
│       ├── global.css
│       └── theme.ts                # Ant Design 5 theme tokens
└── .env
```

---

## 8. Error Handling & Response Specification

### 8.1 HTTP Status Codes

| Code | When |
|------|------|
| `200` | GET / PUT success |
| `201` | POST creation success |
| `204` | DELETE success |
| `400` | Bad request (validation) |
| `401` | Unauthenticated |
| `403` | Forbidden (wrong role) |
| `404` | Not found |
| `409` | Conflict (e.g. duplicate SKU) |
| `422` | Validation failure (Validator crate) |
| `500` | Internal server error |

### 8.2 Error Codes

```rust
pub enum AppErrorCode {
    // General (100xx)
    InternalError,          // 10001
    ValidationError,        // 10002
    NotFound,               // 10003

    // Auth (110xx)
    Unauthorized,           // 11001
    TokenExpired,           // 11002
    Forbidden,              // 11003

    // Item/商品 (120xx)
    ItemNotFound,           // 12001
    SkuDuplicate,           // 12002
    ItemStatusConflict,     // 12003 (e.g. can't delete a disabled/outbound item)

    // Inventory (130xx)
    InsufficientStock,      // 13001
    LocationFull,           // 13002
    LocationNotFound,       // 13003

    // Orders (140xx)
    OrderCannotModify,      // 14001
    OrderNotFound,          // 14002
    OrderNotApproved,       // 14003

    // Inspection/质检 (150xx)
    InspectionNotFound,     // 15001
    AttachmentNotFound,     // 15002

    // Suppliers (160xx)
    SupplierNotFound,       // 16001
    SupplierCodeDuplicate,  // 16002

    // Customers (170xx)
    CustomerNotFound,       // 17001
    CustomerCodeDuplicate,  // 17002

    // Data IO (180xx)
    ExportError,            // 18001
    ImportError,            // 18002

    // Database
    DatabaseError,          // 50001
}
```

### 8.3 Global Error Handling

All errors are caught by Axum's `FromRequestParts` or middleware and converted to the unified `ApiErrorResponse` format (`success: false` + `request_id`). The `x-request-id` response header is propagated via tower-http.

---

## 9. Non-Functional Design

### 9.1 Performance

| Metric | Approach |
|--------|----------|
| **Query ≤ 2s (100k rows)** | Proper indexes, pagination, avoid N+1 |
| **Import 100k rows ≤ 60s** | Batch inserts (1000 rows/transaction) |
| **20+ concurrent users** | SQLite WAL for concurrent reads; write serialization via `busy_timeout` |
| **Large queries** | BI aggregate views or app-level caching if needed |

**SQLite Concurrency**:

```
1. WAL mode: reads don't block writes
2. Writes serialized via Tokio Mutex to avoid SQLITE_BUSY
3. Connection pool: SQLx SqlitePool, max 5 connections
4. Long writes split into smaller batch transactions
5. Periodic PRAGMA wal_checkpoint(TRUNCATE) to limit WAL size
6. Connection string fixed: sqlite://data/erp.db?mode=rwc
```

### 9.2 Config Management

```rust
pub struct AppConfig {
    pub database_url: String,       // sqlite://data/erp.db?mode=rwc
    pub jwt_secret: String,
    pub jwt_expires_in: u64,
    pub host: String,
    pub port: u16,
    pub log_level: String,
    pub upload_dir: String,
    pub max_file_size: usize,
    pub default_language: String,
    pub default_unit_pref: String,
}
```

### 9.3 Logging Strategy

| Layer | Technology | Description |
|-------|------------|-------------|
| **App logs** | `tracing` + `tracing-subscriber` | Structured, JSON output |
| **Audit logs** | `operation_logs` table | All data changes in DB |

| Environment | Level |
|-------------|-------|
| Dev | `debug` |
| Test | `info` |
| Prod | `warn` |

### 9.4 Data Backup

- SQLite is a single file — trivial to back up
- Suggested: daily full backup + hourly WAL archive
- Use `.backup` command for online hot backup

---

## 10. Security Design

### 10.1 Authentication & Authorization

| Layer | Approach |
|-------|----------|
| **Password storage** | Argon2id (m=19456, t=2, p=1) |
| **Session** | JWT (access_token 1h + refresh_token 7d) |
| **API auth** | JWT Bearer middleware (`Extension<JwtSecret>`) |
| **RBAC** | Axum middleware extracts role from token, matches route permissions (role → permission → user, with departments and optional tenants) |
| **Sensitive ops** | DELETE/critical mods require confirmation (frontend + backend audit) |

### 10.2 Role Permission Matrix

| Feature / Role | Admin | Warehouse | Sales | Procurement |
|----------------|:-----:|:---------:|:-----:|:-----------:|
| Item view | ✅ | ✅ | ✅ | ✅ |
| Item create/update/delete | ✅ | ✅ | -- | -- |
| Inbound operations | ✅ | ✅ | -- | ✅ (purchase only) |
| Outbound operations | ✅ | ✅ | ✅ (sales only) | -- |
| Inbound approval (non-purchase) | ✅ | ✅ | -- | -- |
| Outbound approval (non-sales) | ✅ | ✅ | -- | -- |
| Inventory query | ✅ | ✅ | ✅ | ✅ |
| Inventory check | ✅ | ✅ | -- | -- |
| Location/reservation management | ✅ | ✅ | -- | -- |
| Manufacturing & inspection | ✅ | ✅ | -- | -- |
| Purchase orders | ✅ | -- | -- | ✅ |
| Sales orders | ✅ | -- | ✅ | -- |
| Suppliers/Customers | ✅ | -- | customers | suppliers |
| Import/Export | ✅ | ✅ | ✅ | ✅ |
| Reports/BI | ✅ | ✅ | ✅ | ✅ |
| User management | ✅ | -- | -- | -- |
| Operation logs | ✅ | self only | self only | self only |
| System config | ✅ | -- | -- | -- |

### 10.3 Input Validation

- All DTOs validated with `validator` crate
- File uploads: restricted types (PDF/images/Excel), max 50MB, sanitized names
- SQL injection: SQLx parameterized queries only
- XSS: handled on frontend; API returns raw data

### 10.4 Audit Trail

```
Every data modification (create / update / delete) is auto-logged:
- Who (user_id + username)
- When (created_at)
- What (target_type + target_id)
- Action (action)
- Details (detail: JSON, before/after)
- Where from (ip_address)
```

---

## 11. Internationalization & Unit Switching Design

### 11.1 i18n

**Frontend** (`react-i18next`):

```
i18n/
├── resources/
│   ├── zh/                     # Chinese (primary)
│   │   ├── common.json
│   │   ├── items.json
│   │   ├── inventory.json
│   │   ├── purchase.json
│   │   ├── sales.json
│   │   ├── workflow.json
│   │   ├── hr.json
│   │   ├── finance.json
│   │   ├── manufacturing.json
│   │   ├── system.json
│   │   └── validation.json
│   └── en/                     # English
│       ├── common.json
│       ├── items.json
│       ├── inventory.json
│       ├── purchase.json
│       ├── sales.json
│       ├── workflow.json
│       ├── hr.json
│       ├── finance.json
│       ├── manufacturing.json
│       ├── system.json
│       └── validation.json
```

Namespaces are per-feature, lazy-loaded. Switching language updates `language_pref` and triggers re-render via `useTranslation()`.

### 11.2 Unit Switching

**Strategy**: Items carry a `unit` field (kg / m / pc / etc.). Cross-unit conversion (e.g. kg ↔ t, m ↔ km) is handled client-side in `unitStore` by conversion factors; backend storage stays consistent with the item's unit field.

**API**:

```json
// Item unit is returned with master data
{
  "data": {
    "sku": "FG-202608-0001",
    "name": "Finished Good A",
    "unit": "kg",
    "quantity": 1000
  }
}
```

**Implementation**:
- `items.unit` defines the item's measurement unit
- Frontend `unitStore` keeps the user's conversion preference (standard/metric/imperial)
- Display layer converts per user preference without changing backend storage
- Optional `Measurement<T>` wrapper type at the DTO layer carries unit info

---

## Appendix A: Key Decision Records (ADR)

### ADR-001: Monolith over Microservices

| Item | Content |
|------|---------|
| **Context** | Medium project, small team |
| **Decision** | Modular monolith organized by domain |
| **Why** | Less operational complexity; SQLite doesn't do distributed; can split later if needed |
| **Cost** | Must enforce module boundaries rigorously |

### ADR-002: SQLite as the Only Database

| Item | Content |
|------|---------|
| **Context** | 100K+ rows, 20+ concurrent users |
| **Decision** | SQLite3 (`sqlite://data/erp.db?mode=rwc`) WAL + connection pool + write serialization |
| **Why** | Zero config, file-level, plenty for this scale. The 37 legacy migrations are rewritten to SQLite syntax minus the pipe tables. |
| **Cost** | Keep the app-layer referential integrity design; if scale explodes, the SQLx abstraction keeps options open. |

### ADR-003: Single Item Table as the Single Business Entity

| Item | Content |
|------|---------|
| **Context** | The legacy system split tables by industry type, producing field drift and UNION-based cross-type queries |
| **Decision** | One `items` table: sku / name / category / unit / spec / status — no industry-specific fields |
| **Why** | Generic ERP items come in many shapes; a single table with category + spec is more flexible; no UNION needed |
| **Cost** | Industry-specific validation fields are dropped; category and spec become the two generic dimensions |

### ADR-004: Module Keep/Delete Boundary

| Item | Content |
|------|---------|
| **Context** | The steel-pipe-era modules (pipes/threading/labels/quality certs) no longer apply |
| **Decision** | Delete the pipes/threading/labels/quality-cert modules; keep and generalize inventory; manufacturing inspection (BOM/work orders/Inspection/NCR) carries the quality capability; add workflow/HR/finance/project/asset/notification/portal/BI |
| **Why** | A general-purpose ERP must cover procurement, sales, inventory, manufacturing, finance, and HR end-to-end |
| **Cost** | Import/export, search, and reporting all shift to generic items/documents |

---

## Appendix B: Item Category & Unit Reference Data (Seed Data)

Seeded with the `002_create_items.sql` migration:

```sql
-- Item category reference data
INSERT INTO item_categories (code, name) VALUES
('RM', 'Raw material'),
('SF', 'Semi-finished'),
('FG', 'Finished goods'),
('SP', 'Spare parts');

-- Item unit reference data
INSERT INTO item_units (code, name) VALUES
('kg', 'Kilogram'),
('m',   'Meter'),
('pc',  'Piece'),
('box', 'Box'),
('L',   'Liter');
```

> Categories and units are sample seeds only — enterprises can extend them. The SKU auto-generation rule is `{category-code}-{yyyymm}-{seq}`, e.g. `FG-202608-0001`.

---
