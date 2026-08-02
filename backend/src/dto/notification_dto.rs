//! Notification DTOs.

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SendNotificationRequest {
    pub user_id: i64,
    pub title: String,
    pub content: Option<String>,
    pub notify_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePreferenceRequest {
    pub notify_type: String,
    pub channel: Option<String>,
    pub enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateTemplateRequest {
    pub code: String,
    pub title: String,
    pub content_template: String,
    pub channel: Option<String>,
}
