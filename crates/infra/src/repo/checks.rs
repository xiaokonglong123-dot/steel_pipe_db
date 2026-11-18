//! 盘点单仓储（`check_sessions` / `check_items`）。

use sqlx::{FromRow, Sqlite, SqliteConnection};

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

pub async fn find_session<'e, E>(
    executor: E,
    id: i64,
) -> Result<Option<CheckSessionRow>, InfraError>
where
    E: sqlx::Executor<'e, Database = Sqlite>,
{
    let sql = format!("SELECT {SESSION_COLUMNS} FROM check_sessions WHERE id = ?");
    sqlx::query_as::<_, CheckSessionRow>(&sql)
        .bind(id)
        .fetch_optional(executor)
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

pub async fn items<'e, E>(executor: E, check_id: i64) -> Result<Vec<CheckItemRow>, InfraError>
where
    E: sqlx::Executor<'e, Database = Sqlite>,
{
    sqlx::query_as(
        "SELECT id, check_id, pipe_id, result, actual_location, surplus_data, created_at
         FROM check_items WHERE check_id = ? ORDER BY id",
    )
    .bind(check_id)
    .fetch_all(executor)
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
            let rows = sqlx::query(
                "UPDATE check_items SET result = ?, actual_location = ?, surplus_data = ?
                 WHERE id = ? AND check_id = ?
                   AND EXISTS (SELECT 1 FROM check_sessions WHERE id = ? AND status = 'open')",
            )
            .bind(result)
            .bind(actual_location)
            .bind(surplus_data)
            .bind(id)
            .bind(check_id)
            .bind(check_id)
            .execute(conn)
            .await
            .map_err(InfraError::Connect)?
            .rows_affected();
            if rows != 1 {
                return Err(InfraError::NotOpen);
            }
            Ok(id)
        }
        None => {
            let inserted = sqlx::query_scalar::<_, i64>(
                "INSERT INTO check_items (check_id, pipe_id, result, actual_location, surplus_data, created_at)
                 SELECT id, ?, ?, ?, ?, ? FROM check_sessions WHERE id = ? AND status = 'open'
                 RETURNING id",
            )
            .bind(pipe_id)
            .bind(result)
            .bind(actual_location)
            .bind(surplus_data)
            .bind(now)
            .bind(check_id)
            .fetch_optional(conn)
            .await
            .map_err(InfraError::Connect)?;
            inserted.ok_or(InfraError::NotOpen)
        }
    }
}

pub async fn delete_item(
    conn: &mut SqliteConnection,
    check_id: i64,
    item_id: i64,
) -> Result<(), InfraError> {
    let rows = sqlx::query(
        "DELETE FROM check_items WHERE id = ? AND check_id = ?
         AND EXISTS (SELECT 1 FROM check_sessions WHERE id = ? AND status = 'open')",
    )
    .bind(item_id)
    .bind(check_id)
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

/// 在写事务内区分不存在与已关闭，避免将已处理单据误报成 404/500。
pub async fn require_open(
    conn: &mut SqliteConnection,
    check_id: i64,
) -> Result<CheckSessionRow, InfraError> {
    let session = find_session(conn, check_id)
        .await?
        .ok_or(InfraError::NotFound("盘点单"))?;
    if session.status != "open" {
        return Err(InfraError::NotOpen);
    }
    Ok(session)
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
        return Err(InfraError::NotOpen);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{test_support::temp_pool, uow::Uow};

    #[tokio::test]
    async fn closed_session_rejects_new_items_and_keeps_existing_results() {
        let (pool, path) = temp_pool("check-closed-guard").await;
        let mut uow = Uow::begin(&pool).await.unwrap();
        let check_id = insert_session(uow.conn(), "A1", 1, None, "2026-01-01T00:00:00Z")
            .await
            .unwrap();
        upsert_item(
            uow.conn(),
            check_id,
            "P-1",
            "missing",
            None,
            None,
            "2026-01-01T00:00:00Z",
        )
        .await
        .unwrap();
        close_session(uow.conn(), check_id, "cancelled", 1, "2026-01-01T00:00:01Z")
            .await
            .unwrap();

        for pipe_id in ["P-1", "P-2"] {
            let error = upsert_item(
                uow.conn(),
                check_id,
                pipe_id,
                "ok",
                None,
                None,
                "2026-01-01T00:00:02Z",
            )
            .await
            .unwrap_err();
            assert!(matches!(error, InfraError::NotOpen));
        }
        let stored = items(uow.conn(), check_id).await.unwrap();
        assert_eq!(stored.len(), 1);
        assert_eq!(stored[0].result, "missing");
        drop(uow);
        pool.close().await;
        let _ = std::fs::remove_file(path);
    }
}
