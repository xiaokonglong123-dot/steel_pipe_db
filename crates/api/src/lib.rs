//! HTTP 层：路由、DTO、错误映射、OpenAPI。
//!
//! 分层约束：本 crate 不得依赖 sqlx 或 steelpipe-infra。
//! 数据库探活通过 [`DbProbe`] trait 注入，实现位于 `crates/server`。

#![forbid(unsafe_code)]

pub mod error;
pub mod routes;

use std::sync::Arc;
use std::time::Instant;

pub use error::{ApiError, ProblemDetails};
pub use routes::ApiDoc;

/// 数据库探针。API 层只关心“能不能用”，不关心驱动与连接串。
#[async_trait::async_trait]
pub trait DbProbe: Send + Sync {
    /// 返回探活耗时（毫秒）。
    async fn ping(&self) -> Result<u64, String>;
}

/// 路由共享状态。
#[derive(Clone)]
pub struct AppState {
    version: &'static str,
    commit: &'static str,
    started_at: Instant,
    probe: Option<Arc<dyn DbProbe>>,
}

impl AppState {
    pub fn new(version: &'static str, probe: Option<Arc<dyn DbProbe>>) -> Self {
        Self {
            version,
            commit: option_env!("STEELPIPE_GIT_SHA").unwrap_or("unknown"),
            started_at: Instant::now(),
            probe,
        }
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

    pub(crate) fn probe(&self) -> Option<&Arc<dyn DbProbe>> {
        self.probe.as_ref()
    }
}

/// 装配全部 `/api` 路由。
pub fn router(state: AppState) -> axum::Router {
    routes::router().with_state(state)
}

/// OpenAPI 文档（供前端类型生成与 CI 漂移检查）。
pub fn openapi_json_string() -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(&routes::build_openapi())
}
