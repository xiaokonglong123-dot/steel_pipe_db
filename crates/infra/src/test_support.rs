//! 测试辅助：临时 SQLite 库 + 已迁移的池。

#![cfg(test)]

use std::path::PathBuf;

use crate::DbPool;

/// 建一个临时库、跑完全部迁移，返回连接池与文件路径（drop 时文件保留，由调用方删）。
pub async fn temp_pool(tag: &str) -> (DbPool, PathBuf) {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("系统时钟早于 1970")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("steelpipe-{tag}-{nanos}.db"));
    let url = format!("sqlite://{}", path.display());
    let pool = crate::connect(&url, 8).await.expect("打开临时库失败");
    crate::migrate(&pool).await.expect("迁移临时库失败");
    (pool, path)
}
