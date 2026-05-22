use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct ApproveOrderRequest {
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct RejectOrderRequest {
    #[validate(length(min = 1))]
    pub reason: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct LinkOutboundRequest {
    #[validate(range(min = 1))]
    pub outbound_record_id: i64,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateSalesOrderRequest {
    pub order_no: Option<String>,
    #[validate(range(min = 1))]
    pub customer_id: i64,
    #[validate(length(min = 1))]
    pub order_date: String,
    pub notes: Option<String>,
    pub items: Vec<CreateSalesItemRequest>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateSalesOrderRequest {
    pub order_date: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateSalesItemRequest {
    #[validate(length(min = 1))]
    pub pipe_type: String,
    #[validate(length(min = 1))]
    pub grade: String,
    #[validate(range(min = 0.0))]
    pub od: f64,
    #[validate(range(min = 0.0))]
    pub wt: f64,
    #[validate(range(min = 1))]
    pub quantity: i64,
    pub unit_price: Option<f64>,
    pub total_price: Option<f64>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateSalesItemRequest {
    pub pipe_type: Option<String>,
    pub grade: Option<String>,
    pub od: Option<f64>,
    pub wt: Option<f64>,
    pub quantity: Option<i64>,
    pub unit_price: Option<f64>,
    pub total_price: Option<f64>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SalesOrderFilterParams {
    pub q: Option<String>,
    pub status: Option<String>,
    pub customer_id: Option<i64>,
    pub order_date_from: Option<String>,
    pub order_date_to: Option<String>,
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct SalesOrderStatusTransitionRequest {
    #[validate(length(min = 1))]
    pub status: String,
}
