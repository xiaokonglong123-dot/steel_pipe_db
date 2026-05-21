use axum::extract::{Extension, Path, Query};
use axum::Json;
use sqlx::SqlitePool;

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

pub async fn create_contract_handler(
    Extension(pool): Extension<SqlitePool>,
    Json(req): Json<CreateContractRequest>,
) -> Result<Json<ApiResponse<ContractDetailResponse>>, AppError> {
    let result = ContractService::create_contract(&pool, &req).await?;
    Ok(ApiResponse::ok(result))
}

pub async fn get_contract_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<ContractDetailResponse>>, AppError> {
    let result = ContractService::get_contract_detail(&pool, id).await?;
    Ok(ApiResponse::ok(result))
}

pub async fn update_contract_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateContractRequest>,
) -> Result<Json<ApiResponse<Contract>>, AppError> {
    let contract = ContractService::update_contract(&pool, id, &req).await?;
    Ok(ApiResponse::ok(contract))
}

pub async fn delete_contract_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    ContractService::delete_contract(&pool, id).await?;
    Ok(ApiResponse::ok("Contract deleted successfully".into()))
}

pub async fn update_contract_status_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateContractStatusRequest>,
) -> Result<Json<ApiResponse<Contract>>, AppError> {
    let contract = ContractService::update_status(&pool, id, &req.status).await?;
    Ok(ApiResponse::ok(contract))
}

// ━━━ Item Handlers ━━━

pub async fn add_contract_item_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(contract_id): Path<i64>,
    Json(req): Json<CreateContractItemRequest>,
) -> Result<Json<ApiResponse<ContractItem>>, AppError> {
    let item = ContractService::add_item(&pool, contract_id, &req).await?;
    Ok(ApiResponse::ok(item))
}

pub async fn update_contract_item_handler(
    Extension(pool): Extension<SqlitePool>,
    Path((contract_id, item_id)): Path<(i64, i64)>,
    Json(req): Json<UpdateContractItemRequest>,
) -> Result<Json<ApiResponse<ContractItem>>, AppError> {
    let item = ContractService::update_item(&pool, contract_id, item_id, &req).await?;
    Ok(ApiResponse::ok(item))
}

pub async fn delete_contract_item_handler(
    Extension(pool): Extension<SqlitePool>,
    Path((contract_id, item_id)): Path<(i64, i64)>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    ContractService::delete_item(&pool, contract_id, item_id).await?;
    Ok(ApiResponse::ok("Contract item deleted successfully".into()))
}

// ━━━ Payment Handlers ━━━

pub async fn list_contract_payments_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(contract_id): Path<i64>,
) -> Result<Json<ApiResponse<Vec<ContractPayment>>>, AppError> {
    let payments = ContractService::get_payments(&pool, contract_id).await?;
    Ok(ApiResponse::ok(payments))
}

pub async fn add_contract_payment_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(contract_id): Path<i64>,
    Json(req): Json<CreatePaymentRequest>,
) -> Result<Json<ApiResponse<ContractPayment>>, AppError> {
    let payment = ContractService::add_payment(&pool, contract_id, &req).await?;
    Ok(ApiResponse::ok(payment))
}

pub async fn update_contract_payment_handler(
    Extension(pool): Extension<SqlitePool>,
    Path((contract_id, payment_id)): Path<(i64, i64)>,
    Json(req): Json<UpdatePaymentRequest>,
) -> Result<Json<ApiResponse<ContractPayment>>, AppError> {
    let payment = ContractService::update_payment(&pool, contract_id, payment_id, &req).await?;
    Ok(ApiResponse::ok(payment))
}

pub async fn delete_contract_payment_handler(
    Extension(pool): Extension<SqlitePool>,
    Path((contract_id, payment_id)): Path<(i64, i64)>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    ContractService::delete_payment(&pool, contract_id, payment_id).await?;
    Ok(ApiResponse::ok("Contract payment deleted successfully".into()))
}
