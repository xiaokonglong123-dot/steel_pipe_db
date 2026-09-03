//! finance_repo.payment — 付款 payments 与发票已付额核算

use sqlx::Transaction;
use sqlx::Sqlite;
use sqlx::SqlitePool;
use crate::services::finance_service::CreatePaymentRequest;
use crate::error::{AppError, ErrorCode};

const PAYMENT_COLUMNS: &str = "id, payment_no, payment_date, supplier_id, amount, invoice_id, method, notes, created_by, created_at";


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
