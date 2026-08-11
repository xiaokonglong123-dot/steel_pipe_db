# Phase 3 — Frontend: Reports & Statistics Module (P2)

> Based on: `docs/frontend-design.en.md` §4.1

## Task List

### 1.1 Shared Types & API
- [ ] Define `features/reports/types.ts`: StockSummary, TrendDataPoint, ChartConfig, etc.
- [ ] Define `features/reports/api/reportApi.ts`: all report endpoints

### 1.2 Report Landing Page
- [ ] Implement `ReportDashboardPage`:
  - Four KPI cards: total stock, inbound this month, outbound this month, sales amount this month
  - Quick links: clicking a card jumps to the corresponding report

### 1.3 Inventory Reports
- [ ] Implement `StockReportPage`:
  - Overview: KPIs + pie chart by pipe type (Ant Design Pie)
  - Grade distribution: bar chart (stock by grade)
  - Location distribution: tree + quantities
  - Date filter (for historical snapshots)

### 1.4 Inbound / Outbound Reports
- [ ] Implement `InboundReportPage`:
  - Inbound trend line chart (daily/monthly aggregation toggle)
  - Inbound type breakdown pie chart
  - Inbound detail table
- [ ] Implement `OutboundReportPage` (same pattern, outbound side)
- [ ] Implement `MonthlyFlowPage`:
  - Monthly bar chart (inbound blue, outbound orange, side-by-side)
  - Table with monthly breakdown

### 1.5 Purchase / Sales Reports
- [ ] Implement `PurchaseReportPage`:
  - Purchase amount bar chart grouped by supplier
  - Monthly purchase trend
- [ ] Implement `SalesReportPage`:
  - Sales amount bar chart grouped by customer
  - Monthly sales trend

### 1.6 Shared Components
- [ ] Implement `ChartCard`: chart container (title, filters, export button)
- [ ] Implement `KPICard`: stat card (title, value, unit, trend arrow)
- [ ] Implement `DateRangeFilter`: date range picker with presets (this week/month/quarter/year, custom)
- [ ] Implement `ChartExport`: export chart to image button

### 1.7 Dependencies & Third-Party
- [ ] Install `@ant-design/charts` (G2Plot-based)
- [ ] Configure chart locale for Chinese

> **Dependencies**: Reports backend API
