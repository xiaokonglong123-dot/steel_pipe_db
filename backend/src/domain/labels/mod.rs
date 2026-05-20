use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelTemplateDto {
    pub name: String,
    pub width_mm: f64,
    pub height_mm: f64,
    pub config_json: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateLabelTemplateDto {
    pub name: Option<String>,
    pub width_mm: Option<f64>,
    pub height_mm: Option<f64>,
    pub config_json: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateLabelsRequest {
    pub template_id: String,
    pub pipe_numbers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelGenerateResult {
    pub template_id: String,
    pub template_name: String,
    pub total_labels: usize,
    pub pdf_size_bytes: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PrintLog {
    pub id: String,
    pub template_id: String,
    pub template_name: String,
    pub pipe_numbers_json: String,
    pub total_labels: i64,
    pub printed_by: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PrintLogRow {
    pub id: String,
    pub template_id: String,
    pub template_name: String,
    pub pipe_numbers_json: String,
    pub total_labels: i64,
    pub printed_by: String,
    pub created_at: String,
}
