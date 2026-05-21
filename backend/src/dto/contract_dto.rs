use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::models::contract::{Contract, ContractItem, ContractPayment};

// ━━━ Contract ━━━

#[derive(Debug, Deserialize, Validate)]
pub struct CreateContractRequest {
    #[validate(length(min = 1))]
    pub contract_type: String,
    #[validate(length(min = 1))]
    pub title: String,
    #[validate(length(min = 1))]
    pub party_a: String,
    #[validate(length(min = 1))]
    pub party_b: String,
    pub sign_date: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub notes: Option<String>,
    pub items: Vec<CreateContractItemRequest>,
}

#[derive(Debug, Deserialize, Validate)]
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

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateContractItemRequest {
    #[validate(length(min = 1))]
    pub pipe_type: String,
    #[validate(length(min = 1))]
    pub grade: String,
    #[validate(range(min = 0.0))]
    pub od: f64,
    #[validate(range(min = 0.0))]
    pub wt: f64,
    #[validate(range(min = 1))]
    pub quantity: i64,
    pub unit_price: Option<f64>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
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

#[derive(Debug, Deserialize, Validate)]
pub struct CreatePaymentRequest {
    #[validate(length(min = 1))]
    pub due_date: String,
    #[validate(range(min = 0.0))]
    pub amount: f64,
    #[validate(length(min = 1))]
    pub payment_type: String,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdatePaymentRequest {
    pub due_date: Option<String>,
    pub amount: Option<f64>,
    pub payment_type: Option<String>,
    pub is_paid: Option<i64>,
    pub paid_date: Option<String>,
    pub notes: Option<String>,
}

// ━━━ Status Transition ━━━

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateContractStatusRequest {
    #[validate(length(min = 1))]
    pub status: String,
}
