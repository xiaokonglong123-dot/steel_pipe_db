# 004 — 财务会计模块 (Phase 2)

> **版本**: v1.0
> **日期**: 2026-08-02
> **依赖**: 001-auth-identity (用户), 003-hr (薪资集成)
> **父文档**: 015-architecture-overview.md

---

## 1. 目标

实现标准的总账 + 应收应付 + 发票 + 多币种 + 财务核算。

## 2. 功能范围

| 子模块 | 说明 |
| -------- | ------ |
| 会计科目表 | 完整的科目体系 |
| 总账 | 日记账分录 (Journal Entry) |
| 应收款 | Account Receivable (AR) |
| 应付款 | Account Payable (AP) |
| 发票 | 销售发票 + 采购发票 + invoice 匹配 |
| 收/付款 | 收据 + payment confirmation |
| 多币种 | 交易币种 + 汇率 |
| 金融报告 | Trial Balance 利润表 |

## 3. 数据模型

> 数据库为 SQLite3（`sqlite://data/erp.db?mode=rwc`），金额用 REAL，时间用 TEXT。

```sql
CREATE TABLE chart_of_accounts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    company_id INTEGER REFERENCES tenants(id),
    account_code TEXT NOT NULL UNIQUE,   -- e.g. 1001 - Cash
    account_name TEXT NOT NULL,
    account_type TEXT NOT NULL,          -- Expense, Asset, Liability, Revenue, Equity
    parent_id INTEGER REFERENCES chart_of_accounts(id),
    is_active INTEGER DEFAULT 1,
    opening_balance REAL DEFAULT 0,
    description TEXT,
    created_at, updated_at, deleted_at
);

CREATE TABLE journal_entries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    company_id INTEGER NOT NULL,
    entry_no TEXT UNIQUE,   -- like JE-2026-001
    entry_date TEXT NOT NULL,
    reference_document_type TEXT,   -- PO, SO, Invoice, payment
    reference_document_id INTEGER,
    description TEXT,
    total_debits REAL,
    total_credits REAL,
    is_balanced INTEGER NOT NULL,   -- 借贷平衡
    posted INTEGER DEFAULT 0,
    created_by INTEGER REFERENCES users(id),
    created_at, updated_at
);

CREATE TABLE journal_entry_details (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    journal_entry_id INTEGER REFERENCES journal_entries(id),
    account_id INTEGER REFERENCES chart_of_accounts(id),
    amount_debit REAL DEFAULT 0,
    amount_credit REAL DEFAULT 0,
    description TEXT,
    currency TEXT DEFAULT 'CNY',
    exchange_rate REAL DEFAULT 1.0,
    department_id INTEGER,
    created_at TEXT DEFAULT (datetime('now'))
);

-- 应收表 (AR)
CREATE TABLE invoices (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    invoice_type TEXT NOT NULL,   -- 'AR' (应收) or 'AP' (应付)
    source_document_type TEXT,    -- sales_order, purchase_order
    source_document_id INTEGER,
    invoice_no TEXT UNIQUE NOT NULL,
    invoice_number TEXT NOT NULL,
    customer_id_or_supplier_id INTEGER,
    invoice_date TEXT,
    due_date TEXT,
    total_amount REAL,
    tax_amount REAL,
    amount_adjusted REAL DEFAULT 0,
    status TEXT DEFAULT 'draft',  -- pending/posted/paid/due
    currency TEXT DEFAULT 'CNY',
    tax_code TEXT,
    created_at, updated_at, deleted_at
);

CREATE TABLE invoice_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    invoice_id INTEGER REFERENCES invoices(id),
    item_description TEXT,
    quantity REAL,
    unit_price REAL,
    total_price REAL,
    tax_rate REAL DEFAULT 0.0,
    tax_code TEXT
);

CREATE TABLE payments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    payment_direction TEXT NOT NULL,   -- 'IN' (收款), 'OUT' (付款)
    invoice_id INTEGER REFERENCES invoices(id),
    amount REAL NOT NULL,
    currency TEXT DEFAULT 'CNY',
    payment_method TEXT,    -- bank_transfer, cash, alipay
    payment_reference TEXT,
    paid_at TEXT,
    confirmed_by INTEGER REFERENCES users(id),
    created_at TEXT DEFAULT (datetime('now'))
);
```

## 4. 核心业务流程

### 采购付款

```
采购订单批准 → 创建应付记录
         → 采购收货 → 订单状态 = received
         → 自动创建应付发票 (AP invoice)
         → 批准后 → 生成 journal entry:
            Debit: Expense 科目 (amount)
            Credit: AP Account (amount)
         → 付款申请 → 确认支付 → journal entry:
            Credit: Bank Account (amount)
            Debit: AP Account (amount)
```

### 销售收款

```
销售订单 → AR 发票创建
   → 收到付款确认 → journal entry:
      Debit: Bank Account (amount)
      Credit: AR Account (amount)
```

## 5. API

| Method | Path | Description |
| -------- | ------ | ------------- |
| 科目 | | |
| GET | `/api/chart-of-accounts` | 会计科目树 |
| POST | `/api/chart-of-accounts` | 新增科目 |
| 分录 | | |
| GET | `/api/journal-entries` | 分录列表 |
| POST | `/api/journal-entries` | 创建分录 |
| 发票 | | |
| GET | `/api/invoices` | 发票列表 |
| POST | `/api/invoices` | 创建发票 |
| POST | `/api/invoices/:id/confirm` | 确认发票 |
| POST | `/api/invoices/:id/void` | 作废 |
| 付款 | | |
| GET | `/api/payments` | 收/付款记录 |
| POST | `/api/payments` | 登记支付 |
| 报表 | | |
| GET | `/api/finance/trial-balance` | 试算平衡表 |
| GET | `/api/finance/profit-loss` | 利润表 |

## 6. 前后端

**后端** (`backend/src/finance`):

- `services.rs` — AR/AP/GL business logic
- `repos.rs` — expose query helpers for reporting

**前端**:

- `features/finance/pages/ChartOfAccounts.tsx`
- `features/finance/pages/JournalEntryList.tsx`
- `features/finance/pages/InvoiceList.tsx`
- `features/finance/pages/PaymentList.tsx`
- `features/finance/pages/GLReport.tsx`
