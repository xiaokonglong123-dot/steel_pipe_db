//! Notification row models — mirror `036_create_notifications.sql`.

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow)]
pub struct Notification {
    pub id: i64,
    pub tenant_id: i64,
    pub user_id: i64,
    pub title: String,
    pub content: Option<String>,
    pub notify_type: String,
    pub is_read: bool,
    pub created_at: DateTime<Utc>,
    pub read_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct NotificationPreference {
    pub id: i64,
    pub user_id: i64,
    pub notify_type: String,
    pub channel: String,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct NotificationTemplate {
    pub id: i64,
    pub tenant_id: i64,
    pub code: String,
    pub title: String,
    pub content_template: String,
    pub channel: String,
    pub created_at: DateTime<Utc>,
}
