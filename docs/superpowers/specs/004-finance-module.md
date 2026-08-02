# 004 — 财务会计模块 (Phase 2)

> **版本**: v1.0
> **日期**: 2026-08-02
> **依赖**: 001-auth-identity (用户), 003-hr (薪资集成)
> **父文档**: 015-architecture-overview.md

---

## 1. 目标

实现标准的总账 + 应收应付 + 祥式发票 + 多币种 + 财务扣款核算。

## 2. 功能范围

| 子模块 | 说明 |
|--------|------|
| 会计科目表 | 完整的科目体系 |
| 总账 | 日记账分录 (Journal Entry) |
| 应收款 | Account Receivable (AR) |
| 应付款 | Account Payable (AP) |
| 发票 | 销售发票 + 采购发票 + invoice 匹配 |
| 收/付款 | 收据+ payment confirmation |
| 多币种 | 交易币种 + 汇率 |
| 金融报告 | Trial Balance 利润表 |

## 3. 数据模型

```sql
-- Schema: finance

CREATE TABLE finance.chart_of_accounts (
    id BIGSERIAL PRIMARY KEY,
    company_id BIGINT REFERENCES auth.companies(id),
    account_code VARCHAR(50) NOT NULL UNIQUE,   -- (e.g. 1001 - Cash)
    account_name VARCHAR(200) NOT NULL,
    account_type VARCHAR(50) NOT NULL,          -- Expense, Asset, Liability, Revenue, Equity
    parent_id BIGINT REFERENCES finance.chart_of_accounts(id),
    is_active BOOLEAN DEFAULT true,
    opening_balance NUMERIC(18,2) DEFAULT 0,
    description TEXT,
    created_at, updated_at, deleted_at
);

CREATE TABLE finance.journal_entries (
    id BIGSERIAL PRIMARY KEY,
    company_id BIGINT NOT NULL,
    entry_no VARCHAR(100) UNIQUE,   -- like JE-2026-001
    entry_date DATE NOT NULL,
    reference_document_type VARCHAR(50),   -- PO, SO, Invoice, payment
    reference_document_id BIGINT,
    description TEXT,
    total_debits NUMERIC(18,2),
    total_credits NUMERIC(18,2),
    is_balanced BOOLEAN NOT NULL,   -- 借贷平衡
    posted BOOLEAN DEFAULT false
    created_by BIGINT REFERENCES auth.users(id),
    created_at, updated_at
);

CREATE TABLE finance.journal_entry_details (
    id BIGSERIAL PRIMARY KEY,
    journal_entry_id BIGINT REFERENCES finance.journal_entries(id),
    account_id BIGINT REFERENCES finance.chart_of_accounts(id),
    amount_debit NUMERIC(18,2) DEFAULT 0,
    amount_credit NUMERIC(18,2) DEFAULT 0,
    description TEXT,
    currency VARCHAR(3) DEFAULT 'CNY',
    exchange_rate NUMERIC(12,6) DEFAULT 1.0,
    department_id BIGINT,
    created_at TIMESTAMPTZ
);

-- 应收表 (AR)
CREATE TABLE finance.invoices (
    id BIGSERIAL PRIMARY KEY,
    invoice_type VARCHAR(10) NOT NULL,   -- 'AR' (香菜) or  'AP' (红P)
    source_document_type VARCHAR(50),-- sales_order, purchase_order
    source_document_id BIGINT,
    invoice_no VARCHAR(100) UNIQUE NOT NULL,
    invoice_number VARCHAR(200) NOT NULL,
    customer_id_or_supplier_id BIGINT,
    invoice_date DATE,
    due_date DATE,
    total_amount NUMERIC(18,2),
    tax_amount NUMERIC(18,2),
    amount_adjusted NUMERIC(18,2) DEFAULT 0,
    status VARCHAR(20) DEFAULT 'draft',       是 pending/posted/paid/due
    currency VARCHAR(3) DEFAULT 'CNY',
    tax_code VARCHAR(20),
    created_at, updated_at, deleted_at
);

CREATE TABLE finance.invoice_items (
    id BIGSERIAL PRIMARY KEY,
    invoice_id BIGINT REFERENCES finance.invoices(id),
    item_description TEXT,
    quantity NUMERIC(18,4),
    unit_price NUMERIC(18,4),
    total_price NUMERIC(18,2),
    tax_rate NUMERIC(5,3) DEFAULT 0.0,
    tax_code VARCHAR(20)
);

CREATE TABLE finance.payments (
    id BIGSERIAL PRIMARY KEY,
    payment_direction VARCHAR(5) NOT NULL,   -- 'IN' (收款), 'J币'  (付款 = 'OUT')
    invoice_id BIGINT REFERENCES finance.invoices(id),
    amount NUMERIC(18,2) NOT NULL,
    currency VARCHAR(3) DEFAULT 'CNY',
    payment_method VARCHAR(30),    -- bank_transfer, cash, alipay
    payment_reference VARCHAR(200),
    paid_at TIMESTAMPTZ,
    confirmed_by BIGINT REFERENCES auth.users(id),
    created_at TIMESTAMPTZ DEFAULT NOW()
);
```

## 4. 核心业务流程 loop

### 采购付款

```
PO 批准 → 创建应付款录
         → 收到币 格 → PO.status = received
         → 自动创建应付垫款 (AP invoice)
         → 批准后 → 生成 journal entry:
            Debit: Expense 科目 (amount)
 Information: AP Account (amount)
         → 付款申请 → 确认支付 → journal entry:
            Credit: Bank Account (amount)
            Debit: AP Account (amount)
```

### 销售收款

```
SO → AR 发票创建
   → 收到付款确认 → journal entry:
      Debit: Bank Account (amount)
      Credit: AR Account (amount)
```

## 5. API

| Method | Path | Description |
|--------|------|-------------|
| 科目 | | |
| GET | `/api/chart-of-accounts` | 会计科目树 |
| POST | `/api/chart-of-accounts` | 新增科目 |
| 分录 | | |
| GET | `/api/journal-entries` | 分录列表 |
| POST | `/api/journal-entries` | 创建分录 |
| 发票 | | |
| GET | `/api/invoices` | 纸发票列表 |
| POST | `/api/invoices` | 创建发票 |
| POST | `/api/invoices/:id/confirm` | 确认获发 |
| POST | `/api/invoices/:id/void` | 作废 |
| 付款 | | |
| GET | `/api/payments` | 收/付款记录 |
| POST | `/api/payments` | 登记支付 |
| 报表 | | |
| GET | `/api/finance/trial-balance` | 试算平衡表 |
| GET | `/api/finance/profit-loss` | 利润表 simpl Association) |

## 6. 前后端

**后端** (crates/finance):
- `finance_service.rs` — AR/AP/GL business logic
- `finance_repo.rs` — expose query helpers for reporting

**前端**:
- `features/finance/pages/ChartOfAccounts.tsx`
- `features/finance/pages/JournalEntryList.tsx`
- `features/finance/pages/InvoiceList.tsx`
- `features/finance/pages/PaymentList.tsx`
- `features/finance/pages/GLReport.tsx`