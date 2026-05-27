# Seamless Steel Pipe & Screen Pipe Management System — PRD

> **Version**: v1.0
> **Date**: 2026-05-19
> **Standard**: API 5CT (10th Edition / ISO 11960)
> **Stack**: Rust + React
> **Type**: Web App (Frontend/Backend split)

---

## Revision History

| Version | Date | Changes | Author |
|---------|------|---------|--------|
| v1.0 | 2026-05-19 | Initial version | - |

---

## 1. Project Background & Objectives

### 1.1 Background

Seamless steel pipes (Casing / Tubing) and screen pipes are the backbone of oil & gas well construction — both governed by the **API 5CT** standard. Right now there's no integrated system that tracks the full lifecycle of these pipes: procurement, inbound, warehouse storage, sales outbound, and quality traceability. Everything's scattered across spreadsheets and paper forms.

### 1.2 Objective

Build a Rust-based web system that handles **procurement + sales + inventory + quality** for seamless steel pipes and screen pipes in one place. Full traceability from the moment a pipe arrives until it ships out.

---

## 2. Target Users & Roles

| Role | What They Do | Key Concerns |
|------|-------------|--------------|
| **Warehouse Operator** | Inbound, outbound, transfer, stock count, label printing | Stock levels, bin locations, speed |
| **Quality Inspector** | QC data entry, cert management, cert-to-pipe linking | Grade/spec consistency, heat/lot traceability, cert docs |
| **Sales/Procurement Staff** | PO/SO management, supplier/customer management | ATP stock, order status, contract info |
| **Management** | Dashboards, reports, decisions | Inventory turns, stock value, biz metrics |

System is **multi-user**, role-based access (RBAC).

---

## 3. Functional Requirements

### 3.1 Pipe Information Management (P0 — Must Have)

**FR-PIPE-001: Pipe Master Data Management**
- **Description**: CRUD for both seamless steel pipe and screen pipe types
- **Seamless Pipe Fields**:
  - Pipe number (unique across the system)
  - Product type: Casing / Tubing
  - Grade: H40, J55, K55, N80, L80, C90, T95, P110, Q125, etc.
  - Dimensions: OD, WT, Length, Unit Weight
  - End type: SC, LC, BC, Extreme Line, etc.
  - Coupling: type, OD, length
  - Heat number / Lot number
  - Pipe body serial number
- **Screen Pipe Specific Fields**:
  - Screen type: Wire-wrapped, Slotted, Punched, Metal mesh, etc.
  - Base pipe params: OD, WT, grade (the base pipe is itself a seamless pipe)
  - Slot width / aperture
  - Filtration rating (e.g., 150μm, 250μm)
  - Screen pipe length and connection type
- **Common Fields**: Manufacturer, prod date, QC cert number, attachments (PDF scans, etc.)
- **Acceptance Criteria**:
  - Full CRUD
  - Pipe number globally unique
  - Search by any combination of fields
  - Screen and seamless managed separately but searchable from one place

### 3.2 Inventory Management (P0 — Must Have)

**FR-INV-001: Inbound Management**
- Multiple inbound types: purchase, production return, customer return, etc.
- Each record: inbound order number, date, pipe details (number/qty/batch), supplier/source, operator
- Auto-update stock on inbound

**FR-INV-002: Outbound Management**
- Sales outbound, internal requisition, transfer outbound, etc.
- Each record: outbound order number, date, customer/destination, pipe details, operator
- Batch or single-piece outbound (precise by pipe number)

**FR-INV-003: Stock Query & Count**
- Real-time stock by pipe type, grade, spec, location, etc.
- Movement ledger — full inbound/outbound history per pipe
- Stock count: generate count sheets, enter counts, produce variance reports

**FR-INV-004: Location Management**
- Multi-level locations: zone / rack / shelf
- Bind pipes to locations, support moves

### 3.3 Quality Management (P1 — Should Have)

**FR-QA-001: QC Info Management**
- Link QC reports/certs to individual pipes
- Fields: test items, results, date, agency, inspector
- File uploads (PDF/images)

**FR-QA-002: Quality Traceability**
- Trace by heat number → production batch
- Trace by pipe number → full quality history
- Reference API 5CT grade mechanical properties and chemical composition for comparison

### 3.4 Procurement Management (P1 — Should Have)

**FR-PUR-001: Procurement Management**
- PO creation, approval flow, tracking
- Supplier info management (name, contact, qualifications, etc.)
- Purchase arrival links to inbound

**FR-SALE-001: Sales Management**
- SO creation, approval flow, tracking
- Customer info management
- Sales outbound deducts inventory
- Available-to-promise (ATP) stock visibility

**FR-CONTRACT-001: Contract Management**
- Basic contract info for procurement and sales
- Link contracts to orders

### 3.5 Data Import/Export (P1 — Should Have)

**FR-IO-001: Data Import**
- Excel/CSV batch import for pipe data
- Validate format and required fields during import
- Import result report (success/fail counts + reasons)

