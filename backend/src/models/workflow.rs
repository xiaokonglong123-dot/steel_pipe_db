//! Workflow row models — sqlx `FromRow` structs mirroring `026_create_workflow.sql`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// A workflow definition template (ordered approval nodes).
#[derive(Debug, Serialize, FromRow)]
pub struct WorkflowDefinition {
    pub id: i64,
    pub tenant_id: i64,
    pub name: String,
    pub entity_type: String,
    pub description: Option<String>,
    pub definition_json: serde_json::Value,
    pub callback_action: Option<String>,
    pub version: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// A running (or finished) workflow instance for one business entity.
#[derive(Debug, Serialize, FromRow)]
pub struct WorkflowInstance {
    pub id: i64,
    pub definition_id: i64,
    pub tenant_id: i64,
    pub entity_type: String,
    pub entity_id: i64,
    pub amount: Option<rust_decimal::Decimal>,
    pub status: String,
    pub current_step: i32,
    pub initiator_id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
}

/// One approval step within an instance.
#[derive(Debug, Serialize, FromRow)]
pub struct ApprovalNode {
    pub id: i64,
    pub instance_id: i64,
    pub step_index: i32,
    pub node_key: String,
    pub assignee_type: String,
    pub assignee_value: Option<String>,
    pub condition_json: Option<serde_json::Value>,
    pub status: String,
    pub approver_id: Option<i64>,
    pub approval_reason: Option<String>,
    pub due_date: Option<DateTime<Utc>>,
    pub decided_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Delegation — one user proxies another's approvals for a period.
#[derive(Debug, Serialize, FromRow)]
pub struct WorkflowDelegation {
    pub id: i64,
    pub original_user_id: i64,
    pub delegated_user_id: i64,
    pub entity_type: Option<String>,
    pub starts_at: DateTime<Utc>,
    pub ends_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

/// A node's node_key + optional condition, used when instantiating a
/// definition into per-instance approval_nodes rows.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefinitionNode {
    pub node_key: String,
    pub assignee_type: String,
    pub assignee_value: Option<String>,
    pub condition: Option<serde_json::Value>,
}
