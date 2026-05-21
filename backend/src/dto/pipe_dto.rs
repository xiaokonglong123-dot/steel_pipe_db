use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreateSeamlessPipeRequest {
    pub pipe_number: Option<String>,
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
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSeamlessPipeRequest {
    pub batch_number: Option<String>,
    pub pipe_type: Option<String>,
    pub grade: Option<String>,
    pub od: Option<f64>,
    pub wt: Option<f64>,
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
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateScreenPipeRequest {
    pub pipe_number: Option<String>,
    pub batch_number: Option<String>,
    pub screen_type: String,
    pub slot_size: Option<f64>,
    pub filtration_grade: Option<String>,
    pub base_od: f64,
    pub base_wt: f64,
    pub base_grade: String,
    pub base_end_type: Option<String>,
    pub length: Option<f64>,
    pub weight_per_unit: Option<f64>,
    pub heat_number: Option<String>,
    pub serial_number: Option<String>,
    pub manufacturer: Option<String>,
    pub production_date: Option<String>,
    pub cert_number: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateScreenPipeRequest {
    pub batch_number: Option<String>,
    pub screen_type: Option<String>,
    pub slot_size: Option<f64>,
    pub filtration_grade: Option<String>,
    pub base_od: Option<f64>,
    pub base_wt: Option<f64>,
    pub base_grade: Option<String>,
    pub base_end_type: Option<String>,
    pub length: Option<f64>,
    pub weight_per_unit: Option<f64>,
    pub heat_number: Option<String>,
    pub serial_number: Option<String>,
    pub manufacturer: Option<String>,
    pub production_date: Option<String>,
    pub cert_number: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PipeFilterParams {
    pub q: Option<String>,
    pub grade: Option<String>,
    pub pipe_type: Option<String>,
    pub status: Option<String>,
    pub od_min: Option<f64>,
    pub od_max: Option<f64>,
    pub wt_min: Option<f64>,
    pub wt_max: Option<f64>,
    pub location_id: Option<i64>,
    pub manufacturer: Option<String>,
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PipeSearchResult {
    pub pipe_type: String,
    pub pipe: serde_json::Value,
}
