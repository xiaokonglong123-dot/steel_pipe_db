use std::sync::Arc;
use axum::extract::{Path, State};
use axum::Json;

use crate::error::AppResult;
use crate::handler::{list_response, ok_response};
use crate::middleware::AuthUser;
use crate::repository::user_repo::UserRepo;
use crate::service::auth_service::{
    AuthService, CreateUserDto, LoginDto, LoginResponse, RefreshTokenDto, UpdateUserDto, UserInfo,
};
use crate::AppState;

/// POST /api/v1/auth/login
pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(dto): Json<LoginDto>,
) -> AppResult<Json<crate::handler::ApiResponse<LoginResponse>>> {
    let svc = AuthService::new(UserRepo::new(state.db.clone()), state.config.clone());
    let response = svc.login(dto).await?;
    Ok(ok_response(response))
}

/// POST /api/v1/auth/refresh
pub async fn refresh_token(
    State(state): State<Arc<AppState>>,
    Json(dto): Json<RefreshTokenDto>,
) -> AppResult<Json<crate::handler::ApiResponse<LoginResponse>>> {
    let svc = AuthService::new(UserRepo::new(state.db.clone()), state.config.clone());
    let response = svc.refresh_token(dto).await?;
    Ok(ok_response(response))
}

/// GET /api/v1/auth/me
pub async fn me(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
) -> AppResult<Json<crate::handler::ApiResponse<UserInfo>>> {
    let svc = AuthService::new(UserRepo::new(state.db.clone()), state.config.clone());
    let user = svc.get_current_user(&auth_user.user_id).await?;
    Ok(ok_response(user))
}

/// GET /api/v1/auth/users
pub async fn list_users(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
) -> AppResult<Json<crate::handler::ApiListResponse<UserInfo>>> {
    crate::middleware::check_role(&auth_user, &["admin"])?;
    let svc = AuthService::new(UserRepo::new(state.db.clone()), state.config.clone());
    let users = svc.list_users().await?;
    let total = users.len() as i64;
    Ok(list_response(users, total, 1, total.max(1)))
}

/// POST /api/v1/auth/users
pub async fn create_user(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    Json(dto): Json<CreateUserDto>,
) -> AppResult<Json<crate::handler::ApiResponse<UserInfo>>> {
    crate::middleware::check_role(&auth_user, &["admin"])?;
    let svc = AuthService::new(UserRepo::new(state.db.clone()), state.config.clone());
    let user = svc.create_user(dto).await?;
    Ok(ok_response(user))
}

/// PUT /api/v1/auth/users/{id}
pub async fn update_user(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    Path(user_id): Path<String>,
    Json(dto): Json<UpdateUserDto>,
) -> AppResult<Json<crate::handler::ApiResponse<UserInfo>>> {
    crate::middleware::check_role(&auth_user, &["admin"])?;
    let svc = AuthService::new(UserRepo::new(state.db.clone()), state.config.clone());
    let user = svc.update_user(&user_id, dto).await?;
    Ok(ok_response(user))
}

/// DELETE /api/v1/auth/users/{id}
pub async fn delete_user(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    Path(user_id): Path<String>,
) -> AppResult<Json<crate::handler::ApiResponse<&'static str>>> {
    crate::middleware::check_role(&auth_user, &["admin"])?;
    let svc = AuthService::new(UserRepo::new(state.db.clone()), state.config.clone());
    svc.delete_user(&user_id).await?;
    Ok(ok_response("User deleted"))
}
