//! Project HTTP handlers.

use axum::extract::{Extension, Path, Query};
use axum::Json;
use serde::Deserialize;
use sqlx::PgPool;

use crate::dto::project_dto::{
    CreateProjectRequest, CreateTransactionRequest, CreateWbsRequest, UpdateWbsProgressRequest,
};
use crate::error::AppError;
use crate::handlers::auth_handler::AuthenticatedUser;
use crate::models::project::{Project, ProjectFinancials, ProjectTransaction, WbsElement};
use crate::project::services::ProjectService;
use crate::response::ApiResponse;

#[derive(Debug, Deserialize)]
pub struct StatusFilter {
    pub status: Option<String>,
}

pub async fn list_projects(
    Extension(pool): Extension<PgPool>,
    user: AuthenticatedUser,
    Query(f): Query<StatusFilter>,
) -> Result<Json<ApiResponse<Vec<Project>>>, AppError> {
    Ok(ApiResponse::ok(ProjectService::list_projects(&pool, user.0.tenant_id, f.status.as_deref()).await?))
}

pub async fn get_project(
    Extension(pool): Extension<PgPool>,
    user: AuthenticatedUser,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<Project>>, AppError> {
    Ok(ApiResponse::ok(ProjectService::get_project(&pool, user.0.tenant_id, id).await?))
}

pub async fn create_project(
    Extension(pool): Extension<PgPool>,
    user: AuthenticatedUser,
    Json(p): Json<CreateProjectRequest>,
) -> Result<Json<ApiResponse<Project>>, AppError> {
    Ok(ApiResponse::ok(ProjectService::create_project(&pool, user.0.tenant_id, &p).await?))
}

pub async fn update_project_status(
    Extension(pool): Extension<PgPool>,
    user: AuthenticatedUser,
    Path(id): Path<i64>,
    Json(p): Json<StatusFilter>,
) -> Result<Json<ApiResponse<Project>>, AppError> {
    let status = p.status.unwrap_or_default();
    Ok(ApiResponse::ok(ProjectService::update_project_status(&pool, user.0.tenant_id, id, &status).await?))
}

pub async fn wbs_tree(
    Extension(pool): Extension<PgPool>,
    _user: AuthenticatedUser,
    Path(project_id): Path<i64>,
) -> Result<Json<ApiResponse<Vec<WbsElement>>>, AppError> {
    Ok(ApiResponse::ok(ProjectService::wbs_tree(&pool, project_id).await?))
}

pub async fn create_wbs(
    Extension(pool): Extension<PgPool>,
    user: AuthenticatedUser,
    Path(project_id): Path<i64>,
    Json(p): Json<CreateWbsRequest>,
) -> Result<Json<ApiResponse<WbsElement>>, AppError> {
    Ok(ApiResponse::ok(ProjectService::create_wbs(&pool, user.0.tenant_id, project_id, &p).await?))
}

pub async fn update_wbs_progress(
    Extension(pool): Extension<PgPool>,
    _user: AuthenticatedUser,
    Path((project_id, id)): Path<(i64, i64)>,
    Json(p): Json<UpdateWbsProgressRequest>,
) -> Result<Json<ApiResponse<WbsElement>>, AppError> {
    Ok(ApiResponse::ok(ProjectService::update_wbs_progress(&pool, project_id, id, &p).await?))
}

pub async fn financials(
    Extension(pool): Extension<PgPool>,
    user: AuthenticatedUser,
    Path(project_id): Path<i64>,
) -> Result<Json<ApiResponse<ProjectFinancials>>, AppError> {
    Ok(ApiResponse::ok(ProjectService::financials(&pool, user.0.tenant_id, project_id).await?))
}

pub async fn list_transactions(
    Extension(pool): Extension<PgPool>,
    _user: AuthenticatedUser,
    Path(project_id): Path<i64>,
) -> Result<Json<ApiResponse<Vec<ProjectTransaction>>>, AppError> {
    Ok(ApiResponse::ok(ProjectService::list_transactions(&pool, project_id).await?))
}

pub async fn create_transaction(
    Extension(pool): Extension<PgPool>,
    user: AuthenticatedUser,
    Path(project_id): Path<i64>,
    Json(p): Json<CreateTransactionRequest>,
) -> Result<Json<ApiResponse<ProjectTransaction>>, AppError> {
    Ok(ApiResponse::ok(ProjectService::create_transaction(&pool, user.0.tenant_id, project_id, &p, Some(user.0.user_id)).await?))
}
