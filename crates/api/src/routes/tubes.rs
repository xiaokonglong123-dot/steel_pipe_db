//! 管子与库存流转端点（D 域）。全部经 `InventoryService`，本文件不认识 SQL。

use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use steelpipe_domain::{QualityMark, TubeStatus, TubeType};
use steelpipe_service::inventory::{ScreenPipeInput, SeamlessPipeInput, StockInInput};
use steelpipe_service::{
    HistoryView, InventoryService, ScreenSpecView, SeamlessSpecView, TubeDetailView, TubeQuery,
    TubeView,
};
use utoipa::{IntoParams, ToSchema};

use crate::auth::AuthUser;
use crate::error::{ApiReject, ProblemDetails};
use crate::state::AppState;

use super::{TUBES_TAG, fail};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/tubes", get(list_tubes))
        .route("/api/tubes/stock-in", post(stock_in))
        .route("/api/tubes/stock-out/batch", post(batch_stock_out))
        .route("/api/tubes/{pipe_id}", get(tube_detail).delete(soft_delete))
        .route("/api/tubes/{pipe_id}/stock-out", post(stock_out))
        .route("/api/tubes/{pipe_id}/transfer", post(transfer))
        .route("/api/tubes/{pipe_id}/quality", post(set_quality))
        .route("/api/tubes/{pipe_id}/history", get(tube_history))
}

fn service(state: &AppState) -> InventoryService {
    InventoryService::new(state.pool().clone())
}

