//! `steelpipe serve`：装配配置、连接池、用例、中间件与优雅退出。

use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use axum::http::StatusCode;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

use steelpipe_api::{AppState, router};
use steelpipe_auth::Keys;
use steelpipe_service::{AuthService, UserService, bootstrap};

use crate::config::Config;
use crate::probe::DbPingProbe;

pub async fn run() -> anyhow::Result<()> {
    let config = Config::from_env()?;
    init_tracing();
    tracing::debug!(?config, "运行时配置");

    let pool = steelpipe_infra::connect(&config.database_url, config.max_connections)
        .await
        .context("打开数据库失败")?;
    steelpipe_infra::migrate(&pool)
        .await
        .context("应用迁移失败")?;

    if let Some((username, password)) = &config.admin_seed {
        match bootstrap::ensure_admin(&pool, username, password).await {
            Ok(seed) => tracing::info!(?seed, username = %username, "管理员引导完成"),
            Err(error) => tracing::error!(%error, "管理员引导失败"),
        }
    }

    let keys = Keys::from_secret(config.jwt_secret.as_bytes());
    let auth = Arc::new(AuthService::new(pool.clone(), keys));
    let users = Arc::new(UserService::new(pool.clone()));
    let probe: Option<steelpipe_api::DbProbe> = Some(Arc::new(DbPingProbe::new(pool.clone())));

    let state = AppState::new(
        auth,
        users,
        probe,
        config.cookie_secure,
        env!("CARGO_PKG_VERSION"),
    );

    let app = router(state)
        .fallback(crate::assets::static_handler)
        .layer(TraceLayer::new_for_http())
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(30),
        ))
        .layer(RequestBodyLimitLayer::new(config.max_body_bytes));

    let listener = tokio::net::TcpListener::bind(config.bind_addr)
        .await
        .with_context(|| format!("监听 {} 失败", config.bind_addr))?;

    tracing::info!(addr = %config.bind_addr, "SteelPipe 已启动");

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    .context("HTTP 服务异常退出")
}

fn init_tracing() {
    use tracing_subscriber::EnvFilter;

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new(
            "steelpipe_server=info,steelpipe_api=info,steelpipe_service=info,tower_http=info",
        )
    });

    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .compact()
        .try_init();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };

    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => {
                signal.recv().await;
            }
            Err(_) => std::future::pending::<()>().await,
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }

    tracing::info!("收到退出信号，开始优雅关闭");
}
