//! Procurement row models — mirror `029_create_procurement.sql`.

use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow)]
pub struct PurchaseRequisition {
    pub id: i64,
    pub tenant_id: i64,
    pub req_no: String,
    pub title: String,
    pub department_id: Option<i64>,
    pub applicant_id: Option<i64>,
    pub expected_date: Option<NaiveDate>,
    pub status: String,
    pub items_json: serde_json::Value,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct PoReceipt {
    pub id: i64,
    pub tenant_id: i64,
    pub receipt_no: String,
    pub purchase_order_id: i64,
    pub received_at: DateTime<Utc>,
    pub status: String,
    pub notes: Option<String>,
    pub created_by: Option<i64>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct PoReceiptItem {
    pub id: i64,
    pub receipt_id: i64,
    pub item_id: Option<i64>,
    pub sku: Option<String>,
    pub quantity: f64,
    pub remark: Option<String>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct SupplierQuote {
    pub id: i64,
    pub tenant_id: i64,
    pub quote_no: String,
    pub supplier_id: i64,
    pub title: Option<String>,
    pub valid_until: Option<NaiveDate>,
    pub total_amount: f64,
    pub status: String,
    pub items_json: serde_json::Value,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Supplier scorecard: order/quote counts and totals for performance review.
#[derive(Debug, Serialize, FromRow)]
pub struct SupplierScorecard {
    pub supplier_id: i64,
    pub quote_count: i64,
    pub order_count: i64,
    pub order_total: f64,
}
