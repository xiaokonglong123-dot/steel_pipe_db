//! 持久层：连接池、迁移、写事务、审计链写入、仓储。
//!
//! 数据库为 SQLite（WAL，单写者）。写入串行化沿用设计 §12.1 的
//! `BEGIN IMMEDIATE` 语义：sqlx 的 `Pool::begin_with` 允许显式指定事务起始
//! 语句，因此不需要"单进程独占"之外的额外机制。
//!
//! 分层约束：本 crate 允许依赖 sqlx/tokio，但不得依赖 axum；
//! `crates/api` 不得依赖本 crate（见 CI 的分层断言）。

#![forbid(unsafe_code)]

pub mod repo;
pub mod time;
pub mod uow;

#[cfg(test)]
mod test_support;

use std::time::Duration;

use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};

/// 连接池别名：更换驱动时只改这一行，上层不受影响。
pub type DbPool = sqlx::SqlitePool;

pub use sqlx::migrate::Migrator;
pub use steelpipe_audit::GENESIS_HASH as AUDIT_GENESIS_HASH;

/// 审计链写入的事务起始语句（等价旧实现的 `BEGIN IMMEDIATE`）。
pub const WRITE_TXN_BEGIN: &str = "BEGIN IMMEDIATE";

/// 编译期嵌入的迁移集（`crates/infra/migrations`）。
pub static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

#[derive(Debug, thiserror::Error)]
pub enum InfraError {
    #[error("数据库连接失败：{0}")]
    Connect(#[from] sqlx::Error),
    #[error("连接串无效：{0}")]
    Url(String),
    #[error("迁移失败：{0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),
    #[error("{0}不存在")]
    NotFound(&'static str),
    #[error("{0}已存在")]
    Conflict(&'static str),
    #[error("盘点单不是 open 状态")]
    NotOpen,
    #[error("{0}")]
    Other(String),
}

impl InfraError {
    /// 唯一约束冲突映射为 [\`InfraError::Conflict\`]，其余原样包装。
    pub fn from_unique(error: sqlx::Error, what: &'static str) -> Self {
        if error
            .as_database_error()
            .is_some_and(|db_error| db_error.is_unique_violation())
        {
            Self::Conflict(what)
        } else {
            Self::Connect(error)
        }
    }
}

/// 打开（必要时创建）数据库并建立连接池。
///
/// PRAGMA 与设计 §9.1 对齐：`journal_mode=WAL`、`synchronous=NORMAL`、
/// `foreign_keys=ON`、`busy_timeout=5000`。
pub async fn connect(database_url: &str, max_connections: u32) -> Result<DbPool, InfraError> {
    let options: SqliteConnectOptions = database_url
        .parse::<SqliteConnectOptions>()
        .map_err(|e| InfraError::Url(e.to_string()))?
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal)
        .foreign_keys(true)
        .busy_timeout(Duration::from_secs(5));

    let pool = SqlitePoolOptions::new()
        .max_connections(max_connections.max(1))
        .acquire_timeout(Duration::from_secs(10))
        .connect_with(options)
        .await?;

    tracing::info!(max_connections, "数据库连接池已建立");
    Ok(pool)
}

/// 应用全部未执行的迁移。
pub async fn migrate(pool: &DbPool) -> Result<(), InfraError> {
    MIGRATOR.run(pool).await?;
    tracing::info!("迁移已应用到最新版本");
    Ok(())
}

/// 探活：执行一次 `SELECT 1`。
pub async fn ping(pool: &DbPool) -> Result<(), sqlx::Error> {
    sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(pool)
        .await
        .map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::test_support::temp_pool;
    use super::*;

    /// 创世哈希常量必须与参考实现一致。
    #[test]
    fn genesis_hash_matches_reference() {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(b"AYANAMI_REI_GENESIS");
        assert_eq!(format!("{:x}", hasher.finalize()), AUDIT_GENESIS_HASH);
    }

    /// 全部迁移可执行；链尾初值等于创世哈希；11 张业务表齐全。
    #[tokio::test]
    async fn migrations_apply_and_seed_schema() {
        let (pool, path) = temp_pool("migrate").await;

        let (seq, hash): (i64, String) =
            sqlx::query_as("SELECT last_seq, last_hash FROM audit_chain_tip")
                .fetch_one(&pool)
                .await
                .expect("链尾行缺失");

        assert_eq!(seq, 0);
        assert_eq!(hash, AUDIT_GENESIS_HASH);

        let journal: String = sqlx::query_scalar("PRAGMA journal_mode")
            .fetch_one(&pool)
            .await
            .expect("读取 journal_mode 失败");
        assert_eq!(journal.to_lowercase(), "wal");

        // 外键约束必须逐连接开启（设计 §9.1），否则 FK 形同虚设。
        let foreign_keys: i64 = sqlx::query_scalar("PRAGMA foreign_keys")
            .fetch_one(&pool)
            .await
            .expect("读取 foreign_keys 失败");
        assert_eq!(foreign_keys, 1);

        let busy_timeout: i64 = sqlx::query_scalar("PRAGMA busy_timeout")
            .fetch_one(&pool)
            .await
            .expect("读取 busy_timeout 失败");
        assert_eq!(busy_timeout, 5000);

        let mut tables: Vec<String> =
            sqlx::query_scalar("SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' AND name NOT LIKE '\\_sqlx%' ESCAPE '\\' ORDER BY name")
                .fetch_all(&pool)
                .await
                .expect("读取表清单失败");
        tables.retain(|t| t != "_sqlx_migrations");

        let expected = [
            "audit_chain_tip",
            "audit_logs",
            "check_items",
            "check_sessions",
            "locations",
            "login_attempts",
            "screen_pipe_specs",
            "seamless_pipe_specs",
            "standard_specs",
            "transaction_items",
            "transactions",
            "tubes",
            "users",
        ];
        assert_eq!(tables, expected);

        pool.close().await;
        let _ = std::fs::remove_file(&path);
    }
}