**FR-IO-002: Data Export**
- Export query results to Excel/CSV
- Standard reports: inventory, inbound/outbound details, quality reports, etc.

### 3.6 Search & Filter (P0 — Must Have)

**FR-SEARCH-001: Multi-dimensional Search**
- Combined queries: pipe number, type, grade, spec (OD/WT), status, location, etc.
- Fuzzy search (partial pipe number, grade name, etc.)
- Paginated results

### 3.7 Reports & Statistics (P2 — Could Have)

**FR-RPT-001: Inventory Reports**
- Current stock summary (grouped by type/grade/spec)
- Monthly/quarterly movement reports

**FR-RPT-002: Business Reports**
- Inbound/outbound charts
- Inventory turnover analysis
- Procurement/sales trend analysis

### 3.8 Label Printing (P2 — Could Have)

**FR-LABEL-001: Barcode/QR Code Labels**
- Generate barcode or QR code labels from pipe data
- Batch printing
- Configurable templates (fields, layout)

### 3.9 History Traceability (P0 — Must Have)

**FR-TRACE-001: Full Lifecycle Traceability**
- Every operation on every pipe logged — from inbound to outbound
- Every change tracked: who, when, what fields
- View full lifecycle by pipe number

### 3.10 System Management (P1 — Should Have)

**FR-SYS-001: User & Permission Management**
- User management
- RBAC with 4 roles: Warehouse, QC, Sales/Procurement, Admin
- Menus and buttons adapt to role

**FR-SYS-002: Operation Logs**
- Log key user actions: login, data changes, etc.
- Queryable and exportable

---

## 4. Non-Functional Requirements

### 4.1 Performance

| Metric | Target |
|--------|--------|
| Single page query response | ≤ 2s (within 100K records) |
| Data import | ≤ 60s for 100K records |
| Concurrent users | ≥ 20 simultaneous |
| Availability | 99.5% (≤ 44 hrs/year downtime) |

### 4.2 Data Scale

- Pipe master data: 100K+ records
- Inventory movement logs: millions of records
- Storage: SQLite (WAL mode, well-indexed)

> **Note**: SQLite handles single-machine / small-scale concurrency fine. WAL mode + connection pool are key for a web app. If concurrency grows beyond that, PostgreSQL is an easy migration path.

### 4.3 Internationalization & Units

- **UI Language**: Chinese + English, switchable at runtime
- **Unit System**: Metric (mm, kg/m, m) and Imperial (in, lb/ft, ft) — toggle on the fly
- Internal storage unified (metric) with unit metadata

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
|-------|-----------|-----|
| **Backend** | Rust + Axum 0.8 + SQLx 0.8 | Axum is the most ergonomic async web framework in Rust right now. SQLx gives us compile-time checked SQL. No ORM overhead. |
| **Database** | SQLite (WAL mode) | Zero config, file-level, perfect for this scale. WAL handles concurrent reads fine. |
| **Frontend** | React 19 + TypeScript (strict) + Vite | React 19 is the latest stable. Vite is insanely fast for dev. TypeScript strict catches nulls and bad types. |
| **API** | JSON REST | Standard RESTful — easy to integrate, debug with curl, works with any frontend. |

---

## 5. API 5CT Standard Reference

> API 5CT (Specification for Casing and Tubing) is *the* standard for oilfield pipe. Here's the reference data that drives the system's field design.

### 5.1 Grade Classification

| Group | Grade | Type | Key Characteristics |
|-------|-------|------|---------------------|
| **H** | H40 | Casing/Tubing | Lowest strength, non-critical wells |
| **J/K** | J55, K55 | Casing/Tubing | Medium strength, medium-depth wells |
| **N** | N80 | Casing/Tubing | Higher strength, two heat treatments (N80-1 normalized, N80-Q quenched+tempered) |
| **L** | L80 | Casing/Tubing | Corrosion resistant, Cr content, for H₂S environments |
| **C** | C90, C95 | Casing/Tubing | Corrosion resistant, sour service |
| **T** | T95 | Casing | High collapse resistance, sour service |
| **P** | P110 | Casing/Tubing | High strength, deep wells |
| **Q** | Q125 | Casing | Ultra-high strength, ultra-deep wells |

### 5.2 End Types

- **SC** (Short Round Thread) — Shallower wells
- **LC** (Long Round Thread) — Higher connection strength than SC
- **BC** (Buttress Thread) — High connection strength
- **X** (Extreme Line) — Special operating conditions

### 5.3 Units

API 5CT is imperial by default:

| Parameter | Imperial | Metric |
|-----------|----------|--------|
| OD | inch (in) | millimeter (mm) |
| WT | inch (in) | millimeter (mm) |
| Length | foot (ft) | meter (m) |
| Unit Weight | lb/ft | kg/m |
| Yield Strength | psi | MPa |

### 5.4 Standard Size Reference

