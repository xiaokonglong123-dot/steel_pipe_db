//! 路由与 DTO。M1：健康检查 + 认证 + 用户管理；业务路由自 M2 起按领域加文件。

use std::net::SocketAddr;

use axum::extract::{ConnectInfo, Path, State};
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use steelpipe_domain::Role;
use steelpipe_service::{Identity, NewUser, UserView};
use utoipa::{OpenApi, ToSchema};

use crate::auth::{
    AuthUser, check_csrf, clear_session_headers, client_ip, consts, cookie, csrf_headers,
    new_csrf_token, session_headers,
};
use crate::error::{ApiError, ApiReject, ProblemDetails};
use crate::state::AppState;

pub const API_TAG: &str = "meta";
pub const AUTH_TAG: &str = "auth";
pub const USERS_TAG: &str = "users";

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/health/db", get(db_health))
        .route("/api/openapi.json", get(openapi_json))
        .route("/api/auth/csrf", get(issue_csrf))
        .route("/api/auth/login", post(login))
        .route("/api/auth/refresh", post(refresh))
        .route("/api/auth/logout", post(logout))
        .route("/api/auth/me", get(me))
        .route("/api/users", get(list_users).post(create_user))
        .route("/api/users/{id}/active", post(set_active))
        .route("/api/users/{id}/password", post(set_password))
        .route("/api/users/{id}/revoke", post(revoke_sessions))
}

/// 统一失败出口：带上请求头，让 401 能按 Accept 分流成登录页跳转（A11）。
fn fail(error: impl Into<ApiError>, headers: &HeaderMap) -> ApiReject {
    crate::error::reject(error.into(), headers)
}

// ---------------------------------------------------------------------------
// 健康检查
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, ToSchema)]
pub struct Health {
    /// 固定为 "ok"。
    pub status: String,
    /// 构建版本。
    pub version: String,
    /// git 提交短 sha（构建期注入，缺失时为 "unknown"）。
    pub commit: String,
    /// 进程已运行秒数。
    pub uptime_seconds: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DbHealth {
    /// 固定为 "ok"。
    pub status: String,
    /// 探活耗时（毫秒）。
    pub latency_ms: u64,
}

#[utoipa::path(get, path = "/api/health", tag = API_TAG, summary = "进程存活探针",
    responses((status = 200, body = Health)))]
pub async fn health(State(state): State<AppState>) -> Json<Health> {
    Json(Health {
        status: "ok".to_owned(),
        version: state.version().to_owned(),
        commit: state.commit().to_owned(),
        uptime_seconds: state.uptime_seconds(),
    })
}

#[utoipa::path(get, path = "/api/health/db", tag = API_TAG, summary = "数据库探针",
    responses((status = 200, body = DbHealth), (status = 503, body = ProblemDetails)))]
pub async fn db_health(State(state): State<AppState>) -> Result<Json<DbHealth>, ApiError> {
    let Some(probe) = state.probe.clone() else {
        return Err(ApiError::unavailable(
            "未配置数据库（STEELPIPE_DATABASE_URL）",
        ));
    };
    let latency_ms = probe
        .ping()
        .await
        .map_err(|e| ApiError::unavailable(format!("数据库不可用：{e}")))?;
    Ok(Json(DbHealth {
        status: "ok".to_owned(),
        latency_ms,
    }))
}

#[utoipa::path(get, path = "/api/openapi.json", tag = API_TAG, summary = "OpenAPI 文档",
    responses((status = 200, description = "OpenAPI 3.1 JSON")))]
pub async fn openapi_json() -> impl IntoResponse {
    match crate::openapi_json_string() {
        Ok(body) => (
            [(axum::http::header::CONTENT_TYPE, "application/json")],
            body,
        )
            .into_response(),
        Err(err) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            err.to_string(),
        )
            .into_response(),
    }
}

// ---------------------------------------------------------------------------
// 认证
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, ToSchema)]
pub struct CsrfResponse {
    pub csrf_token: String,
}

#[utoipa::path(get, path = "/api/auth/csrf", tag = AUTH_TAG, summary = "领取 CSRF token",
    responses((status = 200, body = CsrfResponse)))]
