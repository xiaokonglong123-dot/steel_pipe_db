# 013 — 供应商 & 客户门户 (Phase 4)

> **版本**: v1.0 | **日期**: 2026-08-02 | **依赖**: 005-procurement, 006-sales-crm | **状态**: Draft

---

## 1. 目标

外部供应商和客户能通过 Web 门户登录查看订单、确认采购订单、提交发运。

## 2. 功能

| 角色 | 看到的 |
|------|--------|
| Supplier | 查看采购订单、确认或拒绝、提交发货跟踪、提交发货发票 |
| Customer | 查看销售订单、历史订单、信用状态、收到发货告知 |

## 3. 相关表

> 数据库为 SQLite3（`sqlite://data/erp.db?mode=rwc`）。

```sql
-- 新表: portal_accounts 链接到 parties (客户/供应商)
CREATE TABLE portal_accounts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    tenant_id INTEGER REFERENCES tenants(id),
    entity_type TEXT, -- supplier, customer
    entity_id INTEGER,
    username TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    created_at TEXT DEFAULT (datetime('now'))
);
```

## 4. API

| Method | Path | Auth |
| -------- | ------ | ------ |
| GET | `/portal/purchases` | supplier JWT |
| POST | `/portal/purchases/:id/accept` | supplier |
| GET | `/portal/sales` | customer JWT |
| POST | `/portal/sales/:id/acknowledge` | customer |

## 5. 前端

- SPA 用 `portal/PortalLoginPage` 将供应商/客户排序进租户 ID 渲染不同视图
