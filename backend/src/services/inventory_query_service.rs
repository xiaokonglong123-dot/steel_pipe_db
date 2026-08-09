use sqlx::SqlitePool;

use crate::dto::common::PaginationParams;
use crate::dto::inventory_dto::{AtpItem, AtpQuery, InventoryFilter, InventoryStatistics, StockItem};
use crate::error::AppError;
use crate::models::inventory::InventoryLog;
use crate::repositories::inventory_log_repo::InventoryLogRepo;
use crate::repositories::inventory_repo::InventoryRepo;

/// Inventory query service — stock listing, logs, stats dashboard, and ATP calculations.
/// Stock is derived from `inventory_logs` per item (quantity-based, no pipes).
pub struct InventoryQueryService;

impl InventoryQueryService {
    /// Paginated stock listing: item master rows joined with computed on-hand quantity.
    /// Filter by category, location, item, and fuzzy SKU/name/spec search.
    /// Returns `(items, total_count)`.
    ///
    /// NOTE: SQL stays in service because it aggregates item master + inventory
    /// logs into one reporting shape with optional bind distribution — not a
    /// reusable CRUD operation.
    pub async fn list_inventory(
        pool: &SqlitePool,
        filter: &InventoryFilter,
    ) -> Result<(Vec<StockItem>, u64), AppError> {
        let pagination = PaginationParams {
            page: filter.page,
            page_size: filter.page_size,
            sort_by: None,
            sort_order: None,
        };
        let page_size = pagination.page_size();
        let offset = pagination.offset();

        let mut conditions: Vec<String> = vec!["i.deleted_at IS NULL".into()];
        let mut bind_values: Vec<String> = Vec::new();

        if let Some(ref q) = filter.q {
            if !q.is_empty() {
                conditions.push("(i.sku LIKE ? OR i.name LIKE ? OR i.spec LIKE ?)".into());
                let pattern = format!("%{}%", q);
                bind_values.push(pattern.clone());
                bind_values.push(pattern.clone());
                bind_values.push(pattern);
            }
        }
        if let Some(item_id) = filter.item_id {
            conditions.push("i.id = ?".into());
            bind_values.push(item_id.to_string());
        }
        if let Some(ref category) = filter.category {
            conditions.push("i.category = ?".into());
            bind_values.push(category.clone());
        }
        if let Some(location_id) = filter.location_id {
            // Location scoping: only count logs that touched this location.
            conditions.push("(l.from_location_id = ? OR l.to_location_id = ?)".into());
            bind_values.push(location_id.to_string());
            bind_values.push(location_id.to_string());
        }

        let where_clause = conditions.join(" AND ");

        let count_sql = format!(
            "SELECT COUNT(*) FROM (
                 SELECT i.id
                 FROM items i
                 LEFT JOIN inventory_logs l ON l.item_id = i.id
                 WHERE {where_clause}
                 GROUP BY i.id
                 HAVING COALESCE(SUM(CASE WHEN l.change_type IN ('inbound', 'check_adjust')
                                           THEN l.quantity ELSE -l.quantity END) , 0.0) > 0
             )",
            where_clause = where_clause,
        );
        let mut count_q = sqlx::query_as::<_, (i64,)>(&count_sql);
        for val in &bind_values {
            count_q = count_q.bind(val.as_str());
        }
        let total: (i64,) = count_q.fetch_one(pool).await?;

        let list_sql = format!(
            "SELECT i.id, i.sku, i.name, i.category, i.unit, i.spec, i.status,
                    CAST(COALESCE(SUM(CASE WHEN l.change_type IN ('inbound', 'check_adjust')
                                      THEN l.quantity ELSE -l.quantity END) , 0.0) AS REAL) AS quantity
             FROM items i
             LEFT JOIN inventory_logs l ON l.item_id = i.id
             WHERE {where_clause}
             GROUP BY i.id, i.sku, i.name, i.category, i.unit, i.spec, i.status
             HAVING COALESCE(SUM(CASE WHEN l.change_type IN ('inbound', 'check_adjust')
                                      THEN l.quantity ELSE -l.quantity END) , 0.0) > 0
             ORDER BY i.id DESC LIMIT ? OFFSET ?",
            where_clause = where_clause,
        );
        let mut list_q = sqlx::query_as::<_, StockItem>(&list_sql);
        for val in &bind_values {
            list_q = list_q.bind(val.as_str());
        }
        let items = list_q
            .bind(page_size as i64)
            .bind(offset as i64)
            .fetch_all(pool)
            .await?;

        Ok((items, total.0 as u64))
    }

    /// Paginated inventory operation logs (inbound, outbound, checks, etc.) — filter by item and time range.
    pub async fn list_inventory_logs(
        pool: &SqlitePool,
        filter: &InventoryFilter,
    ) -> Result<(Vec<InventoryLog>, u64), AppError> {
        InventoryLogRepo::list(pool, filter)
            .await
            .map_err(AppError::from)
    }

    /// Gets inventory overview stats: total stock, breakdown by category, breakdown by location.
    pub async fn inventory_statistics(pool: &SqlitePool) -> Result<InventoryStatistics, AppError> {
        let total = InventoryRepo::get_total_in_stock(pool)
            .await
            .map_err(AppError::from)?;

        let by_category = InventoryRepo::get_count_by_category(pool)
            .await
            .map_err(AppError::from)?;

        let by_location = InventoryRepo::get_count_by_location(pool)
            .await
            .map_err(AppError::from)?;

        Ok(InventoryStatistics {
            total_in_stock: total,
            by_category,
            by_location,
        })
    }

    /// ATP (Available-to-Promise) query — available quantity per item.
    pub async fn check_atp(pool: &SqlitePool, query: &AtpQuery) -> Result<Vec<AtpItem>, AppError> {
        let rows = InventoryRepo::find_atp(pool, &query.item_id, &query.location_id)
            .await
            .map_err(AppError::from)?;
        Ok(rows
            .into_iter()
            .map(|(item_id, sku, quantity, location_id)| AtpItem {
                item_id,
                sku,
                quantity,
                location_id,
            })
            .collect())
    }
}
