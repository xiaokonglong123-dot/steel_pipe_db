use axum::{
    extract::{Path, Query, State},
    Json,
};
use std::sync::Arc;

use crate::error::AppResult;
use crate::handler::{list_response, ok_response};
use crate::service::inventory_service::{
    CreateInboundDto, CreateOutboundDto, InboundFilter, OutboundFilter,
    InventoryCheckFilter, CreateInventoryCheckDto,
};
use crate::AppState;
use crate::middleware::AuthUser;

// ---------------------------------------------------------------------------
// Inbound
// ---------------------------------------------------------------------------

pub async fn create_inbound(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    Json(dto): Json<CreateInboundDto>,
) -> AppResult<Json<impl serde::Serialize>> {
    let service = &state.inventory_service;
    let detail = service.create_inbound(dto, &auth_user.user_id).await?;
    Ok(ok_response(serde_json::json!(detail)))
}

pub async fn list_inbound(
    State(state): State<Arc<AppState>>,
    _auth_user: AuthUser,
    Query(filter): Query<InboundFilter>,
) -> AppResult<Json<impl serde::Serialize>> {
    let service = &state.inventory_service;
    let page = filter.page();
    let page_size = filter.page_size();
    let (records, total) = service.list_inbound_records(filter).await?;
    Ok(list_response(records, total, page, page_size))
}

pub async fn get_inbound(
    State(state): State<Arc<AppState>>,
    _auth_user: AuthUser,
    Path(id): Path<String>,
) -> AppResult<Json<impl serde::Serialize>> {
    let service = &state.inventory_service;
    let detail = service.get_inbound_detail(&id).await?;
    Ok(ok_response(serde_json::json!(detail)))
}

// ---------------------------------------------------------------------------
// Outbound
// ---------------------------------------------------------------------------

pub async fn create_outbound(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    Json(dto): Json<CreateOutboundDto>,
) -> AppResult<Json<impl serde::Serialize>> {
    let service = &state.inventory_service;
    let detail = service.create_outbound(dto, &auth_user.user_id).await?;
    Ok(ok_response(serde_json::json!(detail)))
}

pub async fn list_outbound(
    State(state): State<Arc<AppState>>,
    _auth_user: AuthUser,
    Query(filter): Query<OutboundFilter>,
) -> AppResult<Json<impl serde::Serialize>> {
    let service = &state.inventory_service;
    let page = filter.page();
    let page_size = filter.page_size();
    let (records, total) = service.list_outbound_records(filter).await?;
    Ok(list_response(records, total, page, page_size))
}

pub async fn get_outbound(
    State(state): State<Arc<AppState>>,
    _auth_user: AuthUser,
    Path(id): Path<String>,
) -> AppResult<Json<impl serde::Serialize>> {
    let service = &state.inventory_service;
    let detail = service.get_outbound_detail(&id).await?;
    Ok(ok_response(serde_json::json!(detail)))
}

// ---------------------------------------------------------------------------
// Stock
// ---------------------------------------------------------------------------

pub async fn get_stock_summary(
    State(state): State<Arc<AppState>>,
    _auth_user: AuthUser,
) -> AppResult<Json<impl serde::Serialize>> {
    let service = &state.inventory_service;
    let summary = service.get_stock_summary().await?;
    Ok(ok_response(serde_json::json!(summary)))
}

// ---------------------------------------------------------------------------
// Inventory Check
// ---------------------------------------------------------------------------

pub async fn create_inventory_check(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    Json(dto): Json<CreateInventoryCheckDto>,
) -> AppResult<Json<impl serde::Serialize>> {
    let service = &state.inventory_service;
    let detail = service.create_inventory_check(dto, &auth_user.user_id).await?;
    Ok(ok_response(serde_json::json!(detail)))
}

pub async fn list_inventory_checks(
    State(state): State<Arc<AppState>>,
    _auth_user: AuthUser,
    Query(filter): Query<InventoryCheckFilter>,
) -> AppResult<Json<impl serde::Serialize>> {
    let service = &state.inventory_service;
    let page = filter.page();
    let page_size = filter.page_size();
    let (checks, total) = service.list_inventory_checks(filter).await?;
    Ok(list_response(checks, total, page, page_size))
}
