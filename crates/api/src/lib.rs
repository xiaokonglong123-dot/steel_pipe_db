//! HTTP 层：路由、DTO、错误映射、OpenAPI。
//!
//! 分层约束：本 crate 不得依赖 sqlx 或 steelpipe-infra。
//! 业务能力全部来自 [`steelpipe_service`]；数据库探活通过 [`Probe`] trait 注入。

#![forbid(unsafe_code)]

pub mod auth;
pub mod error;
pub mod routes;
pub mod state;

pub use error::{ApiError, ProblemDetails};
pub use routes::ApiDoc;
pub use state::{AppState, DbProbe, Probe};

/// 装配全部 `/api` 路由。
pub fn router(state: AppState) -> axum::Router {
    routes::router().with_state(state)
}

/// OpenAPI 文档（供前端类型生成与 CI 漂移检查）。
pub fn openapi_json_string() -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(&routes::build_openapi())
}