// ---------------------------------------------------------------------------
// DTO
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct ListQuery {
    /// in_stock / in_transfer / out_stock
    pub status: Option<String>,
    pub location: Option<String>,
    pub steel_grade: Option<String>,
    /// normal / problematic / scrapped
    pub quality_mark: Option<String>,
    /// 是否包含已软删（默认 false）
    pub include_deleted: Option<bool>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TubeDto {
    pub pipe_id: String,
    #[schema(value_type = String)]
    pub pipe_type: TubeType,
    pub outer_diameter_mm: f64,
    pub wall_thickness_mm: f64,
    pub length_m: f64,
    pub steel_grade: String,
    pub thread_type: Option<String>,
    pub api_5ct_cert: Option<String>,
    pub manufacturer: String,
    pub batch_no: Option<String>,
    pub heat_no: Option<String>,
    pub heat_treat_method: Option<String>,
    pub location: Option<String>,
    #[schema(value_type = String)]
    pub status: TubeStatus,
    #[schema(value_type = String)]
    pub quality_mark: QualityMark,
    pub quality_note: Option<String>,
    pub stock_in_time: Option<String>,
    pub stock_out_time: Option<String>,
    pub compliance_pass: Option<bool>,
    pub deleted: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl From<TubeView> for TubeDto {
    fn from(view: TubeView) -> Self {
        Self {
            pipe_id: view.pipe_id,
            pipe_type: view.pipe_type,
            outer_diameter_mm: view.outer_diameter_mm,
            wall_thickness_mm: view.wall_thickness_mm,
            length_m: view.length_m,
            steel_grade: view.steel_grade,
            thread_type: view.thread_type,
            api_5ct_cert: view.api_5ct_cert,
            manufacturer: view.manufacturer,
            batch_no: view.batch_no,
            heat_no: view.heat_no,
            heat_treat_method: view.heat_treat_method,
            location: view.location,
            status: view.status,
            quality_mark: view.quality_mark,
            quality_note: view.quality_note,
            stock_in_time: view.stock_in_time,
            stock_out_time: view.stock_out_time,
            compliance_pass: view.compliance_pass,
            deleted: view.deleted,
            created_at: view.created_at,
            updated_at: view.updated_at,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TubeListResponse {
    pub tubes: Vec<TubeDto>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SeamlessSpecDto {
    pub surface_treatment: Option<String>,
    pub end_protector: Option<String>,
}

impl From<SeamlessSpecView> for SeamlessSpecDto {
    fn from(view: SeamlessSpecView) -> Self {
        Self {
            surface_treatment: view.surface_treatment,
            end_protector: view.end_protector,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ScreenSpecDto {
    pub screen_type: Option<String>,
    pub slot_width_mm: Option<f64>,
    pub slot_density: Option<f64>,
    pub filtration_rating_um: Option<f64>,
    pub base_pipe_grade: Option<String>,
    pub slot_pattern: Option<String>,
}

impl From<ScreenSpecView> for ScreenSpecDto {
    fn from(view: ScreenSpecView) -> Self {
        Self {
            screen_type: view.screen_type,
            slot_width_mm: view.slot_width_mm,
            slot_density: view.slot_density,
            filtration_rating_um: view.filtration_rating_um,
            base_pipe_grade: view.base_pipe_grade,
            slot_pattern: view.slot_pattern,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct HistoryEntryDto {
    pub transaction_id: i64,
    pub kind: String,
    pub source_location: Option<String>,
    pub target_location: Option<String>,
    pub created_at: String,
    pub operator_id: i64,
    pub remark: Option<String>,
}

impl From<HistoryView> for HistoryEntryDto {
    fn from(view: HistoryView) -> Self {
        Self {
            transaction_id: view.transaction_id,
            kind: view.kind,
            source_location: view.source_location,
            target_location: view.target_location,
            created_at: view.created_at,
            operator_id: view.operator_id,
            remark: view.remark,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TubeDetailResponse {
    pub tube: TubeDto,
    pub seamless: Option<SeamlessSpecDto>,
    pub screen: Option<ScreenSpecDto>,
    pub history: Vec<HistoryEntryDto>,
}

impl From<TubeDetailView> for TubeDetailResponse {
    fn from(view: TubeDetailView) -> Self {
        Self {
            tube: TubeDto::from(view.tube),
            seamless: view.seamless.map(SeamlessSpecDto::from),
            screen: view.screen.map(ScreenSpecDto::from),
            history: view
                .history
                .into_iter()
                .map(HistoryEntryDto::from)
                .collect(),
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ComplianceCheckDto {
    pub check_name: String,
    pub passed: bool,
    pub reason: Option<String>,
}

// ---------------------------------------------------------------------------
// 读端点
// ---------------------------------------------------------------------------

#[utoipa::path(get, path = "/api/tubes", tag = TUBES_TAG, summary = "管子列表（登录可见）",
    params(ListQuery),
    responses((status = 200, body = TubeListResponse), (status = 401, body = ProblemDetails)))]
pub async fn list_tubes(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ListQuery>,
) -> Result<Response, ApiReject> {
    let user = AuthUser::from_headers(&headers, &state).await?;
    let _ = user;

    let page = service(&state)
        .list(TubeQuery {
            status: query.status.as_deref().and_then(|value| value.parse().ok()),
            location: query.location,
            steel_grade: query.steel_grade,
            quality_mark: query
                .quality_mark
                .as_deref()
                .and_then(|value| value.parse().ok()),
            include_deleted: query.include_deleted.unwrap_or(false),
            page: query.page.unwrap_or(1),
            page_size: query.page_size.unwrap_or(50),
        })
        .await?;

    Ok(Json(TubeListResponse {
        tubes: page.tubes.into_iter().map(TubeDto::from).collect(),
        total: page.total,
        page: page.page,
        page_size: page.page_size,
    })
    .into_response())
}

#[utoipa::path(get, path = "/api/tubes/{pipe_id}", tag = TUBES_TAG, summary = "管子详情（含扩展参数与流转历史）",
    responses((status = 200, body = TubeDetailResponse), (status = 404, body = ProblemDetails)))]
pub async fn tube_detail(
    State(state): State<AppState>,
    Path(pipe_id): Path<String>,
    headers: HeaderMap,
) -> Result<Response, ApiReject> {
    let user = AuthUser::from_headers(&headers, &state).await?;
    let _ = user;
    let detail = service(&state).detail(&pipe_id).await?;
    Ok(Json(TubeDetailResponse::from(detail)).into_response())
}

#[utoipa::path(get, path = "/api/tubes/{pipe_id}/history", tag = TUBES_TAG, summary = "管子流转历史",
    responses((status = 200, body = Vec<HistoryEntryDto>)))]
pub async fn tube_history(
    State(state): State<AppState>,
    Path(pipe_id): Path<String>,
    headers: HeaderMap,
) -> Result<Response, ApiReject> {
    let user = AuthUser::from_headers(&headers, &state).await?;
    let _ = user;
    let history = service(&state).history(&pipe_id).await?;
    Ok(Json(
        history
            .into_iter()
            .map(HistoryEntryDto::from)
            .collect::<Vec<_>>(),
    )
    .into_response())
}

// ---------------------------------------------------------------------------
// 写端点
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, ToSchema)]
pub struct ScreenInput {
    pub screen_type: Option<String>,
    pub slot_width_mm: f64,
    pub slot_density: Option<f64>,
    pub filtration_rating_um: f64,
    pub slot_pattern: Option<String>,
    pub base_pipe_grade: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SeamlessInput {
    pub surface_treatment: Option<String>,
    pub end_protector: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct StockInRequest {
    pub pipe_id: String,
    #[schema(value_type = String)]
    pub pipe_type: TubeType,
    pub outer_diameter_mm: f64,
    pub wall_thickness_mm: f64,
    pub length_m: f64,
    pub steel_grade: String,
    pub thread_type: Option<String>,
    pub api_5ct_cert: Option<String>,
    pub manufacturer: String,
    pub batch_no: Option<String>,
    pub heat_no: Option<String>,
    pub heat_treat_method: Option<String>,
    pub location: String,
    #[schema(value_type = String)]
    pub quality_mark: QualityMark,
    pub quality_note: Option<String>,
    pub screen: Option<ScreenInput>,
    pub seamless: Option<SeamlessInput>,
    /// 合规未通过时的强制入库（需 admin，写 forced_intake 审计）
    #[serde(default)]
    pub force: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct StockInResponse {
    pub pipe_id: String,
    pub compliance_pass: bool,
    pub forced: bool,
    pub location: String,
}

impl From<StockInRequest> for StockInInput {
    fn from(body: StockInRequest) -> Self {
        Self {
            pipe_id: body.pipe_id,
            pipe_type: body.pipe_type,
            outer_diameter_mm: body.outer_diameter_mm,
            wall_thickness_mm: body.wall_thickness_mm,
            length_m: body.length_m,
            steel_grade: body.steel_grade,
            thread_type: body.thread_type,
            api_5ct_cert: body.api_5ct_cert,
            manufacturer: body.manufacturer,
            batch_no: body.batch_no,
            heat_no: body.heat_no,
            heat_treat_method: body.heat_treat_method,
            location: body.location,
            quality_mark: body.quality_mark,
            quality_note: body.quality_note,
            screen: body.screen.map(|screen| ScreenPipeInput {
                screen_type: screen.screen_type,
                slot_width_mm: screen.slot_width_mm,
                slot_density: screen.slot_density.unwrap_or(0.0),
                filtration_rating_um: screen.filtration_rating_um,
                slot_pattern: screen.slot_pattern,
                base_pipe_grade: screen.base_pipe_grade,
            }),
            seamless: body.seamless.map(|seamless| SeamlessPipeInput {
                surface_treatment: seamless.surface_treatment,
                end_protector: seamless.end_protector,
            }),
            force: body.force,
        }
    }
}

#[utoipa::path(post, path = "/api/tubes/stock-in", tag = TUBES_TAG, summary = "入库（operator+，强制需 admin）",
    request_body = StockInRequest,
    responses(
        (status = 201, body = StockInResponse),
        (status = 400, body = ProblemDetails),
        (status = 403, body = ProblemDetails),
        (status = 409, body = ProblemDetails),
        (status = 422, body = ProblemDetails)
    ))]
pub async fn stock_in(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<StockInRequest>,
) -> Result<Response, ApiReject> {
    let actor = AuthUser::from_headers(&headers, &state).await?;
    actor
        .require_operator()
        .map_err(|error| fail(error, &headers))?;
    crate::auth::check_csrf(&headers).map_err(|error| fail(error, &headers))?;

    let outcome = service(&state).stock_in(&actor.0, body.into()).await?;

    Ok((
        axum::http::StatusCode::CREATED,
        Json(StockInResponse {
            pipe_id: outcome.pipe_id,
            compliance_pass: outcome.compliance_pass,
            forced: outcome.forced,
            location: outcome.location,
        }),
    )
        .into_response())
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct StockOutRequest {
    pub remark: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct StockOutResponse {
    pub pipe_id: String,
    pub location: Option<String>,
}

#[utoipa::path(post, path = "/api/tubes/{pipe_id}/stock-out", tag = TUBES_TAG, summary = "单根出库（operator+，问题管需 admin）",
    request_body = StockOutRequest,
    responses(
        (status = 200, body = StockOutResponse),
        (status = 403, body = ProblemDetails),
        (status = 409, body = ProblemDetails)
    ))]
pub async fn stock_out(
    State(state): State<AppState>,
    Path(pipe_id): Path<String>,
    headers: HeaderMap,
    Json(body): Json<StockOutRequest>,
) -> Result<Response, ApiReject> {
    let actor = AuthUser::from_headers(&headers, &state).await?;
    actor
        .require_operator()
        .map_err(|error| fail(error, &headers))?;
    crate::auth::check_csrf(&headers).map_err(|error| fail(error, &headers))?;

    let outcome = service(&state)
        .stock_out(&actor.0, &pipe_id, body.remark.as_deref())
        .await?;

    Ok(Json(StockOutResponse {
        pipe_id: outcome.pipe_id,
        location: outcome.location,
    })
    .into_response())
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct BatchStockOutRequest {
    pub pipe_ids: Vec<String>,
    pub remark: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BatchStockOutResponse {
    pub pipe_ids: Vec<String>,
    pub transaction_id: i64,
}

#[utoipa::path(post, path = "/api/tubes/stock-out/batch", tag = TUBES_TAG, summary = "批量出库（单事务原子）",
    request_body = BatchStockOutRequest,
    responses(
        (status = 200, body = BatchStockOutResponse),
        (status = 409, body = ProblemDetails),
        (status = 422, body = ProblemDetails)
    ))]
pub async fn batch_stock_out(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<BatchStockOutRequest>,
) -> Result<Response, ApiReject> {
    let actor = AuthUser::from_headers(&headers, &state).await?;
    actor
        .require_operator()
        .map_err(|error| fail(error, &headers))?;
    crate::auth::check_csrf(&headers).map_err(|error| fail(error, &headers))?;

    let outcome = service(&state)
        .batch_stock_out(&actor.0, &body.pipe_ids, body.remark.as_deref())
        .await?;

    Ok(Json(BatchStockOutResponse {
        pipe_ids: outcome.pipe_ids,
        transaction_id: outcome.transaction_id,
    })
    .into_response())
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct TransferRequest {
    pub target_location: String,
    pub remark: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TransferResponse {
    pub pipe_id: String,
    pub from: Option<String>,
    pub to: String,
}

#[utoipa::path(post, path = "/api/tubes/{pipe_id}/transfer", tag = TUBES_TAG, summary = "调拨（单根）",
    request_body = TransferRequest,
    responses(
        (status = 200, body = TransferResponse),
        (status = 400, body = ProblemDetails),
        (status = 409, body = ProblemDetails)
    ))]
pub async fn transfer(
    State(state): State<AppState>,
    Path(pipe_id): Path<String>,
    headers: HeaderMap,
    Json(body): Json<TransferRequest>,
) -> Result<Response, ApiReject> {
    let actor = AuthUser::from_headers(&headers, &state).await?;
    actor
        .require_operator()
        .map_err(|error| fail(error, &headers))?;
    crate::auth::check_csrf(&headers).map_err(|error| fail(error, &headers))?;

    let outcome = service(&state)
        .transfer(
            &actor.0,
            &pipe_id,
            &body.target_location,
            body.remark.as_deref(),
        )
        .await?;

    Ok(Json(TransferResponse {
        pipe_id: outcome.pipe_id,
        from: outcome.from,
        to: outcome.to,
    })
    .into_response())
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct QualityRequest {
    #[schema(value_type = String)]
    pub quality_mark: QualityMark,
    pub note: Option<String>,
}

#[utoipa::path(post, path = "/api/tubes/{pipe_id}/quality", tag = TUBES_TAG, summary = "质量标注（operator+）",
    request_body = QualityRequest,
    responses((status = 204), (status = 404, body = ProblemDetails)))]
pub async fn set_quality(
    State(state): State<AppState>,
    Path(pipe_id): Path<String>,
    headers: HeaderMap,
    Json(body): Json<QualityRequest>,
) -> Result<Response, ApiReject> {
    let actor = AuthUser::from_headers(&headers, &state).await?;
    actor
        .require_operator()
        .map_err(|error| fail(error, &headers))?;
    crate::auth::check_csrf(&headers).map_err(|error| fail(error, &headers))?;

    service(&state)
        .set_quality(&actor.0, &pipe_id, body.quality_mark, body.note.as_deref())
        .await?;
    Ok(axum::http::StatusCode::NO_CONTENT.into_response())
}

#[utoipa::path(delete, path = "/api/tubes/{pipe_id}", tag = TUBES_TAG, summary = "软删管子（admin）",
    responses((status = 204), (status = 403, body = ProblemDetails)))]
pub async fn soft_delete(
    State(state): State<AppState>,
    Path(pipe_id): Path<String>,
    headers: HeaderMap,
) -> Result<Response, ApiReject> {
    let actor = AuthUser::from_headers(&headers, &state).await?;
    actor
        .require_admin()
        .map_err(|error| fail(error, &headers))?;
    crate::auth::check_csrf(&headers).map_err(|error| fail(error, &headers))?;

    service(&state).soft_delete(&actor.0, &pipe_id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT.into_response())
}
