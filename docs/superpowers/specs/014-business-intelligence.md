# 014 — 商业智能 & 报表 (Phase 4)

> **版本**: v1.0 | **日期**: 2026-08-02 | **依赖**: 004-finance, 005-procurement, 006-sales | **状态**: Draft

---

## 1. 目标

全系统报告套件 + 实时仪表板。

## 2. 功能

| 类型 | 报告名 |
| ------ | -------- |
| Finance | 损益表 (P&L), Balance Sheet, Trial Balance, Aging (账龄分析) |
| Operations | Inventory turns, stock report, defective summary, Order fill rate |
| Sales | 销售趋势，by customer / product, top 10 products |
| HR | 员工统计、离职率 |

## 3. API

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/reports/{report_type}` | 返回 JSON + (可选择导出 Excel) |

## 4. 前端

- `features/reports/pages/` 主要页面 + ECharts 生成图表
- 报表前端提供 Time-filter, Export, Print
