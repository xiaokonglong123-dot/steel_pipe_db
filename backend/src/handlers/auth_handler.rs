use axum::{
    extract::{Extension, FromRequestParts, Path, Query},
    http::{request::Parts, StatusCode},
    response::IntoResponse,
    Json,
};
use sqlx::SqlitePool;
use validator::Validate;

use crate::dto::auth_dto::{
    ChangePasswordRequest, ChangeUserRoleRequest, CreateUserRequest, LoginRequest, LoginResponse,
    RefreshTokenRequest, TokenResponse, UpdateUserRequest,
};
use crate::dto::common::PaginationParams;
use crate::error::AppError;
use crate::middleware::auth::{AuthContext, JwtSecret};
use crate::models::user::UserInfo;
use crate::repositories::operation_log_repo::{CreateOperationLog, OperationLogRepo};
use crate::response::ApiResponse;
use crate::services::auth_service::AuthService;

pub struct AuthenticatedUser(pub AuthContext);

impl<S: Sync> FromRequestParts<S> for AuthenticatedUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<AuthContext>()
            .cloned()
            .map(AuthenticatedUser)
            .ok_or_else(|| AppError::Unauthorized("Not authenticated".into()))
    }
}

/// POST `/api/v1/auth/login` — Straight-up user login, dead simple
///
/// Validates username/password credentials via AuthService, returns JWT access + refresh tokens on success.
/// Logs the operation to the operation_log table.
/// Returns 401 on invalid credentials, 429 on rate limit (rate_limit_login middleware).
pub async fn login_handler(
    Extension(pool): Extension<SqlitePool>,
    Extension(jwt_secret): Extension<JwtSecret>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<ApiResponse<LoginResponse>>, AppError> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let cfg = crate::config::Config::from_env();
    let response = AuthService::login(&pool, jwt_secret.as_str(), cfg.jwt_expiry_hours, &req).await?;

    if let Err(e) = OperationLogRepo::create(
        &pool,
        &CreateOperationLog {
            user_id: Some(response.user.id),
            username: Some(response.user.username.clone()),
            action: "login".into(),
            entity_type: "auth".into(),
            entity_id: Some(response.user.id),
            details: None,
            ip_address: None,
        },
    )
    .await {
        tracing::warn!("Failed to log login operation: {}", e);
    }

    Ok(ApiResponse::ok(response))
}

/// POST `/api/v1/auth/refresh` — Refresh that expired-ass token
///
/// Accepts a refresh token, validates it, and returns a new JWT access token + new refresh token.
/// Uses refresh token rotation for security.
/// Returns 401 if the refresh token is invalid or expired.
pub async fn refresh_handler(
    Extension(jwt_secret): Extension<JwtSecret>,
    Json(req): Json<RefreshTokenRequest>,
) -> Result<Json<ApiResponse<TokenResponse>>, AppError> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let cfg = crate::config::Config::from_env();
    let response = AuthService::refresh_token(jwt_secret.as_str(), cfg.jwt_expiry_hours, &req).await?;
    Ok(ApiResponse::ok(response))
}

/// POST `/api/v1/auth/logout` — Log the user the hell out
///
/// Records a logout operation log entry for the authenticated user.
/// Returns a success message; actual token invalidation is client-side.
/// Requires valid JWT in Authorization header.
pub async fn logout_handler(
    Extension(pool): Extension<SqlitePool>,
    AuthenticatedUser(auth): AuthenticatedUser,
) -> Result<Json<ApiResponse<String>>, AppError> {
    if let Err(e) = OperationLogRepo::create(
        &pool,
        &CreateOperationLog {
            user_id: Some(auth.user_id),
            username: Some(auth.username),
            action: "logout".into(),
            entity_type: "auth".into(),
            entity_id: Some(auth.user_id),
            details: None,
            ip_address: None,
        },
    )
    .await {
        tracing::warn!("Failed to log logout operation: {}", e);
    }

    Ok(ApiResponse::ok("Logged out".into()))
}

/// GET `/api/v1/auth/me` — Grab the current user's deets
///
/// Returns the profile of the currently authenticated user, including id, username, role, etc.
/// Requires valid JWT. Returns 401 if not authenticated.
pub async fn me_handler(
    Extension(pool): Extension<SqlitePool>,
    AuthenticatedUser(auth): AuthenticatedUser,
) -> Result<Json<ApiResponse<UserInfo>>, AppError> {
    let user = AuthService::get_me(&pool, auth.user_id).await?;
    Ok(ApiResponse::ok(user))
}

/// GET `/api/v1/users` — Paginated list of all users
///
/// Returns a paginated list of all system users, with optional search query `q`.
/// Admin-only. Supports pagination via `page` and `page_size` query params.
pub async fn list_users_handler(
    Extension(pool): Extension<SqlitePool>,
    Query(params): Query<UserListQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let (users, total) = AuthService::list_users(&pool, &params.pagination, params.q.as_deref()).await?;
    let page = params.pagination.page();
    let page_size = params.pagination.page_size();
    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "items": users,
            "total": total,
            "page": page,
            "page_size": page_size,
            "total_pages": if total == 0 { 0 } else { total.div_ceil(page_size) }
        }
    })))
}

#[derive(serde::Deserialize)]
pub struct UserListQuery {
    #[serde(flatten)]
    pub pagination: PaginationParams,
    pub q: Option<String>,
}

