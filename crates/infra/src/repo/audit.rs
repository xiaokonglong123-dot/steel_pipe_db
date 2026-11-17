//! 审计链读取（分批校验用）。

use crate::{DbPool, InfraError};

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AuditRow {
    pub id: i64,
    pub user_id: Option<i64>,
    pub action: String,
    pub detail: String,
    pub prev_hash: String,
    pub current_hash: String,
    pub ts: String,
}

impl AuditRow {
    pub fn as_entry(&self) -> steelpipe_audit::Entry<'_> {
        steelpipe_audit::Entry {
            prev_hash: &self.prev_hash,
            action: &self.action,
            detail: &self.detail,
            timestamp: &self.ts,
            current_hash: &self.current_hash,
        }
    }
}

/// 按 id 升序取一页（`after_id` 为上一页最后一条的 id，首页传 0）。
pub async fn page_after(
    pool: &DbPool,
    after_id: i64,
    limit: i64,
) -> Result<Vec<AuditRow>, InfraError> {
    sqlx::query_as(
        "SELECT id, user_id, action, detail, prev_hash, current_hash, ts
         FROM audit_logs WHERE id > ? ORDER BY id LIMIT ?",
    )
    .bind(after_id)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(InfraError::Connect)
}

pub async fn find_by_id(pool: &DbPool, id: i64) -> Result<Option<AuditRow>, InfraError> {
    sqlx::query_as(
        "SELECT id, user_id, action, detail, prev_hash, current_hash, ts FROM audit_logs WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(InfraError::Connect)
}

/// 最近 n 条（倒序取，再翻成升序，便于表格展示）。
pub async fn last_n(pool: &DbPool, n: i64) -> Result<Vec<AuditRow>, InfraError> {
    let mut rows: Vec<AuditRow> = sqlx::query_as(
        "SELECT id, user_id, action, detail, prev_hash, current_hash, ts
         FROM audit_logs ORDER BY id DESC LIMIT ?",
    )
    .bind(n.clamp(1, 200))
    .fetch_all(pool)
    .await
    .map_err(InfraError::Connect)?;
    rows.reverse();
    Ok(rows)
}

pub async fn count(pool: &DbPool) -> Result<i64, InfraError> {
    sqlx::query_scalar("SELECT count(*) FROM audit_logs")
        .fetch_one(pool)
        .await
        .map_err(InfraError::Connect)
}

/// 按 action 过滤的分页查询（审计查询页用）。
pub async fn page_filtered(
    pool: &DbPool,
    action: Option<&str>,
    after_id: i64,
    limit: i64,
) -> Result<Vec<AuditRow>, InfraError> {
    let sql = match action {
        Some(_) => {
            "SELECT id, user_id, action, detail, prev_hash, current_hash, ts
             FROM audit_logs WHERE id > ? AND action = ? ORDER BY id LIMIT ?"
        }
        None => {
            "SELECT id, user_id, action, detail, prev_hash, current_hash, ts
             FROM audit_logs WHERE id > ? ORDER BY id LIMIT ?"
        }
    };
    let mut query = sqlx::query_as(sql).bind(after_id);
    if let Some(action) = action {
        query = query.bind(action);
    }
    query
        .bind(limit)
        .fetch_all(pool)
        .await
        .map_err(InfraError::Connect)
}

/// 链尾（seq, hash）。
pub async fn chain_tip(pool: &DbPool) -> Result<(i64, String), InfraError> {
    sqlx::query_as("SELECT last_seq, last_hash FROM audit_chain_tip WHERE id = 1")
        .fetch_one(pool)
        .await
        .map_err(InfraError::Connect)
}
