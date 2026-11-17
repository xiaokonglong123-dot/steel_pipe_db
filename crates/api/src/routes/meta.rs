//! 健康检查与 OpenAPI 自描述。

use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;
use utoipa::ToSchema;

use crate::error::{ApiError, ProblemDetails};
use crate::state::AppState;

use super::API_TAG;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/health/db", get(db_health))
        .route("/api/openapi.json", get(openapi_json))
}

// ---------------------------------------------------------------------------
// 健康检查
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, ToSchema)]
pub struct Health {
    /// 固定为 "ok"。
    pub status: String,
    /// 构建版本。
    pub version: String,
    /// git 提交短 sha（构建期注入，缺失时为 "unknown"）。
    pub commit: String,
    /// 进程已运行秒数。
    pub uptime_seconds: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DbHealth {
    /// 固定为 "ok"。
    pub status: String,
    /// 探活耗时（毫秒）。
    pub latency_ms: u64,
}

#[utoipa::path(get, path = "/api/health", tag = API_TAG, summary = "进程存活探针",
    responses((status = 200, body = Health)))]
pub async fn health(State(state): State<AppState>) -> Json<Health> {
    Json(Health {
        status: "ok".to_owned(),
        version: state.version().to_owned(),
        commit: state.commit().to_owned(),
        uptime_seconds: state.uptime_seconds(),
    })
}

#[utoipa::path(get, path = "/api/health/db", tag = API_TAG, summary = "数据库探针",
    responses((status = 200, body = DbHealth), (status = 503, body = ProblemDetails)))]
pub async fn db_health(State(state): State<AppState>) -> Result<Json<DbHealth>, ApiError> {
    let Some(probe) = state.probe.clone() else {
        return Err(ApiError::unavailable(
            "未配置数据库（STEELPIPE_DATABASE_URL）",
        ));
    };
    let latency_ms = probe
        .ping()
        .await
        .map_err(|e| ApiError::unavailable(format!("数据库不可用：{e}")))?;
    Ok(Json(DbHealth {
        status: "ok".to_owned(),
        latency_ms,
    }))
}

#[utoipa::path(get, path = "/api/openapi.json", tag = API_TAG, summary = "OpenAPI 文档",
    responses((status = 200, description = "OpenAPI 3.1 JSON")))]
pub async fn openapi_json() -> impl IntoResponse {
    match crate::openapi_json_string() {
        Ok(body) => (
            [(axum::http::header::CONTENT_TYPE, "application/json")],
            body,
        )
            .into_response(),
        Err(err) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            err.to_string(),
        )
            .into_response(),
    }
}
