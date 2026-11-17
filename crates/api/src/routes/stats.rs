//! 仪表盘聚合端点。

use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;
use steelpipe_service::{DashboardView, StatsService};
use utoipa::ToSchema;

use crate::auth::AuthUser;
use crate::error::{ApiReject, ProblemDetails};
use crate::state::AppState;

use super::CATALOG_TAG;

pub fn router() -> Router<AppState> {
    Router::new().route("/api/stats", get(dashboard))
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TubeCountsDto {
    pub total: i64,
    pub in_stock: i64,
    pub in_transfer: i64,
    pub out_stock: i64,
    pub missing: i64,
    pub problematic: i64,
    pub scrapped: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LocationStatsDto {
    pub total: i64,
    pub active: i64,
    pub used: i64,
    pub capacity: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TodayStatsDto {
    pub inbound: i64,
    pub outbound: i64,
    pub transfers: i64,
    pub checks: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AuditBriefDto {
    pub id: i64,
    pub user_id: Option<i64>,
    pub action: String,
    pub ts: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AuditStatsDto {
    pub entries: i64,
    pub seq: i64,
    pub tip: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DashboardDto {
    pub tubes: TubeCountsDto,
    pub locations: LocationStatsDto,
    pub today: TodayStatsDto,
    pub recent_audit: Vec<AuditBriefDto>,
    pub audit: AuditStatsDto,
}

impl From<DashboardView> for DashboardDto {
    fn from(view: DashboardView) -> Self {
        Self {
            tubes: TubeCountsDto {
                total: view.tubes.total,
                in_stock: view.tubes.in_stock,
                in_transfer: view.tubes.in_transfer,
                out_stock: view.tubes.out_stock,
                missing: view.tubes.missing,
                problematic: view.tubes.problematic,
                scrapped: view.tubes.scrapped,
            },
            locations: LocationStatsDto {
                total: view.locations.total,
                active: view.locations.active,
                used: view.locations.used,
                capacity: view.locations.capacity,
            },
            today: TodayStatsDto {
                inbound: view.today.inbound,
                outbound: view.today.outbound,
                transfers: view.today.transfers,
                checks: view.today.checks,
            },
            recent_audit: view
                .recent_audit
                .into_iter()
                .map(|row| AuditBriefDto {
                    id: row.id,
                    user_id: row.user_id,
                    action: row.action,
                    ts: row.ts,
                })
                .collect(),
            audit: AuditStatsDto {
                entries: view.audit.entries,
                seq: view.audit.seq,
                tip: view.audit.tip,
            },
        }
    }
}

#[utoipa::path(get, path = "/api/stats", tag = CATALOG_TAG, summary = "仪表盘聚合（登录可见）",
    responses((status = 200, body = DashboardDto), (status = 401, body = ProblemDetails)))]
pub async fn dashboard(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, ApiReject> {
    let user = AuthUser::from_headers(&headers, &state).await?;
    let _ = user;
    let view = StatsService::new(state.pool().clone()).snapshot().await?;
    Ok(Json(DashboardDto::from(view)).into_response())
}
