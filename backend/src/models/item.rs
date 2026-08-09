//! Item (商品) DB row — mirrors the `items` master table (migration 002).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Generic item master row. Every product in the ERP is an Item + SKU;
/// industry-specific pipe columns (grade / od / wt / API 5CT) were removed in
/// favor of the free-form `spec` text column.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Item {
    pub id: i64,
    /// Unique stock keeping unit.
    pub sku: String,
    /// Display name of the item.
    pub name: String,
    /// Category (e.g. 原材料 / 标准件).
    pub category: Option<String>,
    /// Unit of measure (张 / 根 / 千克 / 个 …).
    pub unit: Option<String>,
    /// Free-form specification text.
    pub spec: Option<String>,
    /// Unit price.
    pub price: Option<f64>,
    /// active | inactive.
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}
