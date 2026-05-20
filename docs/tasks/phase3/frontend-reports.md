# Phase 3 — 前端：报表与统计模块 (P2)

> 基于：`docs/前端设计文档.md` §4.1

## 任务清单

### 1.1 共享类型与 API
- [ ] 定义 `features/reports/types.ts`：StockSummary, TrendDataPoint, ChartConfig 等类型
- [ ] 定义 `features/reports/api/reportApi.ts`：所有报表接口

### 1.2 报表 Landing 页面
- [ ] 实现 `ReportDashboardPage`：
  - 四张 KPI 卡片：库存总量、本月入库、本月出库、本月销售金额
  - 快捷入口：点击卡片跳转对应报表详情

### 1.3 库存报表
- [ ] 实现 `StockReportPage`：
  - 库存总览：KPI + 各管材类型饼图（Ant Design Pie）
  - 钢级分布：柱状图（按钢级展示库存量）
  - 库位分布：树形 + 对应数量
  - 日期筛选（查看历史快照）

### 1.4 出入库报表
- [ ] 实现 `InboundReportPage`：
  - 入库趋势折线图（按日/按月聚合切换）
  - 入库类型占比饼图
  - 入库明细表格
- [ ] 实现 `OutboundReportPage`（同上，出库维度）
- [ ] 实现 `MonthlyFlowPage`：
  - 月度柱状图（入库蓝色、出库橙色，并列对比）
  - 表格展示各月统计明细

### 1.5 采购/销售报表
- [ ] 实现 `PurchaseReportPage`：
  - 按供应商聚合的采购金额柱状图
  - 月度采购趋势
- [ ] 实现 `SalesReportPage`：
  - 按客户聚合的销售金额柱状图
  - 月度销售趋势

### 1.6 共享组件
- [ ] 实现 `ChartCard`：图表卡片容器（标题、筛选、导出图表按钮）
- [ ] 实现 `KPICard`：统计数字卡片（标题、数值、单位、趋势箭头）
- [ ] 实现 `DateRangeFilter`：日期范围选择器（预设：本周/月/季/年/自定义）
- [ ] 实现 `ChartExport`：图表导出为图片按钮

### 1.7 依赖与第三方
- [ ] 安装 `@ant-design/charts`（基于 G2Plot）
- [ ] 配置图表中文 locale

> **依赖**: 报表模块后端 API
