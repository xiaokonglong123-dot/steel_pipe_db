use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateQualityCertRequest {
    #[validate(length(min = 1))]
    pub pipe_type: String,
    #[validate(range(min = 1))]
    pub pipe_id: i64,
    pub cert_date: Option<String>,
    pub result: Option<String>,
    pub inspector: Option<String>,
    pub inspection_body: Option<String>,
    pub notes: Option<String>,
    pub cert_number: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateQualityCertRequest {
    pub cert_date: Option<String>,
    pub result: Option<String>,
    pub inspector: Option<String>,
    pub inspection_body: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct QualityCertFilterParams {
    pub pipe_type: Option<String>,
    pub pipe_id: Option<i64>,
    pub result: Option<String>,
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateAttachmentRequest {
    #[validate(length(min = 1))]
    pub pipe_type: String,
    #[validate(range(min = 1))]
    pub pipe_id: i64,
    #[validate(length(min = 1))]
    pub file_name: String,
    #[validate(length(min = 1))]
    pub file_path: String,
    pub file_size: Option<i64>,
    pub content_type: Option<String>,
    pub uploaded_by: Option<i64>,
}


