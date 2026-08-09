//! Sales CRM DTOs.

use chrono::NaiveDate;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CreateShipmentRequest {
    pub sales_order_id: i64,
    pub carrier: Option<String>,
    pub tracking_no: Option<String>,
    pub items: Vec<serde_json::Value>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateShipmentStatusRequest {
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateSalesQuoteRequest {
    pub customer_id: i64,
    pub valid_until: Option<NaiveDate>,
    pub total_amount: f64,
    pub items: Vec<serde_json::Value>,
    pub notes: Option<String>,
}
