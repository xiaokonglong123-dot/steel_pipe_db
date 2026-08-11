use axum::extract::{Extension, Path, Query};
use axum::Json;
use serde::Deserialize;
use sqlx::SqlitePool;

use crate::error::AppError;
use crate::middleware::auth::AuthUser;
use crate::response::{ApiResponse, PaginatedResponse};
use crate::services::finance_service::{
    self, CreateAccountRequest, CreateInvoiceRequest, CreateJournalEntryRequest,
    CreatePaymentRequest,
};

#[derive(Debug, Deserialize)]
pub struct AccountQuery {
    pub account_type: Option<String>,
    #[serde(default)]
    pub active_only: bool,
}

#[derive(Debug, Deserialize)]
pub struct PageQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

pub async fn create_account(
    Extension(pool): Extension<SqlitePool>,
    user: AuthUser,
    Json(req): Json<CreateAccountRequest>,
) -> Result<Json<ApiResponse<crate::repos::finance_repo::AccountRow>>, AppError> {
    Ok(Json(ApiResponse::created(
        finance_service::create_account(&pool, &req, &user).await?,
    )))
}

pub async fn list_accounts(
    Extension(pool): Extension<SqlitePool>,
    Query(query): Query<AccountQuery>,
) -> Result<Json<ApiResponse<Vec<crate::repos::finance_repo::AccountRow>>>, AppError> {
    Ok(Json(ApiResponse::ok(
        finance_service::list_accounts(&pool, query.account_type.as_deref(), query.active_only)
            .await?,
    )))
}

pub async fn create_journal_entry(
    Extension(pool): Extension<SqlitePool>,
    user: AuthUser,
    Json(req): Json<CreateJournalEntryRequest>,
) -> Result<Json<ApiResponse<crate::repos::finance_repo::JournalEntryRow>>, AppError> {
    Ok(Json(ApiResponse::created(
        finance_service::create_journal_entry(&pool, &req, &user).await?,
    )))
}

pub async fn list_journal_entries(
    Extension(pool): Extension<SqlitePool>,
    Query(query): Query<PageQuery>,
) -> Result<Json<PaginatedResponse<crate::repos::finance_repo::JournalEntryRow>>, AppError> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).clamp(1, 200);
    let (rows, total) = finance_service::list_journal_entries(&pool, page, page_size).await?;
    Ok(Json(PaginatedResponse::ok(rows, total, page, page_size)))
}

pub async fn post_journal_entry(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<crate::repos::finance_repo::JournalEntryRow>>, AppError> {
    Ok(Json(ApiResponse::ok(
        finance_service::post_journal_entry(&pool, id).await?,
    )))
}

pub async fn create_invoice(
    Extension(pool): Extension<SqlitePool>,
    user: AuthUser,
    Json(req): Json<CreateInvoiceRequest>,
) -> Result<Json<ApiResponse<crate::repos::finance_repo::InvoiceRow>>, AppError> {
    Ok(Json(ApiResponse::created(
        finance_service::create_invoice(&pool, &req, &user).await?,
    )))
}

pub async fn list_invoices(
    Extension(pool): Extension<SqlitePool>,
    Query(query): Query<PageQuery>,
) -> Result<Json<PaginatedResponse<crate::repos::finance_repo::InvoiceRow>>, AppError> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).clamp(1, 200);
    let (rows, total) = finance_service::list_invoices(&pool, page, page_size).await?;
    Ok(Json(PaginatedResponse::ok(rows, total, page, page_size)))
}

pub async fn create_payment(
    Extension(pool): Extension<SqlitePool>,
    user: AuthUser,
    Json(req): Json<CreatePaymentRequest>,
) -> Result<Json<ApiResponse<crate::repos::finance_repo::PaymentRow>>, AppError> {
    Ok(Json(ApiResponse::created(
        finance_service::create_payment(&pool, &req, &user).await?,
    )))
}

pub async fn list_payments(
    Extension(pool): Extension<SqlitePool>,
    Query(query): Query<PageQuery>,
) -> Result<Json<PaginatedResponse<crate::repos::finance_repo::PaymentRow>>, AppError> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).clamp(1, 200);
    let (rows, total) = finance_service::list_payments(&pool, page, page_size).await?;
    Ok(Json(PaginatedResponse::ok(rows, total, page, page_size)))
}

pub async fn trial_balance(
    Extension(pool): Extension<SqlitePool>,
) -> Result<Json<ApiResponse<Vec<finance_service::TrialBalanceRow>>>, AppError> {
    Ok(Json(ApiResponse::ok(
        finance_service::trial_balance(&pool).await?,
    )))
}
