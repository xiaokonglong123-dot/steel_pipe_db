//! SqlitePool 初始化 + 迁移
//!
//! 连接串：sqlite://data/erp.db?mode=rwc（WAL 模式由 sqlx 默认开启）。
//! 生产/测试均走同一路径：init_pool → migrate → 返回池。

use crate::config::Config;
use crate::error::AppError;

pub async fn init_pool(cfg: &Config) -> Result<sqlx::SqlitePool, AppError> {
    let opts = sqlx::sqlite::SqliteConnectOptions::from_str(&cfg.database_url)
        .map_err(|e| {
            tracing::error!(error = %e, "invalid database url");
            AppError::new(crate::error::ErrorCode::Config, "无效的数据库连接串")
        })?
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .busy_timeout(std::time::Duration::from_secs(5))
        .foreign_keys(true);

    let pool = sqlx::SqlitePool::connect_with(opts).await.map_err(|e| {
        tracing::error!(error = %e, "database connect failed");
        AppError::new(crate::error::ErrorCode::Config, "数据库连接失败")
    })?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "migration failed");
            AppError::new(crate::error::ErrorCode::Config, "数据库迁移失败")
        })?;

    Ok(pool)
}

// 供测试 / 其他场景读取连接串
use std::str::FromStr;
