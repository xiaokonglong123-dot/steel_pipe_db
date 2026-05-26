use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct OrderReportQuery {
    #[validate(length(min = 1, message = "Order type cannot be empty"))]
    pub r#type: Option<String>,
    #[validate(length(min = 1, message = "Period cannot be empty"))]
    pub period: Option<String>,
}
