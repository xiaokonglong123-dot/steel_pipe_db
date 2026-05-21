use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

use crate::error::ApiErrorResponse;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: i64,
    pub username: String,
    pub role: String,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user_id: i64,
    pub username: String,
    pub role: String,
}

pub async fn auth_middleware(
    mut req: Request,
    next: Next,
) -> Response {
    let jwt_secret = req
        .extensions()
        .get::<String>()
        .cloned()
        .unwrap_or_default();

    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));

    let token = match auth_header {
        Some(t) => t,
        None => {
            return (StatusCode::UNAUTHORIZED, Json(ApiErrorResponse {
                code: 11001,
                message: "Missing authorization token".to_string(),
                details: None,
            })).into_response()
        }
    };

    match decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_bytes()),
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
                code,
                message: msg,
                details: None,
            })).into_response()
        }
    }
}
