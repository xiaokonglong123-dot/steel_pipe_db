use serde::Deserialize;
use validator::Validate;

/// Order report query params DTO.
#[derive(Debug, Deserialize, Validate)]
pub struct OrderReportQuery {
    /// Order type: purchase or sales.
    #[validate(length(min = 1, message = "Order type cannot be empty"))]
    pub r#type: Option<String>,
    /// Stats period: daily / monthly / yearly.
    #[validate(length(min = 1, message = "Period cannot be empty"))]
    pub period: Option<String>,
}
