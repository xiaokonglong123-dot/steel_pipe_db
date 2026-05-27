use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Quality certificate DB row. Test reports and mill certs for pipes.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct QualityCert {
    pub id: i64,
    /// Certificate number.
    pub cert_number: String,
    /// Pipe type: seamless or screen.
    pub pipe_type: String,
    /// Pipe ID this cert belongs to.
    pub pipe_id: i64,
    /// Inspection date.
    pub cert_date: Option<String>,
    /// Result: pass or fail. No middle ground.
    pub result: String,
    /// Inspector name.
    pub inspector: Option<String>,
    /// Inspection body / agency.
    pub inspection_body: Option<String>,
    /// Free-form notes.
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}

/// API 5CT grade reference data. Mechanical properties and chemical composition standards.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Api5ctGradeRef {
    pub id: i64,
    /// Grade name: J55 / N80 / L80 / P110, etc.
    pub grade: String,
    /// Minimum yield strength (MPa).
    pub yield_strength_min: Option<f64>,
    /// Maximum yield strength (MPa).
    pub yield_strength_max: Option<f64>,
    /// Minimum tensile strength (MPa).
    pub tensile_strength_min: Option<f64>,
    /// Maximum hardness rating.
    pub hardness_max: Option<String>,
    /// Maximum carbon content (%).
    pub carbon_content_max: Option<f64>,
    /// Maximum manganese content (%).
    pub manganese_content_max: Option<f64>,
    /// Maximum phosphorus content (%).
    pub phosphorus_content_max: Option<f64>,
    /// Maximum sulfur content (%).
    pub sulfur_content_max: Option<f64>,
    /// Free-form notes.
    pub notes: Option<String>,
}

/// Pipe attachment DB row. Cert images, files, and other docs attached to a pipe.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PipeAttachment {
    pub id: i64,
    /// Pipe type: seamless or screen.
    pub pipe_type: String,
    /// Pipe ID this file belongs to.
    pub pipe_id: i64,
    /// File name.
    pub file_name: String,
    /// Storage path for the file.
    pub file_path: String,
    /// File size in bytes.
    pub file_size: Option<i64>,
    /// MIME type of the file.
    pub content_type: Option<String>,
    /// User ID of whoever uploaded this.
    pub uploaded_by: Option<i64>,
    pub created_at: String,
}
