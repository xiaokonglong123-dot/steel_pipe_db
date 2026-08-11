use sqlx::{Sqlite, SqlitePool, Transaction};

use crate::error::{AppError, ErrorCode};
use crate::services::finance_service::{
    CreateAccountRequest, CreateInvoiceRequest, CreatePaymentRequest,
};

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct AccountRow {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub parent_id: Option<i64>,
    pub account_type: String,
    pub is_active: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct JournalEntryRow {
    pub id: i64,
    pub entry_no: String,
    pub entry_date: String,
    pub description: Option<String>,
    pub status: String,
    pub ref_type: Option<String>,
    pub ref_id: Option<i64>,
    pub created_by: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct JournalEntryLineRow {
    pub id: i64,
    pub entry_id: i64,
    pub account_id: i64,
    pub debit: String,
    pub credit: String,
    pub description: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct InvoiceRow {
    pub id: i64,
    pub invoice_no: String,
    pub invoice_date: String,
    pub party_type: String,
    pub party_id: i64,
    pub amount: String,
    pub ref_type: Option<String>,
    pub ref_id: Option<i64>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct PaymentRow {
    pub id: i64,
    pub payment_no: String,
    pub payment_date: String,
    pub supplier_id: Option<i64>,
    pub amount: String,
    pub invoice_id: Option<i64>,
    pub method: Option<String>,
    pub notes: Option<String>,
    pub created_by: Option<i64>,
    pub created_at: String,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PostedLineRow {
    pub account_id: i64,
    pub account_code: String,
    pub account_name: String,
    pub debit: String,
    pub credit: String,
}

const ACCOUNT_COLUMNS: &str = "id, code, name, parent_id, account_type, is_active, created_at";
const JOURNAL_COLUMNS: &str = "id, entry_no, entry_date, description, status, ref_type, ref_id, created_by, created_at, updated_at";
const INVOICE_COLUMNS: &str = "id, invoice_no, invoice_date, party_type, party_id, amount, ref_type, ref_id, status, created_at, updated_at";
const PAYMENT_COLUMNS: &str = "id, payment_no, payment_date, supplier_id, amount, invoice_id, method, notes, created_by, created_at";

pub async fn insert_account(
    pool: &SqlitePool,
    dto: &CreateAccountRequest,
) -> Result<AccountRow, AppError> {
    let result = sqlx::query(
        "INSERT INTO accounts (code, name, parent_id, account_type) VALUES (?, ?, ?, ?)",
    )
    .bind(&dto.code)
    .bind(&dto.name)
    .bind(dto.parent_id)
    .bind(&dto.account_type)
    .execute(pool)
    .await?;
    find_account_by_id(pool, result.last_insert_rowid())
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::Internal, "会计科目创建后读取失败"))
}

pub async fn find_account_by_id(
    pool: &SqlitePool,
    id: i64,
) -> Result<Option<AccountRow>, AppError> {
    Ok(sqlx::query_as::<_, AccountRow>(&format!(
        "SELECT {ACCOUNT_COLUMNS} FROM accounts WHERE id = ?"
    ))
    .bind(id)
    .fetch_optional(pool)
    .await?)
}

pub async fn find_account_by_code(
    pool: &SqlitePool,
    code: &str,
) -> Result<Option<AccountRow>, AppError> {
    Ok(sqlx::query_as::<_, AccountRow>(&format!(
        "SELECT {ACCOUNT_COLUMNS} FROM accounts WHERE code = ?"
    ))
    .bind(code)
    .fetch_optional(pool)
    .await?)
}

pub async fn list_accounts(
    pool: &SqlitePool,
    account_type: Option<&str>,
    active_only: bool,
) -> Result<Vec<AccountRow>, AppError> {
    let mut sql = format!("SELECT {ACCOUNT_COLUMNS} FROM accounts WHERE 1=1");
    if account_type.is_some() {
        sql.push_str(" AND account_type = ?");
    }
    if active_only {
        sql.push_str(" AND is_active = 1");
    }
    sql.push_str(" ORDER BY code");
    let mut query = sqlx::query_as::<_, AccountRow>(&sql);
    if let Some(account_type) = account_type {
        query = query.bind(account_type);
    }
    Ok(query.fetch_all(pool).await?)
}

pub async fn list_accounts_by_type(
    pool: &SqlitePool,
    account_type: &str,
) -> Result<Vec<AccountRow>, AppError> {
    list_accounts(pool, Some(account_type), false).await
}

pub async fn update_account(
    pool: &SqlitePool,
    id: i64,
    code: &str,
    name: &str,
    parent_id: Option<i64>,
    account_type: &str,
) -> Result<AccountRow, AppError> {
    let result = sqlx::query(
        "UPDATE accounts SET code = ?, name = ?, parent_id = ?, account_type = ? WHERE id = ?",
    )
    .bind(code)
    .bind(name)
    .bind(parent_id)
    .bind(account_type)
    .bind(id)
    .execute(pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::new(ErrorCode::AccountNotFound, "会计科目未找到"));
    }
    find_account_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::AccountNotFound, "会计科目未找到"))
}

