//! Threading row models — mirror `033_create_threading.sql`.

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow)]
pub struct ThreadingRecord {
    pub id: i64,
    pub tenant_id: i64,
    pub pipe_id: Option<i64>,
    pub pipe_number: Option<String>,
    pub thread_type: String,
    pub od: f64,
    pub wt: f64,
    pub grade: Option<String>,
    pub threads_per_inch: Option<f64>,
    pub pitch_diameter: Option<f64>,
    pub makeup_torque: Option<f64>,
    pub machined_at: DateTime<Utc>,
    pub operator: Option<i64>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct ThreadGeometryCache {
    pub id: i64,
    pub tenant_id: i64,
    pub pipe_type: String,
    pub od: f64,
    pub wt: f64,
    pub grade: String,
    pub connection_type: String,
    pub joint_efficiency: f64,
    pub burst_pressure: f64,
    pub collapse_pressure: f64,
    pub tension_capacity: f64,
    pub created_at: DateTime<Utc>,
}
