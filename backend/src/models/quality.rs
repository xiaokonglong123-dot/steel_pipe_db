use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct QualityCert {
    pub id: i64,
    pub cert_number: String,
    pub pipe_type: String,
    pub pipe_id: i64,
    pub cert_date: Option<String>,
    pub result: String,
    pub inspector: Option<String>,
    pub inspection_body: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Api5ctGradeRef {
    pub id: i64,
    pub grade: String,
    pub yield_strength_min: Option<f64>,
    pub yield_strength_max: Option<f64>,
    pub tensile_strength_min: Option<f64>,
    pub hardness_max: Option<String>,
    pub carbon_content_max: Option<f64>,
    pub manganese_content_max: Option<f64>,
    pub phosphorus_content_max: Option<f64>,
    pub sulfur_content_max: Option<f64>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PipeAttachment {
    pub id: i64,
    pub pipe_type: String,
    pub pipe_id: i64,
    pub file_name: String,
    pub file_path: String,
    pub file_size: Option<i64>,
    pub content_type: Option<String>,
    pub uploaded_by: Option<i64>,
    pub created_at: String,
}
