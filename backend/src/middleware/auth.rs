use std::fmt;

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::ApiErrorResponse;

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
/// Contains the authenticated user's identity and token metadata
/// (issued-at and expiration timestamps).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: i64,
    pub username: String,
    pub role: String,
    pub exp: usize,
    pub iat: usize,
}

/// Authenticated user context injected into request extensions by [`auth_middleware`].
///
/// Downstream handlers and middlewares extract this via `Extension<AuthContext>`
/// to access the current user's identity and role.
#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user_id: i64,
    pub username: String,
    pub role: String,
}

/// Axum middleware that validates a Bearer JWT from the `Authorization` header.
///
/// On success, inserts an [`AuthContext`] into request extensions for downstream use.
/// On failure, returns 401 with an `ApiErrorResponse` (code 11001 for invalid/missing
/// token, 11002 for expired signature).
pub async fn auth_middleware(
    mut req: Request,
    next: Next,
) -> Response {
    let Some(jwt_secret) = req.extensions().get::<JwtSecret>() else {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiErrorResponse {
            success: false,
            code: 50001,
            request_id: format!("req_{}", Uuid::new_v4()),
            message: "Authentication is not configured".to_string(),
            details: None,
        }))
            .into_response();
    };

    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));

    let token = match auth_header {
        Some(t) => t,
        None => {
            return (StatusCode::UNAUTHORIZED, Json(ApiErrorResponse {
                success: false,
                code: 11001,
                request_id: format!("req_{}", Uuid::new_v4()),
                message: "Missing authorization token".to_string(),
                details: None,
            })).into_response()
        }
    };

    match decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_str().as_bytes()),
        &Validation::default(),
    ) {
        Ok(data) => {
            let ctx = AuthContext {
                user_id: data.claims.sub,
                username: data.claims.username,
                role: data.claims.role,
            };
            req.extensions_mut().insert(ctx);
            next.run(req).await
        }
        Err(e) => {
            let (code, msg) = match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                    (11002, "Token expired".to_string())
                }
                _ => (11001, "Invalid token".to_string()),
            };
            (StatusCode::UNAUTHORIZED, Json(ApiErrorResponse {
                success: false,
                code,
                request_id: format!("req_{}", Uuid::new_v4()),
                message: msg,
                details: None,
            })).into_response()
        }
    }
}
