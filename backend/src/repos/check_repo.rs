use sqlx::{Executor, SqlitePool};

use crate::error::{AppError, ErrorCode};

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct CheckSessionRow {
    pub id: i64,
    pub session_no: String,
    pub location_id: Option<i64>,
    pub scope: String,
    pub status: String,
    pub created_by: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct CheckDetailRow {
    pub id: i64,
    pub session_id: i64,
    pub item_id: i64,
    pub location_id: Option<i64>,
    pub system_qty: f64,
    pub actual_qty: Option<f64>,
    pub diff_qty: Option<f64>,
}

pub async fn insert_session(
    pool: &SqlitePool,
    session_no: &str,
    location_id: i64,
    _scope: &str,
    user_id: i64,
) -> Result<i64, AppError> {
    insert_session_tx(pool, session_no, location_id, user_id).await
}

pub async fn insert_session_tx<'e, E>(
    executor: E,
    session_no: &str,
    location_id: i64,
    user_id: i64,
) -> Result<i64, AppError>
where
    E: Executor<'e, Database = sqlx::Sqlite>,
{
    let result = sqlx::query(
        "INSERT INTO check_records (record_no, location_id, status, created_by)
         VALUES (?, ?, 'draft', ?)",
    )
    .bind(session_no)
    .bind(location_id)
    .bind(user_id)
    .execute(executor)
    .await?;
    Ok(result.last_insert_rowid())
}

pub async fn insert_detail<'e, E>(
    executor: E,
    session_id: i64,
    item_id: i64,
    system_qty: f64,
) -> Result<i64, AppError>
where
    E: Executor<'e, Database = sqlx::Sqlite>,
{
    let result = sqlx::query(
        "INSERT INTO check_items (record_id, item_id, system_qty)
         VALUES (?, ?, ?)",
    )
    .bind(session_id)
    .bind(item_id)
    .bind(system_qty)
    .execute(executor)
    .await?;
    Ok(result.last_insert_rowid())
}

pub async fn list_sessions(
    pool: &SqlitePool,
    page: i64,
    page_size: i64,
) -> Result<(Vec<CheckSessionRow>, i64), AppError> {
    let total: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM check_records WHERE deleted_at IS NULL")
            .fetch_one(pool)
            .await?;
    let rows = sqlx::query_as::<_, CheckSessionRow>(
        "SELECT id, record_no AS session_no, location_id, 'all' AS scope, status,
                created_by, created_at, updated_at
         FROM check_records
         WHERE deleted_at IS NULL
         ORDER BY id DESC LIMIT ? OFFSET ?",
    )
    .bind(page_size)
    .bind((page - 1).max(0) * page_size)
    .fetch_all(pool)
    .await?;
    Ok((rows, total))
}

pub async fn find_session_by_id(
    pool: &SqlitePool,
    id: i64,
) -> Result<Option<CheckSessionRow>, AppError> {
    let row = sqlx::query_as::<_, CheckSessionRow>(
        "SELECT id, record_no AS session_no, location_id, 'all' AS scope, status,
                created_by, created_at, updated_at
         FROM check_records WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn list_details_for_session(
    pool: &SqlitePool,
    session_id: i64,
) -> Result<Vec<CheckDetailRow>, AppError> {
    let rows = sqlx::query_as::<_, CheckDetailRow>(
        "SELECT ci.id, ci.record_id AS session_id, ci.item_id, cr.location_id,
                COALESCE(ci.system_qty, 0.0) AS system_qty, ci.actual_qty,
                ci.diff AS diff_qty
         FROM check_items ci
         JOIN check_records cr ON cr.id = ci.record_id
         WHERE ci.record_id = ?
         ORDER BY ci.id",
    )
    .bind(session_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn find_detail_by_id(
    pool: &SqlitePool,
    detail_id: i64,
) -> Result<Option<CheckDetailRow>, AppError> {
    let row = sqlx::query_as::<_, CheckDetailRow>(
        "SELECT ci.id, ci.record_id AS session_id, ci.item_id, cr.location_id,
                COALESCE(ci.system_qty, 0.0) AS system_qty, ci.actual_qty,
                ci.diff AS diff_qty
         FROM check_items ci
         JOIN check_records cr ON cr.id = ci.record_id
         WHERE ci.id = ?",
    )
    .bind(detail_id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn update_actual_qty(
    pool: &SqlitePool,
    detail_id: i64,
    actual_qty: f64,
) -> Result<(), AppError> {
    let result = sqlx::query(
        "UPDATE check_items
         SET actual_qty = ?, diff = ? - COALESCE(system_qty, 0.0)
         WHERE id = ?",
    )
    .bind(actual_qty)
    .bind(actual_qty)
    .bind(detail_id)
    .execute(pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::new(ErrorCode::CheckNotFound, "盘点明细未找到"));
    }
    Ok(())
}

pub async fn update_session_status(
    pool: &SqlitePool,
    session_id: i64,
    status: &str,
) -> Result<(), AppError> {
    let result = sqlx::query(
        "UPDATE check_records SET status = ?, updated_at = datetime('now')
         WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(status)
    .bind(session_id)
    .execute(pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::new(ErrorCode::CheckNotFound, "盘点单未找到"));
    }
    Ok(())
}

pub async fn update_session_status_tx<'e, E>(
    executor: E,
    session_id: i64,
    status: &str,
) -> Result<(), AppError>
where
    E: Executor<'e, Database = sqlx::Sqlite>,
{
    let result = sqlx::query(
        "UPDATE check_records SET status = ?, updated_at = datetime('now')
         WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(status)
    .bind(session_id)
    .execute(executor)
    .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::new(ErrorCode::CheckNotFound, "盘点单未找到"));
    }
    Ok(())
}

pub async fn system_snapshot_for_location(
    pool: &SqlitePool,
    location_id: i64,
) -> Result<Vec<(i64, i64, f64)>, AppError> {
    system_snapshot_for_location_tx(pool, location_id).await
}

pub async fn system_snapshot_for_location_tx<'e, E>(
    executor: E,
    location_id: i64,
) -> Result<Vec<(i64, i64, f64)>, AppError>
where
    E: Executor<'e, Database = sqlx::Sqlite>,
{
    let rows = sqlx::query_as::<_, (i64, i64, f64)>(
        "SELECT item_id, location_id, quantity FROM inventory
         WHERE location_id = ? ORDER BY item_id",
    )
    .bind(location_id)
    .fetch_all(executor)
    .await?;
    Ok(rows)
}
