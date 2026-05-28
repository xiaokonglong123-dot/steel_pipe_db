use axum::extract::{Extension, Path, Query};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use sqlx::SqlitePool;

use validator::Validate;

use crate::dto::common::PaginationParams;
use crate::dto::contract_dto::{
    ContractDetailResponse, ContractFilterParams, CreateContractItemRequest,
    CreateContractRequest, CreatePaymentRequest, UpdateContractItemRequest,
    UpdateContractRequest, UpdateContractStatusRequest, UpdatePaymentRequest,
};
use crate::error::AppError;
use crate::models::contract::{Contract, ContractItem, ContractPayment};
use crate::response::{ApiResponse, PaginatedResponse};
use crate::services::contract_service::ContractService;

// ━━━ Contract Handlers ━━━

/// GET `/api/v1/contracts` — Paginated list of contracts
///
/// Supports filtering by status, type, date range, etc.
/// Returns paginated contract results.
pub async fn list_contracts_handler(
    Extension(pool): Extension<SqlitePool>,
    Query(filter): Query<ContractFilterParams>,
) -> Result<Json<PaginatedResponse<Contract>>, AppError> {
    let pagination = PaginationParams {
        page: filter.page,
        page_size: filter.page_size,
        sort_by: filter.sort_by.clone(),
        sort_order: filter.sort_order.clone(),
    };
    let page = pagination.page();
    let page_size = pagination.page_size();

    let (items, total) = ContractService::list_contracts(&pool, &filter, &pagination).await?;

    Ok(PaginatedResponse::ok(items, total, page, page_size))
}

/// POST `/api/v1/contracts` — Create a contract
///
/// Creates a new contract with line items and payment milestones.
/// Validates request body. Returns the created contract with details.
pub async fn create_contract_handler(
    Extension(pool): Extension<SqlitePool>,
    Json(req): Json<CreateContractRequest>,
) -> Result<axum::response::Response, AppError> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let result = ContractService::create_contract(&pool, &req).await?;
    Ok(ApiResponse::created(result))
}

/// GET `/api/v1/contracts/{id}` — Get contract details
///
/// Returns the full contract detail with items and payment milestones.
/// Returns 404 if not found.
pub async fn get_contract_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<ContractDetailResponse>>, AppError> {
    let result = ContractService::get_contract_detail(&pool, id).await?;
    Ok(ApiResponse::ok(result))
}

/// PUT `/api/v1/contracts/{id}` — Update a contract
///
/// Updates an existing contract's header fields. Validates request body.
/// Returns 404 if not found.
pub async fn update_contract_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateContractRequest>,
) -> Result<Json<ApiResponse<Contract>>, AppError> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let contract = ContractService::update_contract(&pool, id, &req).await?;
    Ok(ApiResponse::ok(contract))
}

/// DELETE `/api/v1/contracts/{id}` — Delete a contract
///
/// Soft-deletes a contract. Returns 404 if not found.
pub async fn delete_contract_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<axum::response::Response, AppError> {
    ContractService::delete_contract(&pool, id).await?;
    Ok((StatusCode::NO_CONTENT, ()).into_response())
}

/// PUT `/api/v1/contracts/{id}/status` — Update contract status
///
/// Updates the contract status (e.g., active, completed, terminated).
/// Validates request body. Returns 404 if not found.
pub async fn update_contract_status_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateContractStatusRequest>,
) -> Result<Json<ApiResponse<Contract>>, AppError> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let contract = ContractService::update_status(&pool, id, &req.status).await?;
    Ok(ApiResponse::ok(contract))
}

// ━━━ Item Handlers ━━━

/// POST `/api/v1/contracts/{contract_id}/items` — Add a contract line item
///
/// Adds a new line item to a contract. Validates request body.
/// Returns 404 if contract not found.
pub async fn add_contract_item_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(contract_id): Path<i64>,
    Json(req): Json<CreateContractItemRequest>,
) -> Result<axum::response::Response, AppError> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let item = ContractService::add_item(&pool, contract_id, &req).await?;
    Ok(ApiResponse::created(item))
}

/// PUT `/api/v1/contracts/{contract_id}/items/{item_id}` — Update a contract line item
///
/// Updates a specific line item within a contract. Validates request body.
/// Returns 404 if contract or item not found.
pub async fn update_contract_item_handler(
    Extension(pool): Extension<SqlitePool>,
    Path((contract_id, item_id)): Path<(i64, i64)>,
    Json(req): Json<UpdateContractItemRequest>,
) -> Result<Json<ApiResponse<ContractItem>>, AppError> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let item = ContractService::update_item(&pool, contract_id, item_id, &req).await?;
    Ok(ApiResponse::ok(item))
}

/// DELETE `/api/v1/contracts/{contract_id}/items/{item_id}` — Delete a contract line item
///
/// Removes a line item from a contract. Returns 404 if not found.
pub async fn delete_contract_item_handler(
    Extension(pool): Extension<SqlitePool>,
    Path((contract_id, item_id)): Path<(i64, i64)>,
) -> Result<axum::response::Response, AppError> {
    ContractService::delete_item(&pool, contract_id, item_id).await?;
    Ok((StatusCode::NO_CONTENT, ()).into_response())
}

// ━━━ Payment Handlers ━━━

/// GET `/api/v1/contracts/{contract_id}/payments` — List contract payment milestones
///
/// Lists all payment milestones for a contract.
pub async fn list_contract_payments_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(contract_id): Path<i64>,
) -> Result<Json<ApiResponse<Vec<ContractPayment>>>, AppError> {
    let payments = ContractService::get_payments(&pool, contract_id).await?;
    Ok(ApiResponse::ok(payments))
}

/// POST `/api/v1/contracts/{contract_id}/payments` — Add a payment milestone
///
/// Adds a new payment milestone to a contract. Validates request body.
/// Returns 404 if contract not found.
pub async fn add_contract_payment_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(contract_id): Path<i64>,
    Json(req): Json<CreatePaymentRequest>,
) -> Result<axum::response::Response, AppError> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let payment = ContractService::add_payment(&pool, contract_id, &req).await?;
    Ok(ApiResponse::created(payment))
}

/// PUT `/api/v1/contracts/{contract_id}/payments/{payment_id}` — Update a payment milestone
///
/// Updates a specific payment milestone. Validates request body.
/// Returns 404 if contract or payment not found.
pub async fn update_contract_payment_handler(
    Extension(pool): Extension<SqlitePool>,
    Path((contract_id, payment_id)): Path<(i64, i64)>,
    Json(req): Json<UpdatePaymentRequest>,
) -> Result<Json<ApiResponse<ContractPayment>>, AppError> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let payment = ContractService::update_payment(&pool, contract_id, payment_id, &req).await?;
    Ok(ApiResponse::ok(payment))
}

/// DELETE `/api/v1/contracts/{contract_id}/payments/{payment_id}` — Delete a payment milestone
///
/// Removes a payment milestone from a contract. Returns 404 if not found.
pub async fn delete_contract_payment_handler(
    Extension(pool): Extension<SqlitePool>,
    Path((contract_id, payment_id)): Path<(i64, i64)>,
) -> Result<axum::response::Response, AppError> {
    ContractService::delete_payment(&pool, contract_id, payment_id).await?;
    Ok((StatusCode::NO_CONTENT, ()).into_response())
}
