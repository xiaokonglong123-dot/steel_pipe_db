use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Contract {
    pub id: i64,
    pub contract_no: String,
    pub contract_type: String,
    pub title: String,
    pub party_a: String,
    pub party_b: String,
    pub sign_date: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub total_amount: Option<f64>,
    pub status: String,
    pub notes: Option<String>,
    pub created_by: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ContractItem {
    pub id: i64,
    pub contract_id: i64,
    pub pipe_type: String,
    pub grade: String,
    pub od: f64,
    pub wt: f64,
    pub quantity: i64,
    pub unit_price: Option<f64>,
    pub total_price: Option<f64>,
    pub notes: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ContractPayment {
    pub id: i64,
    pub contract_id: i64,
    pub due_date: String,
    pub amount: f64,
    pub payment_type: String,
    pub is_paid: i64,
    pub paid_date: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
}
