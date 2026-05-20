use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};
use serde::Deserialize;

use crate::error::AppResult;
use crate::handler::{list_response, ok_response, ApiListResponse, ApiResponse};
use crate::middleware::AuthUser;
use crate::service::sales_service::{
    AtpQuery, CreateCustomerDto, CreateSalesOrderDto, OrderFilter, SalesService,
    UpdateCustomerDto,
};
use crate::AppState;

// ── Pagination Query ────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct PaginationQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Deserialize)]
pub struct CustomerFilter {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub search: Option<String>,
}

// ── Customers ───────────────────────────────────────────────────────────────

pub async fn list_customers(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthUser>,
    Query(filter): Query<CustomerFilter>,
) -> AppResult<Json<ApiListResponse<serde_json::Value>>> {
    let svc = SalesService::from_state(&state);
    let page = filter.page.unwrap_or(1).max(1);
    let page_size = filter.page_size.unwrap_or(20).clamp(1, 200);
    let (items, total) = svc.list_customers(filter.search.as_deref(), page, page_size).await?;
    let items: Vec<serde_json::Value> = items.into_iter()
        .map(|c| serde_json::to_value(c).unwrap())
        .collect();
    Ok(list_response(items, total, page, page_size))
}

pub async fn create_customer(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthUser>,
    Json(dto): Json<CreateCustomerDto>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let svc = SalesService::from_state(&state);
    let customer = svc.create_customer(dto).await?;
    Ok(ok_response(serde_json::to_value(customer).unwrap()))
}

pub async fn update_customer(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthUser>,
    Path(id): Path<String>,
    Json(dto): Json<UpdateCustomerDto>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let svc = SalesService::from_state(&state);
    let customer = svc.update_customer(&id, dto).await?;
    Ok(ok_response(serde_json::to_value(customer).unwrap()))
}

pub async fn delete_customer(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthUser>,
    Path(id): Path<String>,
) -> AppResult<Json<ApiResponse<()>>> {
    let svc = SalesService::from_state(&state);
    svc.delete_customer(&id).await?;
    Ok(ok_response(()))
}

// ── Sales Orders ────────────────────────────────────────────────────────────

pub async fn create_sales_order(
    State(state): State<Arc<AppState>>,
    Extension(auth): Extension<AuthUser>,
    Json(dto): Json<CreateSalesOrderDto>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let svc = SalesService::from_state(&state);
    let result = svc.create_sales_order(dto, &auth.user_id).await?;
    Ok(ok_response(serde_json::to_value(result).unwrap()))
}

pub async fn list_sales_orders(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthUser>,
    Query(filter): Query<OrderFilter>,
) -> AppResult<Json<ApiListResponse<serde_json::Value>>> {
    let svc = SalesService::from_state(&state);
    let (items, total) = svc.list_sales_orders(&filter).await?;
    let page = filter.page.unwrap_or(1).max(1);
    let page_size = filter.page_size.unwrap_or(20).clamp(1, 200);
    let items: Vec<serde_json::Value> = items.into_iter()
        .map(|so| serde_json::to_value(so).unwrap())
        .collect();
    Ok(list_response(items, total, page, page_size))
}

pub async fn get_sales_order(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthUser>,
    Path(id): Path<String>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let svc = SalesService::from_state(&state);
    let result = svc.get_sales_order(&id).await?;
    Ok(ok_response(serde_json::to_value(result).unwrap()))
}

pub async fn approve_sales_order(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthUser>,
    Path(id): Path<String>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let svc = SalesService::from_state(&state);
    let order = svc.approve_sales_order(&id).await?;
    Ok(ok_response(serde_json::to_value(order).unwrap()))
}

pub async fn cancel_sales_order(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthUser>,
    Path(id): Path<String>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let svc = SalesService::from_state(&state);
    let order = svc.cancel_sales_order(&id).await?;
    Ok(ok_response(serde_json::to_value(order).unwrap()))
}

pub async fn get_atp(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthUser>,
    Query(query): Query<AtpQuery>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let svc = SalesService::from_state(&state);
    let result = svc.get_atp(&query).await?;
    Ok(ok_response(serde_json::to_value(result).unwrap()))
}

pub async fn link_outbound_to_so(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthUser>,
    Path(id): Path<String>,
    Json(body): Json<OutboundLinkBody>,
) -> AppResult<Json<ApiResponse<()>>> {
    let svc = SalesService::from_state(&state);
    svc.link_outbound_to_so(&body.outbound_id, &id).await?;
    Ok(ok_response(()))
}

// ── Shared DTOs ─────────────────────────────────────────────────────────────

#[derive(serde::Deserialize)]
pub struct OutboundLinkBody {
    pub outbound_id: String,
}
