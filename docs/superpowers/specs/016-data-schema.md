# 016 — Steel Pipe ERP: 完整数据 Schema 总览

> **版本**: v1.0
> **日期**: 2026-08-02
> **状态**: Draft
> **依赖**: 015-architecture-overview.md
> **用途**: 开发和维护所有模块实现时的 Schema 参考

---

## 目录

1. [PostgreSQL Schema 隔离设计](#1-postgresql-schema-隔离设计)
2. [Schema: common (共享表)](#2-schema-common)
3. [Schema: auth (认证 & 身份)](#3-schema-auth)
4. [Schema: inventory (库存 & 管材)](#4-schema-inventory)
5. [Schema: orders (采购 & 销售 & 合同)](#5-schema-orders)
6. [Schema: finance (财务会计)](#6-schema-finance)
7. [Schema: hr (人力资源)](#7-schema-hr)
8. [Schema: manufacturing (制造管理)](#8-schema-manufacturing)
9. [Schema: projects (项目管理)](#9-schema-projects)
10. [Schema: assets (固定财产)](#10-schema-assets)
11. [Schema: workflow (审批工作流)](#11-schema-workflow)
12. [Schema: notification (通知)](#12-schema-notification)
13. [Schema: data_io (数据导入导出)](#13-schema-data_io)
14. [交叉索引 (Cross Schema Index)](#14-交叉索引)
15. [未迁移事项](#15-未迁移事项)

---

## 1. PostgreSQL Schema 隔离设计

**Convention**: 所有生产表必须存在于一个指定的 PostgreSQL schema 下（如 `inventory.seamless_pipes`），而非 `public` schema。`public` 用作临时和共享函数。

```
PostgreSQL: steel_pipe_erp
├── common       ← 共享基础表 (users(原), cities, ...)
├── auth         ← 认证 + 权限 (RBAC, tenants)
├── inventory    ← 管材 + 库存
├── orders       ← 订单 (采购, 销售, 合同)
├── finance      ← 会计
├── hr           ← 人事
├── manufacturing ← 制造 + BOM + 质检
├── projects     ← 项目 + WBS
├── assets       ← 固定资产 + 折旧
├── workflow     ← 审批流程
├── notification ← 通知
└── data_io      ← 数据导入导出log
```

## 2. Schema: common

```sql
-- 全局审计日志 (所有模块共用)
CREATE TABLE common.audit_log (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT, tenant_id BIGINT,
    action VARCHAR(50), entity_type VARCHAR(100), entity_id BIGINT,
    old_value JSONB, new_value JSONB,
    ip_address INET, request_id VARCHAR(36),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_audit_entity ON common.audit_log(entity_type, entity_id);
CREATE INDEX idx_audit_user ON common.audit_log(user_id);
```

---

## 3. Schema: auth

参见 `001-auth-identity.md` §3 Data Model 和更新文档。

主要表: `auth.tenants`, `auth.companies`, `auth.departments`, `auth.roles`, `auth.permissions`, `auth.role_permissions`, `auth.users`, `auth.refresh_tokens`。

## 4. Schema: inventory (管材 & 库存)

```sql
-- 管材：seamless_pipes, screen_pipes, welded_pipes 维持现存造
CREATE TABLE inventory.seamless_pipes (
    id BIGSERIAL PRIMARY KEY,
    pipe_number TEXT NOT NULL UNIQUE,
    -- (其余字段与原 SQLite seamless_pipes 梳理一致) +
    tenant_id BIGINT NOT NULL REFERENCES auth.tenants(id),
    created_at TIMESTAMPTZ, updated_at TIMESTAMPTZ, deleted_at TIMESTAMPTZ
);

-- Screen pipes, welded pipes 同理

-- 位置
CREATE TABLE inventory.locations (
    id BIGSERIAL, code TEXT, name TEXT, warehouse_id INT,
    tenant_id BIGINT REFERENCES auth.tenants(id) NOT NULL
);

-- 出入库记录 已增加字段
-- inbound_records, inbound_items, outbound_records, outbound_items, inbound_expected_arrivals，参见 007

-- stock_summary (物化视图)
CREATE MATERIALIZED VIEW inventory.stock_summary AS
  SELECT pipe_number, SUM(qty_on_hand) AS qty, ... GROUP BY pipe_number;
```

---

## 4. 到 13 schema 简要表逐一列出

为节省篇幅，我只列出各 schema 的主要表名和关键索引策略。

| Schema | 主要表 | 索引策略 |
|--------|--------|--------|
| common | audit_log, event_store | (entity_valid, entity_id) + (created_at) |
| auth | tenants, companies, departments, positions, users, roles, permissions | users(email unique！) |
| inventory | seamless_pipes, inbound_records, outbound_records, atp_slots, + 更多 | (deleted_at + pipe_number), (库存日志+ pipe_number) |
| orders | purchase_orders, sales_orders,  line_items, contracts, purchase_requisitions | (deleted_at, order_no), (deleted_at, status) and 用于过滤行为 |
| finance | chart_of_accounts, journal_entries, invoices, payments | (entity_type, entity_id), (due_date, status) |
| hr | employees, attendances, salaries, contracts | (company_id, deleted_at) |
| manufacturing | boms, work_orders, routing_ops, threading_records, quality_inspections | (work_order_id, status) |
| projects | projects, wbs | (project_id, deleted_at) |
| assets | fixed_assets, depreciation_entries | (total/useful_life, deleted_at) |
| workflow | definitions, instances, approval_nodes, delegations | (instance_id) |
| notification | templates, notifications, preferences | (user_id, read_at) |
| data_io | import_records, export_tasks | (entity_type, created_at) |

---

## 5. 关键索引规则

1. 所有用户对存在 `deleted_at IS NULL` 进行过滤的表都需要: `(deleted_at, <query_column>)` 的复合索引（如 `idx_pipe_deleted_number` 用于查询 `SELECT * FROM inventory.seamless_pipes WHERE deleted_at IS NULL AND pipe_number = ?`）

2. 查询日期范围建议: `(deleted_at, status, created_at)` 索引，使软删除 + 状态 + 日期 三种过滤能同时命中。

3. **不建 FK** 但通过 repository 层 validates

## 6. 迁移工作 (Postgres Migration)

维护策略: 使用 [tern](https://github.com/jackc/tern) 生成 SQL。迁移文件前缀不变: `022_.. 023_..` 格式。

---

> **开发负责人**: 当实现任何 schema 时，必须先查询本文件更新新表名与索引。