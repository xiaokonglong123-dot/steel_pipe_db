# Changelog

See `git log` for history. The `legacy/steel-pipe-react` branch preserves the pre-erp-v2 era.

## [Unreleased] — v3 rewrite era

### Frontend productization (M2–M4)
- `types/` strict typing, per-domain `api/` modules, `utils/money` (Decimal-as-string).
- Dedicated purchase/sales pages with `OrderLineEditor` line-detail editor + `ActionBar` buttons driven by backend `allowed_actions`.
- Inventory moved to shared StockMove pages (inbound/outbound form+list+detail); `EntitySelect` remote dropdowns show name but submit id.
- Master data rewritten on `MasterDataCrud`.
- Backend list/detail endpoints now project foreign-key names (supplier_name/item_name/location_name/…).
- **Build** Element Plus switched to on-demand component resolve + manual chunking (vendor/echarts) — entry chunk dropped from ~1.09 MB to ~10 kB.
- **Fix** frontend API base path: `client.ts` `baseURL = "/api"` + Vite dev proxy strips `/api` before forwarding to `:3000`. (Previously `/api/v1` never reached backend's root-path routes.)
- Backend tests now 140 green.

## [Unreleased] — erp-v2 era

### Decimal quantities + docs cleanup
- quantity columns migrated REAL → Decimal TEXT (migration 013) across inventory/logs/items/check/reservations/PO/SO.
- Enforce non-default JWT_SECRET at startup (`Config::validate_security`).
- Emit + propagate `x-request-id` and trace it in spans.
- Removed archived `docs/legacy/` (pre-rewrite React-stack docs); old stack remains recoverable via `legacy/steel-pipe-react` branch.
- Backend tests now 124 green.

### Promotion (2026-08-11)
- Repository migrated to erp-v2 stack: Rust+Axum+SQLx+SQLite/Decimal backend, Vue3+ElementPlus+bun frontend.
- Old React+Antd+npm stack moved to `legacy/steel-pipe-react` branch (51 pre-promotion commits preserved).
- `main` reset to `origin/main` before promotion; the erp-v2 promotion is the first commit of the new era.
- Layout: `erp-v2/{backend,frontend,docs}` promoted to top-level `{backend,frontend,docs}`.
- Old tracked docs preserved at `docs/legacy/` for reference.
- `specs/` removed (steel-pipe-era terminology, no replacement in erp-v2 yet).
- 121 backend tests green; frontend `bunx tsc --noEmit` + `bun run build` green.

### P0 — 核心交易闭环
- Auth+RBAC, Catalog, Parties, Inventory, 采购/销售订单, 审批流（data-driven ERPNext-style）, 收货 + 发货端到端联动

### P1 — 财务 + 报表 + ATP
- Finance (会计科目/日记账/发票/付款/试算平衡), Inventory Check 盘点, ATP 可用量, Reports + ECharts 可视化

### P2 — 增强
- CSV 商品导入, 审批流多级/条件（amount_threshold）
