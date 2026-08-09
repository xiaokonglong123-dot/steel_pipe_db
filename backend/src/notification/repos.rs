//! Notification repositories.

use sqlx::SqlitePool;
use crate::models::notification::{Notification, NotificationPreference, NotificationTemplate};

pub struct NotificationRepo;

impl NotificationRepo {
    pub async fn create(
        pool: &SqlitePool,
        tenant_id: i64,
        user_id: i64,
        title: &str,
        content: Option<&str>,
        notify_type: &str,
    ) -> Result<Notification, sqlx::Error> {
        sqlx::query_as::<_, Notification>(
            "INSERT INTO notifications (tenant_id, user_id, title, content, notify_type) \
             VALUES (?, ?, ?, ?, ?) \
             RETURNING id, tenant_id, user_id, title, content, notify_type, is_read, created_at, read_at",
        )
        .bind(tenant_id)
        .bind(user_id)
        .bind(title)
        .bind(content)
        .bind(notify_type)
        .fetch_one(pool)
        .await
    }

    pub async fn list(pool: &SqlitePool, tenant_id: i64, user_id: i64, unread_only: bool, limit: i64) -> Result<Vec<Notification>, sqlx::Error> {
        sqlx::query_as::<_, Notification>(
            "SELECT id, tenant_id, user_id, title, content, notify_type, is_read, created_at, read_at \
             FROM notifications WHERE tenant_id = ? AND user_id = ? \
             AND (? = 0 OR is_read = 0) \
             ORDER BY id DESC LIMIT ?",
        )
        .bind(tenant_id)
        .bind(user_id)
        .bind(unread_only)
        .bind(limit)
        .fetch_all(pool)
        .await
    }

    pub async fn mark_read(pool: &SqlitePool, tenant_id: i64, user_id: i64, id: i64) -> Result<Option<Notification>, sqlx::Error> {
        sqlx::query_as::<_, Notification>(
            "UPDATE notifications SET is_read = 1, read_at = datetime('now') \
             WHERE tenant_id = ? AND user_id = ? AND id = ? \
             RETURNING id, tenant_id, user_id, title, content, notify_type, is_read, created_at, read_at",
        )
        .bind(tenant_id)
        .bind(user_id)
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    pub async fn unread_count(pool: &SqlitePool, tenant_id: i64, user_id: i64) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar("SELECT COUNT(*) FROM notifications WHERE tenant_id = ? AND user_id = ? AND is_read = 0")
            .bind(tenant_id)
            .bind(user_id)
            .fetch_one(pool)
            .await
    }
}

pub struct PreferenceRepo;

impl PreferenceRepo {
    pub async fn upsert(
        pool: &SqlitePool,
        user_id: i64,
        notify_type: &str,
        channel: &str,
        enabled: bool,
    ) -> Result<NotificationPreference, sqlx::Error> {
        sqlx::query_as::<_, NotificationPreference>(
            "INSERT INTO notification_preferences (user_id, notify_type, channel, enabled) \
             VALUES (?, ?, ?, ?) \
             ON CONFLICT (user_id, notify_type, channel) DO UPDATE SET enabled = EXCLUDED.enabled \
             RETURNING id, user_id, notify_type, channel, enabled, created_at",
        )
        .bind(user_id)
        .bind(notify_type)
        .bind(channel)
        .bind(enabled)
        .fetch_one(pool)
        .await
    }

    pub async fn list(pool: &SqlitePool, user_id: i64) -> Result<Vec<NotificationPreference>, sqlx::Error> {
        sqlx::query_as::<_, NotificationPreference>(
            "SELECT id, user_id, notify_type, channel, enabled, created_at \
             FROM notification_preferences WHERE user_id = ? ORDER BY notify_type",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
    }
}

pub struct TemplateRepo;

impl TemplateRepo {
    pub async fn create(
        pool: &SqlitePool,
        tenant_id: i64,
        code: &str,
        title: &str,
        content_template: &str,
        channel: &str,
    ) -> Result<NotificationTemplate, sqlx::Error> {
        sqlx::query_as::<_, NotificationTemplate>(
            "INSERT INTO notification_templates (tenant_id, code, title, content_template, channel) \
             VALUES (?, ?, ?, ?, ?) ON CONFLICT (tenant_id, code) DO UPDATE SET \
               title = EXCLUDED.title, content_template = EXCLUDED.content_template \
             RETURNING id, tenant_id, code, title, content_template, channel, created_at",
        )
        .bind(tenant_id)
        .bind(code)
        .bind(title)
        .bind(content_template)
        .bind(channel)
        .fetch_one(pool)
        .await
    }
}
