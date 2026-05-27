# Phase 3 — Backend: Reports & Statistics Module (P2)

> Based on: `docs/requirements.en.md` §3.7, `docs/detailed-design.en.md` §10

## Task List

### 1.1 Inventory Stats Reports
- [ ] `GET /api/v1/reports/stock-summary` — Inventory overview
  - Returns: total stock, breakdown by pipe type, breakdown by grade
- [ ] `GET /api/v1/reports/stock-by-grade` — Stock grouped by steel grade
- [ ] `GET /api/v1/reports/stock-by-location` — Stock grouped by location

### 1.2 Inbound / Outbound Stats Reports
- [ ] `GET /api/v1/reports/inbound-summary` — Inbound stats
  - Params: date range, inbound type
  - Returns: qty by type, daily/monthly trend aggregation
- [ ] `GET /api/v1/reports/outbound-summary` — Outbound stats
- [ ] `GET /api/v1/reports/monthly-flow` — Monthly in/out flow stats (bar chart data)

### 1.3 Purchase & Sales Reports
- [ ] `GET /api/v1/reports/purchase-summary` — Purchase stats (grouped by supplier, by status)
- [ ] `GET /api/v1/reports/sales-summary` — Sales stats (grouped by customer, by status)
- [ ] `GET /api/v1/reports/financial-monthly` — Monthly financial stats (purchase amount, sales amount, estimated gross margin)

### 1.4 Implementation Notes
- [ ] Use SQL aggregation (GROUP BY, SUM, COUNT) — don't pull everything into the app layer
- [ ] All report endpoints support `start_date` / `end_date` params
- [ ] Response shape should map directly to what the chart renderer expects (labels + datasets)
- [ ] Create SQLite indexes on frequently queried report columns

### 1.5 Testing
- [ ] Verify each report endpoint returns correct data (spot-check against raw data)
- [ ] Test report performance with large datasets (100k+ rows)
- [ ] Test date range filtering works correctly

> **Dependencies**: All business modules (reports run on their data)
