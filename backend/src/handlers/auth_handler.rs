use axum::{
    extract::{Extension, FromRequestParts, Path, Query},
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use sqlx::PgPool;
use validator::Validate;

use crate::dto::auth_dto::{
    ChangePasswordRequest, ChangeUserRoleRequest, CreateUserRequest, LoginRequest,
    RefreshTokenRequest, UpdateUserRequest,
};
use crate::dto::common::PaginationParams;
use crate::error::AppError;
use crate::middleware::auth::{AuthContext, JwtSecret};
use crate::models::user::UserInfo;
use crate::repositories::operation_log_repo::{CreateOperationLog, OperationLogRepo};
use crate::repositories::user_repo::UserRepo;
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

/// POST `/api/v1/auth/login` — User login with access + refresh token pair.
///
/// Returns JWT access token in body and sets refresh token as httpOnly cookie.
pub async fn login_handler(
    Extension(pool): Extension<PgPool>,
    Extension(jwt_secret): Extension<JwtSecret>,
    Json(req): Json<LoginRequest>,
) -> Result<Response, AppError> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;
    let cfg = crate::config::Config::from_env();
    let response = AuthService::login(
        &pool,
        jwt_secret.as_str(),
        cfg.jwt_expiry_hours,
        cfg.refresh_token_expiry_days,
        &req,
    )
    .await?;

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
    .await
    {
        tracing::warn!("Failed to log login operation: {}", e);
    }

    let cookie = refresh_token_cookie(
        &response.refresh_token,
        cfg.refresh_token_expiry_days,
        &cfg.app_env,
    );
    let mut resp = (StatusCode::OK, ApiResponse::ok(response)).into_response();
    resp.headers_mut()
        .insert("set-cookie", cookie.to_string().parse().map_err(|_| AppError::Internal("Failed to parse cookie".into()))?);
    Ok(resp)
}

/// POST `/api/v1/auth/refresh` — Rotate refresh token (reads from httpOnly cookie).
///
/// Reads refresh token from httpOnly cookie. Returns new access + refresh pair,
/// sets new cookie. Returns 401 if cookie is missing or invalid.
pub async fn refresh_handler(
    Extension(pool): Extension<PgPool>,
    Extension(jwt_secret): Extension<JwtSecret>,
    jar: CookieJar,
) -> Result<Response, AppError> {
    let cfg = crate::config::Config::from_env();

    let refresh_token = jar
        .get("refresh_token")
        .map(|c| c.value().to_string())
        .ok_or_else(|| AppError::Unauthorized("Missing refresh token cookie".into()))?;

    let refresh_req = RefreshTokenRequest { refresh_token };
    let response = AuthService::refresh_token(
        &pool,
        jwt_secret.as_str(),
        cfg.jwt_expiry_hours,
        cfg.refresh_token_expiry_days,
        &refresh_req,
    )
    .await?;

    let cookie = refresh_token_cookie(
        &response.refresh_token,
        cfg.refresh_token_expiry_days,
        &cfg.app_env,
    );
    let mut resp = (StatusCode::OK, ApiResponse::ok(response)).into_response();
    resp.headers_mut()
        .insert("set-cookie", cookie.to_string().parse().map_err(|_| AppError::Internal("Failed to parse cookie".into()))?);
    Ok(resp)
}

/// POST `/api/v1/auth/logout` — Revoke all refresh tokens + clear cookie
pub async fn logout_handler(
    Extension(pool): Extension<PgPool>,
    AuthenticatedUser(auth): AuthenticatedUser,
) -> Result<Response, AppError> {
    AuthService::logout(&pool, auth.user_id).await?;

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
    .await
    {
        tracing::warn!("Failed to log logout operation: {}", e);
    }

    let cookie = Cookie::build(("refresh_token", ""))
        .path("/api/v1/auth")
        .max_age(time::Duration::seconds(0));
    let mut resp = (StatusCode::OK, ApiResponse::ok("Logged out".to_string())).into_response();
    resp.headers_mut()
        .insert("set-cookie", cookie.to_string().parse().map_err(|_| AppError::Internal("Failed to parse cookie".into()))?);
    Ok(resp)
}

/// GET `/api/v1/auth/me` — Grab the current user's deets
///
/// Returns the profile of the currently authenticated user, including id, username, role, etc.
/// Requires valid JWT. Returns 401 if not authenticated.
pub async fn me_handler(
    Extension(pool): Extension<PgPool>,
    AuthenticatedUser(auth): AuthenticatedUser,
) -> Result<Json<ApiResponse<UserInfo>>, AppError> {
    let user = AuthService::get_me(&pool, auth.user_id).await?;
    Ok(ApiResponse::ok(user))
}

