//! finance_repo.invoice — 发票 invoices

use sqlx::Transaction;
use sqlx::Sqlite;
use sqlx::SqlitePool;
use crate::services::finance_service::CreateInvoiceRequest;
use crate::error::{AppError, ErrorCode};

const INVOICE_COLUMNS: &str = "id, invoice_no, invoice_date, party_type, party_id, amount, ref_type, ref_id, status, created_at, updated_at";


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
