# 006 — 销售 & CRM (Phase 2)

> **版本**: v1.0
> **日期**: 2026-08-02
> **依赖**: 004-finance-module, 002-workflow
> **状态**: Draft

---

## 1. 目标

深化销售管理，集成客户关系（CRM），控制信用和发货。

## 2. 功能范围

| 子模块 | 新的/更新 | 说明 |
| -------- | ----------- | ------ |
| Sales Orders | 已存在 | 增强确认/发货/日志/匹配 |
| 发货确认 | 新增 | 出库 + 文件签名 |
| 信用控制 | 新 | 客户信用等级 + 限额 + 分析 |
| 销售报价 | 新 | 价格 + 步骤, 发送到客户 |
| 客户历史 | 新 | 购买历史分析 |
| 销售预测 | 新 | 按期间/按商品 |

## 3. 数据模型

> 数据库为 SQLite3（`sqlite://data/erp.db?mode=rwc`），表名不带 schema 前缀。

```sql
-- customers 增加信用字段
ALTER TABLE customers ADD COLUMN credit_limit REAL DEFAULT 0;
ALTER TABLE customers ADD COLUMN credit_rating TEXT DEFAULT 'A';
ALTER TABLE customers ADD COLUMN payment_terms TEXT;  -- net30, immediate

-- 销售发货
CREATE TABLE shipments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    sales_order_id INTEGER REFERENCES sales_orders(id),
    shipment_date TEXT,
    tracking_number TEXT,
    carrier TEXT,
    status TEXT DEFAULT 'in_transit'
);

-- 销售报价
CREATE TABLE sales_quotes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    quote_no TEXT UNIQUE,
    customer_id INTEGER REFERENCES customers(id),
    valid_until TEXT,
    total_amount REAL,
    status TEXT DEFAULT 'draft',
    workflow_instance_id INTEGER
);
```

## 4. API

| Method | Path | Description |
| -------- | ------ | ------------- |
| GET | `/api/sales-orders/:id/print` | 打印销售订单 |
| POST | `/api/sales-orders/:id/ship` | 标记发货 |
| GET | `/api/customers/:id/credit` | 信用信息 |
| POST | `/api/sales-quotes` | 创建销售报价 |
| POST | `/api/sales-quotes/:id/convert` | 报价 → 订单 |

## 5. 前端

- `features/sales/pages/SalesOrderShipmentPage.tsx`
- `features/sales/pages/SalesQuotePage.tsx`
- `features/customers/pages/CustomerCreditPage.tsx`
- 扩展现有 CustomerFormPage 增加信用字段
