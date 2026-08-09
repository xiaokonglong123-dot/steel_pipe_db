//! Procurement services — requisitions (submit/approve), goods receipts,
//! supplier quotes, scorecard.

use sqlx::SqlitePool;

use crate::dto::procurement_dto::{
    CreateQuoteRequest, CreateReceiptRequest, CreateRequisitionRequest, UpdateQuoteStatusRequest,
};
use crate::error::AppError;
use crate::models::procurement::{
    PoReceipt, PoReceiptItem, PurchaseRequisition, SupplierQuote, SupplierScorecard,
};
use crate::procurement::repos::{QuoteRepo, ReceiptRepo, RequisitionRepo};

pub struct ProcurementService;

impl ProcurementService {
    // -----------------------------------------------------------------------
    // Requisitions
    // -----------------------------------------------------------------------

    pub async fn create_requisition(
        pool: &SqlitePool,
        tenant_id: i64,
        applicant_id: Option<i64>,
        dto: &CreateRequisitionRequest,
    ) -> Result<PurchaseRequisition, AppError> {
        if dto.title.trim().is_empty() {
            return Err(AppError::Validation("Title is required".into()));
        }
        let req_no = format!("PR-{}-{}", chrono::Utc::now().format("%Y%m%d"), seq(pool, "purchase_requisitions").await?);
        RequisitionRepo::create(
            pool, tenant_id, &req_no, dto.title.trim(), dto.department_id, applicant_id,
            dto.expected_date, &serde_json::Value::Array(dto.items.clone()), dto.notes.as_deref(),
        )
        .await
        .map_err(AppError::from)
    }

    pub async fn list_requisitions(
        pool: &SqlitePool,
        tenant_id: i64,
        status: Option<&str>,
    ) -> Result<Vec<PurchaseRequisition>, AppError> {
        RequisitionRepo::list(pool, tenant_id, status).await.map_err(AppError::from)
    }

    pub async fn get_requisition(pool: &SqlitePool, tenant_id: i64, id: i64) -> Result<PurchaseRequisition, AppError> {
        RequisitionRepo::find_by_id(pool, tenant_id, id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Requisition not found: {}", id)))
    }

    /// Transition a requisition's status with a simple state guard.
    pub async fn update_requisition_status(
        pool: &SqlitePool,
        tenant_id: i64,
        id: i64,
        status: &str,
    ) -> Result<PurchaseRequisition, AppError> {
        if !matches!(status, "draft" | "submitted" | "approved" | "rejected") {
            return Err(AppError::Validation(format!("Invalid status: {}", status)));
        }
        let req = Self::get_requisition(pool, tenant_id, id).await?;
        // draft → submitted → approved/rejected; no skipping backward.
        if req.status == "approved" && status != "draft" {
            return Err(AppError::Validation("Approved requisitions cannot change status".into()));
        }
        RequisitionRepo::update_status(pool, tenant_id, id, status)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Requisition not found: {}", id)))
    }

    // -----------------------------------------------------------------------
    // Goods receipts
    // -----------------------------------------------------------------------

    pub async fn create_receipt(
        pool: &SqlitePool,
        tenant_id: i64,
        created_by: Option<i64>,
        dto: &CreateReceiptRequest,
    ) -> Result<PoReceipt, AppError> {
        if dto.items.is_empty() {
            return Err(AppError::Validation("Receipt needs at least one item".into()));
        }
        let receipt_no = format!("GR-{}-{}", chrono::Utc::now().format("%Y%m%d"), seq(pool, "po_receipts").await?);
        let receipt = ReceiptRepo::create(pool, tenant_id, &receipt_no, dto.purchase_order_id, dto.notes.as_deref(), created_by)
            .await
            .map_err(AppError::from)?;
        for item in &dto.items {
            ReceiptRepo::insert_item(
                pool, receipt.id, item.item_id, item.sku.as_deref(), item.quantity, item.remark.as_deref(),
            )
            .await
            .map_err(AppError::from)?;
        }
        Ok(receipt)
    }

    pub async fn list_receipts(
        pool: &SqlitePool,
        tenant_id: i64,
        purchase_order_id: Option<i64>,
    ) -> Result<Vec<PoReceipt>, AppError> {
        ReceiptRepo::list(pool, tenant_id, purchase_order_id).await.map_err(AppError::from)
    }

    pub async fn get_receipt(pool: &SqlitePool, tenant_id: i64, id: i64) -> Result<(PoReceipt, Vec<PoReceiptItem>), AppError> {
        let receipts = ReceiptRepo::list(pool, tenant_id, None).await.map_err(AppError::from)?;
        let receipt = receipts.into_iter().find(|r| r.id == id)
            .ok_or_else(|| AppError::NotFound(format!("Receipt not found: {}", id)))?;
        let items = ReceiptRepo::items_for_receipt(pool, id).await.map_err(AppError::from)?;
        Ok((receipt, items))
    }

    // -----------------------------------------------------------------------
    // Supplier quotes
    // -----------------------------------------------------------------------

    pub async fn create_quote(
        pool: &SqlitePool,
        tenant_id: i64,
        dto: &CreateQuoteRequest,
    ) -> Result<SupplierQuote, AppError> {
        let quote_no = format!("SQ-{}-{}", chrono::Utc::now().format("%Y%m%d"), seq(pool, "supplier_quotes").await?);
        QuoteRepo::create(
            pool, tenant_id, &quote_no, dto.supplier_id, dto.title.as_deref(), dto.valid_until,
            dto.total_amount, &serde_json::Value::Array(dto.items.clone()), dto.notes.as_deref(),
        )
        .await
        .map_err(AppError::from)
    }

    pub async fn list_quotes(pool: &SqlitePool, tenant_id: i64, supplier_id: Option<i64>) -> Result<Vec<SupplierQuote>, AppError> {
        QuoteRepo::list(pool, tenant_id, supplier_id).await.map_err(AppError::from)
    }

    pub async fn update_quote_status(
        pool: &SqlitePool,
        tenant_id: i64,
        id: i64,
        dto: &UpdateQuoteStatusRequest,
    ) -> Result<SupplierQuote, AppError> {
        if !matches!(dto.status.as_str(), "draft" | "sent" | "accepted" | "expired") {
            return Err(AppError::Validation(format!("Invalid quote status: {}", dto.status)));
        }
        QuoteRepo::update_status(pool, tenant_id, id, &dto.status)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Quote not found: {}", id)))
    }

    pub async fn supplier_scorecard(pool: &SqlitePool, tenant_id: i64, supplier_id: i64) -> Result<SupplierScorecard, AppError> {
        QuoteRepo::scorecard(pool, tenant_id, supplier_id).await.map_err(AppError::from)
    }
}

/// Per-table sequence helper for document numbers.
async fn seq(pool: &SqlitePool, table: &str) -> Result<i64, AppError> {
    let n: i64 = sqlx::query_scalar(&format!("SELECT COALESCE(MAX(id), 0) + 1 FROM {}", table))
        .fetch_one(pool)
        .await
        .map_err(AppError::from)?;
    Ok(n)
}
