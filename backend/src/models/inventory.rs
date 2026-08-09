use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Location DB row. A physical spot in the warehouse — zone, shelf, and level.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Location {
    pub id: i64,
    /// Zone code — which area of the warehouse.
    pub zone_code: String,
    /// Shelf / rack code.
    pub shelf_code: String,
    /// Level code within the shelf.
    pub level_code: String,
    /// Full location code — zone + shelf + level concatenated.
    pub full_code: String,
    /// Human-readable description of this location.
    pub description: Option<String>,
    /// Maximum capacity.
    pub capacity: Option<i64>,
    /// How many units are currently stored here (item-quantity based, kept
    /// for compatibility; stock is derived from inventory_logs).
    pub used_count: i64,
    /// Whether this location is active and usable.
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Inbound record DB row. Covers purchase receipts, production returns, and transfers.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct InboundRecord {
    pub id: i64,
    /// Inbound document number.
    pub inbound_no: String,
    /// Inbound type: purchase / production / return.
    pub inbound_type: String,
    /// Related order ID (e.g. purchase order).
    pub order_id: Option<i64>,
    /// Supplier ID.
    pub supplier_id: Option<i64>,
    /// Free-form notes.
    pub notes: Option<String>,
    /// Approval status: auto_approved / pending / approved / rejected.
    pub approval_status: String,
    /// Why it got rejected (if it did).
    pub rejection_reason: Option<String>,
    /// Approval/rejection reason provided by the approver.
    pub approval_reason: Option<String>,
    /// User ID of the handler / operator.
    pub handled_by: Option<i64>,
    /// When the handling happened.
    pub handled_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Inbound item DB row. The actual items in an inbound shipment.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct InboundItem {
    pub id: i64,
    /// FK back to the inbound record.
    pub inbound_id: i64,
    /// FK to the items master table.
    pub item_id: i64,
    /// Received quantity.
    pub quantity: f64,
    pub created_at: DateTime<Utc>,
}

/// Outbound record DB row. Sales, scrapped, or transfer out of the warehouse.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OutboundRecord {
    pub id: i64,
    /// Outbound document number.
    pub outbound_no: String,
    /// Outbound type: sales / transfer / scrapped.
    pub outbound_type: String,
    /// Related order ID (e.g. sales order).
    pub order_id: Option<i64>,
    /// Customer ID this is going to.
    pub customer_id: Option<i64>,
    /// Free-form notes.
    pub notes: Option<String>,
    /// Approval status: auto_approved / pending / approved / rejected.
    pub approval_status: String,
    /// Rejection reason, if applicable.
    pub rejection_reason: Option<String>,
    /// Approval/rejection reason provided by the approver.
    pub approval_reason: Option<String>,
    /// User ID of the handler.
    pub handled_by: Option<i64>,
    /// When the handling happened.
    pub handled_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Outbound item DB row. Items going out the door.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OutboundItem {
    pub id: i64,
    /// FK back to the outbound record.
    pub outbound_id: i64,
    /// FK to the items master table.
    pub item_id: i64,
    /// Shipped quantity.
    pub quantity: f64,
    pub created_at: DateTime<Utc>,
}

/// Inventory log DB row. Audit trail for every item movement, no exceptions.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct InventoryLog {
    pub id: i64,
    /// FK to the items master table.
    pub item_id: i64,
    /// Signed quantity moved (positive = in, negative = out).
    pub quantity: f64,
    /// Change type: inbound / outbound / transfer / check_adjust.
    pub change_type: String,
    /// Reference document type.
    pub ref_type: Option<String>,
    /// Reference document ID.
    pub ref_id: Option<i64>,
    /// Source location ID (where it came from).
    pub from_location_id: Option<i64>,
    /// Destination location ID (where it went).
    pub to_location_id: Option<i64>,
    /// Free-form notes.
    pub notes: Option<String>,
    /// User ID of whoever performed the operation.
    pub created_by: Option<i64>,
    pub created_at: DateTime<Utc>,
}

/// Inventory check record DB row. The header for a stock-counting session.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct InventoryCheckRecord {
    pub id: i64,
    /// Check document number.
    pub check_no: String,
    /// Location ID being checked (null = whole warehouse).
    pub location_id: Option<i64>,
    /// Status: in_progress / completed / cancelled.
    pub status: String,
    /// Notes about this check.
    pub notes: Option<String>,
    /// User ID of whoever created this check.
    pub created_by: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Inventory check item DB row. One row = one item being verified.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct InventoryCheckItem {
    pub id: i64,
    /// FK back to the check record.
    pub check_id: i64,
    /// FK to the items master table.
    pub item_id: i64,
    /// Expected (book) quantity.
    pub expected_quantity: Option<f64>,
    /// What was actually found on the floor.
    pub found_quantity: Option<f64>,
    /// Whether expected and found match up.
    pub is_match: Option<bool>,
    /// Notes about this item.
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}
