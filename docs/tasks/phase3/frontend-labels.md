# Phase 3 — Frontend: Label Printing Module (P2) — **ARCHIVED**

> **状态：模块已删除** — 通用商品/SKU 不再内置标签打印 UI。
> 历史沿革：本系统由钢管行业系统重构而来，本任务文档对应的标签打印前端页面在 ERP 重构中已下线。

## 删除说明

- 旧 `features/labels/` 前端目录（模板列表/模板表单/生成页面）已移除。
- 旧模板预览组件与管材专用批量选择组件已删除；通用商品批量选择由 `features/inventory` 提供。
- 路由表移除 `/labels/*`。
- 通用商品/SKU 的条码/二维码 UI 若业务侧需要，在 `inventory` 或 `manufacturing` 模块下按需扩展。
- 详见 `specs/UBIQUITOUS_LANGUAGE_LATEST.md` 术语表。
