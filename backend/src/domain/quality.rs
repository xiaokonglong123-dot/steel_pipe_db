use serde::{Deserialize, Serialize};
use validator::Validate;

// ── DTOs for QualityCert ──

#[derive(Debug, Deserialize, Validate)]
pub struct CreateQualityCertDto {
    #[validate(length(min = 1, message = "cert_no is required"))]
    pub cert_no: String,

    #[validate(length(min = 1, message = "pipe_type is required"))]
    pub pipe_type: String, // "seamless" | "screen"

    #[validate(length(min = 1, message = "pipe_id is required"))]
    pub pipe_id: String,

    pub inspect_date: Option<String>,
    pub inspector: Option<String>,
    pub agency: Option<String>,
    pub result: Option<String>, // pass / fail / pending

    /// JSON array of inspection items
    pub items_json: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateQualityCertDto {
    pub cert_no: Option<String>,
    pub inspect_date: Option<String>,
    pub inspector: Option<String>,
    pub agency: Option<String>,
    pub result: Option<String>,
    pub items_json: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct QualityCertFilter {
    pub pipe_type: Option<String>,
    pub grade: Option<String>,
    pub result: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

// ── Trace result ──

#[derive(Debug, Serialize)]
pub struct TraceResult {
    pub pipe_type: String,
    pub pipe_id: String,
    pub pipe_number: String,
    pub grade: String,
    pub heat_number: Option<String>,
    pub certs: Vec<super::QualityCert>,
}
