# Pipe Threading Implementation Plan (已归档)

> **状态**: **Archived — 计划已废弃，模块已删除**

## 归档说明

**模块已删除 — 由通用商品/SKU 与制造质检取代。**

历史沿革：本系统由钢管行业系统重构而来，原「螺纹加工（Threading）」实施计划已废弃。

- 几何参数计算、扭矩计算、管柱设计、连接强度校核等任务**不再实施**。
- 制造侧的质量能力统一由 `008-manufacturing.md` 的**质检 (Inspection) / 不合格品单 (NCR)** 承接。
- 对象模型统一为**商品 (Item) + SKU**，见 `016-data-schema.md`。

如需追溯原计划，请查看 Git 历史中本文件删除前的内容。
