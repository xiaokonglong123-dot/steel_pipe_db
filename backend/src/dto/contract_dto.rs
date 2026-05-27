use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::models::contract::{Contract, ContractItem, ContractPayment};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  Contract
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Create contract request DTO.
#[derive(Debug, Deserialize, Validate)]
pub struct CreateContractRequest {
    /// Contract type: purchase or sales.
    #[validate(length(min = 1))]
    pub contract_type: String,
    /// Contract title.
    #[validate(length(min = 1))]
    pub title: String,
    /// Party A name.
    #[validate(length(min = 1))]
    pub party_a: String,
    /// Party B name.
    #[validate(length(min = 1))]
    pub party_b: String,
    /// Signing date.
    pub sign_date: Option<String>,
    /// Effective date.
    pub start_date: Option<String>,
    /// Expiry date.
    pub end_date: Option<String>,
    /// Notes.
    pub notes: Option<String>,
    /// List of contract items.
    pub items: Vec<CreateContractItemRequest>,
}

/// Update contract request DTO.
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateContractRequest {
    /// Title.
    pub title: Option<String>,
    /// Party A name.
    pub party_a: Option<String>,
    /// Party B name.
    pub party_b: Option<String>,
    /// Signing date.
    pub sign_date: Option<String>,
    /// Effective date.
    pub start_date: Option<String>,
    /// Expiry date.
    pub end_date: Option<String>,
    /// Notes.
    pub notes: Option<String>,
}

/// Contract list filter params.
#[derive(Debug, Deserialize)]
pub struct ContractFilterParams {
    /// Full-text search.
    pub q: Option<String>,
    /// Filter by contract type.
    pub contract_type: Option<String>,
    /// Filter by status.
    pub status: Option<String>,
    /// Date range start.
    pub date_from: Option<String>,
    /// Date range end.
    pub date_to: Option<String>,
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

/// Contract detail response DTO (includes items and payment milestones).
#[derive(Debug, Serialize)]
pub struct ContractDetailResponse {
    pub contract: Contract,
    pub items: Vec<ContractItem>,
    pub payments: Vec<ContractPayment>,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  Contract Items
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Create contract item request DTO.
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateContractItemRequest {
    /// Pipe type: seamless or screen.
    #[validate(length(min = 1))]
    pub pipe_type: String,
    /// Steel grade.
    #[validate(length(min = 1))]
    pub grade: String,
    /// Outer diameter (mm).
    #[validate(range(min = 0.0))]
    pub od: f64,
    /// Wall thickness (mm).
    #[validate(range(min = 0.0))]
    pub wt: f64,
    /// Quantity.
    #[validate(range(min = 1))]
    pub quantity: i64,
    /// Unit price.
    pub unit_price: Option<f64>,
    /// Notes.
    pub notes: Option<String>,
}

/// Update contract item request DTO.
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

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  Contract Payments
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Create payment milestone request DTO.
#[derive(Debug, Deserialize, Validate)]
pub struct CreatePaymentRequest {
    /// Payment due date.
    #[validate(length(min = 1))]
    pub due_date: String,
    /// Payment amount.
    #[validate(range(min = 0.0))]
    pub amount: f64,
    /// Payment type: deposit / progress / final / retention.
    #[validate(length(min = 1))]
    pub payment_type: String,
    /// Notes.
    pub notes: Option<String>,
}

/// Update payment milestone request DTO.
#[derive(Debug, Deserialize, Validate)]
pub struct UpdatePaymentRequest {
    /// Due date.
    pub due_date: Option<String>,
    /// Amount.
    pub amount: Option<f64>,
    /// Payment type.
    pub payment_type: Option<String>,
    /// Whether paid (0 = unpaid, 1 = paid).
    pub is_paid: Option<i64>,
    /// Actual payment date.
    pub paid_date: Option<String>,
    /// Notes.
    pub notes: Option<String>,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  Status Transition
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Contract status change request DTO.
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateContractStatusRequest {
    /// Target status: draft / active / completed / terminated / cancelled.
    #[validate(length(min = 1))]
    pub status: String,
}
