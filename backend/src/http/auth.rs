//! Auth HTTP handlers — login / refresh / logout / me / users CRUD / roles / permissions

use axum::extract::{Extension, Path, Query};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use crate::config::Config;
use crate::error::AppError;
use crate::middleware::auth::{AuthUser, JwtSecret, ACCESS_COOKIE, REFRESH_COOKIE};
use crate::repos::auth_repo;
use crate::response::{ApiResponse, PaginatedResponse};
use crate::services::auth_service;

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub user: UserPublic,
}

#[derive(Serialize)]
pub struct UserPublic {
    pub id: i64,
    pub username: String,
    pub display_name: String,
    pub permissions: Vec<String>,
}

pub async fn login(
    Extension(pool): Extension<SqlitePool>,
    Extension(secret): Extension<JwtSecret>,
    Extension(cfg): Extension<Config>,
    Json(req): Json<LoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    let result = auth_service::login(
        &pool,
        &req.username,
        &req.password,
        &secret.0,
        cfg.jwt_expiry_hours,
        cfg.refresh_expiry_days,
    )
    .await?;

    let permissions = auth_repo::list_permissions_for_user(&pool, result.user_id).await?;

    let resp = ApiResponse::ok(LoginResponse {
        access_token: result.access_token.clone(),
        refresh_token: result.refresh_token.clone(),
        user: UserPublic {
            id: result.user_id,
            username: result.username,
            display_name: result.display_name,
            permissions,
        },
    });

    let access_cookie = cookie::Cookie::build(cookie::Cookie::new(
        ACCESS_COOKIE.to_string(),
        result.access_token,
    ))
    .path("/")
    .http_only(true)
    .same_site(cookie::SameSite::Lax)
    .max_age(cookie::time::Duration::hours(cfg.jwt_expiry_hours as i64))
    .build();
    let refresh_cookie = cookie::Cookie::build(cookie::Cookie::new(
        REFRESH_COOKIE.to_string(),
        result.refresh_token,
    ))
    .path("/auth")
    .http_only(true)
    .same_site(cookie::SameSite::Lax)
    .max_age(cookie::time::Duration::days(cfg.refresh_expiry_days as i64))
    .build();

    Ok((
        StatusCode::OK,
        [
            ("set-cookie", access_cookie.to_string()),
            ("set-cookie", refresh_cookie.to_string()),
        ],
        Json(resp),
    ))
}

#[derive(Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: Option<String>,
}

pub async fn refresh(
    Extension(pool): Extension<SqlitePool>,
    Extension(secret): Extension<JwtSecret>,
    Extension(cfg): Extension<Config>,
    jar: axum_extra::extract::cookie::CookieJar,
    Json(req): Json<RefreshRequest>,
) -> Result<impl IntoResponse, AppError> {
    let raw = req
        .refresh_token
        .or_else(|| jar.get(REFRESH_COOKIE).map(|c| c.value().to_string()))
        .ok_or_else(|| AppError::new(ErrorCode::Unauthorized, "缺少 refresh token"))?;

    let result = auth_service::refresh(
        &pool,
        &raw,
        &secret.0,
        cfg.jwt_expiry_hours,
        cfg.refresh_expiry_days,
    )
    .await?;

    let resp = ApiResponse::ok(serde_json::json!({
        "access_token": result.access_token,
        "refresh_token": result.refresh_token,
    }));

    let access_cookie = cookie::Cookie::build(cookie::Cookie::new(
        ACCESS_COOKIE.to_string(),
        result.access_token,
    ))
    .path("/")
    .http_only(true)
    .same_site(cookie::SameSite::Lax)
    .max_age(cookie::time::Duration::hours(cfg.jwt_expiry_hours as i64))
    .build();
    let refresh_cookie = cookie::Cookie::build(cookie::Cookie::new(
        REFRESH_COOKIE.to_string(),
        result.refresh_token,
    ))
    .path("/auth")
    .http_only(true)
    .same_site(cookie::SameSite::Lax)
    .max_age(cookie::time::Duration::days(cfg.refresh_expiry_days as i64))
    .build();

    Ok((
        StatusCode::OK,
        [
            ("set-cookie", access_cookie.to_string()),
            ("set-cookie", refresh_cookie.to_string()),
        ],
        Json(resp),
    ))
}

