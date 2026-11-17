//! 单据仓储（transactions + transaction_items）。

use sqlx::{FromRow, SqliteConnection};
use steelpipe_domain::TransactionType;

use crate::{DbPool, InfraError};

#[derive(Debug, Clone, FromRow)]
pub struct TransactionRow {
    pub id: i64,
    #[sqlx(rename = "type")]
    pub kind: String,
    pub operator_id: i64,
    pub reference_no: Option<String>,
    pub remark: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct ItemRow {
    pub id: i64,
    pub transaction_id: i64,
    pub pipe_id: String,
    pub source_location: Option<String>,
    pub target_location: Option<String>,
}

pub async fn insert(
    conn: &mut SqliteConnection,
    kind: TransactionType,
    operator_id: i64,
    reference_no: Option<&str>,
    remark: Option<&str>,
    created_at: &str,
) -> Result<i64, InfraError> {
    sqlx::query_scalar(
        "INSERT INTO transactions (type, operator_id, reference_no, remark, created_at)
         VALUES (?, ?, ?, ?, ?) RETURNING id",
    )
    .bind(kind.as_str())
    .bind(operator_id)
    .bind(reference_no)
    .bind(remark)
    .bind(created_at)
    .fetch_one(conn)
    .await
    .map_err(InfraError::Connect)
}

pub async fn insert_item(
    conn: &mut SqliteConnection,
    transaction_id: i64,
    pipe_id: &str,
    source_location: Option<&str>,
    target_location: Option<&str>,
) -> Result<(), InfraError> {
    sqlx::query(
        "INSERT INTO transaction_items (transaction_id, pipe_id, source_location, target_location)
         VALUES (?, ?, ?, ?)",
    )
    .bind(transaction_id)
    .bind(pipe_id)
    .bind(source_location)
    .bind(target_location)
    .execute(conn)
    .await
    .map(|_| ())
    .map_err(InfraError::Connect)
}

pub async fn find(pool: &DbPool, id: i64) -> Result<Option<TransactionRow>, InfraError> {
    sqlx::query_as(
        "SELECT id, type, operator_id, reference_no, remark, created_at FROM transactions WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(InfraError::Connect)
}

pub async fn list(
    pool: &DbPool,
    limit: i64,
    offset: i64,
) -> Result<Vec<TransactionRow>, InfraError> {
    sqlx::query_as(
        "SELECT id, type, operator_id, reference_no, remark, created_at
         FROM transactions ORDER BY id DESC LIMIT ? OFFSET ?",
    )
    .bind(limit.clamp(1, 500))
    .bind(offset.max(0))
    .fetch_all(pool)
    .await
    .map_err(InfraError::Connect)
}

pub async fn items(pool: &DbPool, transaction_id: i64) -> Result<Vec<ItemRow>, InfraError> {
    sqlx::query_as(
        "SELECT id, transaction_id, pipe_id, source_location, target_location
         FROM transaction_items WHERE transaction_id = ? ORDER BY id",
    )
    .bind(transaction_id)
    .fetch_all(pool)
    .await
    .map_err(InfraError::Connect)
}
