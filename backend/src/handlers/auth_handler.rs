use axum::{
    extract::{Extension, FromRequestParts, Path, Query},
    http::request::Parts,
    Json,
};
use sqlx::SqlitePool;

use crate::dto::auth_dto::{
    ChangePasswordRequest, CreateUserRequest, LoginRequest, LoginResponse, RefreshTokenRequest,
    TokenResponse, UpdateUserRequest,
};
use crate::dto::common::PaginationParams;
use crate::error::AppError;
use crate::middleware::auth::AuthContext;
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

pub async fn login_handler(
    Extension(pool): Extension<SqlitePool>,
    Extension(jwt_secret): Extension<String>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<ApiResponse<LoginResponse>>, AppError> {
    let cfg = crate::config::Config::from_env();
    let response = AuthService::login(&pool, &jwt_secret, cfg.jwt_expiry_hours, &req).await?;

    let _ = OperationLogRepo::create(
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
    .await;

    Ok(ApiResponse::ok(response))
}

pub async fn refresh_handler(
    Extension(jwt_secret): Extension<String>,
    Json(req): Json<RefreshTokenRequest>,
) -> Result<Json<ApiResponse<TokenResponse>>, AppError> {
    let cfg = crate::config::Config::from_env();
    let response = AuthService::refresh_token(&jwt_secret, cfg.jwt_expiry_hours, &req).await?;
    Ok(ApiResponse::ok(response))
}

pub async fn logout_handler(
    Extension(pool): Extension<SqlitePool>,
    AuthenticatedUser(auth): AuthenticatedUser,
) -> Result<Json<ApiResponse<String>>, AppError> {
    let _ = OperationLogRepo::create(
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
    .await;

    Ok(ApiResponse::ok("Logged out".into()))
}

pub async fn me_handler(
    Extension(pool): Extension<SqlitePool>,
    AuthenticatedUser(auth): AuthenticatedUser,
) -> Result<Json<ApiResponse<UserInfo>>, AppError> {
    let user = AuthService::get_me(&pool, auth.user_id).await?;
    Ok(ApiResponse::ok(user))
}

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
            "total_pages": if total == 0 { 0 } else { (total + page_size - 1) / page_size }
        }
    })))
}

#[derive(serde::Deserialize)]
pub struct UserListQuery {
    #[serde(flatten)]
    pub pagination: PaginationParams,
    pub q: Option<String>,
}

pub async fn create_user_handler(
    Extension(pool): Extension<SqlitePool>,
    AuthenticatedUser(auth): AuthenticatedUser,
    Json(req): Json<CreateUserRequest>,
) -> Result<Json<ApiResponse<UserInfo>>, AppError> {
    let user = AuthService::create_user(&pool, &req).await?;

    let _ = OperationLogRepo::create(
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
    .await;

    Ok(ApiResponse::ok(user))
}

pub async fn update_user_handler(
    Extension(pool): Extension<SqlitePool>,
    AuthenticatedUser(auth): AuthenticatedUser,
    Path(id): Path<i64>,
    Json(req): Json<UpdateUserRequest>,
) -> Result<Json<ApiResponse<UserInfo>>, AppError> {
    let user = AuthService::update_user(&pool, id, &req).await?;

    let _ = OperationLogRepo::create(
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
    .await;

    Ok(ApiResponse::ok(user))
}

pub async fn change_password_handler(
    Extension(pool): Extension<SqlitePool>,
    AuthenticatedUser(auth): AuthenticatedUser,
    Path(id): Path<i64>,
    Json(req): Json<ChangePasswordRequest>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    AuthService::change_password(&pool, id, &auth.role, &req).await?;

    let _ = OperationLogRepo::create(
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
    .await;

    Ok(ApiResponse::ok("Password changed".into()))
}


