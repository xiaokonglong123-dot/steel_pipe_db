use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
pub struct CreateSeamlessPipeRequest {
    pub pipe_number: Option<String>,
    pub batch_number: Option<String>,
    pub pipe_type: Option<String>,
    #[validate(length(min = 1, message = "grade is required"))]
    pub grade: String,
    #[validate(range(min = 0.1, message = "od must be positive"))]
    pub od: f64,
    #[validate(range(min = 0.01, message = "wt must be positive"))]
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

#[derive(Debug, Clone, Deserialize, Validate)]
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
    pub location_id: Option<i64>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
pub struct CreateScreenPipeRequest {
    pub pipe_number: Option<String>,
    pub batch_number: Option<String>,
    pub screen_type: Option<String>,
    pub slot_size: Option<f64>,
    pub filtration_grade: Option<String>,
    #[validate(range(min = 0.1, message = "base_od must be positive"))]
    pub base_od: f64,
    #[validate(range(min = 0.01, message = "base_wt must be positive"))]
    pub base_wt: f64,
    #[validate(length(min = 1, message = "base_grade is required"))]
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

#[derive(Debug, Clone, Deserialize, Validate)]
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
    pub location_id: Option<i64>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
pub struct CreateWeldedPipeRequest {
    pub pipe_number: Option<String>,
    pub batch_number: Option<String>,
    pub pipe_type: Option<String>,
    #[validate(length(min = 1, message = "grade is required"))]
    pub grade: String,
    #[validate(range(min = 0.1, message = "od must be positive"))]
    pub od: f64,
    #[validate(range(min = 0.01, message = "wt must be positive"))]
    pub wt: f64,
    pub length: Option<f64>,
    pub weight_per_unit: Option<f64>,
    pub end_type: Option<String>,
    pub seam_type: Option<String>,
    pub heat_number: Option<String>,
    pub serial_number: Option<String>,
    pub manufacturer: Option<String>,
    pub production_date: Option<String>,
    pub cert_number: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct UpdateWeldedPipeRequest {
    pub batch_number: Option<String>,
    pub pipe_type: Option<String>,
    pub grade: Option<String>,
    pub od: Option<f64>,
    pub wt: Option<f64>,
    pub length: Option<f64>,
    pub weight_per_unit: Option<f64>,
    pub end_type: Option<String>,
    pub seam_type: Option<String>,
    pub heat_number: Option<String>,
    pub serial_number: Option<String>,
    pub manufacturer: Option<String>,
    pub production_date: Option<String>,
    pub cert_number: Option<String>,
    pub notes: Option<String>,
    pub location_id: Option<i64>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
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
    pub heat_number: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PipeSearchResult {
    pub id: i64,
    pub pipe_type: String,
    pub pipe_number: String,
    pub grade: String,
    pub od: f64,
    pub wt: f64,
    pub status: String,
    pub location_id: Option<i64>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
pub struct BatchCreatePipeRequest {
    #[validate(length(min = 1, message = "pipe_type is required"))]
    pub pipe_type: String,
    #[validate(length(min = 1, message = "grade is required"))]
    pub grade: String,
    #[validate(range(min = 0.1, message = "od must be positive"))]
    pub od: f64,
    #[validate(range(min = 0.01, message = "wt must be positive"))]
    pub wt: f64,
    #[validate(range(min = 1, message = "quantity must be at least 1"))]
    pub quantity: i64,
    pub batch_number: Option<String>,
    pub length: Option<f64>,
    pub end_type: Option<String>,
    pub manufacturer: Option<String>,
    pub heat_number: Option<String>,
    pub notes: Option<String>,
}
