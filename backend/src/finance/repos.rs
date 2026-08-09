//! Finance repositories — pure SQL, static methods.

use sqlx::{Sqlite, SqlitePool, Transaction};
use crate::models::finance::{
    Account, FinanceInvoice, FinanceInvoiceItem, FinancePayment, JournalEntry, JournalEntryDetail,
    TrialBalanceRow,
};

pub struct AccountRepo;

impl AccountRepo {
    pub async fn list(pool: &SqlitePool, tenant_id: i64) -> Result<Vec<Account>, sqlx::Error> {
        sqlx::query_as::<_, Account>(
            "SELECT id, tenant_id, code, name, account_type, parent_id, is_active, \
                    created_at, updated_at, deleted_at \
             FROM chart_of_accounts WHERE tenant_id = ? AND deleted_at IS NULL ORDER BY code",
        )
        .bind(tenant_id)
        .fetch_all(pool)
        .await
    }

    pub async fn find_by_code(pool: &SqlitePool, tenant_id: i64, code: &str) -> Result<Option<Account>, sqlx::Error> {
        sqlx::query_as::<_, Account>(
            "SELECT id, tenant_id, code, name, account_type, parent_id, is_active, \
                    created_at, updated_at, deleted_at \
             FROM chart_of_accounts WHERE tenant_id = ? AND code = ? AND deleted_at IS NULL",
        )
        .bind(tenant_id)
        .bind(code)
        .fetch_optional(pool)
        .await
    }

