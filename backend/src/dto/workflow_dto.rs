//! Workflow DTOs — request payloads for definition/task endpoints.

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CreateDefinitionRequest {
    pub name: String,
    pub entity_type: String,
    pub description: Option<String>,
    /// Ordered nodes: [{node_key, assignee_type, assignee_value, condition}]
    pub nodes: Vec<serde_json::Value>,
    pub callback_action: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct StartInstanceRequest {
    pub definition_id: i64,
    pub entity_type: String,
    pub entity_id: i64,
    pub amount: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct ApproveTaskRequest {
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RejectTaskRequest {
    pub reason: String,
}

#[derive(Debug, Deserialize)]
pub struct DelegateTaskRequest {
    pub node_id: i64,
    pub delegated_user_id: i64,
    pub entity_type: Option<String>,
}
