use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct PipeIdentifier {
    pub pipe_type: String,
    pub pipe_id: i64,
}

#[derive(Debug, Deserialize)]
pub struct BatchLabelRequest {
    pub pipe_ids: Vec<PipeIdentifier>,
}

#[derive(Debug, Deserialize)]
pub struct ShippingLabelRequest {
    pub pipe_type: String,
    pub pipe_id: i64,
    pub order_number: Option<String>,
    pub customer_name: Option<String>,
    pub destination: Option<String>,
    pub po_number: Option<String>,
    pub ship_date: Option<String>,
}
