use axum::{
    extract::{Request, State},
    http::header,
    middleware::Next,
    response::Response,
    extract::FromRequestParts,
    http::request::Parts,
};

use crate::auth::validate_token;
use crate::error::{AppError, AppResult};
use std::sync::Arc;
use crate::AppState;

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: String,
    pub username: String,
    pub role: String,
}

pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let config = &state.config;

    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AppError::Unauthorized("Missing authorization header".into()))?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| AppError::Unauthorized("Invalid authorization format".into()))?;

    let claims = validate_token(token, &config.jwt_secret)
        .map_err(|_| AppError::Unauthorized("Invalid or expired token".into()))?;

    let auth_user = AuthUser {
        user_id: claims.sub,
        username: claims.username,
        role: claims.role,
    };

    req.extensions_mut().insert(auth_user);
    Ok(next.run(req).await)
}

pub fn check_role(auth_user: &AuthUser, allowed_roles: &[&str]) -> AppResult<()> {
    if allowed_roles.contains(&auth_user.role.as_str()) || auth_user.role == "admin" {
        Ok(())
    } else {
        Err(AppError::Forbidden("Insufficient permissions".into()))
    }
}

impl FromRequestParts<Arc<AppState>> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<AuthUser>()
            .cloned()
            .ok_or_else(|| AppError::Unauthorized("Not authenticated".into()))
    }
}
