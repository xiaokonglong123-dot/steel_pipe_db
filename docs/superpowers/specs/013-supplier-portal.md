# 013 — 供应商 & 客户门户 (Phase 4)

> **版本**: v1.0 | **日期**: 2026-08-02 | **依赖**: 005-procurement, 006-sales-crm | **状态**: Draft

---

## 1. 目标

外部供应商和客户能通过 Web 门户登录查看订单、确认 PO、提交发运、提交质量手册。

## 2. 功能

| 角色 | 看到的 |
|------|--------|
| Supplier | 查看 PO、确认或拒绝 PO、提交发货跟踪、提交发货发票 -->
| Customer | 查看 SO、历史订单、信用状态、收到发货告知 |

## 3. 相关样式

```sql
-- 新表: portal_tenants 链接到 auth.tenants 的 external
CREATE TABLE auth.portal_tenants (
    id BIGSERIAL PRIMARY KEY,
    tenant_id BIGINT REFERENCES auth.tenants(id),
    entity_type VARCHAR(20), -- supplier, customer (choice)
    entity_id BIGINT
);
```

## 4. API

| Method | Path | Auth |
|--------|------|------|
| GET | `/portal/purchases` | supplier JWT |
| POST | `/portal/purchases/:id/accept` | supplier |
| GET | `/portal/sales` | customer JWT |
| POST | `/portal/sales/:id/acknowledge` | customer |

## 5. 前端

- SPA 用 `gateway/PortalLoginPage` 将供应商/客户排序进 tenant ID 渲染不同视图