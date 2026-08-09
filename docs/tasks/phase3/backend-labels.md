# Phase 3 — Backend: Label Printing Module (P2) — **ARCHIVED**

> **状态：模块已删除** — 通用商品/SKU 不再内置标签打印；按需由业务侧自行扩展。
> 历史沿革：本系统由钢管行业系统重构而来，本任务文档对应的标签打印模块在 ERP 重构中已下线。

## 删除说明

- 旧标签模板表已删除；通用 ERP 不内置模板管理。
- 旧标签生成与打印服务（`POST /api/v1/labels/generate`、`GET /api/v1/labels/print-history`）已下线。
- 旧 `printpdf` / `barcoder` 依赖从 `Cargo.toml` 移除；条码/二维码生成需求不再绑定商品主数据。
- 通用商品/SKU 的条码/二维码若业务侧需要，应在 `inventory_atp` 或 `manufacturing` 扩展点中按需实现，不作为平台级强制能力。
- 详见 `specs/UBIQUITOUS_LANGUAGE_LATEST.md` 术语表。
