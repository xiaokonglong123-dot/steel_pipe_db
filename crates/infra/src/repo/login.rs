//! 登录限流：`login_attempts` 表（A23 落库，重启不清零）。

use sqlx::{Sqlite, SqliteConnection};

use crate::{DbPool, InfraError};

pub const SCOPE_IP: &str = "ip";
pub const SCOPE_USERNAME: &str = "username";

pub async fn record(
    conn: &mut SqliteConnection,
    scope: &str,
    subject: &str,
    at: &str,
    succeeded: bool,
) -> Result<(), InfraError> {
    sqlx::query(
        "INSERT INTO login_attempts (scope, subject, attempted_at, succeeded) VALUES (?, ?, ?, ?)",
    )
    .bind(scope)
    .bind(subject)
    .bind(at)
    .bind(i64::from(succeeded))
    .execute(conn)
    .await
    .map_err(InfraError::Connect)?;
    Ok(())
}

/// 统计窗口内的失败次数。
pub async fn failures_since<'e, E>(
    executor: E,
    scope: &str,
    subject: &str,
    since: &str,
) -> Result<i64, InfraError>
where
    E: sqlx::Executor<'e, Database = Sqlite>,
{
    sqlx::query_scalar(
        "SELECT count(*) FROM login_attempts
         WHERE scope = ? AND subject = ? AND succeeded = 0 AND attempted_at >= ?",
    )
    .bind(scope)
    .bind(subject)
    .bind(since)
    .fetch_one(executor)
    .await
    .map_err(InfraError::Connect)
}

/// 窗口内最后一次失败的时间；用于推导锁定截止时刻。
pub async fn last_failure_since<'e, E>(
    executor: E,
    scope: &str,
    subject: &str,
    since: &str,
) -> Result<Option<String>, InfraError>
where
    E: sqlx::Executor<'e, Database = Sqlite>,
{
    sqlx::query_scalar(
        "SELECT max(attempted_at) FROM login_attempts
         WHERE scope = ? AND subject = ? AND succeeded = 0 AND attempted_at >= ?",
    )
    .bind(scope)
    .bind(subject)
    .bind(since)
    .fetch_one(executor)
    .await
    .map_err(InfraError::Connect)
}

/// 登录成功后清空该用户名的失败记录（A21：清记录但不写审计）。
pub async fn clear_failures(conn: &mut SqliteConnection, subject: &str) -> Result<u64, InfraError> {
    let affected =
        sqlx::query("DELETE FROM login_attempts WHERE scope = ? AND subject = ? AND succeeded = 0")
            .bind(SCOPE_USERNAME)
            .bind(subject)
            .execute(conn)
            .await
            .map_err(InfraError::Connect)?
            .rows_affected();
    Ok(affected)
}

/// 保留窗口外的记录清理（避免表无限增长）。
pub async fn prune_before(pool: &DbPool, before: &str) -> Result<u64, InfraError> {
    let affected = sqlx::query("DELETE FROM login_attempts WHERE attempted_at < ?")
        .bind(before)
        .execute(pool)
        .await
        .map_err(InfraError::Connect)?
        .rows_affected();
    Ok(affected)
}
