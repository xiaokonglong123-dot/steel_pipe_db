//! Procurement repositories.

use sqlx::SqlitePool;
use crate::models::procurement::{
    PoReceipt, PoReceiptItem, PurchaseRequisition, SupplierQuote, SupplierScorecard,
};

pub struct RequisitionRepo;

impl RequisitionRepo {
    pub async fn create(
        pool: &SqlitePool,
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
             VALUES (?, ?, ?, ?, ?, ?, ?, ?) \
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
        pool: &SqlitePool,
        tenant_id: i64,
        status: Option<&str>,
    ) -> Result<Vec<PurchaseRequisition>, sqlx::Error> {
        sqlx::query_as::<_, PurchaseRequisition>(
            "SELECT id, tenant_id, req_no, title, department_id, applicant_id, expected_date, \
                    status, items_json, notes, created_at, updated_at, deleted_at \
             FROM purchase_requisitions WHERE tenant_id = ? AND deleted_at IS NULL \
             AND (? IS NULL OR status = ?) ORDER BY id DESC LIMIT 500",
        )
        .bind(tenant_id)
        .bind(status)
        .bind(status)
        .fetch_all(pool)
        .await
    }

    pub async fn find_by_id(pool: &SqlitePool, tenant_id: i64, id: i64) -> Result<Option<PurchaseRequisition>, sqlx::Error> {
        sqlx::query_as::<_, PurchaseRequisition>(
            "SELECT id, tenant_id, req_no, title, department_id, applicant_id, expected_date, \
                    status, items_json, notes, created_at, updated_at, deleted_at \
             FROM purchase_requisitions WHERE tenant_id = ? AND id = ? AND deleted_at IS NULL",
        )
        .bind(tenant_id)
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    pub async fn update_status(
        pool: &SqlitePool,
        tenant_id: i64,
        id: i64,
        status: &str,
    ) -> Result<Option<PurchaseRequisition>, sqlx::Error> {
        sqlx::query_as::<_, PurchaseRequisition>(
            "UPDATE purchase_requisitions SET status = ?, updated_at = datetime('now') \
             WHERE tenant_id = ? AND id = ? AND deleted_at IS NULL \
             RETURNING id, tenant_id, req_no, title, department_id, applicant_id, expected_date, \
                       status, items_json, notes, created_at, updated_at, deleted_at",
        )
        .bind(status)
        .bind(tenant_id)
        .bind(id)
        .fetch_optional(pool)
        .await
    }
}

pub struct ReceiptRepo;

impl ReceiptRepo {
    pub async fn create(
        pool: &SqlitePool,
        tenant_id: i64,
        receipt_no: &str,
        purchase_order_id: i64,
        notes: Option<&str>,
        created_by: Option<i64>,
    ) -> Result<PoReceipt, sqlx::Error> {
        sqlx::query_as::<_, PoReceipt>(
            "INSERT INTO po_receipts (tenant_id, receipt_no, purchase_order_id, notes, created_by) \
             VALUES (?, ?, ?, ?, ?) \
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
        pool: &SqlitePool,
        receipt_id: i64,
        item_id: Option<i64>,
        sku: Option<&str>,
        quantity: f64,
        remark: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO po_receipt_items (receipt_id, item_id, sku, quantity, remark) \
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(receipt_id)
        .bind(item_id)
        .bind(sku)
        .bind(quantity)
        .bind(remark)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn list(pool: &SqlitePool, tenant_id: i64, purchase_order_id: Option<i64>) -> Result<Vec<PoReceipt>, sqlx::Error> {
        sqlx::query_as::<_, PoReceipt>(
            "SELECT id, tenant_id, receipt_no, purchase_order_id, received_at, status, \
                    notes, created_by, created_at \
             FROM po_receipts WHERE tenant_id = ? \
             AND (? IS NULL OR purchase_order_id = ?) ORDER BY id DESC LIMIT 500",
        )
        .bind(tenant_id)
        .bind(purchase_order_id)
        .bind(purchase_order_id)
        .fetch_all(pool)
        .await
    }

    pub async fn items_for_receipt(pool: &SqlitePool, receipt_id: i64) -> Result<Vec<PoReceiptItem>, sqlx::Error> {
        sqlx::query_as::<_, PoReceiptItem>(
            "SELECT id, receipt_id, item_id, sku, quantity, remark \
             FROM po_receipt_items WHERE receipt_id = ? ORDER BY id",
        )
        .bind(receipt_id)
        .fetch_all(pool)
        .await
    }
}

pub struct QuoteRepo;

impl QuoteRepo {
    pub async fn create(
        pool: &SqlitePool,
        tenant_id: i64,
        quote_no: &str,
        supplier_id: i64,
        title: Option<&str>,
        valid_until: Option<chrono::NaiveDate>,
        total_amount: f64,
        items: &serde_json::Value,
        notes: Option<&str>,
    ) -> Result<SupplierQuote, sqlx::Error> {
        sqlx::query_as::<_, SupplierQuote>(
            "INSERT INTO supplier_quotes \
             (tenant_id, quote_no, supplier_id, title, valid_until, total_amount, items_json, notes) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?) \
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
        pool: &SqlitePool,
        tenant_id: i64,
        id: i64,
        status: &str,
    ) -> Result<Option<SupplierQuote>, sqlx::Error> {
        sqlx::query_as::<_, SupplierQuote>(
            "UPDATE supplier_quotes SET status = ?, updated_at = datetime('now') \
             WHERE tenant_id = ? AND id = ? AND deleted_at IS NULL \
             RETURNING id, tenant_id, quote_no, supplier_id, title, valid_until, total_amount, \
                       status, items_json, notes, created_at, updated_at, deleted_at",
        )
        .bind(status)
        .bind(tenant_id)
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    pub async fn list(pool: &SqlitePool, tenant_id: i64, supplier_id: Option<i64>) -> Result<Vec<SupplierQuote>, sqlx::Error> {
        sqlx::query_as::<_, SupplierQuote>(
            "SELECT id, tenant_id, quote_no, supplier_id, title, valid_until, total_amount, \
                    status, items_json, notes, created_at, updated_at, deleted_at \
             FROM supplier_quotes WHERE tenant_id = ? AND deleted_at IS NULL \
             AND (? IS NULL OR supplier_id = ?) ORDER BY id DESC LIMIT 500",
        )
        .bind(tenant_id)
        .bind(supplier_id)
        .bind(supplier_id)
        .fetch_all(pool)
        .await
    }

    pub async fn scorecard(pool: &SqlitePool, tenant_id: i64, supplier_id: i64) -> Result<SupplierScorecard, sqlx::Error> {
        sqlx::query_as::<_, SupplierScorecard>(
            "SELECT ? AS supplier_id, \
                    (SELECT COUNT(*) FROM supplier_quotes WHERE tenant_id = ? AND supplier_id = ? AND deleted_at IS NULL) AS quote_count, \
                    (SELECT COUNT(*) FROM purchase_orders WHERE supplier_id = ? AND deleted_at IS NULL) AS order_count, \
                    CAST(COALESCE((SELECT SUM(total_amount) FROM purchase_orders WHERE supplier_id = ? AND deleted_at IS NULL), 0.0) AS REAL) AS order_total",
        )
        .bind(supplier_id)
        .bind(tenant_id)
        .bind(supplier_id)
        .bind(supplier_id)
        .bind(supplier_id)
        .fetch_one(pool)
        .await
    }
}
