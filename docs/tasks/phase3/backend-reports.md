# Phase 3 — Backend: Reports & Statistics Module (P2)

> Based on: `docs/requirements.en.md` §3.7, `docs/detailed-design.en.md` §10
> Architecture: backend crate `erp-server` (Rust + Axum + SQLx 0.8 **sqlite** feature)；DB = SQLite3. 报表维度统一为「商品 (Item) + SKU」+ 分类 + 规格 + 库位，无管材专属字段. 用 SQL `GROUP BY` + `SUM` / `COUNT` 聚合，**不要**把全量数据拉进应用层. 响应形状应直接对应 chart 渲染器 (labels + datasets). 术语见 `specs/UBIQUITOUS_LANGUAGE_LATEST.md`.

## Task List

### 1.1 Inventory Stats Reports

- [ ] `GET /api/v1/reports/stock-summary` — 库存总览
  - Returns: total stock, breakdown by category, breakdown by spec
- [ ] `GET /api/v1/reports/stock-by-category` — 库存按商品分类聚合
- [ ] `GET /api/v1/reports/stock-by-location` — 库存按库位聚合

### 1.2 Inbound / Outbound Stats Reports

- [ ] `GET /api/v1/reports/inbound-summary` — 入库统计
  - Params: date range, inbound type
  - Returns: qty by type, daily/monthly trend aggregation
- [ ] `GET /api/v1/reports/outbound-summary` — 出库统计
- [ ] `GET /api/v1/reports/monthly-flow` — 月度进出存统计 (bar chart data)

### 1.3 Purchase & Sales Reports

- [ ] `GET /api/v1/reports/purchase-summary` — 采购统计 (grouped by supplier, by status)
- [ ] `GET /api/v1/reports/sales-summary` — 销售统计 (grouped by customer, by status)
- [ ] `GET /api/v1/reports/financial-monthly` — 月度财务统计 (采购金额, 销售金额, 估算毛利)

### 1.4 Implementation Notes

- [ ] Use SQL aggregation (GROUP BY, SUM, COUNT) — don't pull everything into the app layer
- [ ] All report endpoints support `start_date` / `end_date` params
- [ ] Response shape should map directly to what the chart renderer expects (labels + datasets)
- [ ] Create SQLite indexes on frequently queried report columns (item_sku, category, location_id, date)

### 1.5 Testing

- [ ] Verify each report endpoint returns correct data (spot-check against raw data)
- [ ] Test report performance with large datasets (100k+ rows)
- [ ] Test date range filtering works correctly

> **Dependencies**: All business modules (reports run on their data) — 商品主数据、采购、销售、库存、合同
