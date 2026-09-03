//! finance_repo.journal — 日记账 journal_entries / journal_lines / 已过账行

use sqlx::Transaction;
use sqlx::Sqlite;
use sqlx::SqlitePool;
use crate::error::{AppError, ErrorCode};


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


#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PostedLineRow {
    pub account_id: i64,
    pub account_code: String,
    pub account_name: String,
    pub debit: String,
    pub credit: String,
}

const JOURNAL_COLUMNS: &str = "id, entry_no, entry_date, description, status, ref_type, ref_id, created_by, created_at, updated_at";


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


pub async fn posted_lines(pool: &SqlitePool) -> Result<Vec<PostedLineRow>, AppError> {
    Ok(sqlx::query_as::<_, PostedLineRow>("SELECT l.account_id, a.code AS account_code, a.name AS account_name, l.debit, l.credit FROM journal_entry_lines l JOIN journal_entries j ON j.id = l.entry_id JOIN accounts a ON a.id = l.account_id WHERE j.status = 'posted'").fetch_all(pool).await?)
}
