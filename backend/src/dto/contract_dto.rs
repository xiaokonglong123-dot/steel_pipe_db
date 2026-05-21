use serde::{Deserialize, Serialize};

use crate::models::contract::{Contract, ContractItem, ContractPayment};

// ━━━ Contract ━━━

#[derive(Debug, Deserialize)]
pub struct CreateContractRequest {
    pub contract_type: String,
    pub title: String,
    pub party_a: String,
    pub party_b: String,
    pub sign_date: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub notes: Option<String>,
    pub items: Vec<CreateContractItemRequest>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateContractRequest {
    pub title: Option<String>,
    pub party_a: Option<String>,
    pub party_b: Option<String>,
    pub sign_date: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ContractFilterParams {
    pub q: Option<String>,
    pub contract_type: Option<String>,
    pub status: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ContractDetailResponse {
    pub contract: Contract,
    pub items: Vec<ContractItem>,
    pub payments: Vec<ContractPayment>,
}

// ━━━ Contract Items ━━━

#[derive(Debug, Clone, Deserialize)]
pub struct CreateContractItemRequest {
    pub pipe_type: String,
    pub grade: String,
    pub od: f64,
    pub wt: f64,
    pub quantity: i64,
    pub unit_price: Option<f64>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateContractItemRequest {
    pub pipe_type: Option<String>,
    pub grade: Option<String>,
    pub od: Option<f64>,
    pub wt: Option<f64>,
    pub quantity: Option<i64>,
    pub unit_price: Option<f64>,
    pub notes: Option<String>,
}

// ━━━ Contract Payments ━━━

#[derive(Debug, Deserialize)]
pub struct CreatePaymentRequest {
    pub due_date: String,
    pub amount: f64,
    pub payment_type: String,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePaymentRequest {
    pub due_date: Option<String>,
    pub amount: Option<f64>,
    pub payment_type: Option<String>,
    pub is_paid: Option<i64>,
    pub paid_date: Option<String>,
    pub notes: Option<String>,
}

// ━━━ Status Transition ━━━

#[derive(Debug, Deserialize)]
pub struct UpdateContractStatusRequest {
    pub status: String,
}
