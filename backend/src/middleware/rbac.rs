//! RBAC 中间件 — 查库实时权限校验
//!
//! 用法：`.route_layer(rbac::require_permission("item.read"))`
//! 权限变更即时生效（auth_middleware 每次请求查库注入 AuthUser.permissions）。

use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::Response;

use crate::error::{AppError, ErrorCode};
use crate::middleware::auth::AuthUser;

pub async fn require_permission(
    State(perm): State<&'static str>,
    req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let user = req
        .extensions()
        .get::<AuthUser>()
        .cloned()
        .ok_or_else(|| AppError::new(ErrorCode::Unauthorized, "未认证"))?;
    if user.permissions.iter().any(|p| p == perm) {
        Ok(next.run(req).await)
    } else {
        Err(AppError::new(ErrorCode::Forbidden, "权限不足"))
    }
}

pub async fn require_admin(_: State<()>, req: Request, next: Next) -> Result<Response, AppError> {
    let user = req
        .extensions()
        .get::<AuthUser>()
        .cloned()
        .ok_or_else(|| AppError::new(ErrorCode::Unauthorized, "未认证"))?;
    if user.is_admin() {
        Ok(next.run(req).await)
    } else {
        Err(AppError::new(ErrorCode::Forbidden, "需要管理员权限"))
    }
}
