use serde::Deserialize;
use validator::Validate;

/// Create quality cert request DTO.
#[derive(Debug, Deserialize, Validate)]
pub struct CreateQualityCertRequest {
    /// Pipe type: seamless or screen.
    #[validate(length(min = 1))]
    pub pipe_type: String,
    /// Pipe ID.
    #[validate(range(min = 1))]
    pub pipe_id: i64,
    /// Inspection date.
    pub cert_date: Option<String>,
    /// Result: pass or fail.
    pub result: Option<String>,
    /// Inspector name.
    pub inspector: Option<String>,
    /// Inspection body / agency.
    pub inspection_body: Option<String>,
    /// Notes.
    pub notes: Option<String>,
    /// Cert number (auto-generated if empty).
    pub cert_number: Option<String>,
}

/// Update quality cert request DTO.
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateQualityCertRequest {
    /// Inspection date.
    pub cert_date: Option<String>,
    /// Result.
    pub result: Option<String>,
    /// Inspector name.
    pub inspector: Option<String>,
    /// Inspection body / agency.
    pub inspection_body: Option<String>,
    /// Notes.
    pub notes: Option<String>,
}

/// Quality cert list filter params.
#[derive(Debug, Deserialize)]
pub struct QualityCertFilterParams {
    /// Filter by pipe type.
    pub pipe_type: Option<String>,
    /// Filter by pipe ID.
    pub pipe_id: Option<i64>,
    /// Filter by result.
    pub result: Option<String>,
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

/// Create pipe attachment request DTO.
#[derive(Debug, Deserialize, Validate)]
pub struct CreateAttachmentRequest {
    /// Pipe type: seamless or screen.
    #[validate(length(min = 1))]
    pub pipe_type: String,
    /// Pipe ID.
    #[validate(range(min = 1))]
    pub pipe_id: i64,
    /// File name.
    #[validate(length(min = 1))]
    pub file_name: String,
    /// File storage path.
    #[validate(length(min = 1))]
    pub file_path: String,
    /// File size in bytes.
    pub file_size: Option<i64>,
    /// MIME type.
    pub content_type: Option<String>,
    /// Uploader user ID.
    pub uploaded_by: Option<i64>,
}


