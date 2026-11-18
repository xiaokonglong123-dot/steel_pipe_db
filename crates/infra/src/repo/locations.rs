//! 库位仓储。`used_count` 是反规范化派生列（设计 §12.2）：只做原子增减，
//! 绝不全表 `COUNT(*)`。

use sqlx::{FromRow, Sqlite, SqliteConnection};

use crate::{DbPool, InfraError};

#[derive(Debug, Clone, FromRow)]
pub struct LocationRow {
    pub code: String,
    pub name: Option<String>,
    pub used_count: i64,
    pub capacity: Option<i64>,
    pub is_active: bool,
}

pub async fn list(pool: &DbPool) -> Result<Vec<LocationRow>, InfraError> {
    sqlx::query_as(
        "SELECT code, name, used_count, capacity, is_active FROM locations ORDER BY code",
    )
    .fetch_all(pool)
    .await
    .map_err(InfraError::Connect)
}

pub async fn find<'e, E>(executor: E, code: &str) -> Result<Option<LocationRow>, InfraError>
where
    E: sqlx::Executor<'e, Database = Sqlite>,
{
    sqlx::query_as(
        "SELECT code, name, used_count, capacity, is_active FROM locations WHERE code = ?",
    )
    .bind(code)
    .fetch_optional(executor)
    .await
    .map_err(InfraError::Connect)
}

pub async fn insert(
    conn: &mut SqliteConnection,
    code: &str,
    name: Option<&str>,
    capacity: Option<i64>,
) -> Result<(), InfraError> {
    sqlx::query("INSERT INTO locations (code, name, used_count, capacity, is_active) VALUES (?, ?, 0, ?, 1)")
        .bind(code)
        .bind(name)
        .bind(capacity)
        .execute(conn)
        .await
        .map(|_| ())
        .map_err(|e| InfraError::from_unique(e, "库位编号"))
}

pub async fn rename(
    conn: &mut SqliteConnection,
    code: &str,
    name: Option<&str>,
    capacity: Option<i64>,
) -> Result<(), InfraError> {
    let rows = sqlx::query("UPDATE locations SET name = ?, capacity = ? WHERE code = ?")
        .bind(name)
        .bind(capacity)
        .bind(code)
        .execute(conn)
        .await
        .map_err(InfraError::Connect)?
        .rows_affected();
    if rows == 0 {
        return Err(InfraError::NotFound("库位"));
    }
    Ok(())
}

pub async fn set_active(
    conn: &mut SqliteConnection,
    code: &str,
    active: bool,
) -> Result<(), InfraError> {
    let rows = sqlx::query("UPDATE locations SET is_active = ? WHERE code = ?")
        .bind(i64::from(active))
        .bind(code)
        .execute(conn)
        .await
        .map_err(InfraError::Connect)?
        .rows_affected();
    if rows == 0 {
        return Err(InfraError::NotFound("库位"));
    }
    Ok(())
}

/// 原子增减 `used_count`；减到负数会被 CHECK 约束挡住（转成 Conflict）。
pub async fn adjust_used_count(
    conn: &mut SqliteConnection,
    code: &str,
    delta: i64,
) -> Result<i64, InfraError> {
    adjust_used_count_checked(conn, code, delta, false).await
}

/// 增加库存时可要求目标库位启用；原子条件避免校验后再被停用。
pub async fn adjust_used_count_checked(
    conn: &mut SqliteConnection,
    code: &str,
    delta: i64,
    require_active: bool,
) -> Result<i64, InfraError> {
    let used: Option<i64> = sqlx::query_scalar(
        "UPDATE locations SET used_count = used_count + ?
         WHERE code = ? AND (? = 0 OR is_active = 1) RETURNING used_count",
    )
    .bind(delta)
    .bind(code)
    .bind(i64::from(require_active))
    .fetch_optional(conn)
    .await
    .map_err(|e| {
        if e.as_database_error()
            .is_some_and(|db| db.is_unique_violation() || db.is_check_violation())
        {
            InfraError::Conflict("库位 used_count 不能为负")
        } else {
            InfraError::Connect(e)
        }
    })?;
    used.ok_or(InfraError::NotFound("库位"))
}

/// 对账：返回 used_count 与实际在库管子数不一致的库位（设计 §12.2.1）。
pub async fn drift(pool: &DbPool) -> Result<Vec<(String, i64, i64)>, InfraError> {
    sqlx::query_as(
        "SELECT l.code, l.used_count, count(t.id) AS actual
         FROM locations l
         LEFT JOIN tubes t
                ON t.location = l.code AND t.deleted_at IS NULL AND t.status = 'in_stock'
         GROUP BY l.code
         HAVING l.used_count != count(t.id)
         ORDER BY l.code",
    )
    .fetch_all(pool)
    .await
    .map_err(InfraError::Connect)
}

/// 按实际在库数量重算（对账脚本用）。
pub async fn refresh_used_count(
    conn: &mut SqliteConnection,
    code: &str,
) -> Result<i64, InfraError> {
    let actual: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM tubes
         WHERE location = ? AND deleted_at IS NULL AND status = 'in_stock'",
    )
    .bind(code)
    .fetch_one(&mut *conn)
    .await
    .map_err(InfraError::Connect)?;
    sqlx::query("UPDATE locations SET used_count = ? WHERE code = ?")
        .bind(actual)
        .bind(code)
        .execute(conn)
        .await
        .map_err(InfraError::Connect)?;
    Ok(actual)
}
