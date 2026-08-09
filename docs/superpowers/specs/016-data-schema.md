# 016 — ERP: 完整数据 Schema 总览

> **版本**: v2.0（重构）
> **日期**: 2026-08-02
> **状态**: Draft
> **依赖**: 015-architecture-overview.md
> **用途**: 开发和维护所有模块实现时的 Schema 参考

---

## 目录

1. [SQLite 表设计约定](#1-sqlite-表设计约定)
2. [common (共享表)](#2-common-共享表)
3. [auth (认证 & 身份)](#3-auth-认证--身份)
4. [inventory (库存 & 商品)](#4-inventory-库存--商品)
5. [orders (采购 & 销售 & 合同)](#5-orders-采购--销售--合同)
6. [finance (财务会计)](#6-finance-财务会计)
7. [hr (人力资源)](#7-hr-人力资源)
8. [manufacturing (制造管理)](#8-manufacturing-制造管理)
9. [projects (项目管理)](#9-projects-项目管理)
10. [assets (固定资产)](#10-assets-固定资产)
11. [workflow (审批工作流)](#11-workflow-审批工作流)
12. [notification (通知)](#12-notification-通知)
13. [data_io (数据导入导出)](#13-data_io-数据导入导出)
14. [交叉索引 (Cross Table Index)](#14-交叉索引)
15. [未迁移事项](#15-未迁移事项)

---

## 1. SQLite 表设计约定

**Convention**: 数据库为 SQLite3 单文件（`sqlite://data/erp.db?mode=rwc`）。生产表使用**逻辑分组前缀**（如 `inventory_*`, `finance_*`, `hr_*`），无 schema 隔离（SQLite 单文件数据库不支持 schema）。表名与主键统一 `INTEGER PRIMARY KEY AUTOINCREMENT`，时间列 `TEXT DEFAULT (datetime('now'))`。

```
SQLite: data/erp.db
├── common       ← 共享基础表 (users, audit_log, ...)
├── auth         ← 认证 + 权限 (RBAC, tenants)
├── inventory    ← 商品 (items) + 库存
├── orders       ← 订单 (采购, 销售, 合同)
├── finance      ← 会计
├── hr           ← 人事
├── manufacturing ← 制造 + BOM + 质检
├── projects     ← 项目 + WBS
├── assets       ← 固定资产 + 折旧
├── workflow     ← 审批流程
├── notification ← 通知
└── data_io      ← 数据导入导出 log
```

## 2. common (共享表)

```sql
-- 全局审计日志 (所有模块共用)
CREATE TABLE audit_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER, tenant_id INTEGER,
    action TEXT, entity_type TEXT, entity_id INTEGER,
    old_value TEXT, new_value TEXT,   -- JSON 文本
    ip_address TEXT, request_id TEXT,
    created_at TEXT DEFAULT (datetime('now'))
);

CREATE INDEX idx_audit_entity ON audit_log(entity_type, entity_id);
CREATE INDEX idx_audit_user ON audit_log(user_id);
```

---

## 3. auth (认证 & 身份)

参见 `001-auth-identity.md` §3 Data Model 和更新文档。

主要表: `tenants`, `departments`, `roles`, `permissions`, `role_permissions`, `users`, `refresh_tokens`。

---

## 4. inventory (商品 & 库存)

```sql
-- 商品 master（通用商品化：无管材专属字段）
CREATE TABLE items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    sku TEXT NOT NULL UNIQUE,          -- 商品唯一编码
    name TEXT NOT NULL,                -- 商品名称
    category TEXT,                     -- 商品分类
    unit TEXT,                         -- 单位
    spec TEXT,                         -- 规格（可选描述性属性）
    status TEXT DEFAULT 'active',
    tenant_id INTEGER NOT NULL,
    created_at TEXT DEFAULT (datetime('now')),
    updated_at TEXT DEFAULT (datetime('now')),
    deleted_at TEXT
);
CREATE INDEX idx_items_sku ON items(sku);
CREATE INDEX idx_items_category ON items(category);

-- 位置
CREATE TABLE locations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    code TEXT, name TEXT, warehouse_id INTEGER,
    tenant_id INTEGER NOT NULL
);

-- 出入库记录
-- inbound_records, inbound_items, outbound_records, outbound_items, inbound_expected_arrivals，参见 007

-- 库存汇总（视图）
CREATE VIEW stock_summary AS
  SELECT item_id, SUM(qty_on_hand) AS qty
  FROM inventory_logs GROUP BY item_id;
```

---

## 4. 到 13 各逻辑组简要表逐一列出

为节省篇幅，我只列出各逻辑组的**主要表名**和关键索引策略。

| 逻辑组 | 主要表 | 索引策略 |
| -------- | -------- | -------- |
| common | audit_log | (entity_type, entity_id) + (created_at) |
| auth | tenants, departments, users, roles, permissions | users(email unique!) |
| inventory | items, locations, inbound_records, outbound_records, atp_slots | (deleted_at + sku), (inventory_logs + item_id) |
| orders | purchase_orders, sales_orders, line_items, contracts, purchase_requisitions | (deleted_at, order_no), (deleted_at, status) |
| finance | chart_of_accounts, journal_entries, invoices, payments | (entity_type, entity_id), (due_date, status) |
| hr | employees, attendances, salaries, contracts | (company_id, deleted_at) |
| manufacturing | boms, work_orders, routing_ops, quality_inspections, ncr_outputs | (work_order_id, status) |
| projects | projects, wbs_elements | (project_id, deleted_at) |
| assets | fixed_assets, depreciation_entries | (useful_life, deleted_at) |
| workflow | definitions, instances, approval_nodes, delegations | (instance_id) |
| notification | templates, notifications, preferences | (user_id, read_at) |
| data_io | import_records, export_tasks | (entity_type, created_at) |

---

## 5. 关键索引规则

1. 所有用户对存在 `deleted_at IS NULL` 进行过滤的表都需要: `(deleted_at, <query_column>)` 的复合索引（如 `idx_item_deleted_sku` 用于查询 `SELECT * FROM items WHERE deleted_at IS NULL AND sku = ?`）

2. 查询日期范围建议: `(deleted_at, status, created_at)` 索引，使软删除 + 状态 + 日期 三种过滤能同时命中。

3. **不建 FK** 但通过 repository 层 validates

---

## 6. 迁移工作 (SQLite 重写)

维护策略: 使用 sqlx 内建 migration（`sqlx::migrate!`）。37 个遗留迁移文件重写为 SQLite 语法，命名前缀不变（`001_..`, `022_..` 格式），并删除钢管行业专属表（管材、标签、质检证书、参考数据等；完整清单见 `specs/UBIQUITOUS_LANGUAGE_LATEST.md`），新增 `items` 商品表。

---

> **开发负责人**: 当实现任何 schema 时，必须先查询本文件更新新表名与索引。
