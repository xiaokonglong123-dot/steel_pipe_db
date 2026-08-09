//! Item (商品) DTOs — request/response types for the items master API.

use serde::{Deserialize, Serialize};
use validator::Validate;

/// Create item request.
#[derive(Debug, Deserialize, Validate)]
pub struct CreateItemRequest {
    /// Unique SKU.
    #[validate(length(min = 1, message = "SKU is required"))]
    pub sku: String,
    /// Display name.
    #[validate(length(min = 1, message = "Name is required"))]
    pub name: String,
    /// Category (e.g. 原材料 / 标准件).
    pub category: Option<String>,
    /// Unit of measure.
    pub unit: Option<String>,
    /// Free-form specification text.
    pub spec: Option<String>,
    /// Unit price.
    pub price: Option<f64>,
}

/// Update item request — every field optional; only provided fields are set.
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateItemRequest {
    /// Display name.
    pub name: Option<String>,
    /// Category.
    pub category: Option<String>,
    /// Unit of measure.
    pub unit: Option<String>,
    /// Free-form specification text.
    pub spec: Option<String>,
    /// Unit price.
    pub price: Option<f64>,
    /// active | inactive.
    pub status: Option<String>,
}

/// Item list filter params.
#[derive(Debug, Deserialize)]
pub struct ItemFilter {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    /// Full-text search over sku / name / category / spec.
    pub q: Option<String>,
    /// Filter by category.
    pub category: Option<String>,
    /// Filter by status.
    pub status: Option<String>,
}

/// SKU search query — returns items whose sku contains the term.
#[derive(Debug, Deserialize)]
pub struct ItemSkuQuery {
    /// SKU to search (partial match).
    pub sku: String,
}

/// Item list response item — item master + computed on-hand stock.
#[derive(Debug, Serialize)]
pub struct ItemWithStock {
    #[serde(flatten)]
    pub item: crate::models::item::Item,
    /// Computed on-hand quantity from inventory_logs.
    pub stock: f64,
}
