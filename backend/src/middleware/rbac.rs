use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use uuid::Uuid;

use crate::error::ApiErrorResponse;

pub async fn require_role(
    req: Request,
    next: Next,
    allowed_roles: &'static [&'static str],
) -> Response {
    let auth_ctx = req
        .extensions()
        .get::<super::auth::AuthContext>();

    match auth_ctx {
        Some(ctx) if allowed_roles.contains(&ctx.role.as_str()) => {
            next.run(req).await
        }
        Some(_) => {
            (StatusCode::FORBIDDEN, Json(ApiErrorResponse {
                success: false,
                code: 11003,
                request_id: format!("req_{}", Uuid::new_v4()),
                message: "Insufficient permissions".to_string(),
                details: None,
            })).into_response()
        }
        None => {
            (StatusCode::UNAUTHORIZED, Json(ApiErrorResponse {
                success: false,
                code: 11001,
                request_id: format!("req_{}", Uuid::new_v4()),
                message: "Authentication required".to_string(),
                details: None,
            })).into_response()
        }
    }
}
