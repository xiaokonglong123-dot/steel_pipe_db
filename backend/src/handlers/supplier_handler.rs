use axum::{
    extract::{Extension, Path, Query},
    Json,
};
use serde::Deserialize;
use sqlx::SqlitePool;

use validator::Validate;

use crate::dto::common::PaginationParams;
use crate::dto::supplier_dto::{
    CreateSupplierRequest, SupplierFilterParams, UpdateSupplierRequest,
};
use crate::error::AppError;
use crate::models::supplier::Supplier;
use crate::response::{ApiResponse, PaginatedResponse};
use crate::services::supplier_service::SupplierService;

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: String,
}

/// GET `/api/v1/suppliers` — Paginated list of suppliers
///
/// Supports filtering by code, name, status, etc.
/// Returns paginated supplier results.
pub async fn list_suppliers_handler(
    Extension(pool): Extension<SqlitePool>,
    Query(filter): Query<SupplierFilterParams>,
) -> Result<Json<PaginatedResponse<Supplier>>, AppError> {
    let pagination = PaginationParams {
        page: filter.page,
        page_size: filter.page_size,
        sort_by: filter.sort_by.clone(),
        sort_order: filter.sort_order.clone(),
    };
    let page = pagination.page();
    let page_size = pagination.page_size();

    let (items, total) = SupplierService::list(&pool, &filter, &pagination).await?;

    Ok(PaginatedResponse::ok(items, total, page, page_size))
}

/// POST `/api/v1/suppliers` — Create a supplier
///
/// Creates a new supplier with contact and qualification info.
/// Validates request body. Returns the created supplier.
pub async fn create_supplier_handler(
    Extension(pool): Extension<SqlitePool>,
    Json(req): Json<CreateSupplierRequest>,
) -> Result<Json<ApiResponse<Supplier>>, AppError> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let supplier = SupplierService::create(&pool, &req).await?;
    Ok(ApiResponse::ok(supplier))
}

/// GET `/api/v1/suppliers/{id}` — Get supplier details
///
/// Returns a single supplier by ID. Returns 404 if not found.
pub async fn get_supplier_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<Supplier>>, AppError> {
    let supplier = SupplierService::get(&pool, id).await?;
    Ok(ApiResponse::ok(supplier))
}

/// PUT `/api/v1/suppliers/{id}` — Update supplier info
///
/// Updates an existing supplier. Validates request body.
/// Returns 404 if not found.
pub async fn update_supplier_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateSupplierRequest>,
) -> Result<Json<ApiResponse<Supplier>>, AppError> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let supplier = SupplierService::update(&pool, id, &req).await?;
    Ok(ApiResponse::ok(supplier))
}

/// DELETE `/api/v1/suppliers/{id}` — Soft-delete a supplier
///
/// Soft-deletes a supplier. Returns 404 if not found.
pub async fn delete_supplier_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    SupplierService::delete(&pool, id).await?;
    Ok(ApiResponse::ok("Supplier deleted successfully".into()))
}

/// GET `/api/v1/suppliers/search?q={keyword}` — Search suppliers by keyword
///
/// Searches suppliers by keyword (code, name, contact).
pub async fn search_suppliers_handler(
    Extension(pool): Extension<SqlitePool>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<ApiResponse<Vec<Supplier>>>, AppError> {
    let results = SupplierService::search(&pool, &query.q).await?;
    Ok(ApiResponse::ok(results))
}

/// GET `/api/v1/suppliers/active` — List active suppliers for dropdown
///
/// Returns all active suppliers (for dropdown selection forms).
pub async fn list_active_suppliers_handler(
    Extension(pool): Extension<SqlitePool>,
) -> Result<Json<ApiResponse<Vec<Supplier>>>, AppError> {
    let suppliers = SupplierService::list_active(&pool).await?;
    Ok(ApiResponse::ok(suppliers))
}
