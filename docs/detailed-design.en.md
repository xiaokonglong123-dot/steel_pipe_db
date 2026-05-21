# Seamless Steel Pipe & Screen Pipe Management System — Detailed Design Document

> **Document Version**: v1.1
> **Created**: 2026-05-19
> **Based on Requirements Document**: docs/需求文档.md v1.0
> **Tech Stack**: Rust + Axum + SQLx + SQLite (WAL) | React (Frontend)

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
8. [Error Handling & Response Specification](#8-error-handling--response-specification)
9. [Non-Functional Design](#9-non-functional-design)
10. [Security Design](#10-security-design)
11. [Internationalization & Unit Switching Design](#11-internationalization--unit-switching-design)

---

## 1. System Overview

### 1.1 Project Positioning

An integrated web-based inventory management system for seamless steel pipes (Casing / Tubing) and screen pipes, covering the full data tracking lifecycle from pipe receipt to dispatch, based on API 5CT / ISO 11960 standards.

### 1.2 Core Capabilities

| Capability | Description |
|------------|-------------|
| Full pipe lifecycle management | Complete tracking from procurement receipt to sales dispatch |
| Integrated inventory management | Linked management of procurement, inventory, and sales |
| Quality traceability | Traceability by heat number / pipe number for quality inspection |
| Multi-user RBAC | Four roles: warehouse manager, quality inspector, sales/procurement staff, administrator |
| Internationalization | Chinese/English interface + metric/imperial unit switching |

---

## 2. Tech Stack Decisions

### 2.1 Backend Technology Selection

| Layer | Selection | Version | Rationale |
|-------|-----------|---------|-----------|
| **Web Framework** | Axum | 0.8+ | The most mainstream async web framework in the Rust ecosystem, built on Tower middleware layer, active ecosystem |
| **ORM / SQL** | SQLx | 0.8+ | Compile-time SQL checking, non-intrusive, natively compatible with SQLite |
| **Database** | SQLite (WAL mode) | 3.46+ | Zero configuration, file-level database; WAL mode supports concurrent read/write |
| **Serialization** | Serde + Serde JSON | 1.x | Standard Rust serialization framework |
| **Authentication** | Axum + JWT (jsonwebtoken) | — | Stateless authentication, suitable for frontend-backend separation |
| **Password Hashing** | Argon2 | — | OWASP recommended password hashing algorithm |
| **Async Runtime** | Tokio | 1.x | Standard Rust async runtime |
| **Parameter Validation** | Validator | 0.18+ | Derive macro-based struct validation |
| **Logging** | Tracing + Tracing-Subscriber | — | Structured logging, integrated with Tokio ecosystem |
| **File Upload** | Axum multipart + tokio-util | — | Handles quality inspection file uploads |
| **Internationalization** | rust-i18n / Fluent | — | Chinese/English message templates |
| **Export/Import** | calamine (read) + xlsxwriter (write) | — | Excel file read/write |

### 2.2 Frontend Technology Selection (TBD)

Frontend design will be discussed separately after the backend architecture is finalized. Preliminary intent:

| Layer | Selection | Description |
|-------|-----------|-------------|
| **UI Framework** | React 19 | — |
| **Routing** | React Router 7 | — |
| **State Management** | Zustand + TanStack Query | Client state + server state caching |
| **UI Component Library** | Ant Design 5 | Enterprise-grade component library, suitable for admin dashboards |
| **Internationalization** | react-i18next | — |
| **HTTP Client** | Axios | — |
| **Table** | @tanstack/react-table | Virtual scrolling for large datasets |

### 2.3 Architecture Style

**RESTful frontend-backend separation**: The backend provides a JSON REST API, the frontend calls it over HTTP.

**Monolithic backend architecture** (not microservices): Given the project scale and team size, a modular monolith is adopted, internally organized by business domain.

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
│  (HTTP Routing + Request Parsing + Response Serialization + Auth Checks) │
│  Responsibilities: Parameter validation, call Service, return JSON      │
├──────────────────────────────────────────────────────┤
│                    Service Layer                       │
│  (Business Logic Orchestration)                        │
│  Responsibilities: CRUD orchestration, transaction management, permission checks, logging │
├──────────────────────────────────────────────────────┤
│                    Repository Layer                    │
│  (Data Access + SQL Queries)                           │
│  Responsibilities: Execute SQLx queries, data mapping, pagination & sorting │
├──────────────────────────────────────────────────────┤
│                    Domain Layer                        │
│  (Data Models + Type Definitions)                      │
│  Responsibilities: Struct definitions, enums, business constants, API 5CT reference data │
└──────────────────────────────────────────────────────┘
```

### 3.3 Module Dependency Relationships

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
        │   Real-time Inventory Query)         │
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

| Module | Priority | Core Responsibilities | Independently Testable |
|--------|----------|----------------------|----------------------|
| **Pipe Management** | P0 | Seamless/screen pipe CRUD, search, archive management | Yes |
| **Inventory Management** | P0 | Inbound/outbound/check/location/real-time inventory query | Yes |
| **Quality Management** | P1 | Quality certificate management, quality traceability, API 5CT standard reference | Yes |
| **Procurement Management** | P1 | Supplier management, purchase orders, inbound linkage | Yes |
| **Sales Management** | P1 | Customer management, sales orders, outbound linkage, ATP available-to-promise | Yes |
| **Contract Management** | P2 | Purchase/sales contract management | Yes |
| **Data Import/Export** | P1 | Excel/CSV import/export | Yes |
| **Search & Filtering** | P0 | Multi-dimensional combined search (shared capability across modules) | -- |
| **Reports & Statistics** | P2 | Inventory reports, business charts | Yes |
| **Label Printing** | P2 | Barcode/QR code label generation and printing | Yes |
| **History Traceability** | P0 | Full lifecycle operation logs | Yes |
| **System Management** | P1 | User management, RBAC permission control, operation log audit | Yes |

### 4.2 Pipe Management Module

**Seamless Steel Pipe & Screen Pipe Management**

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
| `update_seamless_pipe(id, dto)` | Update seamless pipe information |
| `delete_seamless_pipe(id)` | Delete a seamless pipe (soft delete / validate inventory) |
| `get_seamless_pipe(id)` | Get single seamless pipe details |
| `list_seamless_pipes(filters)` | Query seamless pipe list with conditions (paginated) |
| `create_screen_pipe(dto)` | Create a screen pipe |
| `update_screen_pipe(id, dto)` | Update screen pipe information |
| `delete_screen_pipe(id)` | Delete a screen pipe |
| `get_screen_pipe(id)` | Get screen pipe details |
| `list_screen_pipes(filters)` | Query screen pipe list with conditions (paginated) |
| `generate_pipe_number(pipe_type)` | Auto-generate pipe number |
| `validate_pipe_number_unique(number)` | Validate pipe number global uniqueness |

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
| `create_inbound(dto)` | Create an inbound record, auto-update inventory. **Constraint**: when inbound_type='purchase', order_id is required and the purchase order must be in approved status; after inbound, auto-update the purchase order's received_quantity. For production/return types, approval_status=pending upon creation, requiring warehouse supervisor approval before taking effect |
| `approve_inbound(id)` | Approve non-purchase inbound records (production/return), update inventory upon approval |
| `reject_inbound(id, reason)` | Reject non-purchase inbound records |
| `create_outbound(dto)` | Create an outbound record, deduct inventory. **Constraint**: when outbound_type='sales', order_id is required and the sales order must be in approved status; after outbound, auto-update the sales order's delivered_quantity. For transfer/scrapped types, approval_status=pending upon creation, requiring warehouse supervisor approval before taking effect |
| `approve_outbound(id)` | Approve non-sales outbound records (transfer/scrapped), deduct inventory upon approval |
| `reject_outbound(id, reason)` | Reject non-sales outbound records |
| `get_stock_status(pipe_type, pipe_id)` | View single-item or single-batch inventory status |
| `list_inventory(filters)` | Real-time inventory query (grouped and aggregated by conditions) |
| `list_inventory_logs(filters)` | Inventory change transaction logs |
| `create_inventory_check(dto)` | Create an inventory check list |
| `submit_check_result(dto)` | Submit check results, generate discrepancy report |
| `create_location(dto)` | Create a location |
| `assign_pipe_to_location(pipe_id, location_id)` | Bind a pipe to a location |
| `transfer_location(pipe_id, new_location_id)` | Transfer pipe location |

### 4.4 Quality Management Module

**Core Interfaces:**

| Interface | Description |
|-----------|-------------|
| `create_quality_cert(dto)` | Upload / enter a quality inspection certificate |
| `update_quality_cert(id, dto)` | Update a quality inspection certificate |
| `get_quality_cert(id)` | Get quality inspection certificate details |
| `list_quality_certs(filters)` | Query quality inspection certificate list with conditions |
| `trace_by_heat_number(heat_no)` | Trace by heat number |
| `trace_by_pipe_number(pipe_no)` | Trace by pipe number |
| `get_api_5ct_grade_ref(grade)` | Get API 5CT grade mechanical/chemical reference data |

### 4.5 Procurement Management Module (PurchaseModule)

**Dependencies**: Inventory Management Module (inbound linkage), Pipe Management Module (pipe spec references)
**Depended by**: Inventory Management Module references purchase orders when creating purchase inbound records

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
| `create_supplier(dto)` | Create a supplier |
| `update_supplier(id, dto)` | Update supplier information |
| `delete_supplier(id)` | Delete a supplier |
| `list_suppliers(filter)` | Supplier list |
| `create_purchase_order(dto)` | Create a purchase order (with line items) |
| `approve_purchase_order(id)` | Approve purchase order (draft -> pending -> approved) |
| `reject_purchase_order(id, reason)` | Reject a purchase order |
| `link_inbound_to_po(inbound_id, po_id)` | Link inbound record to purchase order (update received_quantity, auto-complete when fully received) |

### 4.6 Sales Management Module (SalesModule)

**Dependencies**: Inventory Management Module (outbound linkage + ATP available-to-promise query), Pipe Management Module (pipe spec references)
**Depended by**: Inventory Management Module references sales orders when creating sales outbound records

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
│  │ CustomerRepo    │ │ SalesOrderRepo       │  │
│  │ (Customer data  │ │ (SO + line items     │  │
│  │  access)        │ │  + ATP)              │  │
│  └────────────────┘ └──────────────────────┘  │
└──────────────────────────────────────────────┘
```

**Core Interfaces:**

| Interface | Description |
|-----------|-------------|
| `create_customer(dto)` | Create a customer |
| `update_customer(id, dto)` | Update customer information |
| `delete_customer(id)` | Delete a customer |
| `list_customers(filter)` | Customer list |
| `create_sales_order(dto)` | Create a sales order (with line items) |
| `approve_sales_order(id)` | Approve sales order (draft -> pending -> approved) |
| `reject_sales_order(id, reason)` | Reject a sales order |
| `link_outbound_to_so(outbound_id, so_id)` | Link outbound record to sales order (update delivered_quantity, auto-complete when fully delivered) |
| `get_atp(pipe_type, grade, od, wt)` | Query available-to-promise inventory (in-stock quantity - locked sales order quantity) |

### 4.7 System Management Module

**Core Interfaces:**

| Interface | Description |
|-----------|-------------|
| `create_user(dto)` | Create a user (admin) |
| `update_user(id, dto)` | Update user information |
| `list_users(filters)` | User list |
| `assign_role(user_id, role)` | Assign a role |
| `login(credentials)` | User login (returns JWT) |
| `refresh_token(token)` | Refresh JWT |
| `get_current_user()` | Get current logged-in user information |
| `list_operation_logs(filters)` | Operation log query |

---

## 5. Database Detailed Design

### 5.1 Design Principles

- **SQLite WAL mode**: Enabled via `PRAGMA journal_mode=WAL;`, supports concurrent read-write
- **No foreign key constraints**: SQLite foreign keys have performance overhead; referential integrity is enforced at the application layer
- **Indexing strategy**: Index frequently queried fields, composite indexes for combined queries
- **Time fields**: Uniformly stored in ISO 8601 text format (`TEXT`) for compatibility
- **Enum fields**: Stored as `TEXT` type for readability and extensibility
- **Soft deletes**: Key data tables include a `deleted_at` field; no physical deletion

### 5.2 Database Initialization Configuration

```sql
-- Enable WAL mode
PRAGMA journal_mode = WAL;
-- Enable foreign keys (application-controlled, but validation is on)
PRAGMA foreign_keys = ON;
-- Set busy timeout
PRAGMA busy_timeout = 5000;
-- Sync mode: NORMAL balances performance and safety
PRAGMA synchronous = NORMAL;
-- Cache size: 64MB
PRAGMA cache_size = -64000;
-- Temp storage: memory
PRAGMA temp_store = MEMORY;
```

### 5.3 Table Structures

---

#### 5.3.1 seamless_pipes -- Seamless Steel Pipe Table

**Table name**: `seamless_pipes`

**Purpose**: Stores detailed information for each seamless steel pipe.

| Field | Type | Constraint | Description |
|-------|------|------------|-------------|
| `id` | INTEGER | PK, AUTOINCREMENT | Primary key |
| `pipe_number` | TEXT | NOT NULL, UNIQUE | Pipe number (globally unique, format: `J55 4.500in x 11.60lb SC-H2405-000001`) |
| `batch_number` | TEXT | -- | Batch number (from purchase order, for traceability) |
| `pipe_type` | TEXT | NOT NULL, CHECK IN ('casing','tubing') | Product type: casing / tubing |
| `grade` | TEXT | NOT NULL | Steel grade (H40, J55, K55, N80, etc.) |
| `od` | REAL | NOT NULL | Outer diameter (inches) |
| `wt` | REAL | NOT NULL | Wall thickness (inches) |
| `length` | REAL | -- | Length (feet) |
| `weight_per_unit` | REAL | -- | Weight per unit length (lb/ft) |
| `end_type` | TEXT | -- | End type (SC/LC/BC/X, etc.) |
| `coupling_type` | TEXT | -- | Coupling type |
| `coupling_od` | REAL | -- | Coupling outer diameter (inches) |
| `coupling_length` | REAL | -- | Coupling length (inches) |
| `heat_number` | TEXT | -- | Heat number (for quality traceability) |
| `serial_number` | TEXT | -- | Pipe body serial number |
| `manufacturer` | TEXT | -- | Manufacturer |
| `production_date` | TEXT | -- | Production date (ISO 8601) |
| `cert_number` | TEXT | -- | Quality certificate number |
| `location_id` | INTEGER | -- | Current location ID |
| `status` | TEXT | NOT NULL, DEFAULT 'in_stock' | Status: in_stock / outbound / scrapped |
| `notes` | TEXT | -- | Notes |
| `created_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | Creation time |
| `updated_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | Update time |
| `deleted_at` | TEXT | -- | Soft delete time (NULL = not deleted) |

**Indexes**:

```sql
CREATE INDEX idx_seamless_pipes_grade ON seamless_pipes(grade);
CREATE INDEX idx_seamless_pipes_heat_number ON seamless_pipes(heat_number);
CREATE INDEX idx_seamless_pipes_status ON seamless_pipes(status);
CREATE INDEX idx_seamless_pipes_location_id ON seamless_pipes(location_id);
CREATE INDEX idx_seamless_pipes_pipe_type ON seamless_pipes(pipe_type);
CREATE INDEX idx_seamless_pipes_od_wt ON seamless_pipes(od, wt);
CREATE INDEX idx_seamless_pipes_manufacturer ON seamless_pipes(manufacturer);
-- Composite search index
CREATE INDEX idx_seamless_pipes_search ON seamless_pipes(grade, od, wt, status);
```

---

#### 5.3.2 screen_pipes -- Screen Pipe Table

**Table name**: `screen_pipes`

**Purpose**: Stores detailed information for each screen pipe.

| Field | Type | Constraint | Description |
|-------|------|------------|-------------|
| `id` | INTEGER | PK, AUTOINCREMENT | Primary key |
| `pipe_number` | TEXT | NOT NULL, UNIQUE | Pipe number (globally unique, format: `J55 4.500in x 11.60lb 0.30mm-H2405-000001`) |
| `batch_number` | TEXT | -- | Batch number (from purchase order, for traceability) |
| `screen_type` | TEXT | NOT NULL | Screen type (wire_wrapped / slotted / punched / metal_felt) |
| `slot_size` | REAL | -- | Slot width / hole diameter (mm) |
| `filtration_grade` | TEXT | -- | Filtration precision (e.g., '150um', '250um') |
| `base_od` | REAL | NOT NULL | Base pipe outer diameter (inches) |
| `base_wt` | REAL | NOT NULL | Base pipe wall thickness (inches) |
| `base_grade` | TEXT | NOT NULL | Base pipe steel grade |
| `base_end_type` | TEXT | -- | Base pipe end type |
| `length` | REAL | -- | Screen pipe length (feet) |
| `weight_per_unit` | REAL | -- | Weight per unit length (lb/ft) |
| `heat_number` | TEXT | -- | Heat number |
| `serial_number` | TEXT | -- | Pipe body serial number |
| `manufacturer` | TEXT | -- | Manufacturer |
| `production_date` | TEXT | -- | Production date |
| `cert_number` | TEXT | -- | Quality certificate number |
| `location_id` | INTEGER | -- | Current location ID |
| `status` | TEXT | NOT NULL, DEFAULT 'in_stock' | Status |
| `notes` | TEXT | -- | Notes |
| `created_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | Creation time |
| `updated_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | Update time |
| `deleted_at` | TEXT | -- | Soft delete time |

**Indexes**:

```sql
CREATE INDEX idx_screen_pipes_type ON screen_pipes(screen_type);
CREATE INDEX idx_screen_pipes_heat_number ON screen_pipes(heat_number);
CREATE INDEX idx_screen_pipes_status ON screen_pipes(status);
CREATE INDEX idx_screen_pipes_location_id ON screen_pipes(location_id);
CREATE INDEX idx_screen_pipes_base_grade ON screen_pipes(base_grade);
```

---

#### 5.3.3 locations -- Location Table

**Table name**: `locations`

**Purpose**: Hierarchical management of zones / shelves / levels.

| Field | Type | Constraint | Description |
|-------|------|------------|-------------|
| `id` | INTEGER | PK, AUTOINCREMENT | Primary key |
| `zone_code` | TEXT | NOT NULL | Zone code |
| `shelf_code` | TEXT | NOT NULL | Shelf code |
| `level_code` | TEXT | NOT NULL | Level code |
| `full_code` | TEXT | NOT NULL, UNIQUE | Full code (zone_code + '-' + shelf_code + '-' + level_code) |
| `description` | TEXT | -- | Location description |
| `max_capacity` | INTEGER | -- | Maximum capacity (number of pipes) |
| `current_usage` | INTEGER | NOT NULL, DEFAULT 0 | Current usage count |
| `is_active` | INTEGER | NOT NULL, DEFAULT 1 | Whether enabled |
| `created_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | -- |
| `updated_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | -- |

**Indexes**:

```sql
CREATE UNIQUE INDEX idx_locations_full_code ON locations(full_code);
CREATE INDEX idx_locations_zone ON locations(zone_code);
```

---

#### 5.3.4 inbound_records -- Inbound Record Table (Header)

**Table name**: `inbound_records`

**Purpose**: Inbound order header. One inbound record corresponds to one header + N line items (`inbound_items`).

| Field | Type | Constraint | Description |
|-------|------|------------|-------------|
| `id` | INTEGER | PK, AUTOINCREMENT | Primary key |
| `record_no` | TEXT | NOT NULL, UNIQUE | Inbound record number |
| `inbound_type` | TEXT | NOT NULL, CHECK IN ('purchase','production','return') | Inbound type: purchase / production / return |
| `inbound_date` | TEXT | NOT NULL | Inbound date |
| `order_id` | INTEGER | -- | Associated purchase order ID (required when inbound_type='purchase', must reference an approved purchase order) |
| `supplier_id` | INTEGER | -- | Supplier ID |
| `operator_id` | INTEGER | -- | Operator (user ID) |
| `approval_status` | TEXT | NOT NULL, DEFAULT 'auto_approved' | Approval status: auto_approved / pending / approved / rejected. Purchase type auto-approved; production/return types require warehouse supervisor approval |
| `remark` | TEXT | -- | Remark |
| `created_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | -- |

**Indexes**:

```sql
CREATE INDEX idx_inbound_records_no ON inbound_records(record_no);
CREATE INDEX idx_inbound_records_date ON inbound_records(inbound_date);
CREATE INDEX idx_inbound_records_order ON inbound_records(order_id);
CREATE INDEX idx_inbound_records_supplier ON inbound_records(supplier_id);
```

---

#### 5.3.4a inbound_items -- Inbound Line Items Table

**Table name**: `inbound_items`

**Purpose**: Inbound order line items, one record per pipe. Supports batch inbound (N pipes of the same spec) and per-pipe inbound (single existing pipe).

| Field | Type | Constraint | Description |
|-------|------|------------|-------------|
| `id` | INTEGER | PK, AUTOINCREMENT | Primary key |
| `inbound_id` | INTEGER | NOT NULL | Associated inbound header ID |
| `pipe_type` | TEXT | NOT NULL, CHECK IN ('seamless','screen') | Pipe type |
| `pipe_id` | INTEGER | NOT NULL | Pipe ID (auto-created or selected existing upon inbound) |
| `created_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | -- |

**Indexes**:

```sql
CREATE INDEX idx_inbound_items_inbound ON inbound_items(inbound_id);
CREATE INDEX idx_inbound_items_pipe ON inbound_items(pipe_type, pipe_id);
```

---

#### 5.3.5 outbound_records -- Outbound Record Table (Header)

**Table name**: `outbound_records`

**Purpose**: Outbound order header. Symmetric structure to inbound.

| Field | Type | Constraint | Description |
|-------|------|------------|-------------|
| `id` | INTEGER | PK, AUTOINCREMENT | Primary key |
| `record_no` | TEXT | NOT NULL, UNIQUE | Outbound record number |
| `outbound_type` | TEXT | NOT NULL, CHECK IN ('sales','transfer','scrapped') | Outbound type: sales / transfer / scrapped |
| `outbound_date` | TEXT | NOT NULL | Outbound date |
| `order_id` | INTEGER | -- | Associated sales order ID (required when outbound_type='sales', must reference an approved sales order) |
| `customer_id` | INTEGER | -- | Customer ID |
| `operator_id` | INTEGER | -- | Operator (user ID) |
| `approval_status` | TEXT | NOT NULL, DEFAULT 'auto_approved' | Approval status: auto_approved / pending / approved / rejected. Sales type auto-approved; transfer/scrapped types require warehouse supervisor approval |
| `remark` | TEXT | -- | Remark |
| `created_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | -- |

**Indexes**:

```sql
CREATE INDEX idx_outbound_records_no ON outbound_records(record_no);
CREATE INDEX idx_outbound_records_date ON outbound_records(outbound_date);
CREATE INDEX idx_outbound_records_order ON outbound_records(order_id);
CREATE INDEX idx_outbound_records_customer ON outbound_records(customer_id);
```

---

#### 5.3.5a outbound_items -- Outbound Line Items Table

**Table name**: `outbound_items`

**Purpose**: Outbound order line items, one record per pipe.

| Field | Type | Constraint | Description |
|-------|------|------------|-------------|
| `id` | INTEGER | PK, AUTOINCREMENT | Primary key |
| `outbound_id` | INTEGER | NOT NULL | Associated outbound header ID |
| `pipe_type` | TEXT | NOT NULL, CHECK IN ('seamless','screen') | Pipe type |
| `pipe_id` | INTEGER | NOT NULL | Pipe ID |
| `created_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | -- |

**Indexes**:

```sql
CREATE INDEX idx_outbound_items_outbound ON outbound_items(outbound_id);
CREATE INDEX idx_outbound_items_pipe ON outbound_items(pipe_type, pipe_id);
```

---

#### 5.3.6 inventory_logs -- Inventory Change Log Table

**Table name**: `inventory_logs`

**Purpose**: Records every inventory change for each pipe, forming the foundation for full lifecycle traceability.

| Field | Type | Constraint | Description |
|-------|------|------------|-------------|
| `id` | INTEGER | PK, AUTOINCREMENT | Primary key |
| `pipe_type` | TEXT | NOT NULL, CHECK IN ('seamless','screen') | Pipe type |
| `pipe_id` | INTEGER | NOT NULL | Pipe ID |
| `change_type` | TEXT | NOT NULL | Change type: inbound / outbound / transfer / check_adjust |
| `reference_id` | INTEGER | -- | Associated document ID (inbound/outbound order ID) |
| `reference_no` | TEXT | -- | Associated document number |
| `operator_id` | INTEGER | -- | Operator ID |
| `operator_name` | TEXT | -- | Operator name (denormalized to avoid joins) |
| `remark` | TEXT | -- | Change description |
| `created_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | Change time |

**Indexes**:

```sql
CREATE INDEX idx_inventory_logs_pipe ON inventory_logs(pipe_type, pipe_id);
CREATE INDEX idx_inventory_logs_type ON inventory_logs(change_type);
CREATE INDEX idx_inventory_logs_time ON inventory_logs(created_at);
CREATE INDEX idx_inventory_logs_operator ON inventory_logs(operator_id);
```

---

#### 5.3.7 suppliers -- Supplier Table

**Table name**: `suppliers`

| Field | Type | Constraint | Description |
|-------|------|------------|-------------|
| `id` | INTEGER | PK, AUTOINCREMENT | Primary key |
| `name` | TEXT | NOT NULL | Supplier name |
| `contact_person` | TEXT | -- | Contact person |
| `phone` | TEXT | -- | Contact phone |
| `email` | TEXT | -- | Email |
| `address` | TEXT | -- | Address |
| `qualification_cert` | TEXT | -- | Qualification certificate number |
| `is_active` | INTEGER | NOT NULL, DEFAULT 1 | Whether enabled |
| `created_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | -- |
| `updated_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | -- |

**Indexes**:

```sql
CREATE INDEX idx_suppliers_name ON suppliers(name);
```

---

#### 5.3.8 customers -- Customer Table

**Table name**: `customers`

| Field | Type | Constraint | Description |
|-------|------|------------|-------------|
| `id` | INTEGER | PK, AUTOINCREMENT | Primary key |
| `name` | TEXT | NOT NULL | Customer name |
| `contact_person` | TEXT | -- | Contact person |
| `phone` | TEXT | -- | Contact phone |
| `email` | TEXT | -- | Email |
| `address` | TEXT | -- | Address |
| `is_active` | INTEGER | NOT NULL, DEFAULT 1 | Whether enabled |
| `created_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | -- |
| `updated_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | -- |

---

#### 5.3.9 purchase_orders -- Purchase Order Table

**Table name**: `purchase_orders`

| Field | Type | Constraint | Description |
|-------|------|------------|-------------|
| `id` | INTEGER | PK, AUTOINCREMENT | Primary key |
| `order_no` | TEXT | NOT NULL, UNIQUE | Purchase order number |
| `supplier_id` | INTEGER | NOT NULL | Supplier ID |
| `order_date` | TEXT | NOT NULL | Order date |
| `expected_date` | TEXT | -- | Expected delivery date |
| `status` | TEXT | NOT NULL, DEFAULT 'draft' | Status: draft / pending / approved / completed / cancelled |
| `total_amount` | REAL | -- | Total amount |
| `currency` | TEXT | NOT NULL, DEFAULT 'CNY' | Currency |
| `contract_id` | INTEGER | -- | Associated contract ID |
| `notes` | TEXT | -- | Notes |
| `created_by` | INTEGER | -- | Creator |
| `created_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | -- |
| `updated_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | -- |

**Indexes**:

```sql
CREATE INDEX idx_purchase_orders_supplier ON purchase_orders(supplier_id);
CREATE INDEX idx_purchase_orders_status ON purchase_orders(status);
CREATE INDEX idx_purchase_orders_date ON purchase_orders(order_date);
```

---

#### 5.3.10 purchase_order_items -- Purchase Order Line Items Table

**Table name**: `purchase_order_items`

| Field | Type | Constraint | Description |
|-------|------|------------|-------------|
| `id` | INTEGER | PK, AUTOINCREMENT | Primary key |
| `order_id` | INTEGER | NOT NULL | Purchase order ID |
| `pipe_type` | TEXT | NOT NULL | Pipe type |
| `grade` | TEXT | NOT NULL | Steel grade |
| `od` | REAL | NOT NULL | Outer diameter |
| `wt` | REAL | NOT NULL | Wall thickness |
| `quantity` | INTEGER | NOT NULL | Ordered quantity |
| `received_quantity` | INTEGER | NOT NULL, DEFAULT 0 | Received quantity |
| `unit_price` | REAL | -- | Unit price |
| `notes` | TEXT | -- | -- |

**Indexes**:

```sql
CREATE INDEX idx_poi_order ON purchase_order_items(order_id);
```

---

#### 5.3.11 sales_orders -- Sales Order Table

Symmetric structure to `purchase_orders`, associated with `customer_id` instead of `supplier_id`.

**Table name**: `sales_orders`

| Field | Type | Constraint | Description |
|-------|------|------------|-------------|
| `id` | INTEGER | PK, AUTOINCREMENT | Primary key |
| `order_no` | TEXT | NOT NULL, UNIQUE | Sales order number |
| `customer_id` | INTEGER | NOT NULL | Customer ID |
| `order_date` | TEXT | NOT NULL | Order date |
| `status` | TEXT | NOT NULL, DEFAULT 'draft' | Status: draft / pending / approved / completed / cancelled |
| `total_amount` | REAL | -- | Total amount |
| `currency` | TEXT | NOT NULL, DEFAULT 'CNY' | Currency |
| `contract_id` | INTEGER | -- | Associated contract ID |
| `notes` | TEXT | -- | Notes |
| `created_by` | INTEGER | -- | -- |
| `created_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | -- |
| `updated_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | -- |

**Indexes**:

```sql
CREATE INDEX idx_sales_orders_customer ON sales_orders(customer_id);
CREATE INDEX idx_sales_orders_status ON sales_orders(status);
CREATE INDEX idx_sales_orders_date ON sales_orders(order_date);
```

#### 5.3.12 sales_order_items -- Sales Order Line Items Table

Symmetric structure to `purchase_order_items`.

| Field | Type | Constraint | Description |
|-------|------|------------|-------------|
| `id` | INTEGER | PK, AUTOINCREMENT | Primary key |
| `order_id` | INTEGER | NOT NULL | Sales order ID |
| `pipe_type` | TEXT | NOT NULL | Pipe type |
| `grade` | TEXT | NOT NULL | Steel grade |
| `od` | REAL | NOT NULL | Outer diameter |
| `wt` | REAL | NOT NULL | Wall thickness |
| `quantity` | INTEGER | NOT NULL | Ordered quantity |
| `delivered_quantity` | INTEGER | NOT NULL, DEFAULT 0 | Delivered quantity |
| `unit_price` | REAL | -- | Unit price |
| `notes` | TEXT | -- | -- |

---

#### 5.3.13 contracts -- Contract Table

**Table name**: `contracts`

| Field | Type | Constraint | Description |
|-------|------|------------|-------------|
| `id` | INTEGER | PK, AUTOINCREMENT | Primary key |
| `contract_no` | TEXT | NOT NULL, UNIQUE | Contract number |
| `contract_type` | TEXT | NOT NULL | Contract type: purchase / sales |
| `party_a_id` | INTEGER | -- | Party A ID (our company) |
| `party_b_id` | INTEGER | -- | Party B ID (supplier or customer) |
| `sign_date` | TEXT | -- | Signing date |
| `total_amount` | REAL | -- | Contract amount |
| `status` | TEXT | NOT NULL, DEFAULT 'active' | Status: active / completed / terminated |
| `file_url` | TEXT | -- | Contract scan file path |
| `notes` | TEXT | -- | -- |
| `created_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | -- |
| `updated_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | -- |

---

#### 5.3.14 quality_certs -- Quality Certificate Table

**Table name**: `quality_certs`

| Field | Type | Constraint | Description |
|-------|------|------------|-------------|
| `id` | INTEGER | PK, AUTOINCREMENT | Primary key |
| `pipe_type` | TEXT | NOT NULL | Pipe type: seamless / screen |
| `pipe_id` | INTEGER | NOT NULL | Pipe ID |
| `cert_no` | TEXT | NOT NULL | Quality certificate number |
| `inspect_date` | TEXT | -- | Inspection date |
| `inspector` | TEXT | -- | Inspector |
| `agency` | TEXT | -- | Inspection agency |
| `result` | TEXT | NOT NULL | Inspection result: pass / fail / pending |
| `test_items` | TEXT | -- | Test item list (JSON array) |
| `file_url` | TEXT | -- | Inspection report file path |
| `notes` | TEXT | -- | -- |
| `created_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | -- |
| `updated_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | -- |

**Indexes**:

```sql
CREATE INDEX idx_quality_certs_pipe ON quality_certs(pipe_type, pipe_id);
CREATE INDEX idx_quality_certs_cert_no ON quality_certs(cert_no);
```

---

#### 5.3.15 users -- User Table

**Table name**: `users`

| Field | Type | Constraint | Description |
|-------|------|------------|-------------|
| `id` | INTEGER | PK, AUTOINCREMENT | Primary key |
| `username` | TEXT | NOT NULL, UNIQUE | Username (for login) |
| `password_hash` | TEXT | NOT NULL | Password hash (Argon2) |
| `display_name` | TEXT | NOT NULL | Display name |
| `email` | TEXT | -- | Email |
| `role` | TEXT | NOT NULL | Role: admin / warehouse / qc / sales |
| `language_pref` | TEXT | NOT NULL, DEFAULT 'zh' | Language preference: zh / en |
| `unit_system` | TEXT | NOT NULL, DEFAULT 'metric' | Unit system: metric / imperial |
| `is_active` | INTEGER | NOT NULL, DEFAULT 1 | Whether enabled |
| `last_login_at` | TEXT | -- | Last login time |
| `created_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | -- |
| `updated_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | -- |

**Indexes**:

```sql
CREATE UNIQUE INDEX idx_users_username ON users(username);
CREATE INDEX idx_users_role ON users(role);
```

---

#### 5.3.16 operation_logs -- Operation Log Table

**Table name**: `operation_logs`

**Purpose**: Audit log, records all critical data changes.

| Field | Type | Constraint | Description |
|-------|------|------------|-------------|
| `id` | INTEGER | PK, AUTOINCREMENT | Primary key |
| `user_id` | INTEGER | -- | Operating user ID |
| `username` | TEXT | -- | Username (denormalized) |
| `action` | TEXT | NOT NULL | Action type: create / update / delete / login / export |
| `target_type` | TEXT | NOT NULL | Target object type: pipe / inbound / outbound / order / user, etc. |
| `target_id` | INTEGER | -- | Target object ID |
| `target_summary` | TEXT | -- | Operation summary (e.g., pipe number) |
| `detail` | TEXT | -- | Change details (JSON, records before/after field values) |
| `ip_address` | TEXT | -- | Request source IP |
| `created_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | Operation time |

**Indexes**:

```sql
CREATE INDEX idx_operation_logs_user ON operation_logs(user_id);
CREATE INDEX idx_operation_logs_target ON operation_logs(target_type, target_id);
CREATE INDEX idx_operation_logs_action ON operation_logs(action);
CREATE INDEX idx_operation_logs_time ON operation_logs(created_at);
```

---

#### 5.3.17 pipe_attachments -- Pipe Attachments Table

**Table name**: `pipe_attachments`

**Purpose**: Pipe-related files/photos archive (receipt appearance photos, defect photos, inspection reports, etc.), one-to-many association with pipes.

| Field | Type | Constraint | Description |
|-------|------|------------|-------------|
| `id` | INTEGER | PK, AUTOINCREMENT | Primary key |
| `pipe_type` | TEXT | NOT NULL | Pipe type: seamless / screen |
| `pipe_id` | INTEGER | NOT NULL | Pipe ID |
| `file_name` | TEXT | NOT NULL | Original file name |
| `file_path` | TEXT | NOT NULL | Storage path |
| `file_type` | TEXT | -- | File type: image / pdf / other |
| `file_size` | INTEGER | -- | File size (bytes) |
| `uploaded_by` | INTEGER | -- | Uploader (user ID) |
| `created_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | -- |

**Indexes**:

```sql
CREATE INDEX idx_pipe_attachments_target ON pipe_attachments(pipe_type, pipe_id);
```

---

#### 5.3.18 api_5ct_grade_ref -- API 5CT Grade Reference Table

**Table name**: `api_5ct_grade_ref`

**Purpose**: Stores mechanical properties and chemical composition reference data for API 5CT standard grades, used for quality inspection comparison and validation.

| Field | Type | Constraint | Description |
|-------|------|------------|-------------|
| `id` | INTEGER | PK, AUTOINCREMENT | Primary key |
| `grade` | TEXT | NOT NULL, UNIQUE | Grade code |
| `grade_group` | TEXT | -- | Grade group (H/J/K/N/L/C/T/P/Q) |
| `pipe_type` | TEXT | -- | Applicable pipe type: casing / tubing / both |
| `min_yield_strength_psi` | INTEGER | -- | Minimum yield strength (psi) |
| `max_yield_strength_psi` | INTEGER | -- | Maximum yield strength (psi) |
| `min_tensile_strength_psi` | INTEGER | -- | Minimum tensile strength (psi) |
| `hardness_max` | REAL | -- | Maximum hardness (HRC) |
| `notes` | TEXT | -- | Applicable environment and special notes |
| `created_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | -- |

---

#### 5.3.19 inventory_check_records -- Inventory Check Record Table

**Table name**: `inventory_check_records`

| Field | Type | Constraint | Description |
|-------|------|------------|-------------|
| `id` | INTEGER | PK, AUTOINCREMENT | Primary key |
| `check_no` | TEXT | NOT NULL, UNIQUE | Check record number |
| `check_date` | TEXT | NOT NULL | Check date |
| `status` | TEXT | NOT NULL, DEFAULT 'in_progress' | Status: in_progress / completed |
| `operator_id` | INTEGER | -- | Checker |
| `notes` | TEXT | -- | -- |
| `created_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | -- |
| `updated_at` | TEXT | NOT NULL, DEFAULT (datetime('now')) | -- |

#### 5.3.20 inventory_check_items -- Inventory Check Line Items Table

**Table name**: `inventory_check_items`

**Purpose**: Check line items, one row per pipe. The system pre-populates the list of pipes expected at this location (expected), and the checker confirms the actual presence of each pipe one by one. Each pipe is either "confirmed present" or "missing".
**No longer uses quantity addition/subtraction** -- precise identification of which specific pipe is anomalous.

| Field | Type | Constraint | Description |
|-------|------|------------|-------------|
| `id` | INTEGER | PK, AUTOINCREMENT | Primary key |
| `check_id` | INTEGER | NOT NULL | Check record ID |
| `pipe_type` | TEXT | NOT NULL | Pipe type |
| `pipe_id` | INTEGER | NOT NULL | Pipe ID |
| `expected` | INTEGER | NOT NULL, DEFAULT 1 | Expected on book (flag=1), if this pipe should be in stock |
| `found` | INTEGER | -- | Check result (NULL=not checked, 1=found, 0=missing) |
| `notes` | TEXT | -- | Discrepancy description |

**Indexes**:

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
└──────────┘       └──────────────────┘       └─────────────────┘
```

---

## 6. REST API Design

### 6.1 API Base Specification

**Base path**: `/api/v1`

**Unified response format**:

```json
// Success response
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

// Error response
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

**Authentication**: `Authorization: Bearer <jwt_token>`

**Pagination parameters**:

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `page` | integer | 1 | Page number |
| `page_size` | integer | 20 | Items per page (max 100) |

### 6.2 Pipe Management API

#### 6.2.1 Seamless Steel Pipes

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/v1/seamless-pipes` | List query (supports combined filtering) |
| `POST` | `/api/v1/seamless-pipes` | Add a seamless pipe |
| `GET` | `/api/v1/seamless-pipes/{id}` | Get details |
| `PUT` | `/api/v1/seamless-pipes/{id}` | Full update |
| `DELETE` | `/api/v1/seamless-pipes/{id}` | Delete (soft delete) |

**GET /api/v1/seamless-pipes query parameters**:

| Parameter | Type | Description |
|-----------|------|-------------|
| `q` | string | Fuzzy search (matches pipe_number / heat_number / serial_number) |
| `grade` | string | Exact grade match |
| `pipe_type` | string | casing / tubing |
| `od_min` / `od_max` | number | Outer diameter range |
| `wt_min` / `wt_max` | number | Wall thickness range |
| `status` | string | Inventory status |
| `location_id` | integer | Location ID |
| `manufacturer` | string | Manufacturer |
| `sort_by` | string | Sort field (default `created_at`, whitelist only: `created_at`, `pipe_number`, `grade`, `od`, `wt`, `status`) |
| `sort_order` | string | asc / desc |

**POST /api/v1/seamless-pipes request body**:

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
| `GET` | `/api/v1/screen-pipes` | List query |
| `POST` | `/api/v1/screen-pipes` | Add a screen pipe |
| `GET` | `/api/v1/screen-pipes/{id}` | Get details |
| `PUT` | `/api/v1/screen-pipes/{id}` | Full update |
| `DELETE` | `/api/v1/screen-pipes/{id}` | Delete (soft delete) |

Query parameters are similar to seamless pipes, with additional screen-pipe-specific fields such as `screen_type`, `slot_size`, etc.

#### 6.2.3 Unified Pipe Search

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/v1/pipes/search` | Cross-type unified search (seamless + screen) |

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
| `GET` | `/api/v1/inbound-records` | Inbound record list (headers) |
| `POST` | `/api/v1/inbound-records` | Create an inbound record (header + line items submitted together). **Constraint**: when inbound_type='purchase', order_id is required; the system auto-validates that the purchase order is in approved status |
| `GET` | `/api/v1/inbound-records/{id}` | Inbound record details (with line item list) |
| `GET` | `/api/v1/inbound-records/{id}/items` | Inbound line item list |
| `POST` | `/api/v1/inbound-records/{id}/approve` | Approve non-purchase inbound records (production/return), requires `admin` or `warehouse` role |
| `POST` | `/api/v1/inbound-records/{id}/reject` | Reject non-purchase inbound records |
| `DELETE` | `/api/v1/inbound-records/{id}` | Delete an inbound record (requires permission; only records in auto_approved or rejected status can be deleted) |

**POST /api/v1/inbound-records request body -- Purchase Inbound**:

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

**POST /api/v1/inbound-records request body -- Non-Purchase Inbound (requires approval)**:

```json
{
  "inbound_type": "production",
  "inbound_date": "2026-05-19",
  "supplier_id": 1,
  "operator_id": 1,
  "remark": "Production inbound (finished pipes returned to stock)",
  "pipes": [
    { "pipe_type": "seamless", "pipe_id": 100 }
  ]
}
```

> **Approval workflow**: When inbound_type is 'production' or 'return', approval_status is set to 'pending' upon creation; the approval endpoint must be called for it to take effect. Inventory updates are executed only after approval.
> **Batch inbound**: If `pipes` is empty and the `batch_create` parameter is provided, the system auto-creates N pipes of the same spec and processes inbound. Batch inbound only supports inbound_type='purchase'.
> **Additional batch inbound endpoint**:

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/api/v1/inbound-records/batch` | Batch create pipes + single-step inbound (purchase type only) |

```json
// POST /api/v1/inbound-records/batch request body
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
| `GET` | `/api/v1/outbound-records` | Outbound record list (headers) |
| `POST` | `/api/v1/outbound-records` | Create an outbound record (with line items). **Constraint**: when outbound_type='sales', order_id is required; the system auto-validates that the sales order is in approved status |
| `GET` | `/api/v1/outbound-records/{id}` | Outbound record details (with line item list) |
| `GET` | `/api/v1/outbound-records/{id}/items` | Outbound line item list |
| `POST` | `/api/v1/outbound-records/{id}/approve` | Approve non-sales outbound records (transfer/scrapped), requires `admin` or `warehouse` role |
| `POST` | `/api/v1/outbound-records/{id}/reject` | Reject non-sales outbound records |
| `DELETE` | `/api/v1/outbound-records/{id}` | Delete an outbound record (requires permission; only records in auto_approved or rejected status can be deleted) |

**POST /api/v1/outbound-records request body -- Sales Outbound**:

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

**POST /api/v1/outbound-records request body -- Non-Sales Outbound (requires approval)**:

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

> **Approval workflow**: When outbound_type is 'transfer' or 'scrapped', approval_status is set to 'pending' upon creation; the approval endpoint must be called for it to take effect. Inventory deduction is executed only after approval.

#### 6.3.3 Real-Time Inventory Query

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/v1/inventory` | Real-time inventory list (aggregated query) |
| `GET` | `/api/v1/inventory/statistics` | Inventory statistics summary |
| `GET` | `/api/v1/inventory/logs` | Inventory change transaction logs |

**GET /api/v1/inventory query parameters**:

| Parameter | Description |
|-----------|-------------|
| `pipe_type` | seamless / screen |
| `grade` | Steel grade |
| `od` | Outer diameter |
| `wt` | Wall thickness |
| `status` | Status |
| `location_id` | Location |

**Response example**:

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

#### 6.3.4 Inventory Check Management

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/v1/inventory-checks` | Check record list |
| `POST` | `/api/v1/inventory-checks` | Create a check record |
| `GET` | `/api/v1/inventory-checks/{id}` | Check record details |
| `PUT` | `/api/v1/inventory-checks/{id}` | Update check data |
| `POST` | `/api/v1/inventory-checks/{id}/complete` | Complete the check, generate discrepancy report |

#### 6.3.5 Location Management

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/v1/locations` | Location list |
| `POST` | `/api/v1/locations` | Create a location |
| `PUT` | `/api/v1/locations/{id}` | Update a location |
| `DELETE` | `/api/v1/locations/{id}` | Delete a location |
| `POST` | `/api/v1/locations/{id}/assign` | Bind a pipe to a location |
| `POST` | `/api/v1/pipes/{pipe_id}/transfer-location` | Transfer pipe location |

### 6.4 Quality Management API

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/v1/quality-certs` | Quality certificate list |
| `POST` | `/api/v1/quality-certs` | Add a quality certificate |
| `GET` | `/api/v1/quality-certs/{id}` | Quality certificate details |
| `PUT` | `/api/v1/quality-certs/{id}` | Update a quality certificate |
| `DELETE` | `/api/v1/quality-certs/{id}` | Delete a quality certificate |
| `GET` | `/api/v1/trace/heat-number/{heat_no}` | Trace by heat number |
| `GET` | `/api/v1/trace/pipe-number/{pipe_no}` | Trace by pipe number |
| `GET` | `/api/v1/api-5ct-grades` | API 5CT grade reference list |
| `GET` | `/api/v1/api-5ct-grades/{grade}` | Single grade reference details |

### 6.5 Procurement Management API

#### Suppliers

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/v1/suppliers` | Supplier list |
| `POST` | `/api/v1/suppliers` | Create a supplier |
| `GET` | `/api/v1/suppliers/{id}` | Supplier details |
| `PUT` | `/api/v1/suppliers/{id}` | Update a supplier |
| `DELETE` | `/api/v1/suppliers/{id}` | Delete a supplier |

#### Purchase Orders

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/v1/purchase-orders` | Purchase order list |
| `POST` | `/api/v1/purchase-orders` | Create a purchase order |
| `GET` | `/api/v1/purchase-orders/{id}` | Purchase order details |
| `PUT` | `/api/v1/purchase-orders/{id}` | Update a purchase order |
| `POST` | `/api/v1/purchase-orders/{id}/approve` | Approve |
| `POST` | `/api/v1/purchase-orders/{id}/reject` | Reject |
| `POST` | `/api/v1/purchase-orders/{id}/link-inbound` | Link to inbound record |

### 6.6 Sales Management API

#### Customers

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/v1/customers` | Customer list |
| `POST` | `/api/v1/customers` | Create a customer |
| `GET` | `/api/v1/customers/{id}` | Customer details |
| `PUT` | `/api/v1/customers/{id}` | Update a customer |
| `DELETE` | `/api/v1/customers/{id}` | Delete a customer |

#### Sales Orders

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/v1/sales-orders` | Sales order list |
| `POST` | `/api/v1/sales-orders` | Create a sales order |
| `GET` | `/api/v1/sales-orders/{id}` | Sales order details |
| `PUT` | `/api/v1/sales-orders/{id}` | Update a sales order |
| `POST` | `/api/v1/sales-orders/{id}/approve` | Approve |
| `POST` | `/api/v1/sales-orders/{id}/reject` | Reject |
| `GET` | `/api/v1/atp` | Available-to-promise query (parameters: pipe_type, grade, od, wt) |
| `POST` | `/api/v1/sales-orders/{id}/link-outbound` | Link to outbound record |

### 6.7 Contract Management API

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/v1/contracts` | Contract list |
| `POST` | `/api/v1/contracts` | Create a contract |
| `GET` | `/api/v1/contracts/{id}` | Contract details |
| `PUT` | `/api/v1/contracts/{id}` | Update a contract |

### 6.8 Data Import/Export API

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/api/v1/import/seamless-pipes` | Import seamless pipes (Excel/CSV) |
| `POST` | `/api/v1/import/screen-pipes` | Import screen pipes |
| `POST` | `/api/v1/import/inventory` | Import inventory data |
| `GET` | `/api/v1/export/seamless-pipes` | Export seamless pipe data |
| `GET` | `/api/v1/export/screen-pipes` | Export screen pipe data |
| `GET` | `/api/v1/export/inventory` | Export inventory report |
| `GET` | `/api/v1/export/inventory-logs` | Export inventory change logs |

**Import request**: `multipart/form-data`, includes the file.

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

#### Users & Authentication

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/api/v1/auth/login` | Login |
| `POST` | `/api/v1/auth/refresh` | Refresh Token |
| `POST` | `/api/v1/auth/logout` | Logout |
| `GET` | `/api/v1/auth/me` | Current user information |
| `GET` | `/api/v1/users` | User list (admin only) |
| `POST` | `/api/v1/users` | Create a user (admin only) |
| `PUT` | `/api/v1/users/{id}` | Update a user |
| `PUT` | `/api/v1/users/{id}/role` | Change role (admin only) |
| `DELETE` | `/api/v1/users/{id}` | Disable a user |

**POST /api/v1/auth/login request**:

```json
{
  "username": "admin",
  "password": "********"
}
```

**POST /api/v1/auth/login response**:

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
| `GET` | `/api/v1/operation-logs` | Operation log list (supports conditional filtering) |

**Query parameters**:

| Parameter | Description |
|-----------|-------------|
| `user_id` | Filter by user |
| `action` | Filter by action type |
| `target_type` | Filter by target object type |
| `date_from` / `date_to` | Time range |
| `q` | Fuzzy search |

### 6.10 Reports & Statistics API (P2)

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/v1/reports/inventory-summary` | Inventory summary report |
| `GET` | `/api/v1/reports/inventory-monthly` | Monthly inventory change report |
| `GET` | `/api/v1/reports/in-out-statistics` | Inbound/outbound statistics |
| `GET` | `/api/v1/reports/turnover-rate` | Inventory turnover rate |

### 6.11 Label Printing API (P2)

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/api/v1/labels/generate` | Generate labels (returns PDF or image) |
| `POST` | `/api/v1/labels/batch-generate` | Batch generate labels |

---

## 7. Project Directory Structure

### 7.1 Backend Project Structure (Rust)

```
pipe-management/
├── Cargo.toml                    # Dependency management
├── Cargo.lock
├── .env                          # Environment variables (DB path, JWT Secret, etc.)
├── .env.example                  # Environment variable template
├── sqlx-data.json                # SQLx offline query check data
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
│   └── 010_seed_api_5ct_data.sql # API 5CT grade reference data initialization
├── src/
│   ├── main.rs                   # Entry point: server startup, config loading
│   ├── lib.rs                    # Library entry point
│   ├── config.rs                 # Config reading (env vars -> Config struct)
│   ├── router.rs                 # Route registration (all module routes converge here)
│   ├── error.rs                  # Unified error type (AppError)
│   ├── response.rs               # Unified response format (ApiResponse)
│   ├── middleware/
│   │   ├── mod.rs
│   │   ├── auth.rs               # JWT authentication middleware
│   │   ├── rbac.rs               # Role-based access control middleware
│   │   ├── logging.rs            # Request logging middleware
│   │   └── request_id.rs         # Request ID generation and propagation
│   ├── domain/
│   │   ├── mod.rs
│   │   ├── pipe.rs               # Pipe-related enums, constants
│   │   ├── api_5ct.rs            # API 5CT standard reference data
│   │   └── error.rs              # Domain error types
│   ├── models/
│   │   ├── mod.rs
│   │   ├── seamless_pipe.rs      # SeamlessPipe struct
│   │   ├── screen_pipe.rs        # ScreenPipe struct
│   │   ├── location.rs           # Location struct
│   │   ├── inbound.rs            # InboundRecord struct
│   │   ├── outbound.rs           # OutboundRecord struct
│   │   ├── inventory_log.rs      # InventoryLog struct
│   │   ├── supplier.rs
│   │   ├── customer.rs
│   │   ├── purchase_order.rs     # PurchaseOrder + PurchaseOrderItem
│   │   ├── sales_order.rs        # SalesOrder + SalesOrderItem
│   │   ├── contract.rs
│   │   ├── quality_cert.rs
│   │   ├── inventory_check.rs    # Check + CheckItem
│   │   ├── user.rs
│   │   └── operation_log.rs
│   ├── dto/
│   │   ├── mod.rs
│   │   ├── pipe_dto.rs           # Pipe create/update/query DTOs
│   │   ├── inventory_dto.rs      # Inventory-related DTOs
│   │   ├── order_dto.rs          # Order-related DTOs
│   │   ├── auth_dto.rs           # Login/Token DTOs
│   │   └── common.rs             # Pagination, sorting, search parameter DTOs
│   ├── handlers/
│   │   ├── mod.rs
│   │   ├── pipe_handler.rs       # Pipe-related route handlers
│   │   ├── inventory_handler.rs  # Inventory-related route handlers
│   │   ├── quality_handler.rs    # Quality-related route handlers
│   │   ├── order_handler.rs      # Purchase/sales order route handlers
│   │   ├── supplier_handler.rs
│   │   ├── customer_handler.rs
│   │   ├── contract_handler.rs
│   │   ├── location_handler.rs
│   │   ├── import_export_handler.rs
│   │   ├── auth_handler.rs       # Login/authentication
│   │   ├── user_handler.rs
│   │   ├── log_handler.rs        # Operation logs
│   │   └── report_handler.rs     # Report statistics
│   ├── services/
│   │   ├── mod.rs
│   │   ├── pipe_service.rs
│   │   ├── inventory_service.rs
│   │   ├── location_service.rs
│   │   ├── quality_service.rs
│   │   ├── order_service.rs
│   │   ├── import_export_service.rs
│   │   ├── auth_service.rs
│   │   ├── user_service.rs
│   │   ├── report_service.rs
│   │   └── log_service.rs
│   └── repositories/
│       ├── mod.rs
│       ├── pipe_repo.rs
│       ├── screen_pipe_repo.rs
│       ├── location_repo.rs
│       ├── inbound_repo.rs
│       ├── outbound_repo.rs
│       ├── inventory_log_repo.rs
│       ├── supplier_repo.rs
│       ├── customer_repo.rs
│       ├── purchase_order_repo.rs
│       ├── sales_order_repo.rs
│       ├── contract_repo.rs
│       ├── quality_repo.rs
│       ├── inventory_check_repo.rs
│       ├── user_repo.rs
│       └── operation_log_repo.rs
└── tests/
    ├── common/                   # Shared test modules
    │   ├── mod.rs
    │   └── test_db.rs            # Test in-memory database initialization
    ├── pipe_tests.rs
    ├── inventory_tests.rs
    ├── order_tests.rs
    ├── auth_tests.rs
    └── api_integration_tests.rs  # Integration tests (starts test server)
```

### 7.2 Frontend Project Structure (TBD)

```
pipe-management-frontend/
├── package.json
├── tsconfig.json
├── vite.config.ts
├── public/
│   └── favicon.ico
├── src/
│   ├── main.tsx
│   ├── App.tsx
│   ├── routes/
│   ├── layouts/
│   ├── features/
│   │   ├── auth/
│   │   ├── pipes/
│   │   ├── inventory/
│   │   ├── quality/
│   │   ├── orders/
│   │   ├── reports/
│   │   └── system/
│   ├── shared/
│   │   ├── components/
│   │   ├── hooks/
│   │   └── utils/
│   ├── api/
│   ├── i18n/
│   └── types/
└── ...
```

---

## 8. Error Handling & Response Specification

### 8.1 HTTP Status Code Usage

| Status Code | Scenario |
|-------------|----------|
| `200` | GET / PUT success |
| `201` | POST creation success |
| `204` | DELETE success (no response body) |
| `400` | Request parameter error (validation failure) |
| `401` | Unauthenticated (no token or token expired) |
| `403` | Unauthorized (role does not permit the operation) |
| `404` | Resource not found |
| `409` | Conflict (e.g., duplicate pipe number) |
| `422` | Request body validation failure (Validator) |
| `500` | Server internal error |

### 8.2 Error Code Definitions

```rust
pub enum AppErrorCode {
    // General (100xx)
    InternalError,          // 10001
    ValidationError,        // 10002
    NotFound,               // 10003

    // Authentication (110xx)
    Unauthorized,           // 11001
    TokenExpired,           // 11002
    Forbidden,              // 11003

    // Pipes (120xx)
    PipeNotFound,           // 12001
    PipeNumberDuplicate,    // 12002
    PipeStatusConflict,     // 12003 (e.g., cannot delete an already-outbound pipe)

    // Inventory (130xx)
    InsufficientStock,      // 13001
    LocationFull,           // 13002
    LocationNotFound,       // 13003

    // Orders (140xx)
    OrderCannotModify,      // 14001 (approved orders cannot be modified)
    OrderNotFound,          // 14002
    OrderNotApproved,       // 14003 (unapproved order cannot be used for inbound/outbound)

    // Inventory (130xx continued)
    InboundOrderIdRequired,         // 13004 (purchase inbound must link to a purchase order)
    OutboundOrderIdRequired,        // 13005 (sales outbound must link to a sales order)
    InboundNotApprovedYet,          // 13006 (inbound record not yet approved)
    OutboundNotApprovedYet,         // 13007 (outbound record not yet approved)
    InboundAlreadyApproved,         // 13008 (inbound record already approved)
    OutboundAlreadyApproved,        // 13009 (outbound record already approved)
    InboundItemMismatch,            // 13010 (inbound pipe does not match purchase order)
    OutboundItemMismatch,           // 13011 (outbound pipe does not match sales order)
    InboundApprovalNotAllowed,      // 13012 (no permission to approve non-purchase inbound)
    OutboundApprovalNotAllowed,     // 13013 (no permission to approve non-sales outbound)

    // Import/Export (150xx)
    ImportFileParseError,   // 15001
    ImportValidationError,  // 15002
}
```

### 8.3 Global Error Handling

In Axum, global error capture is implemented via `FromRequestParts` or middleware. All errors are uniformly converted to the `ApiResponse::error(...)` format for return.

---

## 9. Non-Functional Design

### 9.1 Performance Design

| Metric | Approach |
|--------|----------|
| **Query response <= 2s (100k rows)** | Proper index design + paginated queries + avoid N+1 queries |
| **Import 100k rows <= 60s** | Batch inserts (SQLx `execute_many`), transaction batch commits (1000 rows per batch) |
| **20+ concurrent users** | SQLite WAL mode allows concurrent read/write; write serialization via `busy_timeout` |
| **Large data queries** | Complex aggregate queries for reports use caching (SQLite materialized views or application-level caching can be introduced later) |

**SQLite Concurrency Considerations**:

```
1. Use WAL mode (default configuration), reads do not block writes
2. Write operations are serialized via Tokio Mutex to avoid SQLITE_BUSY
3. Connection pool: r2d2_sqlite or deadpool-sqlite, max connections = 5
   (In WAL mode, multiple readers can work simultaneously)
4. Long-running write transactions are split into smaller batch transactions
5. Periodically run PRAGMA wal_checkpoint(TRUNCATE) to limit WAL file size
```

### 9.2 Configuration Management

```rust
// Config struct example
pub struct AppConfig {
    pub database_url: String,       // SQLite file path
    pub jwt_secret: String,         // JWT signing key
    pub jwt_expires_in: u64,        // JWT validity period (seconds)
    pub host: String,               // Listen address
    pub port: u16,                  // Listen port
    pub log_level: String,          // Log level
    pub upload_dir: String,         // File upload directory
    pub max_file_size: usize,       // Maximum upload file size
    pub default_language: String,   // Default language
    pub default_unit_system: String,// Default unit system
}
```

### 9.3 Logging Strategy

| Layer | Technology | Description |
|-------|------------|-------------|
| **Application logs** | `tracing` + `tracing-subscriber` | Structured logging, supports JSON output |
| **Audit logs** | `operation_logs` table | All data changes recorded in the database |

Log level configuration:

| Environment | Level |
|-------------|-------|
| Development | `debug` |
| Testing | `info` |
| Production | `warn` |

### 9.4 Data Backup

- The SQLite database file is a single file, suitable for file-level backup
- Recommended strategy: daily full backup + hourly WAL archiving
- Backup script: use the `.backup` command for online hot backup (SQLite backup API)

---

## 10. Security Design

### 10.1 Authentication & Authorization

| Layer | Approach |
|-------|----------|
| **Password storage** | Argon2id (memory 19MB, iterations 2, parallelism 1) |
| **Session management** | JWT (access_token 1h + refresh_token 7d) |
| **API authentication** | JWT Bearer Token middleware |
| **RBAC implementation** | Axum middleware extracts role from token, matches route's `#[require_role]` attribute |
| **Sensitive operation confirmation** | DELETE / critical modifications require secondary confirmation (confirmation dialog on frontend, logged on backend) |

### 10.2 Role Permission Matrix

| Feature / Role | Admin | Warehouse Manager | Quality Inspector | Sales/Procurement |
|----------------|:-----:|:-----------------:|:-----------------:|:-----------------:|
| Pipe view | Yes | Yes | Yes | Yes |
| Pipe create/update/delete | Yes | Yes | Yes | -- |
| Inbound operations | Yes | Yes | -- | Yes (purchase inbound only, order_id required) |
| Outbound operations | Yes | Yes | -- | Yes (sales outbound only, order_id required) |
| Inbound approval (non-purchase types) | Yes | Yes | -- | -- |
| Outbound approval (non-sales types) | Yes | Yes | -- | -- |
| Inventory query | Yes | Yes | Yes | Yes |
| Inventory check | Yes | Yes | -- | -- |
| Location management | Yes | Yes | -- | -- |
| Quality management | Yes | -- | Yes | -- |
| Purchase orders | Yes | -- | -- | Yes |
| Sales orders | Yes | -- | -- | Yes |
| Suppliers/Customers | Yes | -- | -- | Yes |
| Data import/export | Yes | Yes | Yes | Yes |
| Reports & statistics | Yes | Yes | Yes | Yes |
| User management | Yes | -- | -- | -- |
| Operation logs | Yes | (self-view only) | (self-view only) | (self-view only) |
| System configuration | Yes | -- | -- | -- |

### 10.3 Input Validation

- All DTOs use the `validator` crate for field validation
- File uploads: restricted types (PDF / images / Excel only), size limit (max 50MB), sanitized file names
- SQL injection prevention: SQLx parameterized queries ($1, $2 format), no SQL string concatenation
- XSS prevention: handled on the frontend; API returns raw data

### 10.4 Audit Trail

```
Every data modification (create / update / delete) is automatically recorded
in operation_logs:
- Who (user_id + username)
- When (created_at)
- What object (target_type + target_id)
- What action (action)
- Change details (detail: JSON format, records before/after key field values)
- From where (ip_address)
```

---

## 11. Internationalization & Unit Switching Design

### 11.1 Internationalization Approach

**Backend**:

- Use `rust-i18n` or `fluent-rs` to manage message templates
- Message files organized by language:

```
i18n/
├── zh/
│   ├── messages.ftl
│   ├── errors.ftl
│   └── api_5ct.ftl
└── en/
    ├── messages.ftl
    ├── errors.ftl
    └── api_5ct.ftl
```

- Error messages, validation prompts, and status labels are returned in the corresponding language based on the `Accept-Language` header or user preference
- The user `language_pref` field stores the language preference

**Frontend**:

- Use `react-i18next`, message files kept in sync with the backend
- Switching the language updates `language_pref` and refreshes page content

### 11.2 Unit Switching Approach

**Strategy**: Internal storage uniformly uses **imperial units** (API 5CT standard units). Unit conversion is applied at the API layer based on user preference for both input and output.

| Parameter | Internal Storage (Imperial) | Metric Display | Conversion Formula |
|-----------|---------------------------|----------------|-------------------|
| Outer Diameter (OD) | inch | mm | mm = in x 25.4 |
| Wall Thickness (WT) | inch | mm | mm = in x 25.4 |
| Length | foot | m | m = ft x 0.3048 |
| Weight per Unit | lb/ft | kg/m | kg/m = lb/ft x 1.48816 |
| Yield Strength | psi | MPa | MPa = psi x 0.00689476 |

**API Design**:

```json
// Request optionally specifies unit system
POST /api/v1/seamless-pipes?unit_system=metric

// Response includes unit markers
{
  "data": {
    "od": 114.3,          // metric value
    "od_unit": "mm",
    "wt": 6.35,
    "wt_unit": "mm",
    "length": 12.19,
    "length_unit": "m"
  }
}
```

**Implementation**:

- Define a `Measurement<T>` wrapper type at the DTO layer, carrying unit information
- Retrieve the unit preference from user configuration in `AuthService`
- At the Handler layer, a middleware extracts the user's unit preference and passes it to DTO conversion
- The frontend can also perform client-side unit conversion (user-facing), converting display values based on the `unit_system` preference

---

## Appendix A: Key Decision Records (ADR)

### ADR-001: Monolithic Architecture over Microservices

| Item | Content |
|------|---------|
| **Context** | Medium project scale, small team |
| **Decision** | Adopt a modular monolith, internally organized by domain, not split into independent microservices |
| **Rationale** | Reduces operational complexity; a single SQLite database does not support microservices; future splitting can extract modules as independent services along domain boundaries |
| **Consequences** | Strict module boundary design is required to prevent inter-module coupling |

### ADR-002: SQLite over PostgreSQL

| Item | Content |
|------|---------|
| **Context** | Hundreds of thousands of data rows, 20+ concurrent users |
| **Decision** | SQLite WAL mode, with connection pooling and write serialization |
| **Rationale** | Zero configuration, file-level deployment, suitable for this scale; estimated SQLite can support million-level reads and moderate write concurrency |
| **Consequences** | If scale grows significantly, migration to PostgreSQL is feasible (SQLx has good compatibility with both) |

### ADR-003: Imperial Units as Internal Storage Standard

| Item | Content |
|------|---------|
| **Context** | API 5CT standard uses imperial units; domestic users are accustomed to metric |
| **Decision** | Database uniformly stores imperial values; API layer converts based on user preference |
| **Rationale** | Avoids precision loss from repeated metric-imperial conversions; maintains consistency with the standard |
| **Consequences** | All internal calculations use imperial units; unit conversion is done at API input/output |

### ADR-004: Separate Tables for Seamless Pipes and Screen Pipes

| Item | Content |
|------|---------|
| **Context** | Both pipe types share common attributes, but screen pipes have base pipe parameters, filtration precision, and other unique fields |
| **Decision** | Two independent tables, no shared table structure |
| **Rationale** | Significant field differences (screen pipes have unique fields and the base pipe itself uses seamless pipe parameters); cross-type query scenarios are rare; separate tables are clearer |
| **Consequences** | Cross-type search requires a `UNION` query or two independent queries followed by merging |

---

## Appendix B: API 5CT Grade Reference Data (Pre-seeded Data)

Initialization data will be pre-seeded in `migrations/010_seed_api_5ct_data.sql`:

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
