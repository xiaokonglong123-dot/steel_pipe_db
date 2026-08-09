# Rust Axum + Vue3 Optimization Design (已归档)

> **状态**: **Archived — 计划已废弃**

## 归档说明

本文档描述的是重构前 `rust-axum-vue3` 子项目的优化与扩展设计，**已整体作废**。

历史沿革：本系统由钢管行业系统重构而来，重构后：

- **前端栈变更**：Vue 3 + Pinia + vue-router 已被 **React 19 + Vite + Ant Design 5 + TanStack Query 5 + Zustand 5** 取代（见 `017-frontend-guide.md`）。
- **后端栈变更**：rusqlite + 手写 db.rs 已被 **Rust Axum 0.8 + sqlx (SQLite3) 单 crate `erp-server`** 取代（见 `015-architecture-overview.md`）。
- **业务对象变更**：管材专属对象统一为**商品 (Item) + SKU**；原专属模块全部删除（清单见 `specs/UBIQUITOUS_LANGUAGE_LATEST.md` 已废弃术语表）。
- **数据库**：SQLite3 单文件（`sqlite://data/erp.db?mode=rwc`）。

如需追溯原设计，请查看 Git 历史中本文件删除前的内容。
