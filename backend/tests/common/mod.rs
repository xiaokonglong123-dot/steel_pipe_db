//! 共享测试基建 — 为每个测试创建独立 tempfile SQLite 数据库，跑全量迁移，bootstrap admin

use std::str::FromStr;
use std::sync::atomic::{AtomicU64, Ordering};

use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use tempfile::TempDir;

static COUNTER: AtomicU64 = AtomicU64::new(0);

pub async fn test_pool() -> (SqlitePool, TempDir) {
    let dir = tempfile::tempdir().expect("tempdir");
    let idx = COUNTER.fetch_add(1, Ordering::SeqCst);
    let db_path = dir.path().join(format!("test_{idx}.db"));
    let url = format!("sqlite://{}?mode=rwc", db_path.display());

    let opts = SqliteConnectOptions::from_str(&url)
        .unwrap_or_else(|e| panic!("invalid db url: {e}"))
        .foreign_keys(true);
    let pool = SqlitePoolOptions::new()
        .max_connections(4)
        .connect_with(opts)
        .await
        .expect("connect");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("migrations");
    (pool, dir)
}
