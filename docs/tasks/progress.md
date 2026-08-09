# Master Task Tracker — Overall Progress

> Last updated: 2026-08-08
> Stack: Rust + Axum + SQLx + SQLite (backend, crate `erp-server`) / Vite + React 19 + Ant Design 5 + TanStack Query + Zustand (frontend)
> **ERP 重构状态**：本系统由钢管行业系统重构为通用 ERP（企业资源计划系统）。商品主数据统一收敛为「商品 (Item) + SKU」；钢管行业遗留模块（管材主数据、标签打印、质证书）已下线；术语规范见 `specs/UBIQUITOUS_LANGUAGE_LATEST.md`。

---

## Generated Task Files

| Module | Phase | Backend | Frontend | Task Count | Status |
| -------- | ------- | --------- | ---------- | ------------ | -------- |
| Item & SKU Management (商品/SKU 主数据) | P0 | ✅ | ✅ | 21+18 | **活** |
| Inventory Management (库存) | P0 | ✅ | ✅ | 21+18 | **活** |
| Auth & System Management (认证与系统管理) | P0 | ✅ | ✅ | 20+19 | **活** |
| Traceability (可追溯) | P0 | ✅ | ✅ | 12+4 | **活** |
| Inspection (制造质检) | P1 | ✅ | ✅ | 18+15 | **活** |
| Purchase Management (采购) | P1 | ✅ | ✅ | 16+11 | **活** |
| Sales Management (销售) | P1 | ✅ | ✅ | 16+12 | **活** |
| Data Import / Export (数据导入导出) | P1 | ✅ | ✅ | 14+8 | **活** |
| Contract Management (合同) | P2 | ✅ | ✅ | 14+12 | **活** |
| Reports & Statistics (报表) | P2 | ✅ | ✅ | 14+15 | **活** |
| Internationalization & Unit Switching (国际化与单位切换) | P2 | — | ✅ | —+10 | **活** |
| ~~Pipe Management (管材主数据)~~ | P0 | ✅ | ✅ | 18+15 | **归档 — 模块已删除** |
| ~~Quality Management (管材质检证书)~~ | P1 | ✅ | ✅ | 18+15 | **归档 — 模块已删除** |
| ~~Label Printing (标签打印)~~ | P2 | ✅ | ✅ | 12+10 | **归档 — 模块已删除** |
| **Total (活 + 归档)** | | **14 backend** | **14 frontend** | **~330 items** | |

---

## Phase 1 — MVP / P0 (Highest Priority)

> Goal: Core skeleton — 商品/SKU、库存、认证 三大基础模块跑通

### Backend Modules

- [x] **Pipe Management** `phase1/backend-pipe-management.md` — **归档 — 模块已删除**（由商品/SKU 主数据取代）
- [x] **Item & SKU + Inventory** `phase1/backend-inventory.md` — Locations, inbound, outbound, stocktake, stock query（商品/SKU 维度）
- [x] **Auth & System** `phase1/backend-auth-system.md` — JWT auth, RBAC, user management, security config
- [x] **Traceability** `phase1/backend-tracing.md` — Log infrastructure + trace API + cross-cutting integration（商品/SKU 维度）

### Frontend Modules

- [x] **Pipe Management** `phase1/frontend-pipe-management.md` — **归档 — 模块已删除**（由商品主数据 UI 取代）
- [x] **Item & SKU + Inventory** `phase1/frontend-inventory.md` — Inbound/outbound/stocktake/location pages
- [x] **Auth & System** `phase1/frontend-auth-system.md` — Login, layout, user management, route guards
- [x] **Traceability** `phase1/frontend-tracing.md` — Trace tab on detail pages

---

## Phase 2 — P1 (Important Features)

> Goal: Core business loop — 采购、销售、质检、数据导入导出

### Backend Modules

- [x] **Quality Management** `phase2/backend-quality.md` — **归档 — 模块已删除**（管材质检证书 → 通用制造质检 Inspection）
- [x] **Purchase Management** `phase2/backend-purchase.md` — 采购订单、采购收货、供应商评分
- [x] **Sales Management** `phase2/backend-sales.md` — 销售订单、ATP 库存可用、客户信用
- [x] **Data Import / Export** `phase2/backend-data-io.md` — 商品/SKU 主数据 + 库存报表的批量导入导出

### Frontend Modules

- [x] **Quality Management** `phase2/frontend-quality.md` — **归档 — 模块已删除**
- [x] **Purchase Management** `phase2/frontend-purchase.md` — 采购订单 UI + 供应商管理
- [x] **Sales Management** `phase2/frontend-sales.md` — 销售订单 UI + 客户管理
- [x] **Data Import / Export** `phase2/frontend-data-io.md` — 导入/导出页面（商品/SKU 维度）

---

## Phase 3 — P2 (Enterprise Features)

> Goal: 合同、报表、i18n

### Backend Modules

- [x] **Contract Management** `phase3/backend-contracts.md` — 销售合同 / 采购合同
- [x] **Reports & Statistics** `phase3/backend-reports.md` — 库存、采购、销售的统计报表
- [x] **Label Printing** `phase3/backend-labels.md` — **归档 — 模块已删除**

### Frontend Modules

- [x] **Contract Management** `phase3/frontend-contracts.md` — 合同列表/表单/详情页
- [x] **Reports & Statistics** `phase3/frontend-reports.md` — 库存/进出/采购/销售报表页
- [x] **Label Printing** `phase3/frontend-labels.md` — **归档 — 模块已删除**
- [x] **Internationalization & Unit Switching** `phase3/frontend-i18n-units.md` — i18n + 公制/英制单位切换

---

## Completed Cross-Cutting Tasks

- [x] All backend source code doc comments (`///`) rewritten to English — zero Chinese characters
- [x] All frontend source code doc comments (`/** */`) rewritten to English — zero Chinese characters
- [x] All `.md` documentation files rewritten to English
- [x] **ERP 文档先行（2026-08-08）**：所有 Markdown 文档对齐新架构（SQLite3、商品/SKU、模块清单更新），管材/标签/质证书等旧模块文档已归档

---

## Output File Structure

```
docs/tasks/
├── progress.md                    ← You are here
├── phase1/
│   ├── backend-pipe-management.md      ← ARCHIVED
│   ├── backend-inventory.md            ← 商品/SKU + 库存
│   ├── backend-auth-system.md          ← 认证与系统管理
│   ├── backend-tracing.md              ← 可追溯
│   ├── frontend-pipe-management.md     ← ARCHIVED
│   ├── frontend-inventory.md           ← 商品/SKU + 库存 UI
│   ├── frontend-auth-system.md         ← 认证 UI
│   └── frontend-tracing.md             ← 可追溯 UI
├── phase2/
│   ├── backend-quality.md              ← ARCHIVED
│   ├── backend-purchase.md             ← 采购
│   ├── backend-sales.md                ← 销售
│   ├── backend-data-io.md              ← 数据导入导出（商品/SKU 维度）
│   ├── frontend-quality.md             ← ARCHIVED
│   ├── frontend-purchase.md            ← 采购 UI
│   ├── frontend-sales.md               ← 销售 UI
│   └── frontend-data-io.md             ← 数据导入导出 UI
└── phase3/
    ├── backend-contracts.md            ← 合同
    ├── backend-reports.md              ← 报表
    ├── backend-labels.md               ← ARCHIVED
    ├── frontend-contracts.md           ← 合同 UI
    ├── frontend-reports.md             ← 报表 UI
    ├── frontend-labels.md              ← ARCHIVED
    └── frontend-i18n-units.md          ← i18n + 单位切换
```
