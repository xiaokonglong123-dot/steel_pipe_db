# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed — ERP Refactor (文档先行 / docs-first phase)

> 历史沿革：本系统由钢管行业系统重构而来。自本条目起，项目从钢管行业库存管理系统重构为通用 ERP（企业资源计划系统），以下条目描述新架构；`0.1.0` 及更早条目描述旧版钢管行业系统（legacy steel-pipe system），仅供参考。

- Project renamed to **ERP（通用企业资源计划系统）**; backend crate renamed to `erp-server` (documented target for the code phase)
- Database migrated to **SQLite3** (connection string `sqlite://data/erp.db?mode=rwc`, sqlx 0.8 `sqlite` feature); all legacy DB tooling references removed from docs
- Migration strategy: the 37 legacy migrations are rewritten to SQLite syntax, with pipe-specific tables dropped
- Inventory generalized to 商品 (Item) + SKU master data (`sku` / 名称 / 分类 / 单位 / 规格), no industry-specific fields
- Modules removed: pipe master data, label printing, quality certification records, industry reference data, and pipe-specific search / import-export logic
- Modules kept/generalized: auth/RBAC, workflow 审批, hr, finance, procurement, sales_crm, inventory (商品/SKU), manufacturing, project, assets, notification, portal, bi, customers, suppliers, contracts, purchases, sales
- All documentation rewritten to the new architecture with the terminology of `specs/UBIQUITOUS_LANGUAGE_LATEST.md` (商品/Item+SKU, 采购订单, 销售订单, 质检/Inspection, 工单)

### Added

- RBAC permission matrix documentation in README
- Deployment guide (`docs/deployment.md`)
- Troubleshooting guide (`docs/troubleshooting.md`)
- Contributing guide (`CONTRIBUTING.md`)
- Detailed comments in all migration files
- Module-level documentation in `router.rs` (route organization, middleware layering, RBAC reference)
- Annotated `.env.example` with Chinese comments and production security notes

### Changed

- Translated `README_zh.md` to full Chinese (was previously identical to English version)
- Fixed design document references in README (outdated filenames → actual filenames)
- Updated `backend/AGENTS.md` service module structure (reflected `inventory_service.rs` split into focused modules)

---

## [0.1.0] - 2025-05-27

> 注：以下条目描述旧版钢管行业系统（legacy steel-pipe system）的功能，仅作历史记录，不属于 ERP 新架构。

### Added — Phase 1: Core (P0)

- JWT authentication: login, refresh, logout, password change
- RBAC with 4 roles: admin, warehouse, qc, sales
- Pipe master data (CRUD, search, filter)
- Pipe master data variants (CRUD, search, filter)
- Inventory tracking: per-item granular stock management
- Inbound records: create, approve, reject, batch create
- Outbound records: create, approve, reject
- Warehouse locations: zone/shelf/level hierarchy, assign, transfer
- Inventory checks (盘点): create, submit items, complete
- ATP (Available-to-Promise) calculation
- Full-lifecycle item tracing (by item, batch, order)
- Inventory statistics dashboard

### Added — Phase 2: Business (P1)

- Supplier management (CRUD, search, active list)
- Customer management (CRUD, search, active list)
- Purchase order lifecycle (draft → submitted → approved → completed)
- Sales order lifecycle with ATP checks
- Quality inspection records (CRUD)
- Quality mechanical test results
- Quality NDT results (UT/MI/MPI)
- Data import/export: Excel (.xlsx) and CSV batch operations
- Import templates download
- Operation logs for data IO audit trail
- Rate limiting on login, password change, and import endpoints

### Added — Phase 3: Enterprise (P2)

- Contract management with payment milestones
- Reports: inventory summary, order report, inspection report, dashboard
- Label generation: pipe barcode labels, QC labels, shipping labels, batch labels
- Internationalization: zh-CN / en-US with per-module namespaces

### Technical

- Backend: Rust + Axum 0.8 + SQLx 0.8 + SQLite (WAL mode)
- Frontend: React 19 + Ant Design 5 + TanStack Query 5 + Vite 6
- CI: GitHub Actions (cargo check + tsc + vite build)
- SQLite schema with soft deletes
- ~70 REST API endpoints under `/api/v1/`
- Numeric error codes (100xx–50001) with domain prefixes
- Request ID propagation (UUID v4) in all responses
