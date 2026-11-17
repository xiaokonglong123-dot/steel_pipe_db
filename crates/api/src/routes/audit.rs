//! 审计端点：列表查询与分批校验（admin）。

use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use steelpipe_service::OpsService;
use utoipa::ToSchema;

use crate::auth::AuthUser;
use crate::error::{ApiReject, ProblemDetails};
use crate::state::AppState;

use super::{CATALOG_TAG, fail};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/audit", get(list_audit))
        .route("/api/audit/verify", post(verify_chain))
}

fn service(state: &AppState) -> OpsService {
    OpsService::new(state.pool().clone())
}

#[derive(Debug, Deserialize, utoipa::IntoParams, ToSchema)]
pub struct AuditQuery {
    pub action: Option<String>,
    /// 上一页最后一条的 id（首页传 0）
    pub after_id: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AuditEntryDto {
    pub id: i64,
    pub user_id: Option<i64>,
    pub action: String,
    pub detail: Option<String>,
    pub ts: String,
    pub current_hash: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AuditListResponse {
    pub entries: Vec<AuditEntryDto>,
    pub total: i64,
}

#[utoipa::path(get, path = "/api/audit", tag = CATALOG_TAG, summary = "审计日志（admin，按 id 升序分页）",
    params(AuditQuery),
    responses((status = 200, body = AuditListResponse), (status = 403, body = ProblemDetails)))]
pub async fn list_audit(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<AuditQuery>,
) -> Result<Response, ApiReject> {
    let actor = AuthUser::from_headers(&headers, &state).await?;
    actor
        .require_admin()
        .map_err(|error| fail(error, &headers))?;

    let after_id = query.after_id.unwrap_or(0);
    let limit = query.limit.unwrap_or(100).clamp(1, 500);
    let page = service(&state)
        .list_audit(query.action.as_deref(), after_id, limit)
        .await?;

    Ok(Json(AuditListResponse {
        entries: page
            .entries
            .into_iter()
            .map(|row| AuditEntryDto {
                id: row.id,
                user_id: row.user_id,
                action: row.action,
                detail: row.detail,
                ts: row.ts,
                current_hash: row.current_hash,
            })
            .collect(),
        total: page.total,
    })
    .into_response())
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct VerifyRequest {
    /// 每批条数（默认 1000，上限 20000）
    pub batch: Option<usize>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct VerifyResponse {
    pub entries: i64,
    pub batches: i64,
    pub tip: String,
}

#[utoipa::path(post, path = "/api/audit/verify", tag = CATALOG_TAG, summary = "分批校验审计链（admin）",
    request_body = VerifyRequest,
    responses(
        (status = 200, body = VerifyResponse),
        (status = 403, body = ProblemDetails),
        (status = 409, body = ProblemDetails)
    ))]
pub async fn verify_chain(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<VerifyRequest>,
) -> Result<Response, ApiReject> {
    let actor = AuthUser::from_headers(&headers, &state).await?;
    actor
        .require_admin()
        .map_err(|error| fail(error, &headers))?;
    crate::auth::check_csrf(&headers).map_err(|error| fail(error, &headers))?;

    let summary = service(&state)
        .verify_chain(body.batch.unwrap_or(1000))
        .await?;
    Ok(Json(VerifyResponse {
        entries: summary.entries,
        batches: summary.batches,
        tip: summary.tip,
    })
    .into_response())
}
