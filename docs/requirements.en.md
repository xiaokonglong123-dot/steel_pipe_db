# Seamless Steel Pipe & Screen Pipe Management System — PRD

> **Document Version**: v1.0
> **Date**: 2026-05-19
> **Applicable Standard**: API 5CT (10th Edition / ISO 11960)
> **Development Language**: Rust
> **System Type**: Web Application (Frontend/Backend Separation)

---

## Revision History

| Version | Date | Revisions | Author |
|---------|------|-----------|--------|
| v1.0 | 2026-05-19 | Initial version | - |

---

## 1. Project Background & Objectives

### 1.1 Background

Seamless steel pipes (Casing / Tubing) and screen pipes are critical tubular goods in oil and gas well construction, both conforming to the **API 5CT** standard. Currently, there is no integrated information management system to handle the full lifecycle of these two types of pipe — from procurement, inbound, and inventory management to sales outbound and quality traceability.

### 1.2 Objective

Build a Rust-based web management system for integrated **procurement + sales + inventory + quality** management of seamless steel pipes and screen pipes, covering full-process data traceability from pipe arrival to shipment.

---

## 2. Target Users & Roles

| Role | Responsibilities | Key Focus |
|------|-----------------|-----------|
| **Warehouse Operator** | Pipe inbound, outbound, transfer, inventory count, label printing | Stock quantity, location info, operation efficiency |
| **Quality Inspector** | Quality data entry, QC report management, certificate association | Grade/spec consistency, heat/lot traceability, QC certificates |
| **Sales/Procurement Staff** | Purchase order management, sales order management, customer/supplier management | Available-to-promise stock, order progress, contract info |
| **Management** | Data overview, report viewing, decision analysis | Inventory turnover, inventory value, business statistics |

The system is **multi-user** with role-based access control (RBAC).

---

## 3. Functional Requirements

### 3.1 Pipe Information Management (P0 — Must Have)

**FR-PIPE-001: Pipe Master Data Management**
- **Description**: CRUD operations for both seamless steel pipe and screen pipe types
- **Seamless Pipe Attributes**:
  - Pipe number (unique identifier)
  - Product type: Casing / Tubing
  - Grade: H40, J55, K55, N80, L80, C90, T95, P110, Q125, etc.
  - Dimensions: Outer Diameter (OD), Wall Thickness (WT), Length, Unit Weight
  - End type: Short Round Thread (SC), Long Round Thread (LC), Buttress Thread (BC), Extreme Line, etc.
  - Coupling parameters: coupling type, coupling OD, coupling length
  - Heat number / Lot number
  - Pipe body serial number
- **Screen Pipe Specific Attributes**:
  - Screen type: Wire-wrapped, Slotted, Punched, Metal mesh, etc.
  - Base pipe parameters: OD, WT, grade (the base pipe itself is a seamless steel pipe)
  - Slot width / aperture specifications
  - Filtration rating (e.g., 150μm, 250μm, etc.)
  - Screen pipe length and connection type
- **Common Attributes**: Manufacturer, production date, QC certificate number, attachments (certificate scans, etc.)
- **Acceptance Criteria**:
  - Support add/edit/delete pipe entries
  - Pipe number must be globally unique
  - Support search by any combination of attributes
  - Screen pipes and seamless pipes managed by type but queryable in a unified list

### 3.2 Inventory Management (P0 — Must Have)

**FR-INV-001: Inbound Management**
- Support multiple inbound types: purchase inbound, production inbound, return inbound, etc.
- Record on inbound: inbound order number, date, pipe details (number/quantity/batch), supplier/source, operator
- Auto-update inventory quantity

**FR-INV-002: Outbound Management**
- Support sales outbound, requisition outbound, transfer outbound, etc.
- Record on outbound: outbound order number, date, customer/destination, pipe details, operator
- Support batch/single-piece outbound (precise outbound by pipe number)

**FR-INV-003: Inventory Query & Count**
- Real-time inventory query: by pipe type, grade, specification, location, etc.
- Inventory movement details (ledger): inbound/outbound history for each pipe
- Inventory count: generate count list, enter count data, generate discrepancy report

**FR-INV-004: Location Management**
- Multi-level location management: zone/rack/shelf
- Bind pipes to locations, support location transfers

### 3.3 Quality Management (P1 — Should Have)

**FR-QA-001: QC Information Management**
- Associate QC reports/certificates with pipes
- Record: test items, test results, test date, test agency, inspector
- Support uploading test files (PDF/images)

**FR-QA-002: Quality Traceability**
- Trace to production batch via heat number
- Trace to individual pipe's full quality history via pipe number
- Reference API 5CT standard grade mechanical properties and chemical composition data for comparison

### 3.4 Procurement Management (P1 — Should Have)

**FR-PUR-001: Procurement Management**
- Purchase order creation, approval, tracking
- Supplier information management (name, contact, qualifications, etc.)
- Purchase arrival linked with inbound

**FR-SALE-001: Sales Management**
- Sales order creation, approval, tracking
- Customer information management
- Sales outbound linked with inventory deduction
- View available-to-promise (ATP) stock

