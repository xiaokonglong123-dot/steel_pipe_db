//! Finance HTTP handlers — accounts, journal entries, invoices, payments.

use axum::extract::{Extension, Path, Query};
use axum::Json;
use serde::Deserialize;
use sqlx::SqlitePool;

use crate::dto::finance_dto::{
    CreateAccountRequest, CreateInvoiceRequest, CreateJournalEntryRequest, CreatePaymentRequest,
    UpdateAccountRequest,
};
use crate::error::AppError;
use crate::finance::services::FinanceService;
use crate::handlers::auth_handler::AuthenticatedUser;
use crate::models::finance::{
    Account, FinanceInvoice, FinanceInvoiceItem, FinancePayment, JournalEntry, JournalEntryDetail,
    TrialBalanceRow,
};
use crate::response::ApiResponse;

// ---------------------------------------------------------------------------
// Chart of accounts
// ---------------------------------------------------------------------------

pub async fn list_accounts(
    Extension(pool): Extension<SqlitePool>,
    user: AuthenticatedUser,
) -> Result<Json<ApiResponse<Vec<Account>>>, AppError> {
    let items = FinanceService::list_accounts(&pool, user.0.tenant_id).await?;
    Ok(ApiResponse::ok(items))
}

pub async fn create_account(
    Extension(pool): Extension<SqlitePool>,
    user: AuthenticatedUser,
    Json(payload): Json<CreateAccountRequest>,
) -> Result<Json<ApiResponse<Account>>, AppError> {
    let item = FinanceService::create_account(&pool, user.0.tenant_id, &payload).await?;
    Ok(ApiResponse::ok(item))
}

pub async fn update_account(
    Extension(pool): Extension<SqlitePool>,
    user: AuthenticatedUser,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateAccountRequest>,
) -> Result<Json<ApiResponse<Account>>, AppError> {
    let item = FinanceService::update_account(&pool, user.0.tenant_id, id, &payload).await?;
    Ok(ApiResponse::ok(item))
}

// ---------------------------------------------------------------------------
// Journal entries
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct JournalFilter {
    pub from: Option<chrono::NaiveDate>,
    pub to: Option<chrono::NaiveDate>,
}

pub async fn list_journal_entries(
    Extension(pool): Extension<SqlitePool>,
    user: AuthenticatedUser,
    Query(filter): Query<JournalFilter>,
) -> Result<Json<ApiResponse<Vec<JournalEntry>>>, AppError> {
    let items = FinanceService::list_journal_entries(&pool, user.0.tenant_id, filter.from, filter.to).await?;
    Ok(ApiResponse::ok(items))
}

pub async fn get_journal_entry(
    Extension(pool): Extension<SqlitePool>,
    user: AuthenticatedUser,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<JournalDetailResponse>>, AppError> {
    let (entry, details) = FinanceService::get_journal_entry(&pool, user.0.tenant_id, id).await?;
    Ok(ApiResponse::ok(JournalDetailResponse { entry, details }))
}

#[derive(serde::Serialize)]
pub struct JournalDetailResponse {
    pub entry: JournalEntry,
    pub details: Vec<JournalEntryDetail>,
}

pub async fn create_journal_entry(
    Extension(pool): Extension<SqlitePool>,
    user: AuthenticatedUser,
    Json(payload): Json<CreateJournalEntryRequest>,
) -> Result<Json<ApiResponse<JournalEntry>>, AppError> {
    let item = FinanceService::create_journal_entry(&pool, user.0.tenant_id, &payload, Some(user.0.user_id)).await?;
    Ok(ApiResponse::ok(item))
}

pub async fn trial_balance(
    Extension(pool): Extension<SqlitePool>,
    user: AuthenticatedUser,
) -> Result<Json<ApiResponse<Vec<TrialBalanceRow>>>, AppError> {
    let items = FinanceService::trial_balance(&pool, user.0.tenant_id).await?;
    Ok(ApiResponse::ok(items))
}

// ---------------------------------------------------------------------------
// Invoices
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct InvoiceFilter {
    pub invoice_type: Option<String>,
    pub status: Option<String>,
}

pub async fn list_invoices(
    Extension(pool): Extension<SqlitePool>,
    user: AuthenticatedUser,
    Query(filter): Query<InvoiceFilter>,
) -> Result<Json<ApiResponse<Vec<FinanceInvoice>>>, AppError> {
    let items = FinanceService::list_invoices(&pool, user.0.tenant_id, filter.invoice_type.as_deref(), filter.status.as_deref()).await?;
    Ok(ApiResponse::ok(items))
}

pub async fn get_invoice(
    Extension(pool): Extension<SqlitePool>,
    user: AuthenticatedUser,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<InvoiceDetailResponse>>, AppError> {
    let (invoice, items) = FinanceService::get_invoice(&pool, user.0.tenant_id, id).await?;
    Ok(ApiResponse::ok(InvoiceDetailResponse { invoice, items }))
}

#[derive(serde::Serialize)]
pub struct InvoiceDetailResponse {
    pub invoice: FinanceInvoice,
    pub items: Vec<FinanceInvoiceItem>,
}

pub async fn create_invoice(
    Extension(pool): Extension<SqlitePool>,
    user: AuthenticatedUser,
    Json(payload): Json<CreateInvoiceRequest>,
) -> Result<Json<ApiResponse<FinanceInvoice>>, AppError> {
    let item = FinanceService::create_invoice(&pool, user.0.tenant_id, &payload).await?;
    Ok(ApiResponse::ok(item))
}

pub async fn confirm_invoice(
    Extension(pool): Extension<SqlitePool>,
    user: AuthenticatedUser,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<FinanceInvoice>>, AppError> {
    let item = FinanceService::confirm_invoice(&pool, user.0.tenant_id, id).await?;
    Ok(ApiResponse::ok(item))
}

pub async fn void_invoice(
    Extension(pool): Extension<SqlitePool>,
    user: AuthenticatedUser,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<FinanceInvoice>>, AppError> {
    let item = FinanceService::void_invoice(&pool, user.0.tenant_id, id).await?;
    Ok(ApiResponse::ok(item))
}

// ---------------------------------------------------------------------------
// Payments
// ---------------------------------------------------------------------------

pub async fn list_payments(
    Extension(pool): Extension<SqlitePool>,
    user: AuthenticatedUser,
    Query(filter): Query<InvoiceFilter>,
) -> Result<Json<ApiResponse<Vec<FinancePayment>>>, AppError> {
    let invoice_id = filter.status.and_then(|s| s.parse::<i64>().ok());
    let items = FinanceService::list_payments(&pool, user.0.tenant_id, invoice_id).await?;
    Ok(ApiResponse::ok(items))
}

pub async fn create_payment(
    Extension(pool): Extension<SqlitePool>,
    user: AuthenticatedUser,
    Json(payload): Json<CreatePaymentRequest>,
) -> Result<Json<ApiResponse<FinancePayment>>, AppError> {
    let item = FinanceService::create_payment(&pool, user.0.tenant_id, &payload, Some(user.0.user_id)).await?;
    Ok(ApiResponse::ok(item))
}
