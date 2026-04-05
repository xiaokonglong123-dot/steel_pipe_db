use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteelPipe {
    pub id: Option<i64>,
    pub pipe_id: String,
    pub diameter: f64,
    pub thickness: f64,
    pub length: f64,
    pub material: String,
    pub quantity: i32,
    pub location: Option<String>,
    pub supplier: Option<String>,
    pub entry_date: String,
    pub last_update: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryRecord {
    pub id: Option<i64>,
    pub pipe_id: String,
    pub operation_type: String,
    pub quantity: i32,
    pub operation_date: String,
    pub operator: String,
    pub remarks: Option<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Statistics {
    pub total_types: i32,
    pub total_quantity: i32,
    pub total_in: i32,
    pub total_out: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialStats {
    pub material: String,
    pub type_count: i32,
    pub total_quantity: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationLog {
    pub id: i64,
    pub operation_type: String,
    pub target_type: String,
    pub target_id: String,
    pub snapshot_before: String,
    pub snapshot_after: String,
    pub operator: String,
    pub timestamp: String,
    pub remarks: String,
}
