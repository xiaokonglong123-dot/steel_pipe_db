# Phase 2 — Frontend: Quality Management Module (P1) — **ARCHIVED**

> **状态：模块已删除** — 已被通用制造质检 (Inspection) 取代。
> 历史沿革：本系统由钢管行业系统重构而来，本任务文档对应的管材质检证书前端页面在 ERP 重构中已下线。

## 删除说明

- 旧 `features/quality/` 前端目录（质证书列表/表单/详情/追溯/参考数据页面）已移除。
- 旧证书文件上传组件与管材专用追溯组件已删除；通用版随制造 (manufacturing) 模块提供。
- 行业标准参考页已删除；行业参考数据不再保留。
- 路由表移除 `/quality/*`；制造质检统一走 `/manufacturing/inspections` 路由。
- 详见 `docs/superpowers/specs/008-manufacturing.md` 与 `specs/UBIQUITOUS_LANGUAGE_LATEST.md` 术语表。
