//! Procurement repositories.

use sqlx::PgPool;
use crate::models::procurement::{
    PoReceipt, PoReceiptItem, PurchaseRequisition, SupplierQuote, SupplierScorecard,
};

pub struct RequisitionRepo;

impl RequisitionRepo {
    pub async fn create(
        pool: &PgPool,
        tenant_id: i64,
        req_no: &str,
        title: &str,
        department_id: Option<i64>,
        applicant_id: Option<i64>,
        expected_date: Option<chrono::NaiveDate>,
        items: &serde_json::Value,
        notes: Option<&str>,
    ) -> Result<PurchaseRequisition, sqlx::Error> {
        sqlx::query_as::<_, PurchaseRequisition>(
            "INSERT INTO purchase_requisitions \
             (tenant_id, req_no, title, department_id, applicant_id, expected_date, items_json, notes) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8) \
             RETURNING id, tenant_id, req_no, title, department_id, applicant_id, expected_date, \
                       status, items_json, notes, created_at, updated_at, deleted_at",
        )
        .bind(tenant_id)
        .bind(req_no)
        .bind(title)
        .bind(department_id)
        .bind(applicant_id)
        .bind(expected_date)
        .bind(items)
        .bind(notes)
        .fetch_one(pool)
        .await
    }

    pub async fn list(
        pool: &PgPool,
        tenant_id: i64,
        status: Option<&str>,
    ) -> Result<Vec<PurchaseRequisition>, sqlx::Error> {
        sqlx::query_as::<_, PurchaseRequisition>(
            "SELECT id, tenant_id, req_no, title, department_id, applicant_id, expected_date, \
                    status, items_json, notes, created_at, updated_at, deleted_at \
             FROM purchase_requisitions WHERE tenant_id = $1 AND deleted_at IS NULL \
             AND ($2::text IS NULL OR status = $2) ORDER BY id DESC LIMIT 500",
        )
        .bind(tenant_id)
        .bind(status)
        .fetch_all(pool)
        .await
    }

    pub async fn find_by_id(pool: &PgPool, tenant_id: i64, id: i64) -> Result<Option<PurchaseRequisition>, sqlx::Error> {
        sqlx::query_as::<_, PurchaseRequisition>(
            "SELECT id, tenant_id, req_no, title, department_id, applicant_id, expected_date, \
                    status, items_json, notes, created_at, updated_at, deleted_at \
             FROM purchase_requisitions WHERE tenant_id = $1 AND id = $2 AND deleted_at IS NULL",
        )
        .bind(tenant_id)
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    pub async fn update_status(
        pool: &PgPool,
        tenant_id: i64,
        id: i64,
        status: &str,
    ) -> Result<Option<PurchaseRequisition>, sqlx::Error> {
        sqlx::query_as::<_, PurchaseRequisition>(
            "UPDATE purchase_requisitions SET status = $3, updated_at = NOW() \
             WHERE tenant_id = $1 AND id = $2 AND deleted_at IS NULL \
             RETURNING id, tenant_id, req_no, title, department_id, applicant_id, expected_date, \
                       status, items_json, notes, created_at, updated_at, deleted_at",
        )
        .bind(tenant_id)
        .bind(id)
        .bind(status)
        .fetch_optional(pool)
        .await
    }
}

pub struct ReceiptRepo;

impl ReceiptRepo {
    pub async fn create(
        pool: &PgPool,
        tenant_id: i64,
        receipt_no: &str,
        purchase_order_id: i64,
        notes: Option<&str>,
        created_by: Option<i64>,
    ) -> Result<PoReceipt, sqlx::Error> {
        sqlx::query_as::<_, PoReceipt>(
            "INSERT INTO po_receipts (tenant_id, receipt_no, purchase_order_id, notes, created_by) \
             VALUES ($1, $2, $3, $4, $5) \
             RETURNING id, tenant_id, receipt_no, purchase_order_id, received_at, status, \
                       notes, created_by, created_at",
        )
        .bind(tenant_id)
        .bind(receipt_no)
        .bind(purchase_order_id)
        .bind(notes)
        .bind(created_by)
        .fetch_one(pool)
        .await
    }

