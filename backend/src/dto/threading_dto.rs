//! Threading DTOs.

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CreateThreadingRecordRequest {
    pub pipe_id: Option<i64>,
    pub pipe_number: Option<String>,
    pub thread_type: String,
    pub od: f64,
    pub wt: f64,
    pub grade: Option<String>,
    pub threads_per_inch: Option<f64>,
    pub pitch_diameter: Option<f64>,
    pub makeup_torque: Option<f64>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ThreadCalcRequest {
    pub od: f64,
    pub wt: f64,
    pub grade: String,
    pub connection_type: String,   // round | buttress | premium
}

/// Casing design check inputs (single joint).
#[derive(Debug, Deserialize)]
pub struct DesignCheckRequest {
    pub od: f64,
    pub wt: f64,
    pub grade: String,
    pub connection_type: String,
    pub depth: f64,        // meters
    pub fluid_density: f64, // kg/m³ (default ~1025 seawater)
}
