//! Sales CRM services — shipments lifecycle, quotes, quote→order conversion.

use sqlx::PgPool;

use crate::dto::sales_crm_dto::{CreateSalesQuoteRequest, CreateShipmentRequest};
use crate::error::AppError;
use crate::models::sales_crm::{CustomerCredit, SalesQuote, SalesShipment};
use crate::sales_crm::repos::{SalesQuoteRepo, ShipmentRepo};

pub struct SalesCrmService;

impl SalesCrmService {
    // -----------------------------------------------------------------------
    // Shipments
    // -----------------------------------------------------------------------

    pub async fn create_shipment(
        pool: &PgPool,
        tenant_id: i64,
        created_by: Option<i64>,
        dto: &CreateShipmentRequest,
    ) -> Result<SalesShipment, AppError> {
        if dto.items.is_empty() {
            return Err(AppError::Validation("Shipment needs at least one item".into()));
        }
        let shipment_no = format!("SH-{}-{}", chrono::Utc::now().format("%Y%m%d"), seq(pool, "sales_shipments").await?);
        ShipmentRepo::create(
            pool, tenant_id, &shipment_no, dto.sales_order_id, dto.carrier.as_deref(),
            dto.tracking_no.as_deref(), &serde_json::Value::Array(dto.items.clone()),
            dto.notes.as_deref(), created_by,
        )
        .await
        .map_err(AppError::from)
    }

    pub async fn update_shipment_status(
        pool: &PgPool,
        tenant_id: i64,
        id: i64,
        status: &str,
    ) -> Result<SalesShipment, AppError> {
        if !matches!(status, "pending" | "shipped" | "delivered") {
            return Err(AppError::Validation(format!("Invalid shipment status: {}", status)));
        }
        ShipmentRepo::update_status(pool, tenant_id, id, status)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Shipment not found: {}", id)))
    }

    pub async fn list_shipments(
        pool: &PgPool,
        tenant_id: i64,
        sales_order_id: Option<i64>,
    ) -> Result<Vec<SalesShipment>, AppError> {
        ShipmentRepo::list(pool, tenant_id, sales_order_id).await.map_err(AppError::from)
    }

    // -----------------------------------------------------------------------
    // Quotes
    // -----------------------------------------------------------------------

    pub async fn create_quote(
        pool: &PgPool,
        tenant_id: i64,
        dto: &CreateSalesQuoteRequest,
    ) -> Result<SalesQuote, AppError> {
        let quote_no = format!("QT-{}-{}", chrono::Utc::now().format("%Y%m%d"), seq(pool, "sales_quotes").await?);
        SalesQuoteRepo::create(
            pool, tenant_id, &quote_no, dto.customer_id, dto.valid_until, dto.total_amount,
            &serde_json::Value::Array(dto.items.clone()), dto.notes.as_deref(),
        )
        .await
        .map_err(AppError::from)
    }

    pub async fn list_quotes(pool: &PgPool, tenant_id: i64, customer_id: Option<i64>) -> Result<Vec<SalesQuote>, AppError> {
        SalesQuoteRepo::list(pool, tenant_id, customer_id).await.map_err(AppError::from)
    }

    pub async fn get_quote(pool: &PgPool, tenant_id: i64, id: i64) -> Result<SalesQuote, AppError> {
        SalesQuoteRepo::find_by_id(pool, tenant_id, id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Quote not found: {}", id)))
    }

    /// Convert a confirmed quote into a draft sales order; marks the quote converted.
    pub async fn convert_quote(pool: &PgPool, tenant_id: i64, id: i64) -> Result<i64, AppError> {
        let quote = Self::get_quote(pool, tenant_id, id).await?;
        if quote.status != "confirmed" {
            return Err(AppError::Validation(format!(
                "Only confirmed quotes can be converted (status: {})",
                quote.status
            )));
        }
        let order_no = format!("SO-{}-{}", chrono::Utc::now().format("%Y%m%d"), seq(pool, "sales_orders").await?);
        let order_id = SalesQuoteRepo::convert_to_order(pool, tenant_id, &quote, &order_no)
            .await
            .map_err(AppError::from)?;
        SalesQuoteRepo::update_status(pool, tenant_id, id, "converted")
            .await
            .map_err(AppError::from)?;
        Ok(order_id)
    }

    pub async fn update_quote_status(
        pool: &PgPool,
        tenant_id: i64,
        id: i64,
        status: &str,
    ) -> Result<SalesQuote, AppError> {
        if !matches!(status, "draft" | "confirmed" | "converted" | "expired") {
            return Err(AppError::Validation(format!("Invalid quote status: {}", status)));
        }
        SalesQuoteRepo::update_status(pool, tenant_id, id, status)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Quote not found: {}", id)))
    }

    pub async fn customer_credit(pool: &PgPool, tenant_id: i64, customer_id: i64) -> Result<CustomerCredit, AppError> {
        SalesQuoteRepo::credit(pool, tenant_id, customer_id).await.map_err(AppError::from)
    }
}

/// Per-table sequence helper for document numbers.
async fn seq(pool: &PgPool, table: &str) -> Result<i64, AppError> {
    let n: i64 = sqlx::query_scalar(&format!("SELECT COALESCE(MAX(id), 0) + 1 FROM {}", table))
        .fetch_one(pool)
        .await
        .map_err(AppError::from)?;
    Ok(n)
}
