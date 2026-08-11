# Changelog

See `git log` for history. The `legacy/steel-pipe-react` branch preserves the pre-erp-v2 era.

## [Unreleased] — erp-v2 era

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