    pub async fn find_by_id(pool: &SqlitePool, tenant_id: i64, id: i64) -> Result<Option<Account>, sqlx::Error> {
        sqlx::query_as::<_, Account>(
            "SELECT id, tenant_id, code, name, account_type, parent_id, is_active, \
                    created_at, updated_at, deleted_at \
             FROM chart_of_accounts WHERE tenant_id = ? AND id = ? AND deleted_at IS NULL",
        )
        .bind(tenant_id)
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    pub async fn create(
        pool: &SqlitePool,
        tenant_id: i64,
        code: &str,
        name: &str,
        account_type: &str,
        parent_id: Option<i64>,
    ) -> Result<Account, sqlx::Error> {
        sqlx::query_as::<_, Account>(
            "INSERT INTO chart_of_accounts (tenant_id, code, name, account_type, parent_id) \
             VALUES (?, ?, ?, ?, ?) \
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
        pool: &SqlitePool,
        tenant_id: i64,
        id: i64,
        name: Option<&str>,
        is_active: Option<bool>,
    ) -> Result<Option<Account>, sqlx::Error> {
        sqlx::query_as::<_, Account>(
            "UPDATE chart_of_accounts SET name = COALESCE(?, name), \
                    is_active = COALESCE(?, is_active), updated_at = datetime('now') \
             WHERE tenant_id = ? AND id = ? AND deleted_at IS NULL \
             RETURNING id, tenant_id, code, name, account_type, parent_id, is_active, \
                       created_at, updated_at, deleted_at",
        )
        .bind(name)
        .bind(is_active)
        .bind(tenant_id)
        .bind(id)
        .fetch_optional(pool)
        .await
    }
}

pub struct JournalEntryRepo;

impl JournalEntryRepo {
    pub async fn create(
        tx: &mut Transaction<'_, Sqlite>,
        tenant_id: i64,
        entry_no: &str,
        entry_date: chrono::NaiveDate,
        description: Option<&str>,
        created_by: Option<i64>,
    ) -> Result<JournalEntry, sqlx::Error> {
        sqlx::query_as::<_, JournalEntry>(
            "INSERT INTO journal_entries (tenant_id, entry_no, entry_date, description, created_by) \
             VALUES (?, ?, ?, ?, ?) \
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
        tx: &mut Transaction<'_, Sqlite>,
        entry_id: i64,
        account_id: i64,
        debit: f64,
        credit: f64,
        description: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO journal_entry_details (entry_id, account_id, debit, credit, description) \
             VALUES (?, ?, ?, ?, ?)",
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

    pub async fn post(tx: &mut Transaction<'_, Sqlite>, entry_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE journal_entries SET status = 'posted', posted_at = datetime('now') WHERE id = ?")
            .bind(entry_id)
            .execute(&mut **tx)
            .await?;
        Ok(())
    }

    pub async fn list(
        pool: &SqlitePool,
        tenant_id: i64,
        from: Option<chrono::NaiveDate>,
        to: Option<chrono::NaiveDate>,
    ) -> Result<Vec<JournalEntry>, sqlx::Error> {
        sqlx::query_as::<_, JournalEntry>(
            "SELECT id, tenant_id, entry_no, entry_date, description, status, currency, \
                    created_by, created_at, posted_at \
             FROM journal_entries WHERE tenant_id = ? \
             AND (? IS NULL OR entry_date >= ?) AND (? IS NULL OR entry_date <= ?) \
             ORDER BY entry_date DESC, id DESC LIMIT 500",
        )
        .bind(tenant_id)
        .bind(from)
        .bind(from)
        .bind(to)
        .bind(to)
        .fetch_all(pool)
        .await
    }

    pub async fn details_for_entry(pool: &SqlitePool, entry_id: i64) -> Result<Vec<JournalEntryDetail>, sqlx::Error> {
        sqlx::query_as::<_, JournalEntryDetail>(
            "SELECT id, entry_id, account_id, debit, credit, description \
             FROM journal_entry_details WHERE entry_id = ? ORDER BY id",
        )
        .bind(entry_id)
        .fetch_all(pool)
        .await
    }

    pub async fn find_by_id(pool: &SqlitePool, tenant_id: i64, id: i64) -> Result<Option<JournalEntry>, sqlx::Error> {
        sqlx::query_as::<_, JournalEntry>(
            "SELECT id, tenant_id, entry_no, entry_date, description, status, currency, \
                    created_by, created_at, posted_at \
             FROM journal_entries WHERE tenant_id = ? AND id = ?",
        )
        .bind(tenant_id)
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    pub async fn trial_balance(pool: &SqlitePool, tenant_id: i64) -> Result<Vec<TrialBalanceRow>, sqlx::Error> {
        sqlx::query_as::<_, TrialBalanceRow>(
            "SELECT a.code, a.name, CAST(COALESCE(SUM(d.debit), 0.0) AS REAL) AS debit, \
                    CAST(COALESCE(SUM(d.credit), 0.0) AS REAL) AS credit \
             FROM journal_entry_details d \
             JOIN chart_of_accounts a ON a.id = d.account_id \
             JOIN journal_entries e ON e.id = d.entry_id \
             WHERE e.tenant_id = ? AND e.status = 'posted' \
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
        pool: &SqlitePool,
        tenant_id: i64,
        invoice_no: &str,
        invoice_type: &str,
        party_id: i64,
        order_id: Option<i64>,
        amount: f64,
        tax_amount: f64,
        total_amount: f64,
        due_date: Option<chrono::NaiveDate>,
    ) -> Result<FinanceInvoice, sqlx::Error> {
        sqlx::query_as::<_, FinanceInvoice>(
            "INSERT INTO finance_invoices \
             (tenant_id, invoice_no, invoice_type, party_id, order_id, amount, tax_amount, \
              total_amount, due_date, issued_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, datetime('now')) \
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
        pool: &SqlitePool,
        invoice_id: i64,
        description: Option<&str>,
        quantity: f64,
        unit_price: f64,
        amount: f64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO finance_invoice_items (invoice_id, description, quantity, unit_price, amount) \
             VALUES (?, ?, ?, ?, ?)",
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

    pub async fn find_by_id(pool: &SqlitePool, tenant_id: i64, id: i64) -> Result<Option<FinanceInvoice>, sqlx::Error> {
        sqlx::query_as::<_, FinanceInvoice>(
            "SELECT id, tenant_id, invoice_no, invoice_type, party_id, order_id, amount, \
                    tax_amount, total_amount, status, due_date, issued_at, created_at, updated_at \
             FROM finance_invoices WHERE tenant_id = ? AND id = ?",
        )
        .bind(tenant_id)
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    pub async fn update_status(
        pool: &SqlitePool,
        tenant_id: i64,
        id: i64,
        status: &str,
    ) -> Result<Option<FinanceInvoice>, sqlx::Error> {
        sqlx::query_as::<_, FinanceInvoice>(
            "UPDATE finance_invoices SET status = ?, updated_at = datetime('now') \
             WHERE tenant_id = ? AND id = ? \
             RETURNING id, tenant_id, invoice_no, invoice_type, party_id, order_id, amount, \
                       tax_amount, total_amount, status, due_date, issued_at, created_at, updated_at",
        )
        .bind(status)
        .bind(tenant_id)
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    pub async fn list(
        pool: &SqlitePool,
        tenant_id: i64,
        invoice_type: Option<&str>,
        status: Option<&str>,
    ) -> Result<Vec<FinanceInvoice>, sqlx::Error> {
        sqlx::query_as::<_, FinanceInvoice>(
            "SELECT id, tenant_id, invoice_no, invoice_type, party_id, order_id, amount, \
                    tax_amount, total_amount, status, due_date, issued_at, created_at, updated_at \
             FROM finance_invoices WHERE tenant_id = ? \
             AND (? IS NULL OR invoice_type = ?) AND (? IS NULL OR status = ?) \
             ORDER BY id DESC LIMIT 500",
        )
        .bind(tenant_id)
        .bind(invoice_type)
        .bind(invoice_type)
        .bind(status)
        .bind(status)
        .fetch_all(pool)
        .await
    }

    pub async fn items_for_invoice(pool: &SqlitePool, invoice_id: i64) -> Result<Vec<FinanceInvoiceItem>, sqlx::Error> {
        sqlx::query_as::<_, FinanceInvoiceItem>(
            "SELECT id, invoice_id, description, quantity, unit_price, amount \
             FROM finance_invoice_items WHERE invoice_id = ? ORDER BY id",
        )
        .bind(invoice_id)
        .fetch_all(pool)
        .await
    }
}

pub struct PaymentRepo;

impl PaymentRepo {
    pub async fn create(
        pool: &SqlitePool,
        tenant_id: i64,
        payment_no: &str,
        invoice_id: Option<i64>,
        direction: &str,
        amount: f64,
        method: &str,
        reference: Option<&str>,
        created_by: Option<i64>,
    ) -> Result<FinancePayment, sqlx::Error> {
        sqlx::query_as::<_, FinancePayment>(
            "INSERT INTO finance_payments \
             (tenant_id, payment_no, invoice_id, direction, amount, method, reference, created_by) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?) \
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

    pub async fn list(pool: &SqlitePool, tenant_id: i64, invoice_id: Option<i64>) -> Result<Vec<FinancePayment>, sqlx::Error> {
        sqlx::query_as::<_, FinancePayment>(
            "SELECT id, tenant_id, payment_no, invoice_id, direction, amount, method, \
                    paid_at, reference, created_by, created_at \
             FROM finance_payments WHERE tenant_id = ? \
             AND (? IS NULL OR invoice_id = ?) ORDER BY id DESC LIMIT 500",
        )
        .bind(tenant_id)
        .bind(invoice_id)
        .bind(invoice_id)
        .fetch_all(pool)
        .await
    }
}
