use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRoleRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRoleRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignPermissionsRequest {
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignUserRolesRequest {
    pub role_ids: Vec<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDepartmentRequest {
    pub name: String,
    pub parent_id: Option<i64>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateDepartmentRequest {
    pub name: Option<String>,
    pub parent_id: Option<i64>,
    pub sort_order: Option<i32>,
}
