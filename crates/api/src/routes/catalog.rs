//! 目录端点：库位与标准规格。

use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post, put};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use steelpipe_service::{CatalogService, LocationView, StandardInput, StandardView};
use utoipa::ToSchema;

use crate::auth::AuthUser;
use crate::error::{ApiReject, ProblemDetails};
use crate::state::AppState;

use super::{CATALOG_TAG, fail};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/locations", get(list_locations).post(create_location))
        .route("/api/locations/{code}", post(update_location))
        .route("/api/locations/{code}/active", post(set_location_active))
        .route("/api/standards", get(list_standards).post(create_standard))
        .route(
            "/api/standards/{id}",
            put(update_standard).delete(delete_standard),
        )
}

fn service(state: &AppState) -> CatalogService {
    CatalogService::new(state.pool().clone())
}

// ---------------------------------------------------------------------------
// 库位
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, ToSchema)]
pub struct LocationDto {
    pub code: String,
    pub name: Option<String>,
    pub used_count: i64,
    pub capacity: Option<i64>,
    pub is_active: bool,
}

impl From<LocationView> for LocationDto {
    fn from(view: LocationView) -> Self {
        Self {
            code: view.code,
            name: view.name,
            used_count: view.used_count,
            capacity: view.capacity,
            is_active: view.is_active,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LocationListResponse {
    pub locations: Vec<LocationDto>,
    pub total: usize,
}

#[utoipa::path(get, path = "/api/locations", tag = CATALOG_TAG, summary = "库位列表",
    responses((status = 200, body = LocationListResponse)))]
pub async fn list_locations(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, ApiReject> {
    let user = AuthUser::from_headers(&headers, &state).await?;
    let _ = user;
    let locations = service(&state).list_locations().await?;
    let total = locations.len();
    Ok(Json(LocationListResponse {
        locations: locations.into_iter().map(LocationDto::from).collect(),
        total,
    })
    .into_response())
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateLocationRequest {
    pub code: String,
    pub name: Option<String>,
    pub capacity: Option<i64>,
}

#[utoipa::path(post, path = "/api/locations", tag = CATALOG_TAG, summary = "新建库位（admin）",
    request_body = CreateLocationRequest,
    responses(
        (status = 201, body = LocationDto),
        (status = 400, body = ProblemDetails),
        (status = 409, body = ProblemDetails)
    ))]
pub async fn create_location(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<CreateLocationRequest>,
) -> Result<Response, ApiReject> {
    let actor = AuthUser::from_headers(&headers, &state).await?;
    actor
        .require_admin()
        .map_err(|error| fail(error, &headers))?;
    crate::auth::check_csrf(&headers).map_err(|error| fail(error, &headers))?;

    let view = service(&state)
        .create_location(&actor.0, &body.code, body.name.as_deref(), body.capacity)
        .await?;
    Ok((
        axum::http::StatusCode::CREATED,
        Json(LocationDto::from(view)),
    )
        .into_response())
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateLocationRequest {
    pub name: Option<String>,
    pub capacity: Option<i64>,
}

#[utoipa::path(post, path = "/api/locations/{code}", tag = CATALOG_TAG, summary = "改名/改容量（admin）",
    request_body = UpdateLocationRequest,
    responses((status = 204), (status = 404, body = ProblemDetails)))]
pub async fn update_location(
    State(state): State<AppState>,
    Path(code): Path<String>,
    headers: HeaderMap,
    Json(body): Json<UpdateLocationRequest>,
) -> Result<Response, ApiReject> {
    let actor = AuthUser::from_headers(&headers, &state).await?;
    actor
        .require_admin()
        .map_err(|error| fail(error, &headers))?;
    crate::auth::check_csrf(&headers).map_err(|error| fail(error, &headers))?;

    service(&state)
        .update_location(&actor.0, &code, body.name.as_deref(), body.capacity)
        .await?;
    Ok(axum::http::StatusCode::NO_CONTENT.into_response())
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SetLocationActiveRequest {
    pub active: bool,
}

#[utoipa::path(post, path = "/api/locations/{code}/active", tag = CATALOG_TAG, summary = "停用/启用库位（admin）",
    request_body = SetLocationActiveRequest,
    responses((status = 204), (status = 404, body = ProblemDetails)))]
pub async fn set_location_active(
    State(state): State<AppState>,
    Path(code): Path<String>,
    headers: HeaderMap,
    Json(body): Json<SetLocationActiveRequest>,
) -> Result<Response, ApiReject> {
    let actor = AuthUser::from_headers(&headers, &state).await?;
    actor
        .require_admin()
        .map_err(|error| fail(error, &headers))?;
    crate::auth::check_csrf(&headers).map_err(|error| fail(error, &headers))?;

    service(&state)
        .set_location_active(&actor.0, &code, body.active)
        .await?;
    Ok(axum::http::StatusCode::NO_CONTENT.into_response())
}

