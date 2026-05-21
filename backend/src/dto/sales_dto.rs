use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CreateSalesOrderRequest {
    pub order_no: Option<String>,
    pub customer_id: i64,
    pub order_date: String,
    pub notes: Option<String>,
    pub items: Vec<CreateSalesItemRequest>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSalesOrderRequest {
    pub order_date: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSalesItemRequest {
    pub pipe_type: String,
    pub grade: String,
    pub od: f64,
    pub wt: f64,
    pub quantity: i64,
    pub unit_price: Option<f64>,
    pub total_price: Option<f64>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
pub struct SalesOrderStatusTransitionRequest {
    pub status: String,
    pub notes: Option<String>,
}
