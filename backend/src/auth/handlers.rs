//! RBAC handlers — roles, permissions, departments, tenants, user-role binding.
//!
//! Handlers stay thin: extract → call service → respond. Tenant scoping is
//! derived from the JWT (`AuthenticatedUser.0.tenant_id`), never from the
//! request body, so a client cannot touch another tenant's data.

use axum::{
    extract::{Extension, Path, Query},
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use sqlx::SqlitePool;

use crate::auth::services::IdentityService;
use crate::dto::rbac_dto::{
    AssignUserRolesRequest, CreateDepartmentRequest, CreateRoleRequest, AssignPermissionsRequest,
    UpdateDepartmentRequest, UpdateRoleRequest,
};
use crate::error::AppError;
use crate::middleware::auth::AuthenticatedUser;
use crate::response::ApiResponse;

/// GET `/api/v1/auth/tenants/{id}` — tenant details (self-tenant only).
pub async fn get_tenant(
    Extension(pool): Extension<SqlitePool>,
    user: AuthenticatedUser,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<crate::models::rbac::Tenant>>, AppError> {
    if id != user.0.tenant_id {
        return Err(AppError::Forbidden("Cannot access another tenant".into()));
    }
    let tenant = IdentityService::get_tenant(&pool, id).await?;
    Ok(ApiResponse::ok(tenant))
}

/// GET `/api/v1/auth/permissions` — the full permission dictionary.
pub async fn list_permissions(
    Extension(pool): Extension<SqlitePool>,
) -> Result<Json<ApiResponse<Vec<crate::models::rbac::Permission>>>, AppError> {
    let permissions = IdentityService::list_permissions(&pool).await?;
    Ok(ApiResponse::ok(permissions))
}

/// GET `/api/v1/auth/roles` — roles of the caller's tenant.
pub async fn list_roles(
    Extension(pool): Extension<SqlitePool>,
    user: AuthenticatedUser,
) -> Result<Json<ApiResponse<Vec<crate::models::rbac::Role>>>, AppError> {
    let roles = IdentityService::list_roles(&pool, user.0.tenant_id).await?;
    Ok(ApiResponse::ok(roles))
}

/// POST `/api/v1/auth/roles` — create a custom role in the caller's tenant.
pub async fn create_role(
    Extension(pool): Extension<SqlitePool>,
    user: AuthenticatedUser,
    Json(payload): Json<CreateRoleRequest>,
) -> Result<Response, AppError> {
    let role = IdentityService::create_role(
        &pool,
        user.0.tenant_id,
        &payload.name,
        payload.description.as_deref(),
    )
    .await?;
    Ok(ApiResponse::created(role))
}

/// PUT `/api/v1/auth/roles/{id}` — rename / re-describe a role.
pub async fn update_role(
    Extension(pool): Extension<SqlitePool>,
    user: AuthenticatedUser,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateRoleRequest>,
) -> Result<Json<ApiResponse<crate::models::rbac::Role>>, AppError> {
    let role = IdentityService::update_role(
        &pool,
        user.0.tenant_id,
        id,
        payload.name.as_deref(),
        payload.description.as_deref(),
    )
    .await?;
    Ok(ApiResponse::ok(role))
}

/// DELETE `/api/v1/auth/roles/{id}` — delete a non-system role.
pub async fn delete_role(
    Extension(pool): Extension<SqlitePool>,
    user: AuthenticatedUser,
    Path(id): Path<i64>,
) -> Result<Response, AppError> {
    IdentityService::delete_role(&pool, user.0.tenant_id, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT.into_response())
}

/// GET `/api/v1/auth/roles/{id}/permissions` — permission keys of a role.
pub async fn get_role_permissions(
    Extension(pool): Extension<SqlitePool>,
    user: AuthenticatedUser,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<Vec<String>>>, AppError> {
    let keys = IdentityService::role_permission_keys(&pool, user.0.tenant_id, id).await?;
    Ok(ApiResponse::ok(keys))
}

/// PUT `/api/v1/auth/roles/{id}/permissions` — replace a role's permission set.
pub async fn set_role_permissions(
    Extension(pool): Extension<SqlitePool>,
    user: AuthenticatedUser,
    Path(id): Path<i64>,
    Json(payload): Json<AssignPermissionsRequest>,
) -> Result<Json<ApiResponse<Vec<String>>>, AppError> {
    let keys =
        IdentityService::set_role_permissions(&pool, user.0.tenant_id, id, &payload.permissions)
            .await?;
    Ok(ApiResponse::ok(keys))
}

/// GET `/api/v1/auth/departments` — department tree of the caller's tenant.
pub async fn list_departments(
    Extension(pool): Extension<SqlitePool>,
    user: AuthenticatedUser,
    Query(params): Query<DepartmentListParams>,
) -> Result<Json<ApiResponse<Vec<crate::models::rbac::Department>>>, AppError> {
    let departments = IdentityService::list_departments(&pool, user.0.tenant_id, params.parent_id)
        .await?;
    Ok(ApiResponse::ok(departments))
}

/// POST `/api/v1/auth/departments` — create a department.
pub async fn create_department(
    Extension(pool): Extension<SqlitePool>,
    user: AuthenticatedUser,
    Json(payload): Json<CreateDepartmentRequest>,
) -> Result<Response, AppError> {
    let department = IdentityService::create_department(
        &pool,
        user.0.tenant_id,
        &payload.name,
        payload.parent_id,
        payload.sort_order,
    )
    .await?;
    Ok(ApiResponse::created(department))
}

/// PUT `/api/v1/auth/departments/{id}` — update a department.
pub async fn update_department(
    Extension(pool): Extension<SqlitePool>,
    user: AuthenticatedUser,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateDepartmentRequest>,
) -> Result<Json<ApiResponse<crate::models::rbac::Department>>, AppError> {
    let department = IdentityService::update_department(
        &pool,
        user.0.tenant_id,
        id,
        payload.name.as_deref(),
        payload.parent_id,
        payload.sort_order,
    )
    .await?;
    Ok(ApiResponse::ok(department))
}

/// DELETE `/api/v1/auth/departments/{id}` — delete a leaf department.
pub async fn delete_department(
    Extension(pool): Extension<SqlitePool>,
    user: AuthenticatedUser,
    Path(id): Path<i64>,
) -> Result<Response, AppError> {
    IdentityService::delete_department(&pool, user.0.tenant_id, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT.into_response())
}

/// GET `/api/v1/auth/users/{user_id}/roles` — role ids bound to a user.
pub async fn get_user_roles(
    Extension(pool): Extension<SqlitePool>,
    user: AuthenticatedUser,
    Path(user_id): Path<i64>,
) -> Result<Json<ApiResponse<Vec<i64>>>, AppError> {
    // Only self or a user with `system.admin` may inspect another user's roles.
    if user.0.user_id != user_id && !user.0.permissions.contains(&"system.admin".to_string()) {
        return Err(AppError::Forbidden("insufficient privileges".into()));
    }
    let role_ids = IdentityService::user_role_ids(&pool, user_id).await?;
    Ok(ApiResponse::ok(role_ids))
}

/// PUT `/api/v1/auth/users/{user_id}/roles` — replace a user's role set.
pub async fn assign_user_roles(
    Extension(pool): Extension<SqlitePool>,
    user: AuthenticatedUser,
    Path(user_id): Path<i64>,
    Json(payload): Json<AssignUserRolesRequest>,
) -> Result<Json<ApiResponse<Vec<String>>>, AppError> {
    let permissions =
        IdentityService::assign_user_roles(&pool, user.0.tenant_id, user_id, &payload.role_ids)
            .await?;
    Ok(ApiResponse::ok(permissions))
}

/// GET `/api/v1/auth/users/{user_id}/permissions` — effective permissions.
pub async fn get_user_permissions(
    Extension(pool): Extension<SqlitePool>,
    user: AuthenticatedUser,
    Path(user_id): Path<i64>,
) -> Result<Json<ApiResponse<Vec<String>>>, AppError> {
    // Only self or a user with `system.admin` may inspect another user's permissions.
    if user.0.user_id != user_id && !user.0.permissions.contains(&"system.admin".to_string()) {
        return Err(AppError::Forbidden("insufficient privileges".into()));
    }
    let keys = IdentityService::user_permission_keys(&pool, user_id).await?;
    Ok(ApiResponse::ok(keys))
}

#[derive(Debug, Deserialize)]
pub struct DepartmentListParams {
    pub parent_id: Option<i64>,
}