// ---------------------------------------------------------------------------
// 标准规格
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, ToSchema)]
pub struct StandardDto {
    pub id: i64,
    pub grade: String,
    pub outer_diameter_mm: Option<f64>,
    pub min_wall_thickness_mm: Option<f64>,
    pub max_wall_thickness_mm: Option<f64>,
    pub length_tolerance_mm: Option<f64>,
    pub source: Option<String>,
}

impl From<StandardView> for StandardDto {
    fn from(view: StandardView) -> Self {
        Self {
            id: view.id,
            grade: view.grade,
            outer_diameter_mm: view.outer_diameter_mm,
            min_wall_thickness_mm: view.min_wall_thickness_mm,
            max_wall_thickness_mm: view.max_wall_thickness_mm,
            length_tolerance_mm: view.length_tolerance_mm,
            source: view.source,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct StandardListResponse {
    pub standards: Vec<StandardDto>,
    pub total: usize,
}

#[utoipa::path(get, path = "/api/standards", tag = CATALOG_TAG, summary = "标准规格列表",
    responses((status = 200, body = StandardListResponse)))]
pub async fn list_standards(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, ApiReject> {
    let user = AuthUser::from_headers(&headers, &state).await?;
    let _ = user;
    let standards = service(&state).list_standards().await?;
    let total = standards.len();
    Ok(Json(StandardListResponse {
        standards: standards.into_iter().map(StandardDto::from).collect(),
        total,
    })
    .into_response())
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct StandardRequest {
    pub grade: String,
    pub outer_diameter_mm: Option<f64>,
    pub min_wall_thickness_mm: Option<f64>,
    pub max_wall_thickness_mm: Option<f64>,
    pub length_tolerance_mm: Option<f64>,
    pub source: Option<String>,
}

impl From<StandardRequest> for StandardInput {
    fn from(request: StandardRequest) -> Self {
        Self {
            grade: request.grade,
            outer_diameter_mm: request.outer_diameter_mm,
            min_wall_thickness_mm: request.min_wall_thickness_mm,
            max_wall_thickness_mm: request.max_wall_thickness_mm,
            length_tolerance_mm: request.length_tolerance_mm,
            source: request.source,
        }
    }
}

#[utoipa::path(post, path = "/api/standards", tag = CATALOG_TAG, summary = "新建标准规格（admin）",
    request_body = StandardRequest,
    responses(
        (status = 201, body = StandardDto),
        (status = 400, body = ProblemDetails),
        (status = 409, body = ProblemDetails)
    ))]
pub async fn create_standard(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<StandardRequest>,
) -> Result<Response, ApiReject> {
    let actor = AuthUser::from_headers(&headers, &state).await?;
    actor
        .require_admin()
        .map_err(|error| fail(error, &headers))?;
    crate::auth::check_csrf(&headers).map_err(|error| fail(error, &headers))?;

    let view = service(&state)
        .create_standard(&actor.0, body.into())
        .await?;
    Ok((
        axum::http::StatusCode::CREATED,
        Json(StandardDto::from(view)),
    )
        .into_response())
}

#[utoipa::path(put, path = "/api/standards/{id}", tag = CATALOG_TAG, summary = "修改标准规格（admin）",
    request_body = StandardRequest,
    responses((status = 204), (status = 404, body = ProblemDetails)))]
pub async fn update_standard(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    headers: HeaderMap,
    Json(body): Json<StandardRequest>,
) -> Result<Response, ApiReject> {
    let actor = AuthUser::from_headers(&headers, &state).await?;
    actor
        .require_admin()
        .map_err(|error| fail(error, &headers))?;
    crate::auth::check_csrf(&headers).map_err(|error| fail(error, &headers))?;

    service(&state)
        .update_standard(&actor.0, id, body.into())
        .await?;
    Ok(axum::http::StatusCode::NO_CONTENT.into_response())
}

#[utoipa::path(delete, path = "/api/standards/{id}", tag = CATALOG_TAG, summary = "删除标准规格（admin，硬删+审计快照）",
    responses((status = 204), (status = 404, body = ProblemDetails)))]
pub async fn delete_standard(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    headers: HeaderMap,
) -> Result<Response, ApiReject> {
    let actor = AuthUser::from_headers(&headers, &state).await?;
    actor
        .require_admin()
        .map_err(|error| fail(error, &headers))?;
    crate::auth::check_csrf(&headers).map_err(|error| fail(error, &headers))?;

    service(&state).delete_standard(&actor.0, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT.into_response())
}
