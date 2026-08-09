# Phase 1 — Frontend: Pipe Management Module (P0 MVP) — **ARCHIVED**

> **状态：模块已删除** — 已被通用商品 / SKU（items）功能取代。
> 历史沿革：本系统由钢管行业系统重构而来，本任务文档对应的管材主数据页面在 ERP 重构中已下线。

## 删除说明

- 旧 `features/pipes/` 前端目录（列表/表单/详情/统一搜索页面）已移除；统一由商品主数据 `features/items` 页面提供。
- 旧管材筛选/表格/状态徽章等专用组件已移除；统一使用商品主数据通用组件。
- 旧编码生成/解析工具已删除；改用通用 SKU 生成器。
- 路由表移除 `/pipes/*`；商品主数据走 `/items` 路由。
- 详见 `docs/tasks/phase1/frontend-inventory.md` 与 `specs/UBIQUITOUS_LANGUAGE_LATEST.md` 术语表。
