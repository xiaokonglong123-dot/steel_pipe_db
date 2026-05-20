use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};

use crate::error::AppResult;
use crate::handler::{list_response, ok_response};
use crate::middleware::AuthUser;
use crate::service::contract_service::{
    AddPaymentDto, ContractListFilter, ContractService, CreateContractDto, UpdateContractDto,
    UpdateContractStatusDto, UpdatePaymentDto,
};
use crate::AppState;

// ── Contracts CRUD ──────────────────────────────────────────────────────────

pub async fn list_contracts(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthUser>,
    Query(filter): Query<ContractListFilter>,
) -> AppResult<Json<impl serde::Serialize>> {
    let svc = ContractService::from_state(&state);
    let (contracts, total) = svc.list(&filter).await?;
    let contracts: Vec<serde_json::Value> = contracts
        .into_iter()
        .map(|c| serde_json::to_value(c).unwrap())
        .collect();
    Ok(list_response(contracts, total, filter.page, filter.page_size))
}

pub async fn create_contract(
    State(state): State<Arc<AppState>>,
    Extension(auth): Extension<AuthUser>,
    Json(dto): Json<CreateContractDto>,
) -> AppResult<Json<impl serde::Serialize>> {
    let svc = ContractService::from_state(&state);
    let contract = svc.create(dto, &auth.user_id).await?;
    Ok(ok_response(contract))
}

pub async fn get_contract(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthUser>,
    Path(id): Path<String>,
) -> AppResult<Json<impl serde::Serialize>> {
    let svc = ContractService::from_state(&state);
    let contract = svc.get_with_items(&id).await?;
    Ok(ok_response(contract))
}

pub async fn update_contract(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthUser>,
    Path(id): Path<String>,
    Json(dto): Json<UpdateContractDto>,
) -> AppResult<Json<impl serde::Serialize>> {
    let svc = ContractService::from_state(&state);
    let contract = svc.update(&id, dto).await?;
    Ok(ok_response(contract))
}

pub async fn update_contract_status(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthUser>,
    Path(id): Path<String>,
    Json(dto): Json<UpdateContractStatusDto>,
) -> AppResult<Json<impl serde::Serialize>> {
    let svc = ContractService::from_state(&state);
    let contract = svc.update_status(&id, &dto.status).await?;
    Ok(ok_response(contract))
}

pub async fn delete_contract(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthUser>,
    Path(id): Path<String>,
) -> AppResult<Json<impl serde::Serialize>> {
    let svc = ContractService::from_state(&state);
    svc.delete(&id).await?;
    Ok(ok_response(serde_json::json!({"deleted": true, "id": id})))
}

// ── Payments ─────────────────────────────────────────────────────────────────

pub async fn add_payment(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthUser>,
    Path(id): Path<String>,
    Json(dto): Json<AddPaymentDto>,
) -> AppResult<Json<impl serde::Serialize>> {
    let svc = ContractService::from_state(&state);
    let payment = svc.add_payment(&id, dto).await?;
    Ok(ok_response(payment))
}

pub async fn update_payment(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthUser>,
    Path((_contract_id, payment_id)): Path<(String, String)>,
    Json(dto): Json<UpdatePaymentDto>,
) -> AppResult<Json<impl serde::Serialize>> {
    let svc = ContractService::from_state(&state);
    let payment = svc.update_payment(&payment_id, dto).await?;
    Ok(ok_response(payment))
}

pub async fn delete_payment(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthUser>,
    Path((_contract_id, payment_id)): Path<(String, String)>,
) -> AppResult<Json<impl serde::Serialize>> {
    let svc = ContractService::from_state(&state);
    svc.delete_payment(&payment_id).await?;
    Ok(ok_response(serde_json::json!({"deleted": true, "id": payment_id})))
}