pub async fn issue_csrf(State(state): State<AppState>) -> Response {
    let token = new_csrf_token();
    (
        csrf_headers(&token, state.cookie_secure),
        Json(CsrfResponse { csrf_token: token }),
    )
        .into_response()
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

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
pub struct IdentityDto {
    pub user_id: i64,
    pub username: String,
    pub display_name: String,
    #[schema(value_type = String)]
    pub role: Role,
    pub token_version: i64,
}

impl From<Identity> for IdentityDto {
    fn from(identity: Identity) -> Self {
        Self {
            user_id: identity.user_id,
            username: identity.username,
            display_name: identity.display_name,
            role: identity.role,
            token_version: identity.token_version,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LoginResponse {
    pub user: IdentityDto,
    pub csrf_token: String,
    pub access_expires_at: u64,
}

#[utoipa::path(post, path = "/api/auth/login", tag = AUTH_TAG, summary = "登录（下发会话 cookie）",
    request_body = LoginRequest,
    responses(
        (status = 200, body = LoginResponse),
        (status = 401, body = ProblemDetails),
        (status = 403, body = ProblemDetails),
        (status = 429, body = ProblemDetails)
    ))]
pub async fn login(
    State(state): State<AppState>,
    headers: HeaderMap,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    Json(body): Json<LoginRequest>,
) -> Result<Response, ApiReject> {
    check_csrf(&headers).map_err(|error| fail(error, &headers))?;
    let ip = client_ip(&headers, Some(peer));

    let success = state
        .auth
        .login(&body.username, &body.password, &ip)
        .await
        .map_err(|error| fail(error, &headers))?;

    let payload = LoginResponse {
        user: success.identity.into(),
        csrf_token: success.session.csrf_token.clone(),
        access_expires_at: success.session.access_expires_at,
    };
    Ok((
        session_headers(&success.session, state.cookie_secure),
        Json(payload),
    )
        .into_response())
}

#[utoipa::path(post, path = "/api/auth/refresh", tag = AUTH_TAG, summary = "轮换 refresh（单会话）",
    responses(
        (status = 200, body = LoginResponse),
        (status = 401, body = ProblemDetails),
        (status = 403, body = ProblemDetails)
    ))]
pub async fn refresh(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, ApiReject> {
    check_csrf(&headers).map_err(|error| fail(error, &headers))?;
    let presented = cookie(&headers, consts::REFRESH_COOKIE)
        .ok_or_else(|| fail(ApiError::Unauthorized, &headers))?;

    let success = state
        .auth
        .refresh(presented)
        .await
        .map_err(|error| fail(error, &headers))?;

    let payload = LoginResponse {
        user: success.identity.into(),
        csrf_token: success.session.csrf_token.clone(),
        access_expires_at: success.session.access_expires_at,
    };
    Ok((
        session_headers(&success.session, state.cookie_secure),
        Json(payload),
    )
        .into_response())
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LogoutResponse {
    pub logged_out: bool,
}

#[utoipa::path(post, path = "/api/auth/logout", tag = AUTH_TAG, summary = "登出（吊销 refresh）",
    responses(
        (status = 200, body = LogoutResponse),
        (status = 401, body = ProblemDetails),
        (status = 403, body = ProblemDetails)
    ))]
pub async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, ApiReject> {
    let user = AuthUser::from_headers(&headers, &state)
        .await
        .map_err(|error| fail(error, &headers))?;
    check_csrf(&headers).map_err(|error| fail(error, &headers))?;

    let presented = cookie(&headers, consts::REFRESH_COOKIE);
    state
        .auth
        .logout(presented)
        .await
        .map_err(|error| fail(error, &headers))?;

    tracing::info!(user_id = user.0.user_id, "登出");
    Ok((
        clear_session_headers(state.cookie_secure),
        Json(LogoutResponse { logged_out: true }),
    )
        .into_response())
}

#[utoipa::path(get, path = "/api/auth/me", tag = AUTH_TAG, summary = "当前登录身份",
    responses((status = 200, body = IdentityDto), (status = 401, body = ProblemDetails)))]
pub async fn me(user: AuthUser) -> Json<IdentityDto> {
    Json(user.0.into())
}

// ---------------------------------------------------------------------------
// 用户管理（全部 require_admin）
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// OpenAPI
// ---------------------------------------------------------------------------

#[derive(OpenApi)]
#[openapi(
    info(
        title = "SteelPipe API",
        version = env!("CARGO_PKG_VERSION"),
        description = "无缝钢管与筛管库存管理系统 API（单二进制 + SQLite）"
    ),
    paths(
        health, db_health, openapi_json,
        issue_csrf, login, refresh, logout, me,
        list_users, create_user, set_active, set_password, revoke_sessions
    ),
    components(schemas(
        Health, DbHealth, ProblemDetails,
        CsrfResponse, LoginRequest, LoginResponse, LogoutResponse,
        IdentityDto, UserDto, UserListResponse, CreateUserRequest,
        SetActiveRequest, SetPasswordRequest, RevokeResponse
    ))
)]
pub struct ApiDoc;

pub fn build_openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}
