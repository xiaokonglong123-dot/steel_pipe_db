//! Manufacturing DTOs.

use chrono::NaiveDate;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CreateBomRequest {
    pub name: String,
    pub product_type: String,
    pub notes: Option<String>,
    pub items: Vec<BomItemInput>,
}

#[derive(Debug, Deserialize)]
pub struct BomItemInput {
    pub material: String,
    pub quantity: f64,
    pub unit: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateWorkOrderRequest {
    pub bom_id: Option<i64>,
    pub product_type: String,
    pub quantity: f64,
    pub assigned_to: Option<i64>,
    pub due_date: Option<NaiveDate>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateInspectionRequest {
    pub work_order_id: Option<i64>,
    pub item_id: Option<i64>,
    pub inspection_type: String,
    pub result: String,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateNcrRequest {
    pub work_order_id: Option<i64>,
    pub item_id: Option<i64>,
    pub description: String,
    pub severity: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ResolveNcrRequest {
    pub disposition: String,
}
