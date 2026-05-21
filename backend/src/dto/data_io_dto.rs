use serde::{Deserialize, Serialize};

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

#[derive(Debug, Deserialize)]
pub struct ExportQuery {
    pub format: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct OperationLogQuery {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub user_id: Option<i64>,
    pub action: Option<String>,
    pub entity_type: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ImportResult {
    pub entity_type: String,
    pub imported_count: u64,
    pub failed_count: u64,
    pub errors: Vec<String>,
}
