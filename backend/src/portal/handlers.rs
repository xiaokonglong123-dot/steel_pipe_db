//! Portal HTTP handlers. Two tiers:
//! - `/api/v1/portal/*` — admin management (create accounts)
//! - `/api/v1/portal-api/*` — portal JWT endpoints (login + party-scoped views)

use axum::extract::{Extension, Path, Query};
use axum::response::Response;
use axum::Json;
use sqlx::SqlitePool;

use crate::dto::portal_dto::{AcceptPurchaseRequest, CreatePortalAccountRequest, PortalLoginRequest};
use crate::error::AppError;
use crate::middleware::auth::AuthenticatedUser;
use crate::middleware::auth::JwtSecret;
use crate::models::portal::PortalEvent;
use crate::portal::services::{PortalPurchaseRow, PortalSalesRow, PortalService};
use crate::response::ApiResponse;

// ---------------------------------------------------------------------------
// Admin management (internal auth)
// ---------------------------------------------------------------------------

pub async fn create_account(
    Extension(pool): Extension<SqlitePool>,
    user: AuthenticatedUser,
    Json(p): Json<CreatePortalAccountRequest>,
) -> Result<Response, AppError> {
    Ok(ApiResponse::created(PortalService::create_account(&pool, user.0.tenant_id, &p).await?))
}

// ---------------------------------------------------------------------------
// Portal API (portal JWT)
// ---------------------------------------------------------------------------

#[derive(Debug, serde::Deserialize)]
pub struct PortalClaims {
    pub sub: i64,
    pub username: String,
    pub party_type: String,
    pub party_id: i64,
    pub portal: bool,
}

/// Extract party context from a portal JWT (Bearer token in Authorization).
fn portal_context(headers: &axum::http::HeaderMap, jwt_secret: &JwtSecret) -> Result<(String, i64), AppError> {
    let auth = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| AppError::Unauthorized("Missing bearer token".into()))?;
    let data = jsonwebtoken::decode::<PortalClaims>(
        auth,
        &jsonwebtoken::DecodingKey::from_secret(jwt_secret.0.as_bytes()),
        &jsonwebtoken::Validation::default(),
    )
    .map_err(|_| AppError::Unauthorized("Invalid portal token".into()))?;
    if !data.claims.portal {
        return Err(AppError::Unauthorized("Not a portal token".into()));
    }
    Ok((data.claims.party_type, data.claims.party_id))
}

pub async fn portal_login(
    Extension(pool): Extension<SqlitePool>,
    Extension(jwt_secret): Extension<JwtSecret>,
    Json(p): Json<PortalLoginRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let (token, account) = PortalService::login(&pool, &p, &jwt_secret).await?;
    Ok(ApiResponse::ok(serde_json::json!({
        "token": token,
        "party_type": account.party_type,
        "party_id": account.party_id,
        "username": account.username,
    })))
}

pub async fn portal_purchases(
    Extension(pool): Extension<SqlitePool>,
    Extension(jwt_secret): Extension<JwtSecret>,
    headers: axum::http::HeaderMap,
) -> Result<Json<ApiResponse<Vec<PortalPurchaseRow>>>, AppError> {
    let (party_type, party_id) = portal_context(&headers, &jwt_secret)?;
    if party_type != "supplier" {
        return Err(AppError::Forbidden("Supplier account required".into()));
    }
    Ok(ApiResponse::ok(PortalService::supplier_purchases(&pool, 1, party_id).await?))
}

pub async fn portal_accept_purchase(
    Extension(pool): Extension<SqlitePool>,
    Extension(jwt_secret): Extension<JwtSecret>,
    headers: axum::http::HeaderMap,
    Path(id): Path<i64>,
    Json(p): Json<AcceptPurchaseRequest>,
) -> Result<Json<ApiResponse<PortalEvent>>, AppError> {
    let (party_type, party_id) = portal_context(&headers, &jwt_secret)?;
    if party_type != "supplier" {
        return Err(AppError::Forbidden("Supplier account required".into()));
    }
    Ok(ApiResponse::ok(PortalService::accept_purchase(&pool, 1, party_id, id, &p).await?))
}

pub async fn portal_sales(
    Extension(pool): Extension<SqlitePool>,
    Extension(jwt_secret): Extension<JwtSecret>,
    headers: axum::http::HeaderMap,
) -> Result<Json<ApiResponse<Vec<PortalSalesRow>>>, AppError> {
    let (party_type, party_id) = portal_context(&headers, &jwt_secret)?;
    if party_type != "customer" {
        return Err(AppError::Forbidden("Customer account required".into()));
    }
    Ok(ApiResponse::ok(PortalService::customer_sales(&pool, 1, party_id).await?))
}

pub async fn portal_acknowledge_sales(
    Extension(pool): Extension<SqlitePool>,
    Extension(jwt_secret): Extension<JwtSecret>,
    headers: axum::http::HeaderMap,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<PortalEvent>>, AppError> {
    let (party_type, party_id) = portal_context(&headers, &jwt_secret)?;
    if party_type != "customer" {
        return Err(AppError::Forbidden("Customer account required".into()));
    }
    Ok(ApiResponse::ok(PortalService::acknowledge_sales(&pool, 1, party_id, id).await?))
}

pub async fn portal_events(
    Extension(pool): Extension<SqlitePool>,
    Extension(jwt_secret): Extension<JwtSecret>,
    headers: axum::http::HeaderMap,
    Query(_f): Query<serde_json::Value>,
) -> Result<Json<ApiResponse<Vec<PortalEvent>>>, AppError> {
    let (party_type, party_id) = portal_context(&headers, &jwt_secret)?;
    Ok(ApiResponse::ok(PortalService::events(&pool, 1, &party_type, party_id).await?))
}
