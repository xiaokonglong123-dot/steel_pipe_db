//! Inventory ATP DTOs.

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CreateReservationRequest {
    pub pipe_type: String,
    pub pipe_number: Option<String>,
    pub quantity: rust_decimal::Decimal,
    pub sales_order_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct ReleaseReservationRequest {
    pub reservation_id: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateTransferRequest {
    pub from_location_id: i64,
    pub to_location_id: i64,
    pub pipe_id: Option<i64>,
    pub pipe_number: Option<String>,
    pub quantity: rust_decimal::Decimal,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCountTemplateRequest {
    pub name: String,
    pub description: Option<String>,
    pub location_ids: Vec<i64>,
}

#[derive(Debug, Deserialize)]
pub struct CompleteCountSessionRequest {
    pub session_id: i64,
    pub result: serde_json::Value,
}