**FR-CONTRACT-001: Contract Management**
- Basic information management for procurement and sales contracts
- Contract-to-order association

### 3.5 Data Import/Export (P1 — Should Have)

**FR-IO-001: Data Import**
- Support Excel/CSV batch import of pipe information
- Validate data format and required fields during import
- Provide import result report (success/failure row counts and reasons)

**FR-IO-002: Data Export**
- Export query results to Excel/CSV
- Support exporting standard reports: inventory reports, inbound/outbound details, quality reports, etc.

### 3.6 Search & Filter (P0 — Must Have)

**FR-SEARCH-001: Multi-dimensional Search**
- Combined queries by pipe number, pipe type, grade, specification (OD/WT), status, location, etc.
- Support fuzzy search (partial pipe number, grade name, etc.)
- Paginated search results

### 3.7 Reports & Statistics (P2 — Could Have)

**FR-RPT-001: Inventory Reports**
- Current inventory summary (grouped by type/grade/spec)
- Monthly/quarterly inventory movement reports

**FR-RPT-002: Business Reports**
- Inbound/outbound statistics charts
- Inventory turnover analysis
- Procurement/sales trend analysis

### 3.8 Label Printing (P2 — Could Have)

**FR-LABEL-001: Barcode/QR Code Label Printing**
- Generate barcode or QR code labels from pipe information
- Support batch printing
- Configurable label templates (display fields, layout)

### 3.9 History Traceability (P0 — Must Have)

**FR-TRACE-001: Full Lifecycle Traceability**
- Record all operational logs for each pipe from inbound to outbound
- Record all change logs (who, when, which fields changed)
- Support viewing complete lifecycle by pipe number

### 3.10 System Management (P1 — Should Have)

**FR-SYS-001: User & Permission Management**
- User registration/management
- Role-Based Access Control (RBAC)
  - Warehouse Operator, Quality Inspector, Sales/Procurement Staff, Administrator
- Different roles see different menus and action buttons

**FR-SYS-002: Operation Logs**
- Record key user operations: login, data modifications, etc.
- Logs queryable and exportable

---

## 4. Non-Functional Requirements

### 4.1 Performance

| Metric | Target |
|--------|--------|
| Single page query response time | ≤ 2 seconds (within 100K records) |
| Data import performance | ≤ 60 seconds for 100K records |
| Concurrent users | Support ≥ 20 simultaneous users |
| System availability | 99.5% (downtime ≤ 44 hours/year) |

### 4.2 Data Scale

- Pipe master data: Hundreds of thousands (≥ 100K records)
- Inventory movement logs: Millions of records
- Storage solution: SQLite (requires well-designed indexes for large-scale data)

> **Note**: SQLite is suitable for single-machine/small-scale concurrent scenarios. For multi-user web application write concurrency, WAL mode configuration and connection pool management are important. If future concurrency needs grow, migration to PostgreSQL is possible.

### 4.3 Internationalization & Units

- **UI Language**: Support Chinese and English switching
- **Unit System**: Support metric (mm, kg/m, m) and imperial (in, lb/ft, ft) switching
- Input can switch unit systems; internal storage unified to metric or with unit markers

### 4.4 Security

- User passwords encrypted with bcrypt / argon2
- Sensitive operations (delete/modify critical data) require confirmation
- API request authentication (JWT / Session)
- HTTPS (production environment)

### 4.5 Maintainability

- Rust type-safe system reduces runtime errors
- Modular architecture with clear package structure
- Backend provides RESTful API, frontend communicates via HTTP
- API documentation (OpenAPI / Swagger)

### 4.6 System Architecture & Technology Stack

| Layer | Recommended Technology | Description |
|-------|----------------------|-------------|
| **Backend** | Rust + Axum + SQLx | Axum is the most mainstream async web framework in the Rust ecosystem; SQLx provides compile-time SQL checking |
| **Database** | SQLite (WAL mode) | Zero-configuration, file-level database suitable for small-to-medium scale applications |
| **Frontend** | React / Vue / Leptos | Frontend-backend separation. Leptos for full Rust stack, otherwise React/Vue is more mature |
| **API** | JSON REST API | Standard RESTful interface for easy third-party integration |

---

## 5. API 5CT Standard Reference

> API 5CT (Specification for Casing and Tubing) is one of the most important pipe standards in the oil and gas industry. The following is key reference information to guide system field design.

### 5.1 Common Grade Classification

| Grade Group | Grade | Type | Key Characteristics |
|-------------|-------|------|---------------------|
| **Group H** | H40 | Casing/Tubing | Lowest strength, non-critical wells |
| **Group J/K** | J55, K55 | Casing/Tubing | Medium strength, medium-depth wells |
| **Group N** | N80 | Casing/Tubing | Higher strength, N80-1 and N80-Q two heat treatment states |
| **Group L** | L80 | Casing/Tubing | Corrosion resistant, contains Cr, for H₂S environments |
| **Group C** | C90, C95 | Casing/Tubing | Corrosion resistant, for sour environments |
| **Group T** | T95 | Casing | High collapse resistance, suitable for sour environments |
| **Group P** | P110 | Casing/Tubing | High strength, deep wells |
| **Group Q** | Q125 | Casing | Ultra-high strength, ultra-deep wells |

