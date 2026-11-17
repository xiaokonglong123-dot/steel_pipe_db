//! 仪表盘用的聚合查询。全部走索引/分组，不做全表扫描进内存。

use crate::{DbPool, InfraError};

/// 按状态统计管子（不含软删）。
pub async fn tube_counts_by_status(pool: &DbPool) -> Result<Vec<(String, i64)>, InfraError> {
    sqlx::query_as("SELECT status, count(*) FROM tubes WHERE deleted_at IS NULL GROUP BY status")
        .fetch_all(pool)
        .await
        .map_err(InfraError::Connect)
}

/// 按质量标记统计（不含软删）。
pub async fn tube_counts_by_quality(pool: &DbPool) -> Result<Vec<(String, i64)>, InfraError> {
    sqlx::query_as(
        "SELECT quality_mark, count(*) FROM tubes WHERE deleted_at IS NULL GROUP BY quality_mark",
    )
    .fetch_all(pool)
    .await
    .map_err(InfraError::Connect)
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct LocationSummary {
    pub total: i64,
    pub active: i64,
    pub used: i64,
    pub capacity: i64,
}

/// 库位汇总：总数 / 启用数 / 已用 / 名义容量（容量为空按 0 计）。
pub async fn location_summary(pool: &DbPool) -> Result<LocationSummary, InfraError> {
    sqlx::query_as(
        "SELECT count(*) AS total,
                coalesce(sum(is_active), 0) AS active,
                coalesce(sum(used_count), 0) AS used,
                coalesce(sum(capacity), 0) AS capacity
         FROM locations",
    )
    .fetch_one(pool)
    .await
    .map_err(InfraError::Connect)
}

/// 某时间点之后按单据类型计数。
pub async fn transaction_counts_since(
    pool: &DbPool,
    since: &str,
) -> Result<Vec<(String, i64)>, InfraError> {
    sqlx::query_as("SELECT type, count(*) FROM transactions WHERE created_at >= ? GROUP BY type")
        .bind(since)
        .fetch_all(pool)
        .await
        .map_err(InfraError::Connect)
}
