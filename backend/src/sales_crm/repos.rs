//! Sales CRM repositories.

use sqlx::PgPool;
use crate::models::sales_crm::{CustomerCredit, SalesQuote, SalesShipment};

pub struct ShipmentRepo;

impl ShipmentRepo {
    pub async fn create(
        pool: &PgPool,
        tenant_id: i64,
        shipment_no: &str,
        sales_order_id: i64,
        carrier: Option<&str>,
        tracking_no: Option<&str>,
        items: &serde_json::Value,
        notes: Option<&str>,
        created_by: Option<i64>,
    ) -> Result<SalesShipment, sqlx::Error> {
        sqlx::query_as::<_, SalesShipment>(
            "INSERT INTO sales_shipments \
             (tenant_id, shipment_no, sales_order_id, carrier, tracking_no, items_json, notes, created_by) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8) \
             RETURNING id, tenant_id, shipment_no, sales_order_id, shipped_at, carrier, \
                       tracking_no, status, items_json, notes, created_by, created_at, updated_at",
        )
        .bind(tenant_id)
        .bind(shipment_no)
        .bind(sales_order_id)
        .bind(carrier)
        .bind(tracking_no)
        .bind(items)
        .bind(notes)
        .bind(created_by)
        .fetch_one(pool)
        .await
    }

    pub async fn update_status(
        pool: &PgPool,
        tenant_id: i64,
        id: i64,
        status: &str,
    ) -> Result<Option<SalesShipment>, sqlx::Error> {
        sqlx::query_as::<_, SalesShipment>(
            "UPDATE sales_shipments SET status = $3, \
                    shipped_at = CASE WHEN $3 = 'shipped' AND shipped_at IS NULL THEN NOW() ELSE shipped_at END, \
                    updated_at = NOW() \
             WHERE tenant_id = $1 AND id = $2 \
             RETURNING id, tenant_id, shipment_no, sales_order_id, shipped_at, carrier, \
                       tracking_no, status, items_json, notes, created_by, created_at, updated_at",
        )
        .bind(tenant_id)
        .bind(id)
        .bind(status)
        .fetch_optional(pool)
        .await
    }

    pub async fn list(pool: &PgPool, tenant_id: i64, sales_order_id: Option<i64>) -> Result<Vec<SalesShipment>, sqlx::Error> {
        sqlx::query_as::<_, SalesShipment>(
            "SELECT id, tenant_id, shipment_no, sales_order_id, shipped_at, carrier, \
                    tracking_no, status, items_json, notes, created_by, created_at, updated_at \
             FROM sales_shipments WHERE tenant_id = $1 \
             AND ($2::bigint IS NULL OR sales_order_id = $2) ORDER BY id DESC LIMIT 500",
        )
        .bind(tenant_id)
        .bind(sales_order_id)
        .fetch_all(pool)
        .await
    }
}

pub struct SalesQuoteRepo;

impl SalesQuoteRepo {
    pub async fn create(
        pool: &PgPool,
        tenant_id: i64,
        quote_no: &str,
        customer_id: i64,
        valid_until: Option<chrono::NaiveDate>,
        total_amount: rust_decimal::Decimal,
        items: &serde_json::Value,
        notes: Option<&str>,
    ) -> Result<SalesQuote, sqlx::Error> {
        sqlx::query_as::<_, SalesQuote>(
            "INSERT INTO sales_quotes \
             (tenant_id, quote_no, customer_id, valid_until, total_amount, items_json, notes) \
             VALUES ($1, $2, $3, $4, $5, $6, $7) \
             RETURNING id, tenant_id, quote_no, customer_id, quote_date, valid_until, \
                       total_amount, status, items_json, notes, created_at, updated_at, deleted_at",
        )
        .bind(tenant_id)
        .bind(quote_no)
        .bind(customer_id)
        .bind(valid_until)
        .bind(total_amount)
        .bind(items)
        .bind(notes)
        .fetch_one(pool)
        .await
    }