### 5.2 End Types

- **SC** (Short Round Thread) — For shallower wells
- **LC** (Long Round Thread) — Higher connection strength than SC
- **BC** (Buttress Thread) — High connection strength
- **X** (Extreme Line) — For special operating conditions

### 5.3 Units of Measurement

API 5CT standard uses imperial units:

| Parameter | Imperial Unit | Metric Unit |
|-----------|---------------|-------------|
| Outer Diameter (OD) | inch (in) | millimeter (mm) |
| Wall Thickness (WT) | inch (in) | millimeter (mm) |
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

*(Above are partial common size examples; complete reference table should be included in system reference data)*

---

## 6. Preliminary Data Model

### 6.1 Core Entity Relationships

```
                      ┌──────────────────────┐
                      │   SeamlessPipe        │  ← Seamless pipe (independent table)
                      │   (无缝钢管)           │
                      └────────┬─────────────┘
                               │
Supplier ──→ PurchaseOrder ──→ InboundRecord ──→ SeamlessPipe/ScreenPipe ──→ OutboundRecord ──→ Customer
                     ↑                                                          │
                Contract                                                   SalesOrder
                     │                                                          ↑
                     └──────────────────────────────────────────────────────────┘

                      ┌──────────────────────┐
                      │   ScreenPipe           │  ← Screen pipe (independent table)
                      │   (筛管)               │
                      └──────────────────────┘
```

> SeamlessPipe and ScreenPipe are **two independent data tables**, each with its own complete field system; they do not share table structure.

### 6.2 Main Data Entities

| Entity | Description | Core Fields |
|--------|-------------|-------------|
| **SeamlessPipe** | Seamless steel pipe | id, pipe_number, pipe_type(casing/tubing), grade, od, wt, length, weight, end_type, coupling_type, coupling_od, coupling_length, heat_number, serial_number, manufacturer, production_date, cert_number, location_id, status, created_at, updated_at |
| **ScreenPipe** | Screen pipe | id, pipe_number, screen_type(wire-wrapped/slotted/punched), slot_size, filtration_grade, base_od, base_wt, base_grade, base_end_type, length, weight, heat_number, serial_number, manufacturer, production_date, cert_number, location_id, status, created_at, updated_at |
| **Location** | Storage location | id, zone_code, shelf_code, level_code, description |
| **Supplier** | Supplier | id, name, contact, phone, address, qual_cert |
| **Customer** | Customer | id, name, contact, phone, address |
| **PurchaseOrder** | Purchase order | id, order_no, supplier_id, order_date, status, total_amount |
| **SalesOrder** | Sales order | id, order_no, customer_id, order_date, status, total_amount |
| **InboundRecord** | Inbound record | id, record_no, type, pipe_type(seamless/screen), pipe_id, quantity, date, order_id, operator, remark |
| **OutboundRecord** | Outbound record | id, record_no, type, pipe_type(seamless/screen), pipe_id, quantity, date, order_id, operator, remark |
| **InventoryLog** | Inventory movement log | id, pipe_type(seamless/screen), pipe_id, change_type, quantity_before, quantity_after, operation, operator, timestamp |
| **QualityCert** | Quality certificate | id, pipe_type(seamless/screen), pipe_id, cert_no, inspect_date, inspector, agency, file_url, result, remark |
| **User** | User | id, username, password_hash, role, name, email, language_pref, unit_system |
| **OperationLog** | Operation log | id, user_id, action, target_type, target_id, detail, ip_address, timestamp |

---

## 7. Priority Roadmap

| Phase | Feature Scope | Priority |
|-------|--------------|----------|
| **Phase 1 (MVP)** | Pipe information management (CRUD), Inventory management (inbound/outbound/query), Multi-dimensional search, Basic user permissions, History traceability | P0 |
| **Phase 2** | Quality management (QC info & traceability), Procurement/Sales management, Data import/export (Excel/CSV) | P1 |
| **Phase 3** | Reports & statistics, Label printing, Contract management, Internationalization (zh/en + unit switching) | P2 |

---

## 8. Appendix

### 8.1 Glossary

| Term | English | Description |
|------|---------|-------------|
| 无缝钢管 | Seamless Pipe | Seamless steel pipe made by piercing process, used as casing or tubing |
| 筛管 | Screen Pipe | Filter pipe with slots or wire-wrapped screen on base pipe |
| 套管 | Casing | Steel pipe string run into the well to support the wellbore |
| 油管 | Tubing | Steel pipe run inside casing for oil/gas conveyance |
| API 5CT | API Specification 5CT | American Petroleum Institute specification for casing and tubing |
| 钢级 | Grade | Strength grade designation for pipe |
| 炉批号 | Heat Number | Steel furnace heat number, used for quality traceability |
| 接箍 | Coupling | Connector for joining two pipes |

### 8.2 Related Documents

- API 5CT Standard: Specification for Casing and Tubing (ISO 11960)
- Subsequent documents: Database design document, API interface document, User manual
