//! 盘点端点（设计 §5.3）。

use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use steelpipe_domain::CheckResult;
use steelpipe_service::CheckService;
use steelpipe_service::checks::{CheckDetailView, CheckSessionView, ConfirmOutcome};
use utoipa::ToSchema;

use crate::auth::AuthUser;
use crate::error::{ApiReject, ProblemDetails};
use crate::state::AppState;

use super::tubes::StockInRequest;
use super::{CATALOG_TAG, fail};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/checks", get(list_checks).post(create_check))
        .route("/api/checks/{id}", get(check_detail))
        .route("/api/checks/{id}/items", post(mark_item))
        .route(
            "/api/checks/{id}/items/{item_id}",
            axum::routing::delete(remove_item),
        )
        .route("/api/checks/{id}/surplus", post(add_surplus))
        .route("/api/checks/{id}/confirm", post(confirm))
        .route("/api/checks/{id}/cancel", post(cancel))
        .route("/api/tubes/{pipe_id}/recover", post(recover))
}

fn service(state: &AppState) -> CheckService {
    CheckService::new(state.pool().clone())
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CheckSessionDto {
    pub id: i64,
    pub location: String,
    #[schema(value_type = String)]
    pub status: String,
    pub created_by: i64,
    pub confirmed_by: Option<i64>,
    pub created_at: String,
    pub confirmed_at: Option<String>,
    pub remark: Option<String>,
}

impl From<CheckSessionView> for CheckSessionDto {
    fn from(view: CheckSessionView) -> Self {
        Self {
            id: view.id,
            location: view.location,
            status: view.status.as_str().to_owned(),
            created_by: view.created_by,
            confirmed_by: view.confirmed_by,
            created_at: view.created_at,
            confirmed_at: view.confirmed_at,
            remark: view.remark,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CheckItemDto {
    pub id: i64,
    pub pipe_id: String,
    #[schema(value_type = String)]
    pub result: CheckResult,
    pub actual_location: Option<String>,
    pub has_surplus_data: bool,
    pub created_at: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ExpectedTubeDto {
    pub pipe_id: String,
    pub steel_grade: String,
    pub outer_diameter_mm: f64,
    pub wall_thickness_mm: f64,
    pub length_m: f64,
    #[schema(value_type = String)]
    pub quality_mark: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CheckDetailDto {
    pub session: CheckSessionDto,
    pub items: Vec<CheckItemDto>,
    pub expected: Vec<ExpectedTubeDto>,
    pub active_locations: Vec<String>,
}

impl From<CheckDetailView> for CheckDetailDto {
    fn from(view: CheckDetailView) -> Self {
        Self {
            session: CheckSessionDto::from(view.session),
            items: view
                .items
                .into_iter()
                .map(|item| CheckItemDto {
                    id: item.id,
                    pipe_id: item.pipe_id,
                    result: item.result,
                    actual_location: item.actual_location,
                    has_surplus_data: item.surplus.is_some(),
                    created_at: item.created_at,
                })
                .collect(),
            expected: view
                .expected
                .into_iter()
                .map(|row| ExpectedTubeDto {
                    pipe_id: row.pipe_id,
                    steel_grade: row.steel_grade,
                    outer_diameter_mm: row.outer_diameter_mm,
                    wall_thickness_mm: row.wall_thickness_mm,
                    length_m: row.length_m,
                    quality_mark: row.quality_mark.as_str().to_owned(),
                })
                .collect(),
            active_locations: view.active_locations,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CheckListResponse {
    pub sessions: Vec<CheckSessionDto>,
    pub total: usize,
}

#[utoipa::path(get, path = "/api/checks", tag = CATALOG_TAG, summary = "盘点单列表",
    responses((status = 200, body = CheckListResponse)))]
pub async fn list_checks(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, ApiReject> {
    let user = AuthUser::from_headers(&headers, &state).await?;
    let _ = user;
    let sessions = service(&state).list().await?;
    let total = sessions.len();
    Ok(Json(CheckListResponse {
        sessions: sessions.into_iter().map(CheckSessionDto::from).collect(),
        total,
    })
    .into_response())
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateCheckRequest {
    pub location: String,
    pub remark: Option<String>,
}

#[utoipa::path(post, path = "/api/checks", tag = CATALOG_TAG, summary = "开盘点单（operator+）",
    request_body = CreateCheckRequest,
    responses((status = 201, body = CheckSessionDto), (status = 400, body = ProblemDetails)))]
pub async fn create_check(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<CreateCheckRequest>,
) -> Result<Response, ApiReject> {
    let actor = AuthUser::from_headers(&headers, &state).await?;
    actor
        .require_operator()
        .map_err(|error| fail(error, &headers))?;
    crate::auth::check_csrf(&headers).map_err(|error| fail(error, &headers))?;

    let view = service(&state)
        .create(&actor.0, &body.location, body.remark.as_deref())
        .await?;
    Ok((
        axum::http::StatusCode::CREATED,
        Json(CheckSessionDto::from(view)),
    )
        .into_response())
}

#[utoipa::path(get, path = "/api/checks/{id}", tag = CATALOG_TAG, summary = "盘点单详情（含应盘清单）",
    responses((status = 200, body = CheckDetailDto), (status = 404, body = ProblemDetails)))]
pub async fn check_detail(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    headers: HeaderMap,
) -> Result<Response, ApiReject> {
    let user = AuthUser::from_headers(&headers, &state).await?;
    let _ = user;
    let detail = service(&state).detail(id).await?;
    Ok(Json(CheckDetailDto::from(detail)).into_response())
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct MarkItemRequest {
    pub pipe_id: String,
    #[schema(value_type = String)]
    pub result: CheckResult,
    pub actual_location: Option<String>,
}

#[utoipa::path(post, path = "/api/checks/{id}/items", tag = CATALOG_TAG, summary = "逐根核对（operator+）",
    request_body = MarkItemRequest,
    responses((status = 204), (status = 400, body = ProblemDetails), (status = 409, body = ProblemDetails)))]
pub async fn mark_item(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    headers: HeaderMap,
    Json(body): Json<MarkItemRequest>,
) -> Result<Response, ApiReject> {
    let actor = AuthUser::from_headers(&headers, &state).await?;
    actor
        .require_operator()
        .map_err(|error| fail(error, &headers))?;
    crate::auth::check_csrf(&headers).map_err(|error| fail(error, &headers))?;

    service(&state)
        .mark_item(
            &actor.0,
            id,
            &body.pipe_id,
            body.result,
            body.actual_location.as_deref(),
        )
        .await?;
    Ok(axum::http::StatusCode::NO_CONTENT.into_response())
}

#[utoipa::path(post, path = "/api/checks/{id}/surplus", tag = CATALOG_TAG, summary = "登记盘盈（operator+，确认时才过合规）",
    request_body = StockInRequest,
    responses((status = 204), (status = 409, body = ProblemDetails)))]
pub async fn add_surplus(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    headers: HeaderMap,
    Json(body): Json<StockInRequest>,
) -> Result<Response, ApiReject> {
    let actor = AuthUser::from_headers(&headers, &state).await?;
    actor
        .require_operator()
        .map_err(|error| fail(error, &headers))?;
    crate::auth::check_csrf(&headers).map_err(|error| fail(error, &headers))?;

    service(&state)
        .add_surplus(&actor.0, id, body.into())
        .await?;
    Ok(axum::http::StatusCode::NO_CONTENT.into_response())
}

#[utoipa::path(delete, path = "/api/checks/{id}/items/{item_id}", tag = CATALOG_TAG, summary = "删除盘点明细（operator+）",
    responses((status = 204), (status = 404, body = ProblemDetails)))]