    pub async fn find_by_id(pool: &PgPool, tenant_id: i64, id: i64) -> Result<Option<SalesQuote>, sqlx::Error> {
        sqlx::query_as::<_, SalesQuote>(
            "SELECT id, tenant_id, quote_no, customer_id, quote_date, valid_until, \
                    total_amount, status, items_json, notes, created_at, updated_at, deleted_at \
             FROM sales_quotes WHERE tenant_id = $1 AND id = $2 AND deleted_at IS NULL",
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
    ) -> Result<Option<SalesQuote>, sqlx::Error> {
        sqlx::query_as::<_, SalesQuote>(
            "UPDATE sales_quotes SET status = $3, updated_at = NOW() \
             WHERE tenant_id = $1 AND id = $2 AND deleted_at IS NULL \
             RETURNING id, tenant_id, quote_no, customer_id, quote_date, valid_until, \
                       total_amount, status, items_json, notes, created_at, updated_at, deleted_at",
        )
        .bind(tenant_id)
        .bind(id)
        .bind(status)
        .fetch_optional(pool)
        .await
    }

    pub async fn list(pool: &PgPool, tenant_id: i64, customer_id: Option<i64>) -> Result<Vec<SalesQuote>, sqlx::Error> {
        sqlx::query_as::<_, SalesQuote>(
            "SELECT id, tenant_id, quote_no, customer_id, quote_date, valid_until, \
                    total_amount, status, items_json, notes, created_at, updated_at, deleted_at \
             FROM sales_quotes WHERE tenant_id = $1 AND deleted_at IS NULL \
             AND ($2::bigint IS NULL OR customer_id = $2) ORDER BY id DESC LIMIT 500",
        )
        .bind(tenant_id)
        .bind(customer_id)
        .fetch_all(pool)
        .await
    }

    /// Convert a quote into a sales order: creates a draft sales order with
    /// the quote's items. Returns the new order id.
    pub async fn convert_to_order(
        pool: &PgPool,
        tenant_id: i64,
        quote: &SalesQuote,
        order_no: &str,
    ) -> Result<i64, sqlx::Error> {
        let order_id: i64 = sqlx::query_scalar(
            "INSERT INTO sales_orders (tenant_id, order_number, customer_id, order_date, total_amount, status) \
             VALUES ($1, $2, $3, CURRENT_DATE, $4, 'draft') RETURNING id",
        )
        .bind(tenant_id)
        .bind(order_no)
        .bind(quote.customer_id)
        .bind(quote.total_amount)
        .fetch_one(pool)
        .await?;
        // Copy quote items into sales_order_items (pipe_type/grade/od/wt/length/qty/unit_price/total_price).
        if let Some(items) = quote.items_json.as_array() {
            for item in items {
                sqlx::query(
                    "INSERT INTO sales_order_items \
                     (order_id, pipe_type, grade, od, wt, length, quantity, unit_price, total_price) \
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
                )
                .bind(order_id)
                .bind(item.get("pipe_type").and_then(|v| v.as_str()).unwrap_or(""))
                .bind(item.get("grade").and_then(|v| v.as_str()).unwrap_or(""))
                .bind(item.get("od").and_then(|v| v.as_f64()).unwrap_or(0.0))
                .bind(item.get("wt").and_then(|v| v.as_f64()).unwrap_or(0.0))
                .bind(item.get("length").and_then(|v| v.as_f64()).unwrap_or(0.0))
                .bind(item.get("quantity").and_then(|v| v.as_f64()).unwrap_or(0.0))
                .bind(item.get("unit_price").and_then(|v| v.as_f64()).unwrap_or(0.0))
                .bind(item.get("total_price").and_then(|v| v.as_f64()).unwrap_or(0.0))
                .execute(pool)
                .await?;
            }
        }
        Ok(order_id)
    }

    pub async fn credit(pool: &PgPool, tenant_id: i64, customer_id: i64) -> Result<CustomerCredit, sqlx::Error> {
        sqlx::query_as::<_, CustomerCredit>(
            "SELECT $3::bigint AS customer_id, \
                    COALESCE((SELECT SUM(total_amount) FROM finance_invoices \
                              WHERE tenant_id = $1 AND party_id = $3 AND invoice_type = 'sales' \
                                AND status IN ('confirmed', 'draft')), 0)::BIGINT AS open_invoice_total, \
                    COALESCE((SELECT SUM(total_amount) FROM sales_orders \
                              WHERE customer_id = $3 AND deleted_at IS NULL), 0)::BIGINT AS lifetime_sales",
        )
        .bind(tenant_id)
        .bind(customer_id)
        .bind(customer_id)
        .fetch_one(pool)
        .await
    }
}
