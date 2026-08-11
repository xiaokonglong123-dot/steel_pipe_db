//! ERP v2 — 后端入口
//!
//! 分层（继承 v1 验证过的模式）：
//!   main.rs        → tracing + pool + migrate + bootstrap admin + serve
//!   config.rs      → 环境变量配置
//!   error.rs       → AppError + IntoResponse（不泄露 SQL 细节）
//!   response.rs    → ApiResponse<T> / PaginatedResponse<T> / Meta
//!   db.rs          → SqlitePool 初始化 + 迁移
//!   auth.rs        → JWT 签发/校验（access+refresh 轮换）、bootstrap_admin
//!   middleware/    → auth（JWT 校验 + AuthUser）+ rbac（查库实时权限）
//!   http/          → 每资源一个模块（routes+handlers 合一）
//!   services/      → 业务逻辑（事务边界在 service 层，金额 Decimal 计算）
//!   repos/         → 纯 SQL

use std::net::SocketAddr;

use tracing_subscriber::EnvFilter;

use erp_v2::{auth, config, db, http};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,erp_v2=debug")),
        )
        .init();

    let cfg = config::Config::from_env()?;
    let pool = db::init_pool(&cfg).await?;

    // 初始 admin（Argon2 生成真实哈希，幂等）
    auth::bootstrap_admin(&pool, &cfg.admin_username, &cfg.admin_password).await?;
    tracing::info!("bootstrap admin ensured: {}", cfg.admin_username);

    let app = http::router(pool, cfg.jwt_secret.clone());
    let addr: SocketAddr = cfg.server_addr()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("ERP v2 server listening on {addr}");
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;
    Ok(())
}
