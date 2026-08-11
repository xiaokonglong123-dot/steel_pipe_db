# Seamless Steel Pipe & Screen Pipe Management System — Detailed Design

> **Version**: v1.1
> **Date**: 2026-05-19
> **Based on**: docs/requirements.en.md v1.0
> **Stack**: Rust + Axum + SQLx + SQLite (WAL) | React 19 + Ant Design 5

---

## Revision History

| Version | Date | Changes | Author |
|---------|------|---------|--------|
| v1.0 | 2026-05-19 | Initial version | - |
| v1.1 | 2026-05-19 | Inbound/outbound changed to Header+Items structure; inventory check changed to per-pipe verification; added attachments table; numbering format follows API 5CT standard; removed PATCH endpoints; fixed sort_by injection vulnerability | - |

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
11. [i18n & Unit Switching](#11-internationalization--unit-switching-design)

---

## 1. System Overview

### 1.1 What This Thing Is

A web-based inventory management system for seamless steel pipes (Casing / Tubing) and screen pipes. It tracks the full lifecycle of every pipe from receipt to dispatch, all based on API 5CT / ISO 11960 standards.

### 1.2 Core Capabilities

| Capability | Description |
|------------|-------------|
| Full lifecycle tracking | Every pipe tracked from procurement receipt to sales dispatch |
| Integrated inventory | Procurement, stock, and sales all linked together |
| Quality traceability | Trace by heat number or pipe number |
| Multi-user RBAC | 4 roles: warehouse, QC, sales/procurement, admin |
| i18n | Chinese/English UI + metric/imperial units |

---

## 2. Tech Stack Decisions

### 2.1 Backend

| Layer | Choice | Version | Why |
|-------|--------|---------|-----|
| **Web Framework** | Axum | 0.8+ | Mainstream Rust async framework, tower middleware, great ecosystem |
| **SQL Layer** | SQLx | 0.8+ | Compile-time checked SQL, no ORM overhead, native SQLite support |
| **Database** | SQLite (WAL) | 3.46+ | Zero config, file-level, WAL handles concurrent reads |
| **Serialization** | Serde + serde_json | 1.x | The Rust standard |
| **Auth** | JWT (jsonwebtoken) | — | Stateless, works well with SPA |
| **Password Hashing** | Argon2 | — | OWASP recommended. `m=19456, t=2, p=1` |
| **Async Runtime** | Tokio | 1.x | Standard Rust async |
| **Validation** | Validator | 0.19+ | Derive-macro based struct validation |
| **Logging** | Tracing + tracing-subscriber | — | Structured logging, JSON output |
| **File Upload** | Axum multipart | — | For QC file uploads |
| **Excel** | calamine (read) + rust_xlsxwriter (write) | — | Excel import/export |
| **API Docs** | utoipa + utoipa-swagger-ui | — | OpenAPI 3.0 auto-generated docs |

### 2.2 Frontend

| Layer | Choice | Version | Why |
|-------|--------|---------|-----|
| **UI Framework** | React | 19.x | Latest stable, mature ecosystem |
| **Build Tool** | Vite | 6.x | Fast dev server, ESBuild |
| **Component Lib** | Ant Design | 5.x | Enterprise-grade, great tables/forms |
| **Routing** | React Router | 7.x | Nested routes, lazy loading |
| **Server State** | TanStack Query | 5.x | Caching, stale-while-revalidate, optimistic updates |
| **Client State** | Zustand | 5.x | Lightweight, no boilerplate, localStorage persistence |
| **HTTP Client** | Axios | 1.x | Interceptors for auth + refresh |
| **i18n** | react-i18next | 15.x | Namespace support, lazy loading |
| **Type Safety** | TypeScript 5 (strict) + Zod 3 | — | Runtime response validation |
| **Charts** | @ant-design/charts | 2.x | G2Plot-based, matches Ant Design style |

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
│                  Rust Backend Service (Axum)                 │
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
│                                  │  SQLite (WAL)    │       │
│                                  │  File Database    │       │
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
│  (Business Logic)                                      │
│  Job: CRUD orchestration, transactions, permission checks, logging │
├──────────────────────────────────────────────────────┤
│                    Repository Layer                    │
│  (Data Access + SQL)                                   │
│  Job: Execute SQLx queries, map rows, paginate, sort   │
├──────────────────────────────────────────────────────┤
│                    Domain Layer                        │
│  (Data Models + Types)                                 │
│  Job: Struct definitions, enums, constants, API 5CT ref data │
└──────────────────────────────────────────────────────┘
```

### 3.3 Module Dependencies

```
                    ┌──────────────┐
                    │ System Admin │
                    │ (Users/Roles/│
                    │   Logs)      │
                    └──────┬───────┘
                           │ depends on
              ┌────────────┼────────────────┐
              ▼            ▼                ▼
        ┌──────────┐ ┌──────────┐ ┌──────────────┐
        │ Pipe     │ │ Purchase/│ │  Quality     │
        │ Mgmt     │ │ Sales    │ │  Mgmt        │
        │ Module   │ │ Module   │ │  Module      │
        └─────┬────┘ └─────┬────┘ └──────┬───────┘
              │            │             │
              ▼            ▼             ▼
        ┌─────────────────────────────────────┐
        │          Inventory Mgmt Module       │
        │  (Inbound/Outbound/Check/Location/   │
        │   Real-time Stock Query)             │
        └──────────┬──────────────────────────┘
                   │
                   ▼
        ┌─────────────────────────────────────┐
        │    History Traceability Module       │
        │    (Operation Logs)                  │
        └─────────────────────────────────────┘
```

---

## 4. Module Design

### 4.1 Module Overview

| Module | Priority | Core Stuff | Testable Independently? |
|--------|----------|------------|------------------------|
| **Pipe Management** | P0 | Seamless/screen pipe CRUD, search, archive | Yes |
| **Inventory Management** | P0 | Inbound/outbound/check/location/stock query | Yes |
| **Quality Management** | P1 | QC certs, quality traceability, API 5CT reference | Yes |
| **Procurement Management** | P1 | Suppliers, POs, inbound linkage | Yes |
| **Sales Management** | P1 | Customers, SOs, outbound linkage, ATP | Yes |
| **Contract Management** | P2 | Purchase/sales contracts | Yes |
| **Data Import/Export** | P1 | Excel/CSV import/export | Yes |
| **Search & Filtering** | P0 | Multi-dimensional combined search (shared) | -- |
| **Reports & Stats** | P2 | Inventory reports, charts | Yes |
| **Label Printing** | P2 | Barcode/QR code labels | Yes |
| **History Traceability** | P0 | Full lifecycle operation logs | Yes |
| **System Management** | P1 | Users, RBAC, audit logs | Yes |

### 4.2 Pipe Management Module

**Seamless Steel Pipe & Screen Pipe CRUD**

```
┌─────────────────────────────────────────────────────┐
│               Pipe Management Module (PipeModule)     │
│                                                      │
│  ┌────────────────────┐ ┌─────────────────────┐     │
│  │ SeamlessPipeHandler │ │ ScreenPipeHandler    │     │
│  │  (Seamless Pipe CRUD)│ │ (Screen Pipe CRUD)   │     │
│  └────────┬───────────┘ └─────────┬───────────┘     │
│           │                      │                  │
│  ┌────────▼──────────────────────▼───────────┐     │
│  │           PipeService                      │     │
│  │    (General pipe queries, number uniqueness│     │
│  │     validation)                            │     │
│  └────────┬──────────────────────┬───────────┘     │
│           │                      │                  │
│  ┌────────▼──────────┐ ┌─────────▼──────────┐     │
│  │ SeamlessPipeRepo   │ │ ScreenPipeRepo      │     │
│  │ (Seamless pipe     │ │ (Screen pipe        │     │
│  │  data access)      │ │  data access)       │     │
│  └───────────────────┘ └────────────────────┘     │
└─────────────────────────────────────────────────────┘
```

**Core Interfaces:**

| Interface | Description |
|-----------|-------------|
| `create_seamless_pipe(dto)` | Create a seamless pipe |
| `update_seamless_pipe(id, dto)` | Update seamless pipe info |
| `delete_seamless_pipe(id)` | Soft delete (checks inventory first) |
| `get_seamless_pipe(id)` | Get single pipe details |
| `list_seamless_pipes(filters)` | Query with filters + pagination |
| `create_screen_pipe(dto)` | Create a screen pipe |
| `update_screen_pipe(id, dto)` | Update screen pipe info |
| `delete_screen_pipe(id)` | Soft delete |
| `get_screen_pipe(id)` | Get screen pipe details |
| `list_screen_pipes(filters)` | Query with filters + pagination |
| `generate_pipe_number(pipe_type)` | Auto-generate pipe number |
| `validate_pipe_number_unique(number)` | Check uniqueness |

### 4.3 Inventory Management Module

```
┌──────────────────────────────────────────────────────┐
│              Inventory Management Module              │
│              (InventoryModule)                        │
│                                                        │
│  ┌──────────┐ ┌──────────┐ ┌────────┐ ┌──────────┐   │
│  │ Inbound  │ │ Outbound │ │Inventory│ │ Location │   │
│  │ Handler   │ │ Handler  │ │ Query  │ │ Handler  │   │
│  └─────┬────┘ └─────┬────┘ └───┬────┘ └─────┬────┘   │
│        │            │         │           │           │
│  ┌─────▼────────────▼─────────▼───────────▼─────┐   │
│  │           InventoryService                    │   │
│  │    (Inventory changes + transaction logs     │   │
│  │     + inventory validation)                   │   │
│  └─────┬────────────┬────────────────┬──────────┘   │
│        │            │                │               │
│  ┌─────▼────┐ ┌─────▼──────┐ ┌──────▼───────┐      │
│  │ Inbound  │ │ Outbound   │ │ Location     │      │
│  │ Repo     │ │ Repo       │ │ Repo         │      │
│  └──────────┘ └────────────┘ └──────────────┘      │
└──────────────────────────────────────────────────────┘
```

**Core Interfaces:**

| Interface | Description |
|-----------|-------------|
| `create_inbound(dto)` | Create inbound, auto-update stock. **Constraint**: when type='purchase', order_id required + PO must be approved. Production/return types start as `pending` and need supervisor approval. |
| `approve_inbound(id)` | Approve non-purchase inbound, applies stock changes |
| `reject_inbound(id, reason)` | Reject non-purchase inbound |
| `create_outbound(dto)` | Create outbound, deduct stock. Sales type auto-approved. Transfer/scrapped need approval. |
| `approve_outbound(id)` | Approve non-sales outbound, deducts stock |
| `reject_outbound(id, reason)` | Reject non-sales outbound |
| `get_stock_status(pipe_type, pipe_id)` | Check stock for a single pipe |
| `list_inventory(filters)` | Real-time stock query (aggregated) |
| `list_inventory_logs(filters)` | Transaction log |
| `create_inventory_check(dto)` | Create a stock check |
| `submit_check_result(dto)` | Submit results, generate variance report |
| `create_location(dto)` | Create storage location |
| `assign_pipe_to_location(pipe_id, location_id)` | Bind pipe to location |
| `transfer_location(pipe_id, new_location_id)` | Move pipe |

### 4.4 Quality Management Module

**Core Interfaces:**

| Interface | Description |
|-----------|-------------|
| `create_quality_cert(dto)` | Upload/enter QC cert |
| `update_quality_cert(id, dto)` | Update QC cert |
| `get_quality_cert(id)` | Get cert details |
| `list_quality_certs(filters)` | List with filters |
| `trace_by_heat_number(heat_no)` | Trace by heat number |
| `trace_by_pipe_number(pipe_no)` | Trace by pipe number |
| `get_api_5ct_grade_ref(grade)` | Get API 5CT grade reference data |

### 4.5 Procurement Management Module (PurchaseModule)

**Dependencies**: Inventory (inbound linkage), Pipe Management (spec references)
**Depended by**: Inventory references POs when creating purchase inbound

```
┌─────────────────────────────────────────────┐
│        Procurement Management Module         │
│           (PurchaseModule)                   │
│                                              │
│  ┌────────────────┐ ┌─────────────────────┐ │
│  │ SupplierHandler │ │ PurchaseOrderHandler │ │
│  │ (Supplier CRUD) │ │ (PO CRUD + Approval) │ │
│  └───────┬────────┘ └──────────┬──────────┘ │
│          │                     │            │
│  ┌───────▼─────────────────────▼──────────┐ │
│  │          PurchaseService                │ │
│  │  (Supplier mgmt + PO + inbound linkage) │ │
│  └───────┬─────────────────────┬──────────┘ │
│          │                     │            │
│  ┌───────▼────────┐ ┌──────────▼──────────┐ │
│  │ SupplierRepo    │ │ PurchaseOrderRepo   │ │
│  │ (Supplier data  │ │ (PO + line items)   │ │
│  │  access)        │ │                     │ │
│  └────────────────┘ └─────────────────────┘ │
└─────────────────────────────────────────────┘
```

**Core Interfaces:**

| Interface | Description |
|-----------|-------------|
| `create_supplier(dto)` | Create supplier |
| `update_supplier(id, dto)` | Update supplier |
| `delete_supplier(id)` | Delete supplier |
| `list_suppliers(filter)` | List suppliers |
| `create_purchase_order(dto)` | Create PO (with line items) |
| `approve_purchase_order(id)` | Approve PO (draft → pending → approved) |
| `reject_purchase_order(id, reason)` | Reject PO |
| `link_inbound_to_po(inbound_id, po_id)` | Link inbound to PO, update received qty |

### 4.6 Sales Management Module (SalesModule)

**Dependencies**: Inventory (outbound linkage + ATP), Pipe Management (specs)
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
│  │  (Customer mgmt + SO + outbound linkage   │  │
│  │   + ATP)                                  │  │
│  └───────┬─────────────────────┬────────────┘  │
│          │                     │               │
│  ┌───────▼────────┐ ┌──────────▼───────────┐  │
│  │ CustomerRepo    │ │ SalesOrderRepo        │  │
│  │ (Customer data  │ │ (SO + line items      │  │
│  │  access)        │ │  + ATP)              │  │
│  └────────────────┘ └──────────────────────┘  │
└──────────────────────────────────────────────┘
```

**Core Interfaces:**

| Interface | Description |
|-----------|-------------|
| `create_customer(dto)` | Create customer |
| `update_customer(id, dto)` | Update customer |
| `delete_customer(id)` | Delete customer |
| `list_customers(filter)` | List customers |
| `create_sales_order(dto)` | Create SO (with line items) |
| `approve_sales_order(id)` | Approve SO |
| `reject_sales_order(id, reason)` | Reject SO |
| `link_outbound_to_so(outbound_id, so_id)` | Link outbound to SO, update delivered qty |
| `get_atp(pipe_type, grade, od, wt)` | Available-to-promise: in_stock - locked SO qty |

### 4.7 System Management Module

**Core Interfaces:**

| Interface | Description |
|-----------|-------------|
| `create_user(dto)` | Create user (admin only) |
| `update_user(id, dto)` | Update user |
| `list_users(filters)` | List users |
| `assign_role(user_id, role)` | Assign role |
| `login(credentials)` | Login, returns JWT |
| `refresh_token(token)` | Refresh JWT |
| `get_current_user()` | Get current user info |
| `list_operation_logs(filters)` | Query operation logs |

---

## 5. Database Detailed Design

### 5.1 Design Principles

- **SQLite WAL mode**: `PRAGMA journal_mode=WAL;` — concurrent reads without blocking
- **No FK constraints**: SQLite FK enforcement has perf overhead; we enforce referential integrity in application code
- **Indexes**: Index frequently queried fields, composite indexes for combined queries
- **Timestamps**: ISO 8601 text format (`TEXT`) everywhere
- **Enums**: Stored as `TEXT` — readable and extensible
- **Soft deletes**: Key tables have `deleted_at` field; no physical deletion

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

#### 5.3.1 seamless_pipes — Seamless Steel Pipe Table

**Table**: `seamless_pipes`

**Purpose**: Every individual seamless steel pipe gets a row here.

| Field | Type | Constraint | Description |
|-------|------|------------|-------------|
| `id` | INTEGER | PK, AUTOINCREMENT | Primary key |
| `pipe_number` | TEXT | NOT NULL, UNIQUE | Pipe number (globally unique, e.g. `J55 4.500in x 11.60lb SC-H2405-000001`) |
| `batch_number` | TEXT | -- | Batch number from PO |
| `pipe_type` | TEXT | NOT NULL, CHECK IN ('casing','tubing') | Casing or tubing |
| `grade` | TEXT | NOT NULL | Steel grade (H40, J55, K55, N80, etc.) |
| `od` | REAL | NOT NULL | Outer diameter (inches) |
| `wt` | REAL | NOT NULL | Wall thickness (inches) |
| `length` | REAL | -- | Length (feet) |
| `weight_per_unit` | REAL | -- | Weight per unit length (lb/ft) |
| `end_type` | TEXT | -- | End type (SC/LC/BC/X, etc.) |
| `coupling_type` | TEXT | -- | Coupling type |
| `coupling_od` | REAL | -- | Coupling OD (inches) |
| `coupling_length` | REAL | -- | Coupling length (inches) |
| `heat_number` | TEXT | -- | Heat number for traceability |
| `serial_number` | TEXT | -- | Pipe body serial number |
| `manufacturer` | TEXT | -- | Manufacturer |
| `production_date` | TEXT | -- | Production date (ISO 8601) |
| `cert_number` | TEXT | -- | QC certificate number |
| `location_id` | INTEGER | -- | Current location ID |
| `status` | TEXT | NOT NULL, DEFAULT 'in_stock' | in_stock / outbound / scrapped |
| `notes` | TEXT | -- | Notes |
| `created_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | Created |
| `updated_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | Updated |
| `deleted_at` | TEXT | -- | Soft delete timestamp |

**Indexes:**

```sql
CREATE INDEX idx_seamless_pipes_grade ON seamless_pipes(grade);
CREATE INDEX idx_seamless_pipes_heat_number ON seamless_pipes(heat_number);
CREATE INDEX idx_seamless_pipes_status ON seamless_pipes(status);
CREATE INDEX idx_seamless_pipes_location_id ON seamless_pipes(location_id);
CREATE INDEX idx_seamless_pipes_pipe_type ON seamless_pipes(pipe_type);
CREATE INDEX idx_seamless_pipes_od_wt ON seamless_pipes(od, wt);
CREATE INDEX idx_seamless_pipes_manufacturer ON seamless_pipes(manufacturer);
CREATE INDEX idx_seamless_pipes_search ON seamless_pipes(grade, od, wt, status);
```

---

#### 5.3.2 screen_pipes — Screen Pipe Table

**Table**: `screen_pipes`

**Purpose**: Every individual screen pipe gets a row here.

| Field | Type | Constraint | Description |
|-------|------|------------|-------------|
| `id` | INTEGER | PK, AUTOINCREMENT | Primary key |
| `pipe_number` | TEXT | NOT NULL, UNIQUE | Pipe number (globally unique) |
| `batch_number` | TEXT | -- | Batch number from PO |
| `screen_type` | TEXT | NOT NULL | wire_wrapped / slotted / punched / metal_felt |
| `slot_size` | REAL | -- | Slot width / hole diameter (mm) |
| `filtration_grade` | TEXT | -- | Filtration precision (e.g. '150um', '250um') |
| `base_od` | REAL | NOT NULL | Base pipe OD (inches) |
| `base_wt` | REAL | NOT NULL | Base pipe WT (inches) |
| `base_grade` | TEXT | NOT NULL | Base pipe steel grade |
| `base_end_type` | TEXT | -- | Base pipe end type |
| `length` | REAL | -- | Screen pipe length (feet) |
| `weight_per_unit` | REAL | -- | Weight per unit length (lb/ft) |
| `heat_number` | TEXT | -- | Heat number |
| `serial_number` | TEXT | -- | Pipe body serial number |
| `manufacturer` | TEXT | -- | Manufacturer |
| `production_date` | TEXT | -- | Production date |
| `cert_number` | TEXT | -- | QC certificate number |
| `location_id` | INTEGER | -- | Current location ID |
| `status` | TEXT | NOT NULL, DEFAULT 'in_stock' | Status |
| `notes` | TEXT | -- | Notes |
| `created_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | Created |
| `updated_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | Updated |
| `deleted_at` | TEXT | -- | Soft delete |

**Indexes:**

```sql
CREATE INDEX idx_screen_pipes_type ON screen_pipes(screen_type);
CREATE INDEX idx_screen_pipes_heat_number ON screen_pipes(heat_number);
CREATE INDEX idx_screen_pipes_status ON screen_pipes(status);
CREATE INDEX idx_screen_pipes_location_id ON screen_pipes(location_id);
CREATE INDEX idx_screen_pipes_base_grade ON screen_pipes(base_grade);
```

---

#### 5.3.3 locations — Location Table

**Table**: `locations`

**Purpose**: Hierarchical zone / shelf / level management.

| Field | Type | Constraint | Description |
|-------|------|------------|-------------|
| `id` | INTEGER | PK, AUTOINCREMENT | Primary key |
| `zone_code` | TEXT | NOT NULL | Zone code |
| `shelf_code` | TEXT | NOT NULL | Shelf code |
| `level_code` | TEXT | NOT NULL | Level code |
| `full_code` | TEXT | NOT NULL, UNIQUE | Full code (zone-shelf-level) |
| `description` | TEXT | -- | Description |
| `max_capacity` | INTEGER | -- | Max pipes this location can hold |
| `current_usage` | INTEGER | NOT NULL, DEFAULT 0 | Current count |
| `is_active` | INTEGER | NOT NULL, DEFAULT 1 | Enabled? |
| `created_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | -- |
| `updated_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | -- |

**Indexes:**

```sql
CREATE UNIQUE INDEX idx_locations_full_code ON locations(full_code);
CREATE INDEX idx_locations_zone ON locations(zone_code);
```

---

#### 5.3.4 inbound_records — Inbound Record Header

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

#### 5.3.4a inbound_items — Inbound Line Items

**Table**: `inbound_items`

**Purpose**: Line items for inbound. One row per pipe. Supports batch inbound (N pipes same spec) and individual inbound.

| Field | Type | Constraint | Description |
|-------|------|------------|-------------|
| `id` | INTEGER | PK, AUTOINCREMENT | Primary key |
| `inbound_id` | INTEGER | NOT NULL | Associated header ID |
| `pipe_type` | TEXT | NOT NULL, CHECK IN ('seamless','screen') | Pipe type |
| `pipe_id` | INTEGER | NOT NULL | Pipe ID |
| `created_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | -- |

**Indexes:**

```sql
CREATE INDEX idx_inbound_items_inbound ON inbound_items(inbound_id);
CREATE INDEX idx_inbound_items_pipe ON inbound_items(pipe_type, pipe_id);
```

---

#### 5.3.5 outbound_records — Outbound Record Header

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

#### 5.3.5a outbound_items — Outbound Line Items

**Table**: `outbound_items`

**Purpose**: Line items for outbound. One row per pipe.

| Field | Type | Constraint | Description |
|-------|------|------------|-------------|
| `id` | INTEGER | PK, AUTOINCREMENT | Primary key |
| `outbound_id` | INTEGER | NOT NULL | Associated header ID |
| `pipe_type` | TEXT | NOT NULL, CHECK IN ('seamless','screen') | Pipe type |
| `pipe_id` | INTEGER | NOT NULL | Pipe ID |
| `created_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | -- |

**Indexes:**

```sql
CREATE INDEX idx_outbound_items_outbound ON outbound_items(outbound_id);
CREATE INDEX idx_outbound_items_pipe ON outbound_items(pipe_type, pipe_id);
```

---

#### 5.3.6 inventory_logs — Inventory Change Log

**Table**: `inventory_logs`

**Purpose**: Every inventory change for every pipe. The foundation of lifecycle traceability.

| Field | Type | Constraint | Description |
|-------|------|------------|-------------|
| `id` | INTEGER | PK, AUTOINCREMENT | Primary key |
| `pipe_type` | TEXT | NOT NULL, CHECK IN ('seamless','screen') | Pipe type |
| `pipe_id` | INTEGER | NOT NULL | Pipe ID |
| `change_type` | TEXT | NOT NULL | inbound / outbound / transfer / check_adjust |
| `reference_id` | INTEGER | -- | Associated doc ID |
| `reference_no` | TEXT | -- | Associated doc number |
| `operator_id` | INTEGER | -- | Operator ID |
| `operator_name` | TEXT | -- | Operator name (denormalized) |
| `remark` | TEXT | -- | Description |
| `created_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | Timestamp |

**Indexes:**

```sql
CREATE INDEX idx_inventory_logs_pipe ON inventory_logs(pipe_type, pipe_id);
CREATE INDEX idx_inventory_logs_type ON inventory_logs(change_type);
CREATE INDEX idx_inventory_logs_time ON inventory_logs(created_at);
CREATE INDEX idx_inventory_logs_operator ON inventory_logs(operator_id);
```

---

#### 5.3.7 suppliers — Supplier Table

**Table**: `suppliers`

| Field | Type | Constraint | Description |
|-------|------|------------|-------------|
| `id` | INTEGER | PK, AUTOINCREMENT | Primary key |
| `name` | TEXT | NOT NULL | Name |
| `contact_person` | TEXT | -- | Contact |
| `phone` | TEXT | -- | Phone |
| `email` | TEXT | -- | Email |
| `address` | TEXT | -- | Address |
| `qualification_cert` | TEXT | -- | Qualification cert number |
| `is_active` | INTEGER | NOT NULL, DEFAULT 1 | Enabled |
| `created_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | -- |
| `updated_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | -- |

**Indexes:**

```sql
CREATE INDEX idx_suppliers_name ON suppliers(name);
```

---

#### 5.3.8 customers — Customer Table

**Table**: `customers`

| Field | Type | Constraint | Description |
|-------|------|------------|-------------|
| `id` | INTEGER | PK, AUTOINCREMENT | Primary key |
| `name` | TEXT | NOT NULL | Name |
| `contact_person` | TEXT | -- | Contact |
| `phone` | TEXT | -- | Phone |
| `email` | TEXT | -- | Email |
| `address` | TEXT | -- | Address |
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
| `contract_id` | INTEGER | -- | Associated contract |
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
| `pipe_type` | TEXT | NOT NULL | Pipe type |
| `grade` | TEXT | NOT NULL | Grade |
| `od` | REAL | NOT NULL | OD |
| `wt` | REAL | NOT NULL | WT |
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
| `contract_id` | INTEGER | -- | Associated contract |
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
| `pipe_type` | TEXT | NOT NULL | Pipe type |
| `grade` | TEXT | NOT NULL | Grade |
| `od` | REAL | NOT NULL | OD |
| `wt` | REAL | NOT NULL | WT |
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

#### 5.3.13a contract_items — Contract Line Items

**Table**: `contract_items`

| Field | Type | Constraint | Description |
|-------|------|------------|-------------|
| `id` | INTEGER | PK, AUTOINCREMENT | Primary key |
| `contract_id` | INTEGER | NOT NULL | Parent contract ID |
| `pipe_type` | TEXT | NOT NULL | seamless / screen |
| `grade` | TEXT | NOT NULL | Steel grade |
| `od` | REAL | NOT NULL | Outer diameter |
| `wt` | REAL | NOT NULL | Wall thickness |
| `quantity` | INTEGER | NOT NULL | Quantity |
| `unit_price` | REAL | -- | Unit price |
| `total_price` | REAL | -- | Line total |
| `notes` | TEXT | -- | -- |
| `created_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | -- |

---

#### 5.3.13b contract_payments — Contract Payment Milestones

**Table**: `contract_payments`

| Field | Type | Constraint | Description |
|-------|------|------------|-------------|
| `id` | INTEGER | PK, AUTOINCREMENT | Primary key |
| `contract_id` | INTEGER | NOT NULL | Parent contract ID |
| `due_date` | TEXT | NOT NULL | Payment due date |
| `amount` | REAL | NOT NULL | Payment amount |
| `payment_type` | TEXT | NOT NULL | deposit / milestone / final |
| `is_paid` | INTEGER | NOT NULL, DEFAULT 0 | 0 = unpaid, 1 = paid |
| `paid_date` | TEXT | -- | Actual payment date |
| `notes` | TEXT | -- | -- |
| `created_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | -- |

---

#### 5.3.14 quality_certs — Quality Certificate Table

**Table**: `quality_certs`

| Field | Type | Constraint | Description |
|-------|------|------------|-------------|
| `id` | INTEGER | PK, AUTOINCREMENT | Primary key |
| `pipe_type` | TEXT | NOT NULL | seamless / screen |
| `pipe_id` | INTEGER | NOT NULL | Pipe ID |
| `cert_no` | TEXT | NOT NULL | Cert number |
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
CREATE INDEX idx_quality_certs_pipe ON quality_certs(pipe_type, pipe_id);
CREATE INDEX idx_quality_certs_cert_no ON quality_certs(cert_no);
```

---

#### 5.3.15 users — User Table

**Table**: `users`

| Field | Type | Constraint | Description |
|-------|------|------------|-------------|
| `id` | INTEGER | PK, AUTOINCREMENT | Primary key |
| `username` | TEXT | NOT NULL, UNIQUE | Login username |
| `password_hash` | TEXT | NOT NULL | Argon2 hash |
| `display_name` | TEXT | NOT NULL | Display name |
| `email` | TEXT | -- | Email |
| `role` | TEXT | NOT NULL | admin / warehouse / qc / sales |
| `language_pref` | TEXT | NOT NULL, DEFAULT 'zh' | zh / en |
| `unit_system` | TEXT | NOT NULL, DEFAULT 'metric' | metric / imperial |
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

#### 5.3.16 operation_logs — Operation Log Table

**Table**: `operation_logs`

**Purpose**: Audit trail for all critical data changes.

| Field | Type | Constraint | Description |
|-------|------|------------|-------------|
| `id` | INTEGER | PK, AUTOINCREMENT | Primary key |
| `user_id` | INTEGER | -- | User ID |
| `username` | TEXT | -- | Username (denormalized) |
| `action` | TEXT | NOT NULL | create / update / delete / login / export |
| `target_type` | TEXT | NOT NULL | pipe / inbound / outbound / order / user, etc. |
| `target_id` | INTEGER | -- | Target object ID |
| `target_summary` | TEXT | -- | Summary (e.g. pipe number) |
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

#### 5.3.17 pipe_attachments — Pipe Attachments Table

**Table**: `pipe_attachments`

**Purpose**: Files/photos linked to pipes (receipt photos, defect photos, inspection reports). One-to-many with pipes.

| Field | Type | Constraint | Description |
|-------|------|------------|-------------|
| `id` | INTEGER | PK, AUTOINCREMENT | Primary key |
| `pipe_type` | TEXT | NOT NULL | seamless / screen |
| `pipe_id` | INTEGER | NOT NULL | Pipe ID |
| `file_name` | TEXT | NOT NULL | Original filename |
| `file_path` | TEXT | NOT NULL | Storage path |
| `file_type` | TEXT | -- | image / pdf / other |
| `file_size` | INTEGER | -- | Size in bytes |
| `uploaded_by` | INTEGER | -- | Uploader user ID |
| `created_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | -- |

**Indexes:**

```sql
CREATE INDEX idx_pipe_attachments_target ON pipe_attachments(pipe_type, pipe_id);
```

---

#### 5.3.18 api_5ct_grade_ref — API 5CT Grade Reference Table

**Table**: `api_5ct_grade_ref`

**Purpose**: Mechanical properties and chemical composition reference data per API 5CT standard grades. Used for QC comparison and validation.

| Field | Type | Constraint | Description |
|-------|------|------------|-------------|
| `id` | INTEGER | PK, AUTOINCREMENT | Primary key |
| `grade` | TEXT | NOT NULL, UNIQUE | Grade code |
| `grade_group` | TEXT | -- | H/J/K/N/L/C/T/P/Q |
| `pipe_type` | TEXT | -- | casing / tubing / both |
| `min_yield_strength_psi` | INTEGER | -- | Min yield (psi) |
| `max_yield_strength_psi` | INTEGER | -- | Max yield (psi) |
| `min_tensile_strength_psi` | INTEGER | -- | Min tensile (psi) |
| `hardness_max` | REAL | -- | Max hardness (HRC) |
| `notes` | TEXT | -- | Environment notes |
| `created_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | -- |

---

#### 5.3.19 inventory_check_records — Inventory Check Record Table

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

#### 5.3.20 inventory_check_items — Inventory Check Line Items

**Table**: `inventory_check_items`

**Purpose**: One row per pipe per check. System pre-populates expected pipes, checker confirms each one. Each pipe is either "found" or "missing" — no quantity math.

| Field | Type | Constraint | Description |
|-------|------|------------|-------------|
| `id` | INTEGER | PK, AUTOINCREMENT | Primary key |
| `check_id` | INTEGER | NOT NULL | Check record ID |
| `pipe_type` | TEXT | NOT NULL | Pipe type |
| `pipe_id` | INTEGER | NOT NULL | Pipe ID |
| `expected` | INTEGER | NOT NULL, DEFAULT 1 | Expected flag (1 = should be here) |
| `found` | INTEGER | -- | NULL=not checked, 1=found, 0=missing |
| `notes` | TEXT | -- | Discrepancy description |

**Indexes:**

```sql
CREATE INDEX idx_check_items_check ON inventory_check_items(check_id);
CREATE INDEX idx_check_items_status ON inventory_check_items(check_id, found);
```

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

┌───────────────┐     ┌──────────────────┐    ┌─────────────────┐
│ seamless_pipes │──N:1│    locations      │1:N─│  screen_pipes    │
└───────┬───────┘     └──────────────────┘    └────────┬────────┘
        │                                             │
        │ 1:N                                          │ 1:N
        ▼                                             ▼
┌─────────────────────────────────────────────────────────┐
│               inventory_logs                              │
│     (Records all inventory changes for both pipe types)   │
└─────────────────────────────────────────────────────────┘

┌────────────────┐     ┌──────────────────┐     ┌─────────────────┐
│ inbound_records │──1:N│   inbound_items   │──N:1│  seamless_pipes  │
│   (Header)      │     │   (Line Items)    │     │  / screen_pipes  │
└────────────────┘     └──────────────────┘     └─────────────────┘

┌────────────────┐     ┌──────────────────┐     ┌─────────────────┐
│ outbound_records│──1:N│  outbound_items  │──N:1│  seamless_pipes  │
│   (Header)      │     │   (Line Items)    │     │  / screen_pipes  │
└────────────────┘     └──────────────────┘     └─────────────────┘

┌───────────────┐     ┌──────────────────┐    ┌─────────────────┐
│ seamless_pipes │─────│   quality_certs   │────│  screen_pipes    │
└───────────────┘     └──────────────────┘    └─────────────────┘
                              │
                              ▼
                    ┌──────────────────┐
                    │ api_5ct_grade_ref │
                    └──────────────────┘

┌──────────┐       ┌──────────────────┐       ┌─────────────────┐
│   users   │──1:N──│  operation_logs   │       │     contracts    │
└──────────┘       └──────────────────┘       └────────┬────────┘
                                                        │ 1:N
                                          ┌─────────────┴─────────────┐
                                          ▼                           ▼
                                ┌──────────────────┐      ┌──────────────────┐
                                │  contract_items   │      │ contract_payments │
                                └──────────────────┘      └──────────────────┘
```

---

## 6. REST API Design

### 6.1 API Base Spec

**Base path**: `/api/v1`

**Response format**:

```json
// Success
{
  "success": true,
  "data": { ... },
  "meta": {
    "page": 1,
    "page_size": 20,
    "total": 100
  },
  "request_id": "req_xxxxx"
}

// Error
{
  "success": false,
  "error": {
    "code": "PIPE_NOT_FOUND",
    "message": "Pipe not found",
    "details": { "pipe_id": 123 }
  },
  "request_id": "req_xxxxx"
}
```

**Auth**: `Authorization: Bearer <jwt_token>`

**Pagination params**:

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `page` | integer | 1 | Page number |
| `page_size` | integer | 20 | Items per page (max 100) |

### 6.2 Pipe Management API

#### 6.2.1 Seamless Steel Pipes

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/v1/seamless-pipes` | List with filters |
| `POST` | `/api/v1/seamless-pipes` | Create |
| `GET` | `/api/v1/seamless-pipes/{id}` | Get details |
| `PUT` | `/api/v1/seamless-pipes/{id}` | Update |
| `DELETE` | `/api/v1/seamless-pipes/{id}` | Soft delete |

**GET /api/v1/seamless-pipes query params**:

| Parameter | Type | Description |
|-----------|------|-------------|
| `q` | string | Fuzzy search (pipe_number / heat_number / serial_number) |
| `grade` | string | Exact grade match |
| `pipe_type` | string | casing / tubing |
| `od_min` / `od_max` | number | OD range |
| `wt_min` / `wt_max` | number | WT range |
| `status` | string | Stock status |
| `location_id` | integer | Location |
| `manufacturer` | string | Manufacturer |
| `sort_by` | string | Sort field (whitelist: `created_at`, `pipe_number`, `grade`, `od`, `wt`, `status`) |
| `sort_order` | string | asc / desc |

**POST /api/v1/seamless-pipes body**:

```json
{
  "pipe_number": "CSG-4.5-J55-001",
  "pipe_type": "casing",
  "grade": "J55",
  "od": 4.5,
  "wt": 0.250,
  "length": 40.0,
  "weight_per_unit": 11.60,
  "end_type": "SC",
  "coupling_type": "standard",
  "coupling_od": 5.0,
  "coupling_length": 8.0,
  "heat_number": "HT20260501-01",
  "serial_number": "SN-20260501-001",
  "manufacturer": "Some Steel Mill",
  "production_date": "2026-05-01",
  "cert_number": "QC-20260501-001",
  "notes": ""
}
```

**GET /api/v1/seamless-pipes/{id} response**:

```json
{
  "success": true,
  "data": {
    "id": 1,
    "pipe_number": "CSG-4.5-J55-001",
    "pipe_type": "casing",
    "grade": "J55",
    "od": 4.5,
    "wt": 0.250,
    "length": 40.0,
    "weight_per_unit": 11.60,
    "end_type": "SC",
    "coupling_type": "standard",
    "coupling_od": 5.0,
    "coupling_length": 8.0,
    "heat_number": "HT20260501-01",
    "serial_number": "SN-20260501-001",
    "manufacturer": "Some Steel Mill",
    "production_date": "2026-05-01",
    "cert_number": "QC-20260501-001",
    "location": { "id": 1, "full_code": "A-01-01" },
    "status": "in_stock",
    "created_at": "2026-05-19T10:00:00Z",
    "updated_at": "2026-05-19T10:00:00Z"
  }
}
```

#### 6.2.2 Screen Pipes

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/v1/screen-pipes` | List |
| `POST` | `/api/v1/screen-pipes` | Create |
| `GET` | `/api/v1/screen-pipes/{id}` | Details |
| `PUT` | `/api/v1/screen-pipes/{id}` | Update |
| `DELETE` | `/api/v1/screen-pipes/{id}` | Soft delete |

Query params similar to seamless, plus screen-specific fields (`screen_type`, `slot_size`, etc.)

#### 6.2.3 Unified Pipe Search

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/v1/pipes/search` | Cross-type search (seamless + screen) |

```json
// Response
{
  "success": true,
  "data": {
    "seamless_pipes": [ ... ],
    "screen_pipes": [ ... ]
  },
  "meta": { "total_seamless": 50, "total_screen": 10 }
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
  "inbound_date": "2026-05-19",
  "order_id": 1,
  "supplier_id": 1,
  "operator_id": 1,
  "remark": "Purchase inbound",
  "pipes": [
    { "pipe_type": "seamless", "pipe_id": 1 },
    { "pipe_type": "seamless", "pipe_id": 2 }
  ]
}
```

**POST /api/v1/inbound-records — Non-Purchase (needs approval)**:

```json
{
  "inbound_type": "production",
  "inbound_date": "2026-05-19",
  "supplier_id": 1,
  "operator_id": 1,
  "remark": "Production inbound",
  "pipes": [
    { "pipe_type": "seamless", "pipe_id": 100 }
  ]
}
```

> **Approval flow**: production/return inbound are created as `pending`, must be approved before stock is updated.
> **Batch inbound**: If `pipes` is empty + `batch_create` provided, system auto-creates N pipes of same spec.

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/api/v1/inbound-records/batch` | Batch create pipes + single-step inbound |

```json
{
  "inbound_type": "purchase",
  "inbound_date": "2026-05-19",
  "order_id": 1,
  "supplier_id": 1,
  "operator_id": 1,
  "remark": "Batch purchase inbound",
  "pipe_spec": {
    "pipe_type": "seamless",
    "grade": "J55",
    "od": 4.5,
    "wt": 0.250,
    "length": 40.0,
    "heat_number": "HT20260501-01",
    "manufacturer": "Some Steel Mill"
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
  "outbound_date": "2026-05-19",
  "order_id": 1,
  "customer_id": 1,
  "operator_id": 1,
  "remark": "Sales outbound",
  "pipes": [
    { "pipe_type": "seamless", "pipe_id": 1 },
    { "pipe_type": "seamless", "pipe_id": 2 }
  ]
}
```

**POST /api/v1/outbound-records — Non-Sales (needs approval)**:

```json
{
  "outbound_type": "scrapped",
  "outbound_date": "2026-05-19",
  "operator_id": 1,
  "remark": "Scrapped outbound",
  "pipes": [
    { "pipe_type": "seamless", "pipe_id": 50 }
  ]
}
```

#### 6.3.3 Real-Time Inventory Query

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/v1/inventory` | Aggregated stock list |
| `GET` | `/api/v1/inventory/statistics` | Statistics summary |
| `GET` | `/api/v1/inventory/logs` | Transaction logs |

**GET /api/v1/inventory query params**:

| Parameter | Description |
|-----------|-------------|
| `pipe_type` | seamless / screen |
| `grade` | Grade |
| `od` | OD |
| `wt` | WT |
| `status` | Status |
| `location_id` | Location |

**Response**:

```json
{
  "success": true,
  "data": [
    {
      "pipe_type": "seamless",
      "grade": "J55",
      "od": 4.5,
      "wt": 0.250,
      "total_quantity": 500,
      "in_stock": 480,
      "outbound": 20,
      "locations": ["A-01-01", "A-01-02"]
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

#### 6.3.5 Location Management

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/v1/locations` | List |
| `POST` | `/api/v1/locations` | Create |
| `PUT` | `/api/v1/locations/{id}` | Update |
| `DELETE` | `/api/v1/locations/{id}` | Delete |
| `POST` | `/api/v1/locations/{id}/assign` | Bind pipe to location |
| `POST` | `/api/v1/pipes/{pipe_id}/transfer-location` | Transfer pipe |

### 6.4 Quality Management API

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/v1/quality-certs` | List |
| `POST` | `/api/v1/quality-certs` | Create |
| `GET` | `/api/v1/quality-certs/{id}` | Details |
| `PUT` | `/api/v1/quality-certs/{id}` | Update |
| `DELETE` | `/api/v1/quality-certs/{id}` | Delete |
| `GET` | `/api/v1/trace/heat-number/{heat_no}` | Trace by heat number |
| `GET` | `/api/v1/trace/pipe-number/{pipe_no}` | Trace by pipe number |
| `GET` | `/api/v1/api-5ct-grades` | API 5CT grade reference |
| `GET` | `/api/v1/api-5ct-grades/{grade}` | Single grade reference |

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
| `POST` | `/api/v1/purchase-orders/{id}/approve` | Approve |
| `POST` | `/api/v1/purchase-orders/{id}/reject` | Reject |
| `POST` | `/api/v1/purchase-orders/{id}/link-inbound` | Link to inbound |

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
| `POST` | `/api/v1/sales-orders/{id}/approve` | Approve |
| `POST` | `/api/v1/sales-orders/{id}/reject` | Reject |
| `GET` | `/api/v1/atp` | ATP query (pipe_type, grade, od, wt) |
| `POST` | `/api/v1/sales-orders/{id}/link-outbound` | Link to outbound |

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
| `POST` | `/api/v1/import/seamless-pipes` | Import seamless pipes (Excel/CSV) |
| `POST` | `/api/v1/import/screen-pipes` | Import screen pipes |
| `POST` | `/api/v1/import/inventory` | Import inventory data |
| `GET` | `/api/v1/export/seamless-pipes` | Export seamless pipes |
| `GET` | `/api/v1/export/screen-pipes` | Export screen pipes |
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
      { "row": 23, "reason": "Duplicate pipe number: CSG-4.5-J55-001" },
      { "row": 67, "reason": "Grade 'H50' is not in the allowed range" }
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
      "unit_system": "metric"
    }
  }
}
```

#### Operation Logs

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/v1/operation-logs` | List with filters |

**Query params**: `user_id`, `action`, `target_type`, `date_from`/`date_to`, `q`

### 6.10 Reports & Statistics API (P2)

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/v1/reports/inventory-summary` | Inventory summary |
| `GET` | `/api/v1/reports/inventory-monthly` | Monthly changes |
| `GET` | `/api/v1/reports/in-out-statistics` | In/out stats |
| `GET` | `/api/v1/reports/turnover-rate` | Turnover rate |

### 6.11 Label Printing API (P2)

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/api/v1/labels/generate` | Generate label (PDF/image) |
| `POST` | `/api/v1/labels/batch-generate` | Batch generate |

---

## 7. Project Directory Structure

### 7.1 Backend (Rust)

```
pipe-management/
├── Cargo.toml                    # Dependencies
├── Cargo.lock
├── .env                          # Env vars (DB path, JWT Secret, etc.)
├── .env.example                  # Template
├── sqlx-data.json                # SQLx offline check data
├── migrations/
│   ├── 001_create_users.sql
│   ├── 002_create_seamless_pipes.sql
│   ├── 003_create_screen_pipes.sql
│   ├── 004_create_locations.sql
│   ├── 005_create_inventory.sql
│   ├── 006_create_orders.sql
│   ├── 007_create_quality.sql
│   ├── 008_create_logs.sql
│   ├── 009_create_ref_data.sql
│   ├── 010_seed_api_5ct_data.sql
│   └── 011_add_rejection_reason.sql
├── src/
│   ├── main.rs                   # Entry: server startup, config
│   ├── lib.rs                    # Library entry
│   ├── config.rs                 # Config (env → Config struct)
│   ├── router.rs                 # Route registration (~70 endpoints)
│   ├── error.rs                  # AppError enum
│   ├── response.rs               # ApiResponse, PaginatedResponse
│   ├── middleware/
│   │   ├── mod.rs
│   │   ├── auth.rs               # JWT auth middleware
│   │   ├── rbac.rs               # Role-based access control
│   │   ├── logging.rs            # Request logging
│   │   └── request_id.rs         # Request ID generation
│   ├── domain/
│   │   ├── mod.rs
│   │   ├── pipe.rs               # Pipe enums, constants
│   │   ├── api_5ct.rs            # API 5CT ref data
│   │   └── error.rs              # Domain error types
│   ├── models/
│   │   ├── mod.rs
│   │   ├── seamless_pipe.rs
│   │   ├── screen_pipe.rs
│   │   ├── location.rs
│   │   ├── inbound.rs
│   │   ├── outbound.rs
│   │   ├── inventory_log.rs
│   │   ├── supplier.rs
│   │   ├── customer.rs
│   │   ├── purchase_order.rs
│   │   ├── sales_order.rs
│   │   ├── contract.rs
│   │   ├── quality_cert.rs
│   │   ├── inventory_check.rs
│   │   ├── user.rs
│   │   └── operation_log.rs
│   ├── dto/
│   │   ├── mod.rs
│   │   ├── pipe_dto.rs
│   │   ├── inventory_dto.rs
│   │   ├── order_dto.rs
│   │   ├── auth_dto.rs
│   │   └── common.rs
│   ├── handlers/                 # 13 handler files
│   │   ├── mod.rs
│   │   ├── pipe_handler.rs
│   │   ├── inventory_handler.rs
│   │   ├── quality_handler.rs
│   │   ├── purchase_handler.rs
│   │   ├── sales_handler.rs
│   │   ├── supplier_handler.rs
│   │   ├── customer_handler.rs
│   │   ├── contract_handler.rs
│   │   ├── report_handler.rs
│   │   ├── label_handler.rs
│   │   ├── data_io_handler.rs
│   │   ├── auth_handler.rs
│   │   └── atp_handler.rs
│   ├── services/                 # 12 service files
│   │   ├── mod.rs
│   │   ├── pipe_service.rs
│   │   ├── inventory_service.rs
│   │   ├── quality_service.rs
│   │   ├── purchase_sales_service.rs
│   │   ├── contract_service.rs
│   │   ├── customer_service.rs
│   │   ├── supplier_service.rs
│   │   ├── label_service.rs
│   │   ├── report_service.rs
│   │   ├── data_io_service.rs
│   │   ├── auth_service.rs
│   │   └── trace_service.rs
│   └── repositories/            # 13 repo files
│       ├── mod.rs
│       ├── pipe_repo.rs
│       ├── inventory_repo.rs
│       ├── purchase_order_repo.rs
│       ├── sales_order_repo.rs
│       ├── quality_repo.rs
│       ├── contract_repo.rs
│       ├── customer_repo.rs
│       ├── supplier_repo.rs
│       ├── label_repo.rs
│       ├── report_repo.rs
│       ├── data_io_repo.rs
│       ├── user_repo.rs
│       └── operation_log_repo.rs
└── tests/
    ├── common/
    │   ├── mod.rs
    │   └── test_db.rs
    ├── pipe_tests.rs
    ├── inventory_tests.rs
    ├── order_tests.rs
    ├── auth_tests.rs
    └── api_integration_tests.rs
```

### 7.2 Frontend (React 19)

```
pipe-management-frontend/
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
│   ├── features/                   # 11 feature modules
│   │   ├── auth/                   # Login, authStore, authApi
│   │   ├── pipes/                  # Seamless/screen pipe CRUD, search
│   │   ├── inventory/              # Inbound/outbound/stock/check/locations
│   │   ├── quality/                # QC certs, traceability, API 5CT ref
│   │   ├── purchases/              # POs, suppliers
│   │   ├── sales/                  # SOs, customers, ATP
│   │   ├── contracts/              # Contract management
│   │   ├── reports/                # Dashboard, inventory reports, trends
│   │   ├── labels/                 # Label printing
│   │   ├── data-io/                # Excel/CSV import/export
│   │   └── system/                 # Users, operation logs, profile
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
| `409` | Conflict (e.g. duplicate pipe number) |
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

    // Pipes (120xx)
    PipeNotFound,           // 12001
    PipeNumberDuplicate,    // 12002
    PipeStatusConflict,     // 12003

    // Inventory (130xx)
    InsufficientStock,      // 13001
    LocationFull,           // 13002
    LocationNotFound,       // 13003

    // Orders (140xx)
    OrderCannotModify,      // 14001
    OrderNotFound,          // 14002
    OrderNotApproved,       // 14003

    // Inventory (130xx continued)
    InboundOrderIdRequired,         // 13004
    OutboundOrderIdRequired,        // 13005
    InboundNotApprovedYet,          // 13006
    OutboundNotApprovedYet,         // 13007
    InboundAlreadyApproved,         // 13008
    OutboundAlreadyApproved,        // 13009
    InboundItemMismatch,            // 13010
    OutboundItemMismatch,           // 13011
    InboundApprovalNotAllowed,      // 13012
    OutboundApprovalNotAllowed,     // 13013

    // Import/Export (150xx)
    ImportFileParseError,   // 15001
    ImportValidationError,  // 15002
}
```

### 8.3 Global Error Handling

All errors are caught by Axum's `FromRequestParts` or middleware and converted to the unified `ApiResponse::error(...)` format. Every response includes `success: false` and `request_id` — no exceptions.

---

## 9. Non-Functional Design

### 9.1 Performance

| Metric | Approach |
|--------|----------|
| **Query ≤ 2s (100k rows)** | Proper indexes, pagination, avoid N+1 |
| **Import 100k rows ≤ 60s** | Batch inserts (1000 rows/transaction) |
| **20+ concurrent users** | SQLite WAL for concurrent reads; write serialization via `busy_timeout` |
| **Large queries** | Materialized aggregate views or app-level caching if needed |

**SQLite Concurrency**:

```
1. WAL mode: reads don't block writes
2. Writes serialized via Tokio Mutex to avoid SQLITE_BUSY
3. Connection pool: deadpool-sqlite, max 5 connections
4. Long writes split into smaller batch transactions
5. Periodic PRAGMA wal_checkpoint(TRUNCATE) to limit WAL size
```

### 9.2 Config Management

```rust
pub struct AppConfig {
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_expires_in: u64,
    pub host: String,
    pub port: u16,
    pub log_level: String,
    pub upload_dir: String,
    pub max_file_size: usize,
    pub default_language: String,
    pub default_unit_system: String,
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
| **API auth** | JWT Bearer middleware |
| **RBAC** | Axum middleware extracts role from token, matches route permissions |
| **Sensitive ops** | DELETE/critical mods require confirmation (frontend + backend audit) |

### 10.2 Role Permission Matrix

| Feature / Role | Admin | Warehouse | QC | Sales |
|----------------|:-----:|:---------:|:--:|:-----:|
| Pipe view | ✅ | ✅ | ✅ | ✅ |
| Pipe create/update/delete | ✅ | ✅ | ✅ | -- |
| Inbound operations | ✅ | ✅ | -- | ✅ (purchase only) |
| Outbound operations | ✅ | ✅ | -- | ✅ (sales only) |
| Inbound approval | ✅ | ✅ | -- | -- |
| Outbound approval | ✅ | ✅ | -- | -- |
| Inventory query | ✅ | ✅ | ✅ | ✅ |
| Inventory check | ✅ | ✅ | -- | -- |
| Location management | ✅ | ✅ | -- | -- |
| Quality management | ✅ | -- | ✅ | -- |
| Purchase orders | ✅ | -- | -- | ✅ |
| Sales orders | ✅ | -- | -- | ✅ |
| Suppliers/Customers | ✅ | -- | -- | ✅ |
| Import/Export | ✅ | ✅ | ✅ | ✅ |
| Reports | ✅ | ✅ | ✅ | ✅ |
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

**Backend**: Error messages and status labels returned based on user's `language_pref` setting or `Accept-Language` header. Currently the i18n is primarily on the frontend side via `react-i18next`.

**Frontend** (`react-i18next`):

```
i18n/
├── resources/
│   ├── zh/                     # Chinese
│   │   ├── common.json
│   │   ├── pipes.json
│   │   ├── inventory.json
│   │   ├── quality.json
│   │   ├── purchase.json
│   │   ├── sales.json
│   │   ├── system.json
│   │   └── validation.json
│   └── en/                     # English
│       ├── common.json
│       ├── pipes.json
│       ├── inventory.json
│       ├── quality.json
│       ├── purchase.json
│       ├── sales.json
│       ├── system.json
│       └── validation.json
```

Namespaces are per-feature, lazy-loaded. Switching language updates `language_pref` and triggers re-render via `useTranslation()`.

### 11.2 Unit Switching

**Strategy**: Database stores everything in **imperial units** (API 5CT native). Conversion happens at API layer based on user preference.

| Parameter | Storage (Imperial) | Metric Display | Formula |
|-----------|-------------------|----------------|---------|
| OD | inch | mm | mm = in × 25.4 |
| WT | inch | mm | mm = in × 25.4 |
| Length | foot | m | m = ft × 0.3048 |
| Weight | lb/ft | kg/m | kg/m = lb/ft × 1.48816 |
| Yield | psi | MPa | MPa = psi × 0.00689476 |

**API**:

```json
// Request optionally specifies unit system
POST /api/v1/seamless-pipes?unit_system=metric

// Response includes unit markers
{
  "data": {
    "od": 114.3,
    "od_unit": "mm",
    "wt": 6.35,
    "wt_unit": "mm",
    "length": 12.19,
    "length_unit": "m"
  }
}
```

**Implementation**:
- `Measurement<T>` wrapper type at DTO layer carries unit info
- Auth middleware extracts user's unit preference
- Handler layer converts based on preference
- Frontend can also do client-side conversion for display

---

## Appendix A: Key Decision Records (ADR)

### ADR-001: Monolith over Microservices

| Item | Content |
|------|---------|
| **Context** | Medium project, small team |
| **Decision** | Modular monolith organized by domain |
| **Why** | Less operational complexity; SQLite doesn't do distributed; can split later if needed |
| **Cost** | Must enforce module boundaries rigorously |

### ADR-002: SQLite over PostgreSQL

| Item | Content |
|------|---------|
| **Context** | 100K+ rows, 20+ concurrent users |
| **Decision** | SQLite WAL + connection pool + write serialization |
| **Why** | Zero config, file-level, plenty for this scale |
| **Cost** | If scale explodes, SQLx makes migration to PostgreSQL straightforward |

### ADR-003: Imperial Units as Internal Standard

| Item | Content |
|------|---------|
| **Context** | API 5CT is imperial; Chinese users prefer metric |
| **Decision** | Store imperial in DB, convert at API layer |
| **Why** | Avoid precision loss from back-and-forth conversion; spec-native |
| **Cost** | All internal math is imperial; conversion at boundaries |

### ADR-004: Separate Tables for Seamless and Screen Pipes

| Item | Content |
|------|---------|
| **Context** | Both are pipes but have different fields |
| **Decision** | Two independent tables |
| **Why** | Fields are significantly different (screen pipes have base pipe params, filtration specs, etc.); cross-type queries are rare; separate tables are cleaner |
| **Cost** | Cross-type search needs UNION or two queries + merge |

---

## Appendix B: API 5CT Grade Reference Data

Pre-seeded in `migrations/010_seed_api_5ct_data.sql`:

```sql
INSERT INTO api_5ct_grade_ref (grade, grade_group, pipe_type, min_yield_strength_psi, max_yield_strength_psi, min_tensile_strength_psi, notes) VALUES
('H40',  'H', 'both',   40000, 80000, 60000, 'Minimum strength, non-critical wells'),
('J55',  'J', 'both',   55000, 80000, 75000, 'Medium strength, medium-depth wells'),
('K55',  'K', 'both',   55000, 80000, 95000, 'Medium strength, medium-depth wells'),
('N80-1','N', 'both',   80000, 110000, 100000, 'Higher strength, N80-1 normalized'),
('N80-Q','N', 'both',   80000, 110000, 100000, 'Higher strength, N80-Q quenched + tempered'),
('L80-1','L', 'both',   80000, 95000, 95000, 'Corrosion resistant, contains Cr, for H2S environments'),
('C90',  'C', 'both',   90000, 105000, 100000, 'Corrosion resistant, for sour environments'),
('C95',  'C', 'both',   95000, 110000, 105000, 'Corrosion resistant, for sour environments'),
('T95',  'T', 'casing', 95000, 110000, 105000, 'High collapse resistance, suitable for sour environments'),
('P110', 'P', 'both',   110000, 140000, 125000, 'High strength, deep wells'),
('Q125', 'Q', 'casing', 125000, 150000, 135000, 'Ultra-high strength, ultra-deep wells');
```

---