pub async fn deactivate_account(pool: &SqlitePool, id: i64) -> Result<(), AppError> {
    let result = sqlx::query("UPDATE accounts SET is_active = 0 WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::new(ErrorCode::AccountNotFound, "会计科目未找到"));
    }
    Ok(())
}

pub async fn insert_journal_entry_tx(
    tx: &mut Transaction<'_, Sqlite>,
    entry_no: &str,
    entry_date: &str,
    description: Option<&str>,
    ref_type: Option<&str>,
    ref_id: Option<i64>,
    created_by: i64,
) -> Result<i64, AppError> {
    let result = sqlx::query("INSERT INTO journal_entries (entry_no, entry_date, description, ref_type, ref_id, created_by) VALUES (?, ?, ?, ?, ?, ?)")
        .bind(entry_no).bind(entry_date).bind(description).bind(ref_type).bind(ref_id).bind(created_by)
        .execute(&mut **tx).await?;
    Ok(result.last_insert_rowid())
}

pub async fn insert_journal_entry(
    pool: &SqlitePool,
    entry_no: &str,
    entry_date: &str,
    description: Option<&str>,
    ref_type: Option<&str>,
    ref_id: Option<i64>,
    created_by: i64,
) -> Result<i64, AppError> {
    let mut tx = pool.begin().await?;
    let id = insert_journal_entry_tx(
        &mut tx,
        entry_no,
        entry_date,
        description,
        ref_type,
        ref_id,
        created_by,
    )
    .await?;
    tx.commit().await?;
    Ok(id)
}

pub async fn insert_journal_entry_line_tx(
    tx: &mut Transaction<'_, Sqlite>,
    entry_id: i64,
    account_id: i64,
    debit: &str,
    credit: &str,
    description: Option<&str>,
) -> Result<(), AppError> {
    sqlx::query("INSERT INTO journal_entry_lines (entry_id, account_id, debit, credit, description) VALUES (?, ?, ?, ?, ?)")
        .bind(entry_id).bind(account_id).bind(debit).bind(credit).bind(description).execute(&mut **tx).await?;
    Ok(())
}

