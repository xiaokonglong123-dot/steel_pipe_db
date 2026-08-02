//! Finance repositories — pure SQL, static methods.

use sqlx::{PgPool, Postgres, Transaction};
use crate::models::finance::{
    Account, FinanceInvoice, FinanceInvoiceItem, FinancePayment, JournalEntry, JournalEntryDetail,
    TrialBalanceRow,
};

pub struct AccountRepo;

impl AccountRepo {
    pub async fn list(pool: &PgPool, tenant_id: i64) -> Result<Vec<Account>, sqlx::Error> {
        sqlx::query_as::<_, Account>(
            "SELECT id, tenant_id, code, name, account_type, parent_id, is_active, \
                    created_at, updated_at, deleted_at \
             FROM chart_of_accounts WHERE tenant_id = $1 AND deleted_at IS NULL ORDER BY code",
        )
        .bind(tenant_id)
        .fetch_all(pool)
        .await
    }

    pub async fn find_by_code(pool: &PgPool, tenant_id: i64, code: &str) -> Result<Option<Account>, sqlx::Error> {
        sqlx::query_as::<_, Account>(
            "SELECT id, tenant_id, code, name, account_type, parent_id, is_active, \
                    created_at, updated_at, deleted_at \
             FROM chart_of_accounts WHERE tenant_id = $1 AND code = $2 AND deleted_at IS NULL",
        )
        .bind(tenant_id)
        .bind(code)
        .fetch_optional(pool)
        .await
    }

    pub async fn find_by_id(pool: &PgPool, tenant_id: i64, id: i64) -> Result<Option<Account>, sqlx::Error> {
        sqlx::query_as::<_, Account>(
            "SELECT id, tenant_id, code, name, account_type, parent_id, is_active, \
                    created_at, updated_at, deleted_at \
             FROM chart_of_accounts WHERE tenant_id = $1 AND id = $2 AND deleted_at IS NULL",
        )
        .bind(tenant_id)
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    pub async fn create(
        pool: &PgPool,
        tenant_id: i64,
        code: &str,
        name: &str,
        account_type: &str,
        parent_id: Option<i64>,
    ) -> Result<Account, sqlx::Error> {
        sqlx::query_as::<_, Account>(
            "INSERT INTO chart_of_accounts (tenant_id, code, name, account_type, parent_id) \
             VALUES ($1, $2, $3, $4, $5) \
             RETURNING id, tenant_id, code, name, account_type, parent_id, is_active, \
                       created_at, updated_at, deleted_at",
        )
        .bind(tenant_id)
        .bind(code)
        .bind(name)
        .bind(account_type)
        .bind(parent_id)
        .fetch_one(pool)
        .await
    }

    pub async fn update(
        pool: &PgPool,
        tenant_id: i64,
        id: i64,
        name: Option<&str>,
        is_active: Option<bool>,
    ) -> Result<Option<Account>, sqlx::Error> {
        sqlx::query_as::<_, Account>(
            "UPDATE chart_of_accounts SET name = COALESCE($3, name), \
                    is_active = COALESCE($4, is_active), updated_at = NOW() \
             WHERE tenant_id = $1 AND id = $2 AND deleted_at IS NULL \
             RETURNING id, tenant_id, code, name, account_type, parent_id, is_active, \
                       created_at, updated_at, deleted_at",
        )
        .bind(tenant_id)
        .bind(id)
        .bind(name)
        .bind(is_active)
        .fetch_optional(pool)
        .await
    }
}

pub struct JournalEntryRepo;

impl JournalEntryRepo {
    pub async fn create(
        tx: &mut Transaction<'_, Postgres>,
        tenant_id: i64,
        entry_no: &str,
        entry_date: chrono::NaiveDate,
        description: Option<&str>,
        created_by: Option<i64>,
    ) -> Result<JournalEntry, sqlx::Error> {
        sqlx::query_as::<_, JournalEntry>(
            "INSERT INTO journal_entries (tenant_id, entry_no, entry_date, description, created_by) \
             VALUES ($1, $2, $3, $4, $5) \
             RETURNING id, tenant_id, entry_no, entry_date, description, status, currency, \
                       created_by, created_at, posted_at",
        )
        .bind(tenant_id)
        .bind(entry_no)
        .bind(entry_date)
        .bind(description)
        .bind(created_by)
        .fetch_one(&mut **tx)
        .await
    }

