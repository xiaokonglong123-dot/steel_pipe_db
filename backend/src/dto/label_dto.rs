use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct PipeIdentifier {
    #[validate(length(min = 1))]
    pub pipe_type: String,
    #[validate(range(min = 1))]
    pub pipe_id: i64,
}

#[derive(Debug, Deserialize, Validate)]
pub struct BatchLabelRequest {
    pub pipe_ids: Vec<PipeIdentifier>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ShippingLabelRequest {
    #[validate(length(min = 1))]
    pub pipe_type: String,
    #[validate(range(min = 1))]
    pub pipe_id: i64,
    pub order_number: Option<String>,
    pub customer_name: Option<String>,
    pub destination: Option<String>,
    pub po_number: Option<String>,
    pub ship_date: Option<String>,
}