pub async fn logout(
    Extension(pool): Extension<SqlitePool>,
    user: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    auth_service::logout(&pool, user.id).await?;
    Ok((StatusCode::OK, Json(ApiResponse::ok(serde_json::json!({})))))
}

pub async fn me(user: AuthUser) -> impl IntoResponse {
    Json(ApiResponse::ok(UserPublic {
        id: user.id,
        username: user.username,
        display_name: user.display_name,
        permissions: user.permissions,
    }))
}

// —— Users CRUD ——
use crate::error::ErrorCode;

#[derive(Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
    pub display_name: String,
    pub role_ids: Vec<i64>,
}

#[derive(Serialize)]
pub struct UserRowResponse {
    pub id: i64,
    pub username: String,
    pub display_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub is_active: bool,
    pub created_at: String,
}

impl From<auth_repo::UserRow> for UserRowResponse {
    fn from(r: auth_repo::UserRow) -> Self {
        Self {
            id: r.id,
            username: r.username,
            display_name: r.display_name,
            email: r.email,
            phone: r.phone,
            is_active: r.is_active,
            created_at: r.created_at,
        }
    }
}

#[derive(Deserialize)]
pub struct PageQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

pub async fn list_users(
    Extension(pool): Extension<SqlitePool>,
    Query(q): Query<PageQuery>,
) -> Result<impl IntoResponse, AppError> {
    let page = q.page.unwrap_or(1).max(1);
    let page_size = q.page_size.unwrap_or(20).clamp(1, 200);
    let (rows, total) = auth_service::list_users(&pool, page, page_size).await?;
    let items: Vec<UserRowResponse> = rows.into_iter().map(Into::into).collect();
    Ok(Json(PaginatedResponse::ok(items, total, page, page_size)))
}

pub async fn create_user(
    Extension(pool): Extension<SqlitePool>,
    Json(req): Json<CreateUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    let id = auth_service::create_user(
        &pool,
        &req.username,
        &req.password,
        &req.display_name,
        &req.role_ids,
    )
    .await?;
    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::ok(serde_json::json!({ "id": id }))),
    ))
}

#[derive(Deserialize)]
pub struct UpdateUserRequest {
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub is_active: Option<bool>,
    pub role_ids: Option<Vec<i64>>,
    pub password: Option<String>,
}

pub async fn update_user(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    // 取当前用户作为默认值来源
    let current = auth_repo::find_by_id(&pool, id)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::NotFound, "用户未找到"))?;

    let display_name = req.display_name.unwrap_or(current.display_name);
    let email = req.email.or(current.email);
    let phone = req.phone.or(current.phone);
    let is_active = req.is_active.unwrap_or(current.is_active);

    auth_service::update_user(
        &pool,
        id,
        &display_name,
        email.as_deref(),
        phone.as_deref(),
        is_active,
        req.role_ids.as_deref(),
    )
    .await?;

    if let Some(new_pw) = req.password {
        if !new_pw.is_empty() {
            let hash = auth_service::hash_password(&new_pw)?;
            auth_repo::update_password_hash(&pool, id, &hash).await?;
        }
    }
    Ok((
        StatusCode::OK,
        Json(ApiResponse::ok(serde_json::json!({ "id": id }))),
    ))
}

pub async fn delete_user(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    auth_service::disable_user(&pool, id).await?;
    Ok((
        StatusCode::OK,
        Json(ApiResponse::ok(serde_json::json!({ "id": id }))),
    ))
}

pub async fn list_roles(
    Extension(pool): Extension<SqlitePool>,
) -> Result<impl IntoResponse, AppError> {
    let roles = auth_service::list_roles(&pool).await?;
    Ok(Json(ApiResponse::ok(roles)))
}

pub async fn list_permissions(
    Extension(pool): Extension<SqlitePool>,
) -> Result<impl IntoResponse, AppError> {
    let perms = auth_service::list_permissions(&pool).await?;
    Ok(Json(ApiResponse::ok(perms)))
}

pub async fn list_operation_logs(
    Extension(pool): Extension<SqlitePool>,
    Query(q): Query<PageQuery>,
) -> Result<impl IntoResponse, AppError> {
    let page = q.page.unwrap_or(1).max(1);
    let page_size = q.page_size.unwrap_or(20).clamp(1, 200);
    let (rows, total) = auth_service::list_operation_logs(&pool, page, page_size).await?;
    Ok(Json(PaginatedResponse::ok(rows, total, page, page_size)))
}
