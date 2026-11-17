//! 路由共享状态。

use std::sync::Arc;
use std::time::Instant;

use steelpipe_service::{AuthService, UserService};

/// 连接池类型经 service 重新导出，api 不直接依赖 sqlx。
pub use steelpipe_service::DbPool;

/// 数据库探针：API 层只关心“能不能用”，不关心驱动。
#[async_trait::async_trait]
pub trait Probe: Send + Sync {
    async fn ping(&self) -> Result<u64, String>;
}

pub type DbProbe = Arc<dyn Probe>;

#[derive(Clone)]
pub struct AppState {
    pub(crate) auth: Arc<AuthService>,
    pub(crate) users: Arc<UserService>,
    pub(crate) probe: Option<DbProbe>,
    pub(crate) cookie_secure: bool,
    pub(crate) pool: DbPool,
    version: &'static str,
    commit: &'static str,
    started_at: Instant,
}

impl AppState {
    pub fn new(
        auth: Arc<AuthService>,
        users: Arc<UserService>,
        probe: Option<DbProbe>,
        cookie_secure: bool,
        version: &'static str,
    ) -> Self {
        let pool = auth.pool().clone();
        Self {
            auth,
            users,
            probe,
            cookie_secure,
            pool,
            version,
            commit: option_env!("STEELPIPE_GIT_SHA").unwrap_or("unknown"),
            started_at: Instant::now(),
        }
    }

    /// 路由用它构造领域服务；api 不直接接触 sqlx 类型。
    pub fn pool(&self) -> &DbPool {
        &self.pool
    }

    pub(crate) fn version(&self) -> &'static str {
        self.version
    }

    pub(crate) fn commit(&self) -> &'static str {
        self.commit
    }

    pub(crate) fn uptime_seconds(&self) -> u64 {
        self.started_at.elapsed().as_secs()
    }
}
