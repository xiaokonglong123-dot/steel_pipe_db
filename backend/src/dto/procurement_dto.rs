//! Procurement DTOs.

use chrono::NaiveDate;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CreateRequisitionRequest {
    pub title: String,
    pub department_id: Option<i64>,
    pub expected_date: Option<NaiveDate>,
    pub items: Vec<serde_json::Value>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateReceiptRequest {
    pub purchase_order_id: i64,
    pub notes: Option<String>,
    pub items: Vec<ReceiptItemInput>,
}

#[derive(Debug, Deserialize)]
pub struct ReceiptItemInput {
    pub item_id: Option<i64>,
    pub sku: Option<String>,
    pub quantity: f64,
    pub remark: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateQuoteRequest {
    pub supplier_id: i64,
    pub title: Option<String>,
    pub valid_until: Option<NaiveDate>,
    pub total_amount: f64,
    pub items: Vec<serde_json::Value>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateQuoteStatusRequest {
    pub status: String,
}
