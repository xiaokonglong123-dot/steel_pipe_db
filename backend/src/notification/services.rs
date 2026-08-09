//! Notification services — send, list, mark read, preferences.

use sqlx::SqlitePool;

use crate::dto::notification_dto::{
    CreateTemplateRequest, SendNotificationRequest, UpdatePreferenceRequest,
};
use crate::error::AppError;
use crate::models::notification::{Notification, NotificationPreference, NotificationTemplate};
use crate::notification::repos::{NotificationRepo, PreferenceRepo, TemplateRepo};

pub struct NotificationService;

impl NotificationService {
    pub async fn send(
        pool: &SqlitePool,
        tenant_id: i64,
        dto: &SendNotificationRequest,
    ) -> Result<Notification, AppError> {
        if dto.title.trim().is_empty() {
            return Err(AppError::Validation("Notification title is required".into()));
        }
        let notify_type = dto.notify_type.clone().unwrap_or_else(|| "system".into());
        NotificationRepo::create(pool, tenant_id, dto.user_id, dto.title.trim(), dto.content.as_deref(), &notify_type)
            .await
            .map_err(AppError::from)
    }

    /// Send via template: render {placeholders} with values.
    pub async fn send_from_template(
        pool: &SqlitePool,
        tenant_id: i64,
        user_id: i64,
        template_code: &str,
        placeholders: &[(String, String)],
    ) -> Result<Notification, AppError> {
        let template = sqlx::query_as::<_, NotificationTemplate>(
            "SELECT id, tenant_id, code, title, content_template, channel, created_at \
             FROM notification_templates WHERE tenant_id = ? AND code = ?",
        )
        .bind(tenant_id)
        .bind(template_code)
        .fetch_optional(pool)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound(format!("Notification template not found: {}", template_code)))?;

        let mut content = template.content_template.clone();
        for (k, v) in placeholders {
            content = content.replace(&format!("{{{}}}", k), v);
        }
        NotificationRepo::create(pool, tenant_id, user_id, &template.title, Some(&content), "system")
            .await
            .map_err(AppError::from)
    }

    pub async fn list(pool: &SqlitePool, tenant_id: i64, user_id: i64, unread_only: bool) -> Result<Vec<Notification>, AppError> {
        NotificationRepo::list(pool, tenant_id, user_id, unread_only, 100)
            .await
            .map_err(AppError::from)
    }

    pub async fn mark_read(pool: &SqlitePool, tenant_id: i64, user_id: i64, id: i64) -> Result<Notification, AppError> {
        NotificationRepo::mark_read(pool, tenant_id, user_id, id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Notification not found: {}", id)))
    }

    pub async fn unread_count(pool: &SqlitePool, tenant_id: i64, user_id: i64) -> Result<i64, AppError> {
        NotificationRepo::unread_count(pool, tenant_id, user_id).await.map_err(AppError::from)
    }

    pub async fn list_preferences(pool: &SqlitePool, user_id: i64) -> Result<Vec<NotificationPreference>, AppError> {
        PreferenceRepo::list(pool, user_id).await.map_err(AppError::from)
    }

    pub async fn update_preference(pool: &SqlitePool, user_id: i64, dto: &UpdatePreferenceRequest) -> Result<NotificationPreference, AppError> {
        let channel = dto.channel.clone().unwrap_or_else(|| "in_app".into());
        PreferenceRepo::upsert(pool, user_id, &dto.notify_type, &channel, dto.enabled)
            .await
            .map_err(AppError::from)
    }

    pub async fn create_template(pool: &SqlitePool, tenant_id: i64, dto: &CreateTemplateRequest) -> Result<NotificationTemplate, AppError> {
        if dto.code.trim().is_empty() || dto.title.trim().is_empty() {
            return Err(AppError::Validation("Template code and title are required".into()));
        }
        TemplateRepo::create(pool, tenant_id, dto.code.trim(), dto.title.trim(), &dto.content_template, dto.channel.as_deref().unwrap_or("in_app"))
            .await
            .map_err(AppError::from)
    }
}