    pub async fn insert_detail(
        tx: &mut Transaction<'_, Postgres>,
        entry_id: i64,
        account_id: i64,
        debit: rust_decimal::Decimal,
        credit: rust_decimal::Decimal,
        description: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO journal_entry_details (entry_id, account_id, debit, credit, description) \
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(entry_id)
        .bind(account_id)
        .bind(debit)
        .bind(credit)
        .bind(description)
        .execute(&mut **tx)
        .await?;
        Ok(())
    }

    pub async fn post(tx: &mut Transaction<'_, Postgres>, entry_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE journal_entries SET status = 'posted', posted_at = NOW() WHERE id = $1")
            .bind(entry_id)
            .execute(&mut **tx)
            .await?;
        Ok(())
    }

    pub async fn list(
        pool: &PgPool,
        tenant_id: i64,
        from: Option<chrono::NaiveDate>,
        to: Option<chrono::NaiveDate>,
    ) -> Result<Vec<JournalEntry>, sqlx::Error> {
        sqlx::query_as::<_, JournalEntry>(
            "SELECT id, tenant_id, entry_no, entry_date, description, status, currency, \
                    created_by, created_at, posted_at \
             FROM journal_entries WHERE tenant_id = $1 \
             AND ($2::date IS NULL OR entry_date >= $2) AND ($3::date IS NULL OR entry_date <= $3) \
             ORDER BY entry_date DESC, id DESC LIMIT 500",
        )
        .bind(tenant_id)
        .bind(from)
        .bind(to)
        .fetch_all(pool)
        .await
    }

    pub async fn details_for_entry(pool: &PgPool, entry_id: i64) -> Result<Vec<JournalEntryDetail>, sqlx::Error> {
        sqlx::query_as::<_, JournalEntryDetail>(
            "SELECT id, entry_id, account_id, debit, credit, description \
             FROM journal_entry_details WHERE entry_id = $1 ORDER BY id",
        )
        .bind(entry_id)
        .fetch_all(pool)
        .await
    }

    pub async fn find_by_id(pool: &PgPool, tenant_id: i64, id: i64) -> Result<Option<JournalEntry>, sqlx::Error> {
        sqlx::query_as::<_, JournalEntry>(
            "SELECT id, tenant_id, entry_no, entry_date, description, status, currency, \
                    created_by, created_at, posted_at \
             FROM journal_entries WHERE tenant_id = $1 AND id = $2",
        )
        .bind(tenant_id)
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    pub async fn trial_balance(pool: &PgPool, tenant_id: i64) -> Result<Vec<TrialBalanceRow>, sqlx::Error> {
        sqlx::query_as::<_, TrialBalanceRow>(
            "SELECT a.code, a.name, COALESCE(SUM(d.debit), 0) AS debit, \
                    COALESCE(SUM(d.credit), 0) AS credit \
             FROM journal_entry_details d \
             JOIN chart_of_accounts a ON a.id = d.account_id \
             JOIN journal_entries e ON e.id = d.entry_id \
             WHERE e.tenant_id = $1 AND e.status = 'posted' \
             GROUP BY a.code, a.name ORDER BY a.code",
        )
        .bind(tenant_id)
        .fetch_all(pool)
        .await
    }
}

pub struct InvoiceRepo;

impl InvoiceRepo {
    pub async fn create(
        pool: &PgPool,
        tenant_id: i64,
        invoice_no: &str,
        invoice_type: &str,
        party_id: i64,
        order_id: Option<i64>,
        amount: rust_decimal::Decimal,
        tax_amount: rust_decimal::Decimal,
        total_amount: rust_decimal::Decimal,
        due_date: Option<chrono::NaiveDate>,
    ) -> Result<FinanceInvoice, sqlx::Error> {
        sqlx::query_as::<_, FinanceInvoice>(
            "INSERT INTO finance_invoices \
             (tenant_id, invoice_no, invoice_type, party_id, order_id, amount, tax_amount, \
              total_amount, due_date, issued_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW()) \
             RETURNING id, tenant_id, invoice_no, invoice_type, party_id, order_id, amount, \
                       tax_amount, total_amount, status, due_date, issued_at, created_at, updated_at",
        )
        .bind(tenant_id)
        .bind(invoice_no)
        .bind(invoice_type)
        .bind(party_id)
        .bind(order_id)
        .bind(amount)
        .bind(tax_amount)
        .bind(total_amount)
        .bind(due_date)
        .fetch_one(pool)
        .await
    }

