use serde::{Deserialize, Serialize};
use validator::Validate;

/// Create seamless pipe request DTO.
#[derive(Debug, Deserialize, Validate)]
pub struct CreateSeamlessPipeRequest {
    /// Pipe number (auto-generated if empty).
    pub pipe_number: Option<String>,
    /// Batch number.
    pub batch_number: Option<String>,
    /// Pipe type: casing or tubing.
    #[validate(length(min = 1))]
    pub pipe_type: String,
    /// Steel grade: J55 / N80 / L80 / P110, etc.
    #[validate(length(min = 1))]
    pub grade: String,
    /// Outer diameter (mm).
    #[validate(range(min = 0.0))]
    pub od: f64,
    /// Wall thickness (mm).
    #[validate(range(min = 0.0))]
    pub wt: f64,
    /// Length (m).
    pub length: Option<f64>,
    /// Weight per unit length (kg/m).
    pub weight_per_unit: Option<f64>,
    /// End type: STC / LTC / BTC.
    pub end_type: Option<String>,
    /// Coupling type.
    pub coupling_type: Option<String>,
    /// Coupling OD (mm).
    pub coupling_od: Option<f64>,
    /// Coupling length (mm).
    pub coupling_length: Option<f64>,
    /// Heat number.
    pub heat_number: Option<String>,
    /// Serial number.
    pub serial_number: Option<String>,
    /// Manufacturer name.
    pub manufacturer: Option<String>,
    /// Production date.
    pub production_date: Option<String>,
    /// Mill cert number.
    pub cert_number: Option<String>,
    /// Notes.
    pub notes: Option<String>,
}

/// Update seamless pipe request DTO (all fields optional — only provided fields are updated).
#[derive(Debug, Deserialize, Validate)]
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

/// Create screen pipe request DTO.
#[derive(Debug, Deserialize, Validate)]
pub struct CreateScreenPipeRequest {
    /// Pipe number (auto-generated if empty).
    pub pipe_number: Option<String>,
    /// Batch number.
    pub batch_number: Option<String>,
    /// Screen type: wire_wrapped / slotted / premium.
    #[validate(length(min = 1))]
    pub screen_type: String,
    /// Slot / aperture size (mm).
    pub slot_size: Option<f64>,
    /// Filtration grade.
    pub filtration_grade: Option<String>,
    /// Base pipe outer diameter (mm).
    #[validate(range(min = 0.0))]
    pub base_od: f64,
    /// Base pipe wall thickness (mm).
    #[validate(range(min = 0.0))]
    pub base_wt: f64,
    /// Base pipe steel grade.
    #[validate(length(min = 1))]
    pub base_grade: String,
    /// Base pipe end type.
    pub base_end_type: Option<String>,
    /// Length (m).
    pub length: Option<f64>,
    /// Weight per unit length (kg/m).
    pub weight_per_unit: Option<f64>,
    /// Heat number.
    pub heat_number: Option<String>,
    /// Serial number.
    pub serial_number: Option<String>,
    /// Manufacturer name.
    pub manufacturer: Option<String>,
    /// Production date.
    pub production_date: Option<String>,
    /// Mill cert number.
    pub cert_number: Option<String>,
    /// Notes.
    pub notes: Option<String>,
}

/// Update screen pipe request DTO (all fields optional — only provided fields are updated).
#[derive(Debug, Deserialize, Validate)]
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

/// Pipe list filter params DTO.
#[derive(Debug, Deserialize)]
pub struct PipeFilterParams {
    /// Full-text search (matches pipe number / batch / heat, etc.).
    pub q: Option<String>,
    /// Filter by steel grade.
    pub grade: Option<String>,
    /// Filter by pipe type.
    pub pipe_type: Option<String>,
    /// Filter by status.
    pub status: Option<String>,
    /// Min outer diameter (mm).
    pub od_min: Option<f64>,
    /// Max outer diameter (mm).
    pub od_max: Option<f64>,
    /// Min wall thickness (mm).
    pub wt_min: Option<f64>,
    /// Max wall thickness (mm).
    pub wt_max: Option<f64>,
    /// Filter by location.
    pub location_id: Option<i64>,
    /// Filter by manufacturer.
    pub manufacturer: Option<String>,
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

/// Pipe search result DTO (unified wrapper for seamless and screen types).
#[derive(Debug, Serialize)]
pub struct PipeSearchResult {
    /// Pipe type: seamless or screen.
    pub pipe_type: String,
    /// Pipe data (JSON object, shape depends on pipe_type).
    pub pipe: serde_json::Value,
}
