use serde::{Deserialize, Serialize};
use validator::Validate;

pub const ENTITY_SEAMLESS_PIPES: &str = "seamless_pipes";
pub const ENTITY_SCREEN_PIPES: &str = "screen_pipes";
pub const ENTITY_INVENTORY: &str = "inventory";
pub const ENTITY_PURCHASE_ORDERS: &str = "purchase_orders";
pub const ENTITY_SALES_ORDERS: &str = "sales_orders";
pub const ENTITY_QUALITY_CERTS: &str = "quality_certs";

pub const VALID_ENTITY_TYPES: &[&str] = &[
    ENTITY_SEAMLESS_PIPES,
    ENTITY_SCREEN_PIPES,
    ENTITY_INVENTORY,
    ENTITY_PURCHASE_ORDERS,
    ENTITY_SALES_ORDERS,
    ENTITY_QUALITY_CERTS,
];

#[derive(Debug, Deserialize, Validate)]
pub struct ExportQuery {
    #[validate(length(min = 1, message = "Export format cannot be empty"))]
    pub format: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct OperationLogQuery {
    #[validate(range(min = 1, message = "Page must be at least 1"))]
    pub page: Option<u64>,
    #[validate(range(min = 1, max = 1000, message = "Page size must be between 1 and 1000"))]
    pub page_size: Option<u64>,
    #[validate(range(min = 1, message = "User ID must be positive"))]
    pub user_id: Option<i64>,
    #[validate(length(min = 1, message = "Action cannot be empty"))]
    pub action: Option<String>,
    #[validate(length(min = 1, message = "Entity type cannot be empty"))]
    pub entity_type: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ImportResult {
    pub entity_type: String,
    pub imported_count: u64,
    pub failed_count: u64,
    pub errors: Vec<String>,
}
