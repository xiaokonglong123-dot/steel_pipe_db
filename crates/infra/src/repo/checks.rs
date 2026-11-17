//! 盘点单仓储（`check_sessions` / `check_items`）。

use sqlx::{FromRow, SqliteConnection};

use crate::{DbPool, InfraError};

#[derive(Debug, Clone, FromRow)]
pub struct CheckSessionRow {
    pub id: i64,
    pub location: String,
    pub status: String,
    pub created_by: i64,
    pub confirmed_by: Option<i64>,
    pub created_at: String,
    pub confirmed_at: Option<String>,
    pub remark: Option<String>,
}

#[derive(Debug, Clone, FromRow)]
pub struct CheckItemRow {
    pub id: i64,
    pub check_id: i64,
    pub pipe_id: String,
    pub result: String,
    pub actual_location: Option<String>,
    pub surplus_data: Option<String>,
    pub created_at: String,
}

const SESSION_COLUMNS: &str =
    "id, location, status, created_by, confirmed_by, created_at, confirmed_at, remark";

pub async fn insert_session(
    conn: &mut SqliteConnection,
    location: &str,
    created_by: i64,
    remark: Option<&str>,
    now: &str,
) -> Result<i64, InfraError> {
    sqlx::query_scalar(
        "INSERT INTO check_sessions (location, status, created_by, created_at, remark)
         VALUES (?, 'open', ?, ?, ?) RETURNING id",
    )
    .bind(location)
    .bind(created_by)
    .bind(now)
    .bind(remark)
    .fetch_one(conn)
    .await
    .map_err(InfraError::Connect)
}

pub async fn find_session(pool: &DbPool, id: i64) -> Result<Option<CheckSessionRow>, InfraError> {
    let sql = format!("SELECT {SESSION_COLUMNS} FROM check_sessions WHERE id = ?");
    sqlx::query_as::<_, CheckSessionRow>(&sql)
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(InfraError::Connect)
}

pub async fn list_sessions(
    pool: &DbPool,
    limit: i64,
    offset: i64,
) -> Result<Vec<CheckSessionRow>, InfraError> {
    let sql =
        format!("SELECT {SESSION_COLUMNS} FROM check_sessions ORDER BY id DESC LIMIT ? OFFSET ?");
    sqlx::query_as::<_, CheckSessionRow>(&sql)
        .bind(limit.clamp(1, 200))
        .bind(offset.max(0))
        .fetch_all(pool)
        .await
        .map_err(InfraError::Connect)
}

pub async fn items(pool: &DbPool, check_id: i64) -> Result<Vec<CheckItemRow>, InfraError> {
    sqlx::query_as(
        "SELECT id, check_id, pipe_id, result, actual_location, surplus_data, created_at
         FROM check_items WHERE check_id = ? ORDER BY id",
    )
    .bind(check_id)
    .fetch_all(pool)
    .await
    .map_err(InfraError::Connect)
}

/// 同一盘点单内同一根管子只保留一条结论（重新核对即覆盖）。
pub async fn upsert_item(
    conn: &mut SqliteConnection,
    check_id: i64,
    pipe_id: &str,
    result: &str,
    actual_location: Option<&str>,
    surplus_data: Option<&str>,
    now: &str,
) -> Result<i64, InfraError> {
    let existing: Option<i64> =
        sqlx::query_scalar("SELECT id FROM check_items WHERE check_id = ? AND pipe_id = ?")
            .bind(check_id)
            .bind(pipe_id)
            .fetch_optional(&mut *conn)
            .await
            .map_err(InfraError::Connect)?;

    match existing {
        Some(id) => {
            sqlx::query(
                "UPDATE check_items SET result = ?, actual_location = ?, surplus_data = ? WHERE id = ?",
            )
            .bind(result)
            .bind(actual_location)
            .bind(surplus_data)
            .bind(id)
            .execute(conn)
            .await
            .map_err(InfraError::Connect)?;
            Ok(id)
        }
        None => sqlx::query_scalar(
            "INSERT INTO check_items (check_id, pipe_id, result, actual_location, surplus_data, created_at)
             VALUES (?, ?, ?, ?, ?, ?) RETURNING id",
        )
        .bind(check_id)
        .bind(pipe_id)
        .bind(result)
        .bind(actual_location)
        .bind(surplus_data)
        .bind(now)
        .fetch_one(conn)
        .await
        .map_err(InfraError::Connect),
    }
}

pub async fn delete_item(
    conn: &mut SqliteConnection,
    check_id: i64,
    item_id: i64,
) -> Result<(), InfraError> {
    let rows = sqlx::query("DELETE FROM check_items WHERE id = ? AND check_id = ?")
        .bind(item_id)
        .bind(check_id)
        .execute(conn)
        .await
        .map_err(InfraError::Connect)?
        .rows_affected();
    if rows == 0 {
        return Err(InfraError::NotFound("盘点明细"));
    }
    Ok(())
}

/// 状态推进：只有 `open` 能被确认/取消（条件 UPDATE，0 行 = 已被处理）。
pub async fn close_session(
    conn: &mut SqliteConnection,
    check_id: i64,
    status: &str,
    actor_id: i64,
    now: &str,
) -> Result<(), InfraError> {
    let rows = sqlx::query(
        "UPDATE check_sessions SET status = ?, confirmed_by = ?, confirmed_at = ?
         WHERE id = ? AND status = 'open'",
    )
    .bind(status)
    .bind(actor_id)
    .bind(now)
    .bind(check_id)
    .execute(conn)
    .await
    .map_err(InfraError::Connect)?
    .rows_affected();
    if rows == 0 {
        return Err(InfraError::Conflict("盘点单已被处理"));
    }
    Ok(())
}
