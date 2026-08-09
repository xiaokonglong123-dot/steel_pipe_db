//! Sales CRM row models — mirror `030_create_sales_crm.sql`.

use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow)]
pub struct SalesShipment {
    pub id: i64,
    pub tenant_id: i64,
    pub shipment_no: String,
    pub sales_order_id: i64,
    pub shipped_at: Option<DateTime<Utc>>,
    pub carrier: Option<String>,
    pub tracking_no: Option<String>,
    pub status: String,
    pub items_json: serde_json::Value,
    pub notes: Option<String>,
    pub created_by: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct SalesQuote {
    pub id: i64,
    pub tenant_id: i64,
    pub quote_no: String,
    pub customer_id: i64,
    pub quote_date: NaiveDate,
    pub valid_until: Option<NaiveDate>,
    pub total_amount: f64,
    pub status: String,
    pub items_json: serde_json::Value,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Customer credit snapshot: open invoices + historical total.
#[derive(Debug, Serialize, FromRow)]
pub struct CustomerCredit {
    pub customer_id: i64,
    pub open_invoice_total: f64,
    pub lifetime_sales: f64,
}