/// POST `/api/v1/users` — Create a brand new user
///
/// Creates a new system user with the specified username, password, role, and display name.
/// Admin-only. Logs the operation. Returns 400 on validation error, 409 on duplicate username.
pub async fn create_user_handler(
    Extension(pool): Extension<SqlitePool>,
    AuthenticatedUser(auth): AuthenticatedUser,
    Json(req): Json<CreateUserRequest>,
) -> Result<axum::response::Response, AppError> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let user = AuthService::create_user(&pool, &req).await?;

    if let Err(e) = OperationLogRepo::create(
        &pool,
        &CreateOperationLog {
            user_id: Some(auth.user_id),
            username: Some(auth.username),
            action: "create_user".into(),
            entity_type: "user".into(),
            entity_id: Some(user.id),
            details: Some(format!("Created user: {}", user.username)),
            ip_address: None,
        },
    )
    .await {
        tracing::warn!("Failed to log create_user operation: {}", e);
    }

    Ok(ApiResponse::created(user))
}

/// PUT `/api/v1/users/{id}` — Update user info like a boss
///
/// Updates user fields (username, display_name, active status) by user ID.
/// Admin-only. Logs the operation. Returns 404 if user not found.
pub async fn update_user_handler(
    Extension(pool): Extension<SqlitePool>,
    AuthenticatedUser(auth): AuthenticatedUser,
    Path(id): Path<i64>,
    Json(req): Json<UpdateUserRequest>,
) -> Result<Json<ApiResponse<UserInfo>>, AppError> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let user = AuthService::update_user(&pool, id, &req).await?;

    if let Err(e) = OperationLogRepo::create(
        &pool,
        &CreateOperationLog {
            user_id: Some(auth.user_id),
            username: Some(auth.username),
            action: "update_user".into(),
            entity_type: "user".into(),
            entity_id: Some(user.id),
            details: Some(format!("Updated user: {}", user.username)),
            ip_address: None,
        },
    )
    .await {
        tracing::warn!("Failed to log update_user operation: {}", e);
    }

    Ok(ApiResponse::ok(user))
}

/// POST `/api/v1/users/{id}/change-password` — Change user password
///
/// Changes password for the specified user. Non-admin users can only change their own password.
/// Rate-limited (rate_limit_password_change middleware). Logs the operation.
/// Returns 403 if non-admin tries to change another user's password.
pub async fn change_password_handler(
    Extension(pool): Extension<SqlitePool>,
    AuthenticatedUser(auth): AuthenticatedUser,
    Path(id): Path<i64>,
    Json(req): Json<ChangePasswordRequest>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;

    // Self-service or admin only: non-admin users can only change their own password
    if auth.role != "admin" && auth.user_id != id {
        return Err(AppError::Forbidden(
            "You can only change your own password".into(),
        ));
    }

    AuthService::change_password(&pool, id, &auth.role, &req).await?;

    if let Err(e) = OperationLogRepo::create(
        &pool,
        &CreateOperationLog {
            user_id: Some(auth.user_id),
            username: Some(auth.username),
            action: "change_password".into(),
            entity_type: "user".into(),
            entity_id: Some(id),
            details: None,
            ip_address: None,
        },
    )
    .await {
        tracing::warn!("Failed to log change_password operation: {}", e);
    }

    Ok(ApiResponse::ok("Password changed".into()))
}

/// PUT `/api/v1/users/{id}/role` — Swap the user's role
///
/// Changes the role of a user (e.g., admin, warehouse, qc, sales).
/// Admin-only. Logs the operation. Returns 404 if user not found.
pub async fn change_role_handler(
    Extension(pool): Extension<SqlitePool>,
    AuthenticatedUser(auth): AuthenticatedUser,
    Path(id): Path<i64>,
    Json(req): Json<ChangeUserRoleRequest>,
) -> Result<Json<ApiResponse<UserInfo>>, AppError> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let user = AuthService::change_role(&pool, id, &req.role).await?;

    if let Err(e) = OperationLogRepo::create(
        &pool,
        &CreateOperationLog {
            user_id: Some(auth.user_id),
            username: Some(auth.username),
            action: "change_role".into(),
            entity_type: "user".into(),
            entity_id: Some(user.id),
            details: Some(format!("Changed role to: {}", req.role)),
            ip_address: None,
        },
    )
    .await {
        tracing::warn!("Failed to log change_role operation: {}", e);
    }

    Ok(ApiResponse::ok(user))
}

/// DELETE `/api/v1/users/{id}` — Soft-delete a user (gone but not gone)
///
/// Soft-deletes a user by setting `deleted_at`. Admin-only.
/// Logs the operation. Returns 404 if user not found.
pub async fn delete_user_handler(
    Extension(pool): Extension<SqlitePool>,
    AuthenticatedUser(auth): AuthenticatedUser,
    Path(id): Path<i64>,
) -> Result<axum::response::Response, AppError> {
    AuthService::delete_user(&pool, id).await?;

    if let Err(e) = OperationLogRepo::create(
        &pool,
        &CreateOperationLog {
            user_id: Some(auth.user_id),
            username: Some(auth.username),
            action: "delete_user".into(),
            entity_type: "user".into(),
            entity_id: Some(id),
            details: Some(format!("Deleted user id: {}", id)),
            ip_address: None,
        },
    )
    .await {
        tracing::warn!("Failed to log delete_user operation: {}", e);
    }

    Ok((StatusCode::NO_CONTENT, ()).into_response())
}
