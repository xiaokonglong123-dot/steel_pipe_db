use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};
use serde::Deserialize;

use crate::error::AppResult;
use crate::handler::{list_response, ok_response, ApiListResponse, ApiResponse};
use crate::middleware::AuthUser;
use crate::service::purchase_service::{
    CreatePurchaseOrderDto, CreateSupplierDto, OrderFilter, PurchaseService, UpdateSupplierDto,
};
use crate::AppState;

// ── Pagination Query ────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct PaginationQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Deserialize)]
pub struct SupplierFilter {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub search: Option<String>,
}

// ── Suppliers ───────────────────────────────────────────────────────────────

pub async fn list_suppliers(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthUser>,
    Query(filter): Query<SupplierFilter>,
) -> AppResult<Json<ApiListResponse<serde_json::Value>>> {
    let svc = PurchaseService::from_state(&state);
    let page = filter.page.unwrap_or(1).max(1);
    let page_size = filter.page_size.unwrap_or(20).clamp(1, 200);
    let (items, total) = svc.list_suppliers(filter.search.as_deref(), page, page_size).await?;
    let items: Vec<serde_json::Value> = items.into_iter()
        .map(|s| serde_json::to_value(s).unwrap())
        .collect();
    Ok(list_response(items, total, page, page_size))
}

pub async fn create_supplier(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthUser>,
    Json(dto): Json<CreateSupplierDto>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let svc = PurchaseService::from_state(&state);
    let supplier = svc.create_supplier(dto).await?;
    Ok(ok_response(serde_json::to_value(supplier).unwrap()))
}

pub async fn update_supplier(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthUser>,
    Path(id): Path<String>,
    Json(dto): Json<UpdateSupplierDto>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let svc = PurchaseService::from_state(&state);
    let supplier = svc.update_supplier(&id, dto).await?;
    Ok(ok_response(serde_json::to_value(supplier).unwrap()))
}

pub async fn delete_supplier(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthUser>,
    Path(id): Path<String>,
) -> AppResult<Json<ApiResponse<()>>> {
    let svc = PurchaseService::from_state(&state);
    svc.delete_supplier(&id).await?;
    Ok(ok_response(()))
}

// ── Purchase Orders ─────────────────────────────────────────────────────────

pub async fn create_purchase_order(
    State(state): State<Arc<AppState>>,
    Extension(auth): Extension<AuthUser>,
    Json(dto): Json<CreatePurchaseOrderDto>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let svc = PurchaseService::from_state(&state);
    let result = svc.create_purchase_order(dto, &auth.user_id).await?;
    Ok(ok_response(serde_json::to_value(result).unwrap()))
}

pub async fn list_purchase_orders(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthUser>,
    Query(filter): Query<OrderFilter>,
) -> AppResult<Json<ApiListResponse<serde_json::Value>>> {
    let svc = PurchaseService::from_state(&state);
    let (items, total) = svc.list_purchase_orders(&filter).await?;
    let page = filter.page.unwrap_or(1).max(1);
    let page_size = filter.page_size.unwrap_or(20).clamp(1, 200);
    let items: Vec<serde_json::Value> = items.into_iter()
        .map(|po| serde_json::to_value(po).unwrap())
        .collect();
    Ok(list_response(items, total, page, page_size))
}

pub async fn get_purchase_order(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthUser>,
    Path(id): Path<String>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let svc = PurchaseService::from_state(&state);
    let result = svc.get_purchase_order(&id).await?;
    Ok(ok_response(serde_json::to_value(result).unwrap()))
}

pub async fn approve_purchase_order(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthUser>,
    Path(id): Path<String>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let svc = PurchaseService::from_state(&state);
    let order = svc.approve_purchase_order(&id).await?;
    Ok(ok_response(serde_json::to_value(order).unwrap()))
}

pub async fn cancel_purchase_order(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthUser>,
    Path(id): Path<String>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let svc = PurchaseService::from_state(&state);
    let order = svc.cancel_purchase_order(&id).await?;
    Ok(ok_response(serde_json::to_value(order).unwrap()))
}

pub async fn link_inbound_to_po(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthUser>,
    Path(id): Path<String>,
    Json(body): Json<LinkBody>,
) -> AppResult<Json<ApiResponse<()>>> {
    let svc = PurchaseService::from_state(&state);
    svc.link_inbound_to_po(&body.inbound_id, &id).await?;
    Ok(ok_response(()))
}

// ── Shared DTOs ─────────────────────────────────────────────────────────────

#[derive(serde::Deserialize)]
pub struct LinkBody {
    pub inbound_id: String,
}
