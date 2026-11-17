//! 用户管理端点（全部 require_admin）。

use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use steelpipe_domain::Role;
use steelpipe_service::{NewUser, UserView};
use utoipa::ToSchema;

use crate::auth::{AuthUser, check_csrf};
use crate::error::{ApiReject, ProblemDetails};
use crate::state::AppState;

use super::{USERS_TAG, fail};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/users", get(list_users).post(create_user))
        .route("/api/users/{id}/active", post(set_active))
        .route("/api/users/{id}/password", post(set_password))
        .route("/api/users/{id}/revoke", post(revoke_sessions))
}
// ---------------------------------------------------------------------------
// 用户管理（全部 require_admin）
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, ToSchema)]
pub struct UserDto {
    pub id: i64,
    pub username: String,
    pub display_name: String,
    #[schema(value_type = String)]
    pub role: Role,
    pub active: bool,
    pub created_at: String,
}

impl From<UserView> for UserDto {
    fn from(view: UserView) -> Self {
        Self {
            id: view.id,
            username: view.username,
            display_name: view.display_name,
            role: view.role,
            active: view.active,
            created_at: view.created_at,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UserListResponse {
    pub users: Vec<UserDto>,
    pub total: usize,
}

#[utoipa::path(get, path = "/api/users", tag = USERS_TAG, summary = "用户列表（admin）",
    responses((status = 200, body = UserListResponse), (status = 403, body = ProblemDetails)))]
pub async fn list_users(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, ApiReject> {
    let user = AuthUser::from_headers(&headers, &state)
        .await
        .map_err(|error| fail(error, &headers))?;
    user.require_admin()
        .map_err(|error| fail(error, &headers))?;

    let users = state
        .users
        .list()
        .await
        .map_err(|error| fail(error, &headers))?;
    let total = users.len();
    Ok(Json(UserListResponse {
        users: users.into_iter().map(UserDto::from).collect(),
        total,
    })
    .into_response())
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
    pub display_name: String,
    #[schema(value_type = String)]
    pub role: Role,
}

#[utoipa::path(post, path = "/api/users", tag = USERS_TAG, summary = "新建用户（admin）",
    request_body = CreateUserRequest,
    responses(
        (status = 201, body = UserDto),
        (status = 400, body = ProblemDetails),
        (status = 403, body = ProblemDetails),
        (status = 409, body = ProblemDetails)
    ))]
pub async fn create_user(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<CreateUserRequest>,
) -> Result<Response, ApiReject> {
    let actor = AuthUser::from_headers(&headers, &state)
        .await
        .map_err(|error| fail(error, &headers))?;
    actor
        .require_admin()
        .map_err(|error| fail(error, &headers))?;
    check_csrf(&headers).map_err(|error| fail(error, &headers))?;

    let view = state
        .users
        .create(
            &actor.0,
            NewUser {
                username: body.username,
                password: body.password,
                display_name: body.display_name,
                role: body.role,
            },
        )
        .await
        .map_err(|error| fail(error, &headers))?;

    Ok((axum::http::StatusCode::CREATED, Json(UserDto::from(view))).into_response())
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SetActiveRequest {
    pub active: bool,
}

#[utoipa::path(post, path = "/api/users/{id}/active", tag = USERS_TAG, summary = "启用/停用（admin）",
    request_body = SetActiveRequest,
    responses((status = 204), (status = 400, body = ProblemDetails), (status = 403, body = ProblemDetails)))]
pub async fn set_active(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    headers: HeaderMap,
    Json(body): Json<SetActiveRequest>,
) -> Result<Response, ApiReject> {
    let actor = AuthUser::from_headers(&headers, &state)
        .await
        .map_err(|error| fail(error, &headers))?;
    actor
        .require_admin()
        .map_err(|error| fail(error, &headers))?;
    check_csrf(&headers).map_err(|error| fail(error, &headers))?;

    state
        .users
        .set_active(&actor.0, id, body.active)
        .await
        .map_err(|error| fail(error, &headers))?;
    Ok(axum::http::StatusCode::NO_CONTENT.into_response())
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SetPasswordRequest {
    pub password: String,
}

#[utoipa::path(post, path = "/api/users/{id}/password", tag = USERS_TAG, summary = "重置口令（admin，吊销该用户会话）",
    request_body = SetPasswordRequest,
    responses((status = 204), (status = 400, body = ProblemDetails), (status = 403, body = ProblemDetails)))]
pub async fn set_password(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    headers: HeaderMap,
    Json(body): Json<SetPasswordRequest>,
) -> Result<Response, ApiReject> {
    let actor = AuthUser::from_headers(&headers, &state)
        .await
        .map_err(|error| fail(error, &headers))?;
    actor
        .require_admin()
        .map_err(|error| fail(error, &headers))?;
    check_csrf(&headers).map_err(|error| fail(error, &headers))?;

    state
        .users
        .set_password(&actor.0, id, &body.password)
        .await
        .map_err(|error| fail(error, &headers))?;
    Ok(axum::http::StatusCode::NO_CONTENT.into_response())
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RevokeResponse {
    pub token_version: i64,
}

#[utoipa::path(post, path = "/api/users/{id}/revoke", tag = USERS_TAG, summary = "吊销全部会话（admin）",
    responses((status = 200, body = RevokeResponse), (status = 403, body = ProblemDetails)))]
pub async fn revoke_sessions(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    headers: HeaderMap,
) -> Result<Response, ApiReject> {
    let actor = AuthUser::from_headers(&headers, &state)
        .await
        .map_err(|error| fail(error, &headers))?;
    actor
        .require_admin()
        .map_err(|error| fail(error, &headers))?;
    check_csrf(&headers).map_err(|error| fail(error, &headers))?;

    let token_version = state
        .users
        .revoke_sessions(&actor.0, id)
        .await
        .map_err(|error| fail(error, &headers))?;
    Ok(Json(RevokeResponse { token_version }).into_response())
}
