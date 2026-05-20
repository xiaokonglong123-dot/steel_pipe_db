# Phase 3 — 后端：报表与统计模块 (P2)

> 基于：`docs/需求文档.md` §3.7；`docs/详细设计文档.md` §10

## 任务清单

### 1.1 库存统计报表
- [ ] `GET /api/v1/reports/stock-summary` — 库存总览
  - 返回：总库存数、各管材类型数量、各钢级数量分布
- [ ] `GET /api/v1/reports/stock-by-grade` — 按钢级分组库存统计
- [ ] `GET /api/v1/reports/stock-by-location` — 按库位库存统计

### 1.2 出入库统计报表
- [ ] `GET /api/v1/reports/inbound-summary` — 入库统计
  - 参数：日期范围、入库类型
  - 返回：各类型入库数量、按日/月聚合趋势
- [ ] `GET /api/v1/reports/outbound-summary` — 出库统计
- [ ] `GET /api/v1/reports/monthly-flow` — 月度出入库流水统计（月度柱状图数据）

### 1.3 采购与销售报表
- [ ] `GET /api/v1/reports/purchase-summary` — 采购统计（按供应商、按状态聚合）
- [ ] `GET /api/v1/reports/sales-summary` — 销售统计（按客户、按状态聚合）
- [ ] `GET /api/v1/reports/financial-monthly` — 月度财务统计（采购金额、销售金额、毛利估算）

### 1.4 实现要点
- [ ] 使用 SQL 聚合查询（GROUP BY, SUM, COUNT），避免全表加载到应用层
- [ ] 所有报表接口支持 `start_date` / `end_date` 参数
- [ ] 返回数据结构直接对应前端图表渲染器所需格式（labels + datasets）
- [ ] 为频繁查询的报表创建必要的 SQLite 索引

### 1.5 测试
- [ ] 测试每个报表接口返回数据正确性（与原始数据逐条核对）
- [ ] 测试大数据量下报表性能（10 万+条数据）
- [ ] 测试日期范围的正确过滤

> **依赖**: 全部业务模块（依赖基础数据产生统计）
