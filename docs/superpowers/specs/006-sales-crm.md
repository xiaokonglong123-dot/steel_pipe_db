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
|--------|-----------|------|
| Sales Orders | 已存在 | 增强确认/发货/日志/匹配 |
| 发货确认 | 新增 | 出库 + 文件签名 |
| 信用控制 | 新 | 客户信用等级 + 限额 + 贷款分析 |
| 销售报价 | 新 | 价格 + 步骤, 发送到客户 |
| 客户历史 | 新 | 购买历史 history 分析 |
| 销售预测 | 新 | 按期间/按产品 |

## 3. 数据模型

```sql
-- orders.customers
ALTER TABLE orders.customers ADD COLUMN credit_limit NUMERIC(18,2) DEFAULT 0;
ALTER TABLE orders.customers ADD COLUMN credit_rating VARCHAR(5) DEFAULT 'A';
ALTER TABLE orders.customers ADD COLUMN payment_terms VARCHAR(30);  --> net30, immediate

-- 销售发货
CREATE TABLE orders.shipments (
    id BIGSERIAL PRIMARY KEY,
    sales_order_id BIGINT REFERENCES orders.sales_orders(id),
    shipment_date DATE,
    tracking_number VARCHAR(200),
    carrier VARCHAR(100),
    status VARCHAR(20) DEFAULT 'in_transit'
);

-- 销售报价
CREATE TABLE orders.sales_quotes (
    id BIGSERIAL PRIMARY KEY,
    quote_no VARCHAR(100) UNIQUE,
    customer_id BIGINT REFERENCES customers(id),
    valid_until DATE,
    total_amount NUMERIC(18,2),
    status VARCHAR(20) DEFAULT 'draft',
    workflow_instance_id BIGINT
);
```

## 4. API

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/sales-orders/:id/print` | 打印销售订单 |
| POST | `/api/sales-orders/:id/ship` | 标记配送 |
| GET | `/api/customers/:id/credit` | 信用信息 |
| POST | `/api/sales-quotes` | 创建报价 |
| POST | `/api/sales-quotes/:id/convert` | 报价 → 订单 |

## 5. 前端

- `features/orders/pages/SalesOrderShipmentPage.tsx`
- `features/orders/pages/SalesQuotePage.tsx`
- `features/customers/pages/CustomerCreditPage.tsx`
- 扩展现有 CustomerFormPage 增加信用字段