#[derive(serde::Deserialize)]
pub struct UpdateOwnProfileRequest {
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
}

/// PUT `/api/v1/auth/me` — Update own profile (display_name, email, phone).
///
/// Allows any authenticated user to update their own profile fields.
/// Role, is_active, and password changes are NOT permitted here —
/// those require the admin-only `PUT /api/v1/users/{id}` endpoint.
pub async fn update_own_profile_handler(
    Extension(pool): Extension<PgPool>,
    AuthenticatedUser(auth): AuthenticatedUser,
    Json(req): Json<UpdateOwnProfileRequest>,
) -> Result<Json<ApiResponse<UserInfo>>, AppError> {
    let dto = UpdateUserRequest {
        display_name: req.display_name,
        role: None,
        email: req.email,
        phone: req.phone,
        is_active: None,
    };
    dto.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;
    let updated = UserRepo::update(&pool, auth.user_id, &dto)
        .await
        .map_err(AppError::from)?;
    Ok(ApiResponse::ok(UserInfo {
        id: updated.id,
        username: updated.username,
        display_name: updated.display_name,
        role: updated.role,
        email: updated.email,
        phone: updated.phone,
    }))
}

/// GET `/api/v1/users` — Paginated list of all users
///
/// Returns a paginated list of all system users, with optional search query `q`.
/// Admin-only. Supports pagination via `page` and `page_size` query params.
pub async fn list_users_handler(
    Extension(pool): Extension<PgPool>,
    Query(params): Query<UserListQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let (users, total) =
        AuthService::list_users(&pool, &params.pagination, params.q.as_deref()).await?;
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
    Extension(pool): Extension<PgPool>,
    AuthenticatedUser(auth): AuthenticatedUser,
    Json(req): Json<CreateUserRequest>,
) -> Result<axum::response::Response, AppError> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;
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
    .await
    {
        tracing::warn!("Failed to log create_user operation: {}", e);
    }

    Ok(ApiResponse::created(user))
}

/// PUT `/api/v1/users/{id}` — Update user info like a boss
///
/// Updates user fields (username, display_name, active status) by user ID.
/// Admin-only. Logs the operation. Returns 404 if user not found.
pub async fn update_user_handler(
    Extension(pool): Extension<PgPool>,
    AuthenticatedUser(auth): AuthenticatedUser,
    Path(id): Path<i64>,
    Json(req): Json<UpdateUserRequest>,
) -> Result<Json<ApiResponse<UserInfo>>, AppError> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;
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
    .await
    {
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
    Extension(pool): Extension<PgPool>,
    AuthenticatedUser(auth): AuthenticatedUser,
    Path(id): Path<i64>,
    Json(req): Json<ChangePasswordRequest>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

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
    .await
    {
        tracing::warn!("Failed to log change_password operation: {}", e);
    }

    Ok(ApiResponse::ok("Password changed".into()))
}

/// PUT `/api/v1/users/{id}/role` — Swap the user's role
///
/// Changes the role of a user (e.g., admin, warehouse, qc, sales).
/// Admin-only. Logs the operation. Returns 404 if user not found.
pub async fn change_role_handler(
    Extension(pool): Extension<PgPool>,
    AuthenticatedUser(auth): AuthenticatedUser,
    Path(id): Path<i64>,
    Json(req): Json<ChangeUserRoleRequest>,
) -> Result<Json<ApiResponse<UserInfo>>, AppError> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;
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
    .await
    {
        tracing::warn!("Failed to log change_role operation: {}", e);
    }

    Ok(ApiResponse::ok(user))
}

/// DELETE `/api/v1/users/{id}` — Soft-delete a user (gone but not gone)
///
/// Soft-deletes a user by setting `deleted_at`. Admin-only.
/// Logs the operation. Returns 404 if user not found.
pub async fn delete_user_handler(
    Extension(pool): Extension<PgPool>,
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
    .await
    {
        tracing::warn!("Failed to log delete_user operation: {}", e);
    }

    Ok((StatusCode::NO_CONTENT, ()).into_response())
}

fn refresh_token_cookie(token: &str, expiry_days: i64, app_env: &str) -> Cookie<'static> {
    let is_production = app_env == "production" || app_env == "prod";
    let mut builder = Cookie::build(("refresh_token", token.to_string()))
        .path("/api/v1/auth")
        .max_age(time::Duration::days(expiry_days))
        .http_only(true)
        .same_site(SameSite::Strict);
    if is_production {
        builder = builder.secure(true);
    }
    builder.build()
}
