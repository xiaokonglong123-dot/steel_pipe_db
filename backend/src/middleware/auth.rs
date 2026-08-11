//! JWT 认证中间件 — 从 Authorization: Bearer / Cookie 提取 token，校验后注入 AuthUser
//!
//! 对齐 detailed-design §7.1：token 只带 user_id + username + exp，
//! 权限动态从 DB 查（rbac.rs），权限变更即时生效。

use axum::extract::{Extension, Request};
use axum::http::header::AUTHORIZATION;
use axum::middleware::Next;
use axum::response::Response;
use axum_extra::extract::cookie::CookieJar;

use crate::error::{AppError, ErrorCode};
use crate::repos::auth_repo;

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: i64,
    pub username: String,
    pub display_name: String,
    pub permissions: Vec<String>,
}

impl AuthUser {
    pub fn has_permission(&self, perm: &str) -> bool {
        self.permissions.iter().any(|p| p == perm)
    }

    pub fn is_admin(&self) -> bool {
        self.permissions.iter().any(|p| p == "user.manage")
    }
}

impl<S> axum::extract::FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<AuthUser>()
            .cloned()
            .ok_or_else(|| AppError::new(ErrorCode::Unauthorized, "未认证"))
    }
}

pub const ACCESS_COOKIE: &str = "access_token";
pub const REFRESH_COOKIE: &str = "refresh_token";

pub async fn auth_middleware(
    Extension(secret): Extension<JwtSecret>,
    Extension(pool): Extension<sqlx::SqlitePool>,
    cookies: CookieJar,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    // 1. 提取 token：优先 Authorization header，其次 access_token cookie
    let token = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(|s| s.to_string())
        .or_else(|| cookies.get(ACCESS_COOKIE).map(|c| c.value().to_string()));

    let Some(token) = token else {
        return Err(AppError::new(ErrorCode::Unauthorized, "缺少认证令牌"));
    };

    // 2. 校验 JWT
    let claims = crate::auth::verify_token(&token, &secret.0)?;

    // 3. 查库实时取权限（user → roles → permissions）
    let permissions = auth_repo::list_permissions_for_user(&pool, claims.sub).await?;

    let user = AuthUser {
        id: claims.sub,
        username: claims.username,
        display_name: claims.display_name,
        permissions,
    };

    req.extensions_mut().insert(user);
    Ok(next.run(req).await)
}

// —— JwtSecret newtype（防 Extension 类型碰撞 + Debug 不泄露 secret）——
#[derive(Clone)]
pub struct JwtSecret(pub String);

impl std::fmt::Debug for JwtSecret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("JwtSecret(********)")
    }
}

/// 从 CookieJar 取 refresh token
pub fn refresh_token_from_jar(jar: &CookieJar) -> Option<String> {
    jar.get(REFRESH_COOKIE).map(|c| c.value().to_string())
}
