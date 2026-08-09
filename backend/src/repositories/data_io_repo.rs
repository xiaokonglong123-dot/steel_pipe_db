use chrono::{DateTime, Utc};
use sqlx::SqlitePool;

use crate::error::AppError;

type ItemExportRow = (
    i64,
    String,
    String,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<f64>,
    String,
);
type OrderExportRow = (
    i64,
    String,
    i64,
    DateTime<Utc>,
    String,
    Option<f64>,
    Option<String>,
    Option<i64>,
);

/// Bulk data export/import for Excel/CSV operations.
///
/// All exports are item-based for the generic ERP: item master (`items`) and
/// per-item inventory movement totals (`inventory_logs`). The former
/// seamless/screen pipe and quality-cert exports were removed together with the
/// dropped pipe tables.
pub struct DataIORepo;

impl DataIORepo {
    /// Export all non-deleted items as JSON rows.
    pub async fn export_items(pool: &SqlitePool) -> Result<Vec<serde_json::Value>, AppError> {
        let rows: Vec<ItemExportRow> = sqlx::query_as(
            "SELECT id, sku, name, category, unit, spec, price, status \
             FROM items WHERE deleted_at IS NULL ORDER BY id"
        )
        .fetch_all(pool)
        .await
        .map_err(AppError::from)?;

        Ok(rows
            .into_iter()
            .map(
                |(id, sku, name, category, unit, spec, price, status)| {
                    serde_json::json!({
                        "id": id,
                        "sku": sku,
                        "name": name,
                        "category": category,
                        "unit": unit,
                        "spec": spec,
                        "price": price,
                        "status": status,
                    })
                },
            )
            .collect())
    }

    /// Export current stock per item (sum of inbound minus outbound inventory logs).
    pub async fn export_inventory(pool: &SqlitePool) -> Result<Vec<serde_json::Value>, AppError> {
        let rows: Vec<(i64, String, String, Option<f64>)> = sqlx::query_as(
            "SELECT i.id, i.sku, i.name, \
             CAST(COALESCE(SUM(CASE WHEN il.change_type = 'inbound' THEN il.quantity \
                               WHEN il.change_type = 'outbound' THEN -il.quantity \
                               ELSE 0 END), 0.0) AS REAL) as stock \
             FROM items i \
             LEFT JOIN inventory_logs il ON il.item_id = i.id \
             WHERE i.deleted_at IS NULL \
             GROUP BY i.id, i.sku, i.name ORDER BY i.id"
        )
        .fetch_all(pool)
        .await
        .map_err(AppError::from)?;

        Ok(rows
            .into_iter()
            .map(|(id, sku, name, stock)| {
                serde_json::json!({
                    "item_id": id,
                    "sku": sku,
                    "name": name,
                    "stock": stock.unwrap_or(0.0),
                })
            })
            .collect())
    }

    /// Export all purchase orders (with item count) as JSON rows.
    pub async fn export_purchase_orders(
        pool: &SqlitePool,
    ) -> Result<Vec<serde_json::Value>, AppError> {
        let rows: Vec<OrderExportRow> = sqlx::query_as(
            "SELECT id, order_no, supplier_id, order_date, status, total_amount, notes, created_by \
             FROM purchase_orders WHERE deleted_at IS NULL ORDER BY id"
        )
        .fetch_all(pool)
        .await
        .map_err(AppError::from)?;

        let mut result = Vec::new();
        for (id, order_no, supplier_id, order_date, status, total_amount, notes, created_by) in rows
        {
            let count: (i64,) =
                sqlx::query_as("SELECT COUNT(*) FROM purchase_order_items WHERE order_id = ?")
                    .bind(id)
                    .fetch_one(pool)
                    .await
                    .map_err(AppError::from)?;

            result.push(serde_json::json!({
                "id": id,
                "order_no": order_no,
                "supplier_id": supplier_id,
                "order_date": order_date,
                "status": status,
                "total_amount": total_amount,
                "notes": notes,
                "created_by": created_by,
                "items_count": count.0,
            }));
        }
        Ok(result)
    }

    /// Export all sales orders (with item count) as JSON rows.
    pub async fn export_sales_orders(
        pool: &SqlitePool,
    ) -> Result<Vec<serde_json::Value>, AppError> {
        let rows: Vec<OrderExportRow> = sqlx::query_as(
            "SELECT id, order_no, customer_id, order_date, status, total_amount, notes, created_by \
             FROM sales_orders WHERE deleted_at IS NULL ORDER BY id"
        )
        .fetch_all(pool)
        .await
        .map_err(AppError::from)?;

        let mut result = Vec::new();
        for (id, order_no, customer_id, order_date, status, total_amount, notes, created_by) in rows
        {
            let count: (i64,) =
                sqlx::query_as("SELECT COUNT(*) FROM sales_order_items WHERE order_id = ?")
                    .bind(id)
                    .fetch_one(pool)
                    .await
                    .map_err(AppError::from)?;

            result.push(serde_json::json!({
                "id": id,
                "order_no": order_no,
                "customer_id": customer_id,
                "order_date": order_date,
                "status": status,
                "total_amount": total_amount,
                "notes": notes,
                "created_by": created_by,
                "items_count": count.0,
            }));
        }
        Ok(result)
    }

    /// Batch insert items from import rows. Skips duplicates by `sku`.
    /// Returns (imported_count, errors).
    pub async fn import_items(
        pool: &SqlitePool,
        rows: &[serde_json::Value],
    ) -> Result<(u64, Vec<String>), AppError> {
        let mut tx = pool
            .begin()
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        let mut imported = 0u64;
        let mut errors = Vec::new();

        for row in rows {
            let sku = match row.get("sku").and_then(|v| v.as_str()) {
                Some(v) => v.to_string(),
                None => {
                    errors.push("Missing sku field".into());
                    continue;
                }
            };
            let name = match row.get("name").and_then(|v| v.as_str()) {
                Some(v) => v.to_string(),
                None => {
                    errors.push("Missing name field".into());
                    continue;
                }
            };

            let category = row.get("category").and_then(|v| v.as_str());
            let unit = row.get("unit").and_then(|v| v.as_str());
            let spec = row.get("spec").and_then(|v| v.as_str());
            let price = row.get("price").and_then(|v| v.as_f64());
            let status = row
                .get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("active");

            let result: Option<(i64,)> = sqlx::query_as(
                "INSERT INTO items (sku, name, category, unit, spec, price, status) \
                 VALUES (?, ?, ?, ?, ?, ?, ?) \
                 ON CONFLICT DO NOTHING \
                 RETURNING id",
            )
            .bind(&sku)
            .bind(&name)
            .bind(category)
            .bind(unit)
            .bind(spec)
            .bind(price)
            .bind(status)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| AppError::ImportError(format!("Failed to insert {}: {}", sku, e)))?;

            if result.is_some() {
                imported += 1;
            } else {
                errors.push(format!("Item sku {} already exists, skipped", sku));
            }
        }

        tx.commit().await?;
        Ok((imported, errors))
    }
}
