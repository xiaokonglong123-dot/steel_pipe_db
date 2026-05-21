use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SeamlessPipe {
    pub id: i64,
    pub pipe_number: String,
    pub batch_number: Option<String>,
    pub pipe_type: String,
    pub grade: String,
    pub od: f64,
    pub wt: f64,
    pub length: Option<f64>,
    pub weight_per_unit: Option<f64>,
    pub end_type: Option<String>,
    pub coupling_type: Option<String>,
    pub coupling_od: Option<f64>,
    pub coupling_length: Option<f64>,
    pub heat_number: Option<String>,
    pub serial_number: Option<String>,
    pub manufacturer: Option<String>,
    pub production_date: Option<String>,
    pub cert_number: Option<String>,
    pub location_id: Option<i64>,
    pub status: String,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}
