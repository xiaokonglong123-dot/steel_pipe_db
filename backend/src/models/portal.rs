//! Portal row models — mirror `037_create_portal.sql`.

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow)]
pub struct PortalAccount {
    pub id: i64,
    pub tenant_id: i64,
    pub party_type: String,
    pub party_id: i64,
    pub username: String,
    pub password_hash: String,
    pub is_active: bool,
    pub last_login_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct PortalEvent {
    pub id: i64,
    pub tenant_id: i64,
    pub party_type: String,
    pub party_id: i64,
    pub event_type: String,
    pub ref_id: i64,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}
