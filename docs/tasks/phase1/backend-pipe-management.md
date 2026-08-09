# Phase 1 — Backend: Pipe Management Module (P0 MVP) — **ARCHIVED**

> **状态：模块已删除** — 已被通用商品 / SKU（inventory）取代。
> 历史沿革：本系统由钢管行业系统重构而来，本任务文档对应的管材主数据模块在 ERP 重构中已下线。

## 删除说明

- 旧管材主数据表与管材专属字段（类型/端部/筛缝等）已删除；主数据统一收敛为通用 **商品 (Item)** + **SKU**。
- 旧 REST 端点已下线；统一由商品主数据端点 `GET/POST/PUT/DELETE /api/v1/items` 提供。
- 唯一业务编码改为 **SKU**：不可为空、不可重复、跨商品类型唯一。
- 详见 `docs/tasks/phase1/backend-inventory.md`（商品/SKU 主数据 + 库存）与 `specs/UBIQUITOUS_LANGUAGE_LATEST.md` 术语表。
