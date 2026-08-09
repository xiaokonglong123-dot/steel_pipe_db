use std::fmt;

use axum::{
    extract::{FromRequestParts, Request},
    http::request::Parts,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::auth::repos::UserRoleRepo;
use crate::error::{ApiErrorResponse, AppError};

#[derive(Clone)]
pub struct JwtSecret(pub String);

impl JwtSecret {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for JwtSecret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("JwtSecret").field(&"<redacted>").finish()
    }
}

/// JWT payload claims extracted from the access token.
///
/// The token carries identity (`sub`/`username`/`tenant_id`) plus a role and
/// permission **snapshot** taken at issue time. Authorization is *not* trusted
/// from the token: [`auth_middleware`] re-resolves the user's role and
/// permissions from the DB on every request, so changes take effect
/// immediately. The snapshot fields are kept for backward compatibility with
/// old tokens and for frontend display only.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: i64,
    pub tenant_id: i64,
    pub username: String,
    pub role: String,
    pub permissions: Vec<String>,
    pub exp: usize,
    pub iat: usize,
}

/// Authenticated user context injected into request extensions by [`auth_middleware`].
///
/// Downstream handlers and middlewares extract this via `Extension<AuthContext>`
/// to access the current user's identity, tenant scope, and effective permissions.
/// `role` and `permissions` are freshly resolved from the DB on every request.
#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user_id: i64,
    pub tenant_id: i64,
    pub username: String,
    pub role: String,
    pub permissions: Vec<String>,
}

/// Axum extractor for the authenticated user — reads the [`AuthContext`] that
/// [`auth_middleware`] inserted into request extensions.
///
/// Handlers accept `AuthenticatedUser` as an extractor argument instead of
/// reaching into `Extension<AuthContext>` directly. Rejects with 401
/// (`AppError::Unauthorized`) when no auth context is present.
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

fn err_response(status: StatusCode, code: u32, message: &str) -> Response {
    (
        status,
        Json(ApiErrorResponse {
            success: false,
            code,
            request_id: format!("req_{}", Uuid::new_v4()),
            message: message.to_string(),
            details: None,
        }),
    )
        .into_response()
}

/// Axum middleware that validates a Bearer JWT from the `Authorization` header.
///
/// On success, inserts an [`AuthContext`] into request extensions for downstream use.
/// The user's role and permissions are resolved from the DB **on every request**,
/// so role changes, account deactivation, and deletion take effect immediately —
/// no need to wait for the access token to expire.
///
/// On failure, returns 401 with an `ApiErrorResponse` (code 11001 for invalid/missing
/// token, 11002 for expired signature), or 500 if the DB lookup cannot complete.
pub async fn auth_middleware(mut req: Request, next: Next) -> Response {
    let Some(jwt_secret) = req.extensions().get::<JwtSecret>() else {
        return err_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            50001,
            "Authentication is not configured",
        );
    };

    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));

    let token = match auth_header {
        Some(t) => t,
        None => {
            return err_response(StatusCode::UNAUTHORIZED, 11001, "Missing authorization token")
        }
    };

    let claims = match decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_str().as_bytes()),
        &Validation::default(),
    ) {
        Ok(data) => data.claims,
        Err(e) => {
            let (code, msg) = match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                    (11002, "Token expired".to_string())
                }
                _ => (11001, "Invalid token".to_string()),
            };
            return err_response(StatusCode::UNAUTHORIZED, code, &msg);
        }
    };

    // Re-resolve role/permissions from the DB. The pool lives on request
    // extensions because `Extension(pool)` is layered outside this middleware.
    let Some(pool) = req.extensions().get::<SqlitePool>() else {
        return err_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            50001,
            "Authentication is not configured",
        );
    };

    let role_row: Option<(String, bool)> =
        match sqlx::query_as("SELECT role, is_active FROM users WHERE id = ? AND deleted_at IS NULL")
            .bind(claims.sub)
            .fetch_optional(pool)
            .await
        {
            Ok(row) => row,
            Err(e) => {
                tracing::error!("Failed to resolve user {} role: {}", claims.sub, e);
                return err_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    50001,
                    "Failed to resolve user permissions",
                );
            }
        };

    let Some((role, is_active)) = role_row else {
        return err_response(
            StatusCode::UNAUTHORIZED,
            11001,
            "Account no longer exists",
        );
    };
    if !is_active {
        return err_response(StatusCode::FORBIDDEN, 11003, "Account is disabled");
    }

    let permissions = match UserRoleRepo::permission_keys_for_user(pool, claims.sub).await {
        Ok(p) => p,
        Err(e) => {
            tracing::error!("Failed to resolve user {} permissions: {}", claims.sub, e);
            return err_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                50001,
                "Failed to resolve user permissions",
            );
        }
    };

    let ctx = AuthContext {
        user_id: claims.sub,
        tenant_id: claims.tenant_id,
        username: claims.username,
        role,
        permissions,
    };
    req.extensions_mut().insert(ctx);
    next.run(req).await
}
