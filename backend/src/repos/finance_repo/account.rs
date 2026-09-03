//! finance_repo.account — 会计科目表 accounts

use sqlx::SqlitePool;
use crate::services::finance_service::CreateAccountRequest;
use crate::error::{AppError, ErrorCode};

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct AccountRow {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub parent_id: Option<i64>,
    pub parent_name: Option<String>,
    pub account_type: String,
    pub is_active: i64,
    pub created_at: String,
}


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
    Ok(sqlx::query_as::<_, AccountRow>(
        "SELECT a.id, a.code, a.name, a.parent_id, p.name AS parent_name,
                a.account_type, a.is_active, a.created_at
         FROM accounts a
         LEFT JOIN accounts p ON p.id = a.parent_id
         WHERE a.id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?)
}


pub async fn find_account_by_code(
    pool: &SqlitePool,
    code: &str,
) -> Result<Option<AccountRow>, AppError> {
    Ok(sqlx::query_as::<_, AccountRow>(
        "SELECT a.id, a.code, a.name, a.parent_id, p.name AS parent_name,
                a.account_type, a.is_active, a.created_at
         FROM accounts a
         LEFT JOIN accounts p ON p.id = a.parent_id
         WHERE a.code = ?",
    )
    .bind(code)
    .fetch_optional(pool)
    .await?)
}


pub async fn list_accounts(
    pool: &SqlitePool,
    account_type: Option<&str>,
    active_only: bool,
) -> Result<Vec<AccountRow>, AppError> {
    let mut sql = String::from(
        "SELECT a.id, a.code, a.name, a.parent_id, p.name AS parent_name,
                a.account_type, a.is_active, a.created_at
         FROM accounts a
         LEFT JOIN accounts p ON p.id = a.parent_id
         WHERE 1=1",
    );
    if account_type.is_some() {
        sql.push_str(" AND a.account_type = ?");
    }
    if active_only {
        sql.push_str(" AND a.is_active = 1");
    }
    sql.push_str(" ORDER BY a.code");
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