    pub async fn insert_item(
        pool: &PgPool,
        invoice_id: i64,
        description: Option<&str>,
        quantity: rust_decimal::Decimal,
        unit_price: rust_decimal::Decimal,
        amount: rust_decimal::Decimal,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO finance_invoice_items (invoice_id, description, quantity, unit_price, amount) \
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(invoice_id)
        .bind(description)
        .bind(quantity)
        .bind(unit_price)
        .bind(amount)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn find_by_id(pool: &PgPool, tenant_id: i64, id: i64) -> Result<Option<FinanceInvoice>, sqlx::Error> {
        sqlx::query_as::<_, FinanceInvoice>(
            "SELECT id, tenant_id, invoice_no, invoice_type, party_id, order_id, amount, \
                    tax_amount, total_amount, status, due_date, issued_at, created_at, updated_at \
             FROM finance_invoices WHERE tenant_id = $1 AND id = $2",
        )
        .bind(tenant_id)
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    pub async fn update_status(
        pool: &PgPool,
        tenant_id: i64,
        id: i64,
        status: &str,
    ) -> Result<Option<FinanceInvoice>, sqlx::Error> {
        sqlx::query_as::<_, FinanceInvoice>(
            "UPDATE finance_invoices SET status = $3, updated_at = NOW() \
             WHERE tenant_id = $1 AND id = $2 \
             RETURNING id, tenant_id, invoice_no, invoice_type, party_id, order_id, amount, \
                       tax_amount, total_amount, status, due_date, issued_at, created_at, updated_at",
        )
        .bind(tenant_id)
        .bind(id)
        .bind(status)
        .fetch_optional(pool)
        .await
    }

    pub async fn list(
        pool: &PgPool,
        tenant_id: i64,
        invoice_type: Option<&str>,
        status: Option<&str>,
    ) -> Result<Vec<FinanceInvoice>, sqlx::Error> {
        sqlx::query_as::<_, FinanceInvoice>(
            "SELECT id, tenant_id, invoice_no, invoice_type, party_id, order_id, amount, \
                    tax_amount, total_amount, status, due_date, issued_at, created_at, updated_at \
             FROM finance_invoices WHERE tenant_id = $1 \
             AND ($2::text IS NULL OR invoice_type = $2) AND ($3::text IS NULL OR status = $3) \
             ORDER BY id DESC LIMIT 500",
        )
        .bind(tenant_id)
        .bind(invoice_type)
        .bind(status)
        .fetch_all(pool)
        .await
    }

    pub async fn items_for_invoice(pool: &PgPool, invoice_id: i64) -> Result<Vec<FinanceInvoiceItem>, sqlx::Error> {
        sqlx::query_as::<_, FinanceInvoiceItem>(
            "SELECT id, invoice_id, description, quantity, unit_price, amount \
             FROM finance_invoice_items WHERE invoice_id = $1 ORDER BY id",
        )
        .bind(invoice_id)
        .fetch_all(pool)
        .await
    }
}

pub struct PaymentRepo;

impl PaymentRepo {
    pub async fn create(
        pool: &PgPool,
        tenant_id: i64,
        payment_no: &str,
        invoice_id: Option<i64>,
        direction: &str,
        amount: rust_decimal::Decimal,
        method: &str,
        reference: Option<&str>,
        created_by: Option<i64>,
    ) -> Result<FinancePayment, sqlx::Error> {
        sqlx::query_as::<_, FinancePayment>(
            "INSERT INTO finance_payments \
             (tenant_id, payment_no, invoice_id, direction, amount, method, reference, created_by) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8) \
             RETURNING id, tenant_id, payment_no, invoice_id, direction, amount, method, \
                       paid_at, reference, created_by, created_at",
        )
        .bind(tenant_id)
        .bind(payment_no)
        .bind(invoice_id)
        .bind(direction)
        .bind(amount)
        .bind(method)
        .bind(reference)
        .bind(created_by)
        .fetch_one(pool)
        .await
    }

    pub async fn list(pool: &PgPool, tenant_id: i64, invoice_id: Option<i64>) -> Result<Vec<FinancePayment>, sqlx::Error> {
        sqlx::query_as::<_, FinancePayment>(
            "SELECT id, tenant_id, payment_no, invoice_id, direction, amount, method, \
                    paid_at, reference, created_by, created_at \
             FROM finance_payments WHERE tenant_id = $1 \
             AND ($2::bigint IS NULL OR invoice_id = $2) ORDER BY id DESC LIMIT 500",
        )
        .bind(tenant_id)
        .bind(invoice_id)
        .fetch_all(pool)
        .await
    }
}
