// 客户管理入口：客户主数据 CRUD + 搜索
// 客户编码自动生成，支持按名称/编码搜索

use axum::{
    extract::{Extension, Path, Query},
    Json,
};
use serde::Deserialize;
use sqlx::SqlitePool;

use validator::Validate;

use crate::dto::common::PaginationParams;
use crate::dto::customer_dto::{
    CreateCustomerRequest, CustomerFilterParams, UpdateCustomerRequest,
};
use crate::error::AppError;
use crate::models::customer::Customer;
use crate::response::{ApiResponse, PaginatedResponse};
use crate::services::customer_service::CustomerService;

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: String,
}

pub async fn list_customers_handler(
    Extension(pool): Extension<SqlitePool>,
    Query(filter): Query<CustomerFilterParams>,
) -> Result<Json<PaginatedResponse<Customer>>, AppError> {
    let pagination = PaginationParams {
        page: filter.page,
        page_size: filter.page_size,
        sort_by: filter.sort_by.clone(),
        sort_order: filter.sort_order.clone(),
    };
    let page = pagination.page();
    let page_size = pagination.page_size();

    let (items, total) = CustomerService::list(&pool, &filter, &pagination).await?;

    Ok(PaginatedResponse::ok(items, total, page, page_size))
}

pub async fn create_customer_handler(
    Extension(pool): Extension<SqlitePool>,
    Json(req): Json<CreateCustomerRequest>,
) -> Result<Json<ApiResponse<Customer>>, AppError> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let customer = CustomerService::create(&pool, &req).await?;
    Ok(ApiResponse::ok(customer))
}

pub async fn get_customer_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<Customer>>, AppError> {
    let customer = CustomerService::get(&pool, id).await?;
    Ok(ApiResponse::ok(customer))
}

pub async fn update_customer_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateCustomerRequest>,
) -> Result<Json<ApiResponse<Customer>>, AppError> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let customer = CustomerService::update(&pool, id, &req).await?;
    Ok(ApiResponse::ok(customer))
}

pub async fn delete_customer_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    CustomerService::delete(&pool, id).await?;
    Ok(ApiResponse::ok("Customer deleted successfully".into()))
}

pub async fn search_customers_handler(
    Extension(pool): Extension<SqlitePool>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<ApiResponse<Vec<Customer>>>, AppError> {
    let results = CustomerService::search(&pool, &query.q).await?;
    Ok(ApiResponse::ok(results))
}

pub async fn list_active_customers_handler(
    Extension(pool): Extension<SqlitePool>,
) -> Result<Json<ApiResponse<Vec<Customer>>>, AppError> {
    let customers = CustomerService::list_active(&pool).await?;
    Ok(ApiResponse::ok(customers))
}
