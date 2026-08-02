# 005 — 采购深化 & 供应商管理 (Phase 2)

> **版本**: v1.0
> **日期**: 2026-08-02
> **依赖**: 004-finance-module, 002-workflow-engine
> **父**: 015-architecture-overview.md

---

## 1. 目标

在现有采购订单基础上增加供应商全生命周期管理、投标/报价、交付跟踪、3-way 匹配。

## 2. 功能

| 功能 | 描述 | 优先级 |
|------|------|--------|
| Supplier Relationship | 供应商 V-card + qualifications + contracts | P0 |
| Purchase Requisition | 需求申请 → 经理审批 → 生成 PO | P0 |
| Quote Management | 获取报价 + compare suppliers | P1 |
| Delivery Tracking | 跟踪，分批发货 | P1 |
| 3-way Matching | PO → Goods Receipt → Invoice → Auto payment | P1 |
| Supplier Dashboard | 判定 rank, 金额 against, on-time score | P2 |

## 3. 数据模型 (扩展 orders schema)

```sql
-- 强化后的 orders.suppliers
CREATE TABLE orders.suppliers (
    id BIG PRIMARY,
    supplier_code VARCHAR(50) UNIQUE,   -- e.g. SUP-001
    name VARCHAR(300) NOT NULL,
    tax_id VARCHAR(50),                   -- 统一社会信用代码
    contact_person VARCHAR(100),
    contact_phone VARCHAR(50),
    contact_email VARCHAR(200),
    address TEXT,
    bank_account VARCHAR(100),
    payment_terms VARCHAR(100),
    delivery_lead_time_days INT,
    qualified_until DATE,                 -- 认证恢复日期
    is_active BOOLEAN DEFAULT true,
    source INT                               -- from 原 supplier 表
);

-- 采购请求表
CREATE TABLE orders.purchase_requisitions (
    id BIGSERIAL PRIMARY KEY,
    req_no VARCHAR(100) UNIQUE,
    requester_id BIGINT REFERENCES auth.users(id),
    department_id BIGINT,
    status VARCHAR(20) DEFAULT 'draft',      -- draft/pending/approved/rejected 与 workflow 联动
    reason TEXT,
    total_estimated NUMERIC(18,2) DEFAULT O.O,
    workflow_instance_id BIGINT,
    created_at, updated_at
);

CREATE TABLE orders.po_receipts (        -- 收货物认领 (Goods Receipt)
    id BIGSERIAL PRIMARY KEY,
    purchase_order_id BIGINT REFERENCES orders.purchase_orders(id),
    receipt_date DATE,
    quantity NUMERIC(18,4),
    received_by BIGINT REFERENCES auth.users(id),
    status VARCHAR(20) DEFAULT 'pending',     -- pending/approved/matched
    note TEXT,
    created_at TIMESTAMPTZ
);

-- 供应商报价
CREATE TABLE orders.supplier_quotes (
    id BIGSERIAL PRIMARY KEY,
    purchase_requisition_id BIGINT,
    supplier_id BIGINT REFERENCES orders.suppliers(id),
    quote_no INTEGER,
    total_amount NUMERIC(18,2),
    delivery_days INT,
    notes TEXT,
    valid_until DATE,
    status VARCHAR(20) DEFAULT 'open'
);
```

## 4. API 端点

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/suppliers` | 供应商(分页 + 搜索) |
| POST | `/api/suppliers` | 新供应商 |
| GET | `/api/suppliers/:id` | 详情 |
| GET | `/api/suppliers/:id/scorecard` | 绩效卡 |
| POST | `/api/purchase-requisitions` | 创建请购单 |
| POST | `/api/purchase-requisitions/:id/submit` | (workflow → 审批) |
| GET | `/api/purchase-requisitions` | 列表 |
| 继承现有 purchase_orders CRUD → api: `/api/purchase-orders/*` |

## 5. 前端

- `features/suppliers` 加供应商评分页面
- `features/orders/pages/PurchaseRequisitionPage.tsx`
- `features/orders/pages/GoodsReceiptPage.tsx`
- `features/orders/pages/SupplierScorecardPage.tsx`