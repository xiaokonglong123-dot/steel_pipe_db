use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SalesOrder {
    pub id: i64,
    pub order_no: String,
    pub customer_id: i64,
    pub order_date: String,
    pub status: String,
    pub total_amount: Option<f64>,
    pub notes: Option<String>,
    pub created_by: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SalesOrderItem {
    pub id: i64,
    pub order_id: i64,
    pub pipe_type: String,
    pub grade: String,
    pub od: f64,
    pub wt: f64,
    pub quantity: i64,
    pub delivered_quantity: i64,
    pub unit_price: Option<f64>,
    pub total_price: Option<f64>,
    pub notes: Option<String>,
    pub created_at: String,
}
