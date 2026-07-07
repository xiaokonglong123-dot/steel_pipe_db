# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- RBAC permission matrix documentation in README
- Deployment guide (`docs/deployment.md`)
- Troubleshooting guide (`docs/troubleshooting.md`)
- Contributing guide (`CONTRIBUTING.md`)
- Detailed comments in all migration files (001–012)
- Module-level documentation in `router.rs` (route organization, middleware layering, RBAC reference)
- Annotated `.env.example` with Chinese comments and production security notes

### Changed
- Translated `README_zh.md` to full Chinese (was previously identical to English version)
- Fixed design document references in README (outdated filenames → actual filenames)
- Updated `backend/AGENTS.md` service module structure (reflected `inventory_service.rs` split into 5 focused modules)

---

## [0.1.0] - 2025-05-27

### Added — Phase 1: Core (P0)
- JWT authentication: login, refresh, logout, password change
- RBAC with 4 roles: admin, warehouse, qc, sales
- API 5CT seamless pipe master data (CRUD, search, filter)
- API 5CT screen pipe master data (CRUD, search, filter)
- Inventory tracking: per-pipe granular stock management
- Inbound records: create, approve, reject, batch create
- Outbound records: create, approve, reject
- Warehouse locations: zone/shelf/level hierarchy, assign, transfer
- Inventory checks (盘点): create, submit items, complete
- ATP (Available-to-Promise) calculation
- Full-lifecycle pipe tracing (by pipe, heat number, order)
- Inventory statistics dashboard

### Added — Phase 2: Business (P1)
- Supplier management (CRUD, search, active list)
- Customer management (CRUD, search, active list)
- Purchase order lifecycle (draft → submitted → approved → completed)
- Sales order lifecycle with ATP checks
- Quality inspection certificates (CRUD)
- Quality mechanical test results
- Quality NDT results (UT/MI/MPI)
- Data import/export: Excel (.xlsx) and CSV batch operations
- Import templates download
- Operation logs for data IO audit trail
- Rate limiting on login, password change, and import endpoints

### Added — Phase 3: Enterprise (P2)
- Contract management with payment milestones
- Reports: inventory summary, order report, quality report, dashboard
- Label generation: pipe barcode labels, QC labels, shipping labels, batch labels
- Internationalization: zh-CN / en-US with per-module namespaces

### Technical
- Backend: Rust + Axum 0.8 + SQLx 0.8 + SQLite (WAL mode)
- Frontend: React 19 + Ant Design 5 + TanStack Query 5 + Vite 6
- CI: GitHub Actions (cargo check + tsc + vite build)
- 19-table SQLite schema with soft deletes
- ~70 REST API endpoints under `/api/v1/`
- Numeric error codes (100xx–50001) with domain prefixes
- Request ID propagation (UUID v4) in all responses