pub async fn insert_journal_entry_line(
    pool: &SqlitePool,
    entry_id: i64,
    account_id: i64,
    debit: &str,
    credit: &str,
    description: Option<&str>,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO journal_entry_lines (entry_id, account_id, debit, credit, description) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(entry_id)
    .bind(account_id)
    .bind(debit)
    .bind(credit)
    .bind(description)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn find_journal_entry_by_id(
    pool: &SqlitePool,
    id: i64,
) -> Result<Option<JournalEntryRow>, AppError> {
    Ok(sqlx::query_as::<_, JournalEntryRow>(&format!(
        "SELECT {JOURNAL_COLUMNS} FROM journal_entries WHERE id = ?"
    ))
    .bind(id)
    .fetch_optional(pool)
    .await?)
}

pub async fn find_lines_for_entry(
    pool: &SqlitePool,
    entry_id: i64,
) -> Result<Vec<JournalEntryLineRow>, AppError> {
    Ok(sqlx::query_as::<_, JournalEntryLineRow>("SELECT id, entry_id, account_id, debit, credit, description, created_at FROM journal_entry_lines WHERE entry_id = ? ORDER BY id").bind(entry_id).fetch_all(pool).await?)
}

pub async fn list_journal_entries(
    pool: &SqlitePool,
    page: i64,
    page_size: i64,
) -> Result<(Vec<JournalEntryRow>, i64), AppError> {
    let total = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM journal_entries")
        .fetch_one(pool)
        .await?;
    let rows = sqlx::query_as::<_, JournalEntryRow>(&format!(
        "SELECT {JOURNAL_COLUMNS} FROM journal_entries ORDER BY id DESC LIMIT ? OFFSET ?"
    ))
    .bind(page_size)
    .bind((page - 1).max(0) * page_size)
    .fetch_all(pool)
    .await?;
    Ok((rows, total))
}

pub async fn update_journal_entry_status(
    pool: &SqlitePool,
    id: i64,
    status: &str,
) -> Result<(), AppError> {
    let result = sqlx::query("UPDATE journal_entries SET status = ?, updated_at = datetime('now') WHERE id = ? AND status = 'draft'")
        .bind(status).bind(id).execute(pool).await?;
    if result.rows_affected() == 0 {
        return Err(AppError::new(
            ErrorCode::JournalNotFound,
            "日记账未找到或状态不可变更",
        ));
    }
    Ok(())
}

pub async fn insert_invoice(
    pool: &SqlitePool,
    dto: &CreateInvoiceRequest,
    amount: &str,
) -> Result<InvoiceRow, AppError> {
    let result = sqlx::query("INSERT INTO invoices (invoice_no, invoice_date, party_type, party_id, amount, ref_type, ref_id) VALUES (?, ?, ?, ?, ?, ?, ?)")
        .bind(&dto.invoice_no).bind(&dto.invoice_date).bind(&dto.party_type).bind(dto.party_id).bind(amount).bind(dto.ref_type.as_deref()).bind(dto.ref_id).execute(pool).await?;
    find_invoice_by_id(pool, result.last_insert_rowid())
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::Internal, "发票创建后读取失败"))
}

pub async fn find_invoice_by_id(
    pool: &SqlitePool,
    id: i64,
) -> Result<Option<InvoiceRow>, AppError> {
    Ok(sqlx::query_as::<_, InvoiceRow>(&format!(
        "SELECT {INVOICE_COLUMNS} FROM invoices WHERE id = ?"
    ))
    .bind(id)
    .fetch_optional(pool)
    .await?)
}

pub async fn list_invoices(
    pool: &SqlitePool,
    page: i64,
    page_size: i64,
) -> Result<(Vec<InvoiceRow>, i64), AppError> {
    let total = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM invoices")
        .fetch_one(pool)
        .await?;
    let rows = sqlx::query_as::<_, InvoiceRow>(&format!(
        "SELECT {INVOICE_COLUMNS} FROM invoices ORDER BY id DESC LIMIT ? OFFSET ?"
    ))
    .bind(page_size)
    .bind((page - 1).max(0) * page_size)
    .fetch_all(pool)
    .await?;
    Ok((rows, total))
}

pub async fn update_invoice_status(
    pool: &SqlitePool,
    id: i64,
    status: &str,
) -> Result<(), AppError> {
    let result =
        sqlx::query("UPDATE invoices SET status = ?, updated_at = datetime('now') WHERE id = ?")
            .bind(status)
            .bind(id)
            .execute(pool)
            .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::new(ErrorCode::InvoiceNotFound, "发票未找到"));
    }
    Ok(())
}