    pub async fn insert_item(
        pool: &PgPool,
        receipt_id: i64,
        pipe_id: Option<i64>,
        pipe_number: Option<&str>,
        quantity: rust_decimal::Decimal,
        remark: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO po_receipt_items (receipt_id, pipe_id, pipe_number, quantity, remark) \
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(receipt_id)
        .bind(pipe_id)
        .bind(pipe_number)
        .bind(quantity)
        .bind(remark)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn list(pool: &PgPool, tenant_id: i64, purchase_order_id: Option<i64>) -> Result<Vec<PoReceipt>, sqlx::Error> {
        sqlx::query_as::<_, PoReceipt>(
            "SELECT id, tenant_id, receipt_no, purchase_order_id, received_at, status, \
                    notes, created_by, created_at \
             FROM po_receipts WHERE tenant_id = $1 \
             AND ($2::bigint IS NULL OR purchase_order_id = $2) ORDER BY id DESC LIMIT 500",
        )
        .bind(tenant_id)
        .bind(purchase_order_id)
        .fetch_all(pool)
        .await
    }

    pub async fn items_for_receipt(pool: &PgPool, receipt_id: i64) -> Result<Vec<PoReceiptItem>, sqlx::Error> {
        sqlx::query_as::<_, PoReceiptItem>(
            "SELECT id, receipt_id, pipe_id, pipe_number, quantity, remark \
             FROM po_receipt_items WHERE receipt_id = $1 ORDER BY id",
        )
        .bind(receipt_id)
        .fetch_all(pool)
        .await
    }
}

pub struct QuoteRepo;

impl QuoteRepo {
    pub async fn create(
        pool: &PgPool,
        tenant_id: i64,
        quote_no: &str,
        supplier_id: i64,
        title: Option<&str>,
        valid_until: Option<chrono::NaiveDate>,
        total_amount: rust_decimal::Decimal,
        items: &serde_json::Value,
        notes: Option<&str>,
    ) -> Result<SupplierQuote, sqlx::Error> {
        sqlx::query_as::<_, SupplierQuote>(
            "INSERT INTO supplier_quotes \
             (tenant_id, quote_no, supplier_id, title, valid_until, total_amount, items_json, notes) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8) \
             RETURNING id, tenant_id, quote_no, supplier_id, title, valid_until, total_amount, \
                       status, items_json, notes, created_at, updated_at, deleted_at",
        )
        .bind(tenant_id)
        .bind(quote_no)
        .bind(supplier_id)
        .bind(title)
        .bind(valid_until)
        .bind(total_amount)
        .bind(items)
        .bind(notes)
        .fetch_one(pool)
        .await
    }

    pub async fn update_status(
        pool: &PgPool,
        tenant_id: i64,
        id: i64,
        status: &str,
    ) -> Result<Option<SupplierQuote>, sqlx::Error> {
        sqlx::query_as::<_, SupplierQuote>(
            "UPDATE supplier_quotes SET status = $3, updated_at = NOW() \
             WHERE tenant_id = $1 AND id = $2 AND deleted_at IS NULL \
             RETURNING id, tenant_id, quote_no, supplier_id, title, valid_until, total_amount, \
                       status, items_json, notes, created_at, updated_at, deleted_at",
        )
        .bind(tenant_id)
        .bind(id)
        .bind(status)
        .fetch_optional(pool)
        .await
    }

    pub async fn list(pool: &PgPool, tenant_id: i64, supplier_id: Option<i64>) -> Result<Vec<SupplierQuote>, sqlx::Error> {
        sqlx::query_as::<_, SupplierQuote>(
            "SELECT id, tenant_id, quote_no, supplier_id, title, valid_until, total_amount, \
                    status, items_json, notes, created_at, updated_at, deleted_at \
             FROM supplier_quotes WHERE tenant_id = $1 AND deleted_at IS NULL \
             AND ($2::bigint IS NULL OR supplier_id = $2) ORDER BY id DESC LIMIT 500",
        )
        .bind(tenant_id)
        .bind(supplier_id)
        .fetch_all(pool)
        .await
    }

    pub async fn scorecard(pool: &PgPool, tenant_id: i64, supplier_id: i64) -> Result<SupplierScorecard, sqlx::Error> {
        sqlx::query_as::<_, SupplierScorecard>(
            "SELECT $3::bigint AS supplier_id, \
                    (SELECT COUNT(*) FROM supplier_quotes WHERE tenant_id = $1 AND supplier_id = $3 AND deleted_at IS NULL) AS quote_count, \
                    (SELECT COUNT(*) FROM purchase_orders WHERE supplier_id = $3 AND deleted_at IS NULL) AS order_count, \
                    COALESCE((SELECT SUM(total_amount) FROM purchase_orders WHERE supplier_id = $3 AND deleted_at IS NULL), 0)::NUMERIC AS order_total",
        )
        .bind(tenant_id)
        .bind(supplier_id)
        .bind(supplier_id)
        .fetch_one(pool)
        .await
    }
}
