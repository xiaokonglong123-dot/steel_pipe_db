# Phase 2 — Backend: Quality Management Module (P1) — **ARCHIVED**

> **状态：模块已删除** — 已被通用制造质检 (Inspection) 取代。
> 历史沿革：本系统由钢管行业系统重构而来，本任务文档对应的管材质检证书模块在 ERP 重构中已下线。

## 删除说明

- 旧质证书相关表与行业标准参考数据表已删除；行业专属参考数据不再保留。
- 旧质证书实体与附件上传逻辑已移除；附件上传改为工单 (Work Order) 下的通用附件。
- 质检维度改为通用 **SKU** + **规格 (Spec)**。
- 旧 REST 端点已下线；制造质检统一走 `manufacturing/inspections` 端点。
- 质检功能保留为「制造质检 (Inspection)」（工单下的质量检验记录），由 `docs/superpowers/specs/008-manufacturing.md` 定义。
- 详见 `specs/UBIQUITOUS_LANGUAGE_LATEST.md` 术语表。
