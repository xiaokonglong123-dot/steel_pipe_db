//! Inventory ATP DTOs.

use serde::Deserialize;

/// Reserve a quantity of an item (available-to-promise slot).
#[derive(Debug, Deserialize)]
pub struct CreateReservationRequest {
    /// FK to the items master table.
    pub item_id: i64,
    /// Quantity to reserve.
    pub quantity: f64,
    /// Sales order this reservation is for.
    pub sales_order_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct ReleaseReservationRequest {
    pub reservation_id: i64,
}

/// Internal location-to-location transfer request.
#[derive(Debug, Deserialize)]
pub struct CreateTransferRequest {
    pub from_location_id: i64,
    pub to_location_id: i64,
    /// FK to the items master table.
    pub item_id: i64,
    /// Quantity to transfer.
    pub quantity: f64,
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
