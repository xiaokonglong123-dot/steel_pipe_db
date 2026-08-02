use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// A tenant (organization) that scopes all business data.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Tenant {
    pub id: i64,
    pub name: String,
    pub domain: String,
    pub config: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// A department inside a tenant; supports a self-referencing hierarchy.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Department {
    pub id: i64,
    pub tenant_id: i64,
    pub name: String,
    pub parent_id: Option<i64>,
    pub path: Option<String>,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// A role within a tenant; grants permissions via role_permissions.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Role {
    pub id: i64,
    pub tenant_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub is_system: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// A single permission key in the global dictionary (e.g. `purchase.order.write`).
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Permission {
    pub id: i64,
    pub key: String,
    pub description: Option<String>,
}
