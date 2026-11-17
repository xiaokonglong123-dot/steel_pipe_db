//! 认证端点：CSRF 领取、登录、刷新轮换、登出、当前身份。

use std::net::SocketAddr;

use axum::extract::{ConnectInfo, State};
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use steelpipe_domain::Role;
use steelpipe_service::Identity;
use utoipa::ToSchema;

use crate::auth::{
    AuthUser, check_csrf, clear_session_headers, client_ip, consts, cookie, csrf_headers,
    new_csrf_token, session_headers,
};
use crate::error::{ApiError, ApiReject, ProblemDetails};
use crate::state::AppState;

use super::{AUTH_TAG, fail};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/auth/csrf", get(issue_csrf))
        .route("/api/auth/login", post(login))
        .route("/api/auth/refresh", post(refresh))
        .route("/api/auth/logout", post(logout))
        .route("/api/auth/me", get(me))
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
