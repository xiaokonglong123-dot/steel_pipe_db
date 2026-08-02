//! Notification HTTP handlers.

use axum::extract::{Extension, Path, Query};
use axum::Json;
use serde::Deserialize;
use sqlx::PgPool;

use crate::dto::notification_dto::{
    CreateTemplateRequest, SendNotificationRequest, UpdatePreferenceRequest,
};
use crate::error::AppError;
use crate::handlers::auth_handler::AuthenticatedUser;
use crate::models::notification::{Notification, NotificationPreference, NotificationTemplate};
use crate::notification::services::NotificationService;
use crate::response::ApiResponse;

#[derive(Debug, Deserialize)]
pub struct UnreadFilter {
    pub unread_only: Option<bool>,
}

pub async fn list_notifications(
    Extension(pool): Extension<PgPool>,
    user: AuthenticatedUser,
    Query(f): Query<UnreadFilter>,
) -> Result<Json<ApiResponse<Vec<Notification>>>, AppError> {
    Ok(ApiResponse::ok(NotificationService::list(&pool, user.0.tenant_id, user.0.user_id, f.unread_only.unwrap_or(false)).await?))
}

pub async fn unread_count(
    Extension(pool): Extension<PgPool>,
    user: AuthenticatedUser,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let n = NotificationService::unread_count(&pool, user.0.tenant_id, user.0.user_id).await?;
    Ok(ApiResponse::ok(serde_json::json!({ "unread": n })))
}

pub async fn mark_read(
    Extension(pool): Extension<PgPool>,
    user: AuthenticatedUser,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<Notification>>, AppError> {
    Ok(ApiResponse::ok(NotificationService::mark_read(&pool, user.0.tenant_id, user.0.user_id, id).await?))
}

pub async fn send_notification(
    Extension(pool): Extension<PgPool>,
    user: AuthenticatedUser,
    Json(p): Json<SendNotificationRequest>,
) -> Result<Json<ApiResponse<Notification>>, AppError> {
    // Only admin can push notifications to other users.
    if p.user_id != user.0.user_id && !user.0.permissions.contains(&"system.admin".to_string()) {
        return Err(AppError::Forbidden("Only admins may send notifications to other users".into()));
    }
    Ok(ApiResponse::ok(NotificationService::send(&pool, user.0.tenant_id, &p).await?))
}

pub async fn list_preferences(
    Extension(pool): Extension<PgPool>,
    user: AuthenticatedUser,
) -> Result<Json<ApiResponse<Vec<NotificationPreference>>>, AppError> {
    Ok(ApiResponse::ok(NotificationService::list_preferences(&pool, user.0.user_id).await?))
}

pub async fn update_preference(
    Extension(pool): Extension<PgPool>,
    user: AuthenticatedUser,
    Json(p): Json<UpdatePreferenceRequest>,
) -> Result<Json<ApiResponse<NotificationPreference>>, AppError> {
    Ok(ApiResponse::ok(NotificationService::update_preference(&pool, user.0.user_id, &p).await?))
}

pub async fn create_template(
    Extension(pool): Extension<PgPool>,
    user: AuthenticatedUser,
    Json(p): Json<CreateTemplateRequest>,
) -> Result<Json<ApiResponse<NotificationTemplate>>, AppError> {
    Ok(ApiResponse::ok(NotificationService::create_template(&pool, user.0.tenant_id, &p).await?))
}
