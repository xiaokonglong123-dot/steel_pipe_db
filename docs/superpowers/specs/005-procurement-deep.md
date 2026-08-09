# 005 — 采购深化 & 供应商管理 (Phase 2)

> **版本**: v1.0
> **日期**: 2026-08-02
> **依赖**: 004-finance-module, 002-workflow-engine
> **父**: 015-architecture-overview.md

---

## 1. 目标

在现有采购订单基础上增加供应商全生命周期管理、采购报价、交付跟踪、3-way 匹配。

## 2. 功能

| 功能 | 描述 | 优先级 |
| ------ | ------ | -------- |
| Supplier Relationship | 供应商档案 + qualifications + contracts | P0 |
| 采购申请 | 需求申请 → 经理审批 → 生成采购订单 | P0 |
| 采购报价管理 | 获取供应商报价 + 比价 | P1 |
| Delivery Tracking | 跟踪，分批发货 | P1 |
| 3-way Matching | 采购订单 → 采购收货 → 发票 → Auto payment | P1 |
| Supplier Dashboard | 供应商评分、准时率 | P2 |

## 3. 数据模型

> 数据库为 SQLite3（`sqlite://data/erp.db?mode=rwc`），表名不带 schema 前缀。

```sql
CREATE TABLE suppliers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    supplier_code TEXT UNIQUE,   -- e.g. SUP-001
    name TEXT NOT NULL,
    tax_id TEXT,                   -- 统一社会信用代码
    contact_person TEXT,
    contact_phone TEXT,
    contact_email TEXT,
    address TEXT,
    bank_account TEXT,
    payment_terms TEXT,
    delivery_lead_time_days INTEGER,
    qualified_until TEXT,          -- 认证有效期
    is_active INTEGER DEFAULT 1,
    source INTEGER
);

-- 采购申请表
CREATE TABLE purchase_requisitions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    req_no TEXT UNIQUE,
    requester_id INTEGER REFERENCES users(id),
    department_id INTEGER,
    status TEXT DEFAULT 'draft',   -- draft/pending/approved/rejected 与 workflow 联动
    reason TEXT,
    total_estimated REAL DEFAULT 0,
    workflow_instance_id INTEGER,
    created_at, updated_at
);

CREATE TABLE po_receipts (         -- 采购收货 (Goods Receipt)
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    purchase_order_id INTEGER REFERENCES purchase_orders(id),
    receipt_date TEXT,
    quantity REAL,
    received_by INTEGER REFERENCES users(id),
    status TEXT DEFAULT 'pending', -- pending/approved/matched
    note TEXT,
    created_at TEXT DEFAULT (datetime('now'))
);

-- 供应商报价 (采购报价)
CREATE TABLE supplier_quotes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    purchase_requisition_id INTEGER,
    supplier_id INTEGER REFERENCES suppliers(id),
    quote_no TEXT,
    total_amount REAL,
    delivery_days INTEGER,
    notes TEXT,
    valid_until TEXT,
    status TEXT DEFAULT 'open'
);
```

## 4. API 端点

| Method | Path | Description |
| -------- | ------ | ------------- |
| GET | `/api/suppliers` | 供应商(分页 + 搜索) |
| POST | `/api/suppliers` | 新供应商 |
| GET | `/api/suppliers/:id` | 详情 |
| GET | `/api/suppliers/:id/scorecard` | 供应商评分 |
| POST | `/api/purchase-requisitions` | 创建采购申请 |
| POST | `/api/purchase-requisitions/:id/submit` | (workflow → 审批) |
| GET | `/api/purchase-requisitions` | 列表 |
| 继承现有 purchase_orders CRUD → api: `/api/purchase-orders/*` |

## 5. 前端

- `features/suppliers` 加供应商评分页面
- `features/procurement/pages/PurchaseRequisitionPage.tsx`
- `features/procurement/pages/GoodsReceiptPage.tsx`
- `features/procurement/pages/SupplierScorecardPage.tsx`