pub async fn remove_item(
    State(state): State<AppState>,
    Path((id, item_id)): Path<(i64, i64)>,
    headers: HeaderMap,
) -> Result<Response, ApiReject> {
    let actor = AuthUser::from_headers(&headers, &state).await?;
    actor
        .require_operator()
        .map_err(|error| fail(error, &headers))?;
    crate::auth::check_csrf(&headers).map_err(|error| fail(error, &headers))?;

    service(&state).remove_item(id, item_id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT.into_response())
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ConfirmResponse {
    pub check_id: i64,
    pub missing: i64,
    pub moved: i64,
    pub surplus: i64,
    pub transaction_id: i64,
}

impl From<ConfirmOutcome> for ConfirmResponse {
    fn from(outcome: ConfirmOutcome) -> Self {
        Self {
            check_id: outcome.check_id,
            missing: outcome.missing,
            moved: outcome.moved,
            surplus: outcome.surplus,
            transaction_id: outcome.transaction_id,
        }
    }
}

#[utoipa::path(post, path = "/api/checks/{id}/confirm", tag = CATALOG_TAG, summary = "确认盘点（admin，原子应用差异）",
    responses(
        (status = 200, body = ConfirmResponse),
        (status = 403, body = ProblemDetails),
        (status = 409, body = ProblemDetails),
        (status = 422, body = ProblemDetails)
    ))]
pub async fn confirm(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    headers: HeaderMap,
) -> Result<Response, ApiReject> {
    let actor = AuthUser::from_headers(&headers, &state).await?;
    actor
        .require_admin()
        .map_err(|error| fail(error, &headers))?;
    crate::auth::check_csrf(&headers).map_err(|error| fail(error, &headers))?;

    let outcome = service(&state).confirm(&actor.0, id).await?;
    Ok(Json(ConfirmResponse::from(outcome)).into_response())
}

#[utoipa::path(post, path = "/api/checks/{id}/cancel", tag = CATALOG_TAG, summary = "取消盘点（operator+）",
    responses((status = 204), (status = 409, body = ProblemDetails)))]
pub async fn cancel(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    headers: HeaderMap,
) -> Result<Response, ApiReject> {
    let actor = AuthUser::from_headers(&headers, &state).await?;
    actor
        .require_operator()
        .map_err(|error| fail(error, &headers))?;
    crate::auth::check_csrf(&headers).map_err(|error| fail(error, &headers))?;

    service(&state).cancel(&actor.0, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT.into_response())
}

#[utoipa::path(post, path = "/api/tubes/{pipe_id}/recover", tag = CATALOG_TAG, summary = "找回盘亏管（operator+）",
    responses((status = 204), (status = 409, body = ProblemDetails)))]
pub async fn recover(
    State(state): State<AppState>,
    Path(pipe_id): Path<String>,
    headers: HeaderMap,
) -> Result<Response, ApiReject> {
    let actor = AuthUser::from_headers(&headers, &state).await?;
    actor
        .require_operator()
        .map_err(|error| fail(error, &headers))?;
    crate::auth::check_csrf(&headers).map_err(|error| fail(error, &headers))?;

    service(&state).recover(&actor.0, &pipe_id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT.into_response())
}
