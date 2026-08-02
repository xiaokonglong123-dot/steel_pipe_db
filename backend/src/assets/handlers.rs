//! Fixed asset HTTP handlers.

use axum::extract::{Extension, Path, Query};
use axum::Json;
use serde::Deserialize;
use sqlx::PgPool;

use crate::assets::services::AssetService;
use crate::dto::assets_dto::{CreateAssetRequest, DepreciateRequest, UpdateAssetRequest};
use crate::error::AppError;
use crate::handlers::auth_handler::AuthenticatedUser;
use crate::models::assets::{DepreciationEntry, FixedAsset};
use crate::response::ApiResponse;

#[derive(Debug, Deserialize)]
pub struct StatusFilter {
    pub status: Option<String>,
}

pub async fn list_assets(
    Extension(pool): Extension<PgPool>,
    user: AuthenticatedUser,
    Query(f): Query<StatusFilter>,
) -> Result<Json<ApiResponse<Vec<FixedAsset>>>, AppError> {
    Ok(ApiResponse::ok(AssetService::list_assets(&pool, user.0.tenant_id, f.status.as_deref()).await?))
}

pub async fn get_asset(
    Extension(pool): Extension<PgPool>,
    user: AuthenticatedUser,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<FixedAsset>>, AppError> {
    Ok(ApiResponse::ok(AssetService::get_asset(&pool, user.0.tenant_id, id).await?))
}

pub async fn create_asset(
    Extension(pool): Extension<PgPool>,
    user: AuthenticatedUser,
    Json(p): Json<CreateAssetRequest>,
) -> Result<Json<ApiResponse<FixedAsset>>, AppError> {
    Ok(ApiResponse::ok(AssetService::create_asset(&pool, user.0.tenant_id, &p).await?))
}

pub async fn update_asset(
    Extension(pool): Extension<PgPool>,
    user: AuthenticatedUser,
    Path(id): Path<i64>,
    Json(p): Json<UpdateAssetRequest>,
) -> Result<Json<ApiResponse<FixedAsset>>, AppError> {
    Ok(ApiResponse::ok(AssetService::update_asset(&pool, user.0.tenant_id, id, &p).await?))
}

pub async fn depreciate(
    Extension(pool): Extension<PgPool>,
    user: AuthenticatedUser,
    Path(id): Path<i64>,
    Json(p): Json<DepreciateRequest>,
) -> Result<Json<ApiResponse<DepreciationEntry>>, AppError> {
    Ok(ApiResponse::ok(AssetService::depreciate(&pool, user.0.tenant_id, id, &p).await?))
}

pub async fn dispose_asset(
    Extension(pool): Extension<PgPool>,
    user: AuthenticatedUser,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<FixedAsset>>, AppError> {
    Ok(ApiResponse::ok(AssetService::dispose_asset(&pool, user.0.tenant_id, id).await?))
}