| OD (in) | WT (in) | Unit Weight (lb/ft) | Common Grades |
|---------|---------|-------------------|---------------|
| 4½ | 0.250 ~ 0.337 | 11.60 ~ 15.10 | J55, N80, L80 |
| 5½ | 0.304 ~ 0.415 | 17.00 ~ 23.00 | J55, N80, L80, P110 |
| 7 | 0.317 ~ 0.582 | 23.00 ~ 41.00 | J55, N80, L80, P110 |
| 9⅝ | 0.395 ~ 0.595 | 40.00 ~ 59.20 | J55, N80, L80, P110 |
| 13⅜ | 0.514 ~ 0.672 | 72.00 ~ 92.50 | H40, J55, K55 |

*(Partial — full reference table is seeded in the DB via migration 010)*

---

## 6. Preliminary Data Model

### 6.1 Core Entity Relationships

```
                      ┌──────────────────────┐
                      │   SeamlessPipe        │  ← Seamless pipe (own table)
                       │   (Seamless Pipe)       │
                      └────────┬─────────────┘
                               │
Supplier ──→ PurchaseOrder ──→ InboundRecord ──→ SeamlessPipe/ScreenPipe ──→ OutboundRecord ──→ Customer
                     ↑                                                          │
                Contract                                                   SalesOrder
                     │                                                          ↑
                     └──────────────────────────────────────────────────────────┘

                      ┌──────────────────────┐
                      │   ScreenPipe           │  ← Screen pipe (own table)
                       │   (Screen Pipe)         │
                      └──────────────────────┘
```

> SeamlessPipe and ScreenPipe are **two independent tables** — their field sets are different enough that sharing a table would be more trouble than it's worth.

### 6.2 Main Data Entities

| Entity | Description | Core Fields |
|--------|-------------|-------------|
| **SeamlessPipe** | Seamless steel pipe | id, pipe_number, pipe_type(casing/tubing), grade, od, wt, length, weight, end_type, coupling_type, coupling_od, coupling_length, heat_number, serial_number, manufacturer, prod_date, cert_number, location_id, status, created_at, updated_at |
| **ScreenPipe** | Screen pipe | id, pipe_number, screen_type(wire-wrapped/slotted/punched), slot_size, filtration_grade, base_od, base_wt, base_grade, base_end_type, length, weight, heat_number, serial_number, manufacturer, prod_date, cert_number, location_id, status, created_at, updated_at |
| **Location** | Storage spot | id, zone_code, shelf_code, level_code, description |
| **Supplier** | Supplier | id, name, contact, phone, address, qual_cert |
| **Customer** | Customer | id, name, contact, phone, address |
| **PurchaseOrder** | Purchase order | id, order_no, supplier_id, order_date, status, total_amount |
| **SalesOrder** | Sales order | id, order_no, customer_id, order_date, status, total_amount |
| **InboundRecord** | Inbound record | id, record_no, type, pipe_type(seamless/screen), pipe_id, qty, date, order_id, operator, remark |
| **OutboundRecord** | Outbound record | id, record_no, type, pipe_type(seamless/screen), pipe_id, qty, date, order_id, operator, remark |
| **InventoryLog** | Movement log | id, pipe_type, pipe_id, change_type, qty_before, qty_after, operation, operator, timestamp |
| **QualityCert** | QC certificate | id, pipe_type, pipe_id, cert_no, inspect_date, inspector, agency, file_url, result, remark |
| **User** | System user | id, username, password_hash, role, name, email, language_pref, unit_system |
| **OperationLog** | Audit log | id, user_id, action, target_type, target_id, detail, ip_address, timestamp |

---

## 7. Priority Roadmap

| Phase | Scope | Priority |
|-------|-------|----------|
| **Phase 1 (MVP)** | Pipe CRUD, Inventory (inbound/outbound/query), Multi-dimensional search, User permissions, History traceability | P0 |
| **Phase 2** | Quality management, Procurement/Sales management, Data import/export (Excel/CSV) | P1 |
| **Phase 3** | Reports & dashboards, Label printing, Contract management, i18n (zh/en + unit switch) | P2 |

---

## 8. Appendix

### 8.1 Glossary

| Term | English | What It Is |
|------|---------|------------|
| 无缝钢管 | Seamless Pipe | Steel pipe made by piercing — used as casing or tubing |
| 筛管 | Screen Pipe | Filter pipe with slots or wire wrap over a base pipe |
| 套管 | Casing | Pipe string that supports the wellbore |
| 油管 | Tubing | Pipe inside casing that carries oil/gas up |
| API 5CT | API Specification 5CT | The governing spec for casing and tubing |
| 钢级 | Grade | Strength rating of the pipe |
| 炉批号 | Heat Number | Steel furnace batch ID — key for traceability |
| 接箍 | Coupling | Connector that joins two pipe joints together |

### 8.2 Related Documents

- API 5CT: Specification for Casing and Tubing (ISO 11960)
- Detailed design: DB schema, API endpoints, architecture
- Frontend design: Component tree, routing, state management