pub async fn insert_payment_tx(
    tx: &mut Transaction<'_, Sqlite>,
    dto: &CreatePaymentRequest,
    amount: &str,
    user_id: i64,
) -> Result<i64, AppError> {
    let result = sqlx::query("INSERT INTO payments (payment_no, payment_date, supplier_id, amount, invoice_id, method, notes, created_by) VALUES (?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(&dto.payment_no).bind(&dto.payment_date).bind(dto.supplier_id).bind(amount).bind(dto.invoice_id).bind(dto.method.as_deref()).bind(dto.notes.as_deref()).bind(user_id).execute(&mut **tx).await?;
    Ok(result.last_insert_rowid())
}

pub async fn insert_payment(
    pool: &SqlitePool,
    dto: &CreatePaymentRequest,
    amount: &str,
    user_id: i64,
) -> Result<PaymentRow, AppError> {
    let mut tx = pool.begin().await?;
    let id = insert_payment_tx(&mut tx, dto, amount, user_id).await?;
    tx.commit().await?;
    find_payment_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::Internal, "付款创建后读取失败"))
}

pub async fn find_payment_by_id(
    pool: &SqlitePool,
    id: i64,
) -> Result<Option<PaymentRow>, AppError> {
    Ok(sqlx::query_as::<_, PaymentRow>(&format!(
        "SELECT {PAYMENT_COLUMNS} FROM payments WHERE id = ?"
    ))
    .bind(id)
    .fetch_optional(pool)
    .await?)
}

pub async fn list_payments(
    pool: &SqlitePool,
    page: i64,
    page_size: i64,
) -> Result<(Vec<PaymentRow>, i64), AppError> {
    let total = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM payments")
        .fetch_one(pool)
        .await?;
    let rows = sqlx::query_as::<_, PaymentRow>(&format!(
        "SELECT {PAYMENT_COLUMNS} FROM payments ORDER BY id DESC LIMIT ? OFFSET ?"
    ))
    .bind(page_size)
    .bind((page - 1).max(0) * page_size)
    .fetch_all(pool)
    .await?;
    Ok((rows, total))
}

pub async fn posted_lines(pool: &SqlitePool) -> Result<Vec<PostedLineRow>, AppError> {
    Ok(sqlx::query_as::<_, PostedLineRow>("SELECT l.account_id, a.code AS account_code, a.name AS account_name, l.debit, l.credit FROM journal_entry_lines l JOIN journal_entries j ON j.id = l.entry_id JOIN accounts a ON a.id = l.account_id WHERE j.status = 'posted'").fetch_all(pool).await?)
}

pub async fn paid_amount_for_invoice(
    tx: &mut Transaction<'_, Sqlite>,
    invoice_id: i64,
) -> Result<Vec<String>, AppError> {
    Ok(
        sqlx::query_scalar::<_, String>("SELECT amount FROM payments WHERE invoice_id = ?")
            .bind(invoice_id)
            .fetch_all(&mut **tx)
            .await?,
    )
}

pub async fn update_invoice_status_tx(
    tx: &mut Transaction<'_, Sqlite>,
    id: i64,
    status: &str,
) -> Result<(), AppError> {
    let result =
        sqlx::query("UPDATE invoices SET status = ?, updated_at = datetime('now') WHERE id = ?")
            .bind(status)
            .bind(id)
            .execute(&mut **tx)
            .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::new(ErrorCode::InvoiceNotFound, "发票未找到"));
    }
    Ok(())
}

pub async fn find_invoice_by_id_tx(
    tx: &mut Transaction<'_, Sqlite>,
    id: i64,
) -> Result<Option<InvoiceRow>, AppError> {
    Ok(sqlx::query_as::<_, InvoiceRow>(&format!(
        "SELECT {INVOICE_COLUMNS} FROM invoices WHERE id = ?"
    ))
    .bind(id)
    .fetch_optional(&mut **tx)
    .await?)
}
