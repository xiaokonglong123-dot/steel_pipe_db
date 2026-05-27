use serde::Deserialize;
use validator::Validate;

/// Pipe identifier DTO — used to specify target pipes for label generation.
#[derive(Debug, Deserialize, Validate)]
pub struct PipeIdentifier {
    /// Pipe type: seamless or screen.
    #[validate(length(min = 1))]
    pub pipe_type: String,
    /// Pipe ID.
    #[validate(range(min = 1))]
    pub pipe_id: i64,
}

/// Batch label print request DTO.
#[derive(Debug, Deserialize, Validate)]
pub struct BatchLabelRequest {
    /// List of pipes to print labels for.
    pub pipe_ids: Vec<PipeIdentifier>,
}

/// Shipping label print request DTO.
#[derive(Debug, Deserialize, Validate)]
pub struct ShippingLabelRequest {
    /// Pipe type: seamless or screen.
    #[validate(length(min = 1))]
    pub pipe_type: String,
    /// Pipe ID.
    #[validate(range(min = 1))]
    pub pipe_id: i64,
    /// Order number.
    pub order_number: Option<String>,
    /// Customer name.
    pub customer_name: Option<String>,
    /// Destination.
    pub destination: Option<String>,
    /// Purchase order number.
    pub po_number: Option<String>,
    /// Ship date.
    pub ship_date: Option<String>,
}
