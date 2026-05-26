use sqlx::SqlitePool;

use crate::dto::common::PaginationParams;
use crate::dto::inventory_dto::{AtpItem, AtpQuery, InventoryFilter};
use crate::error::AppError;
use crate::models::inventory::InventoryLog;
use crate::repositories::inventory_repo::{InventoryLogRepo, InventoryRepo};

pub struct InventoryQueryService;

impl InventoryQueryService {
    pub async fn list_inventory(
        pool: &SqlitePool,
        filter: &InventoryFilter,
    ) -> Result<(Vec<serde_json::Value>, u64), AppError> {
        let pagination = PaginationParams {
            page: filter.page,
            page_size: filter.page_size,
            sort_by: None,
            sort_order: None,
        };
        let page_size = pagination.page_size();
        let offset = pagination.offset();

        let mut seamless_conditions: Vec<String> = vec!["deleted_at IS NULL".into()];
        let mut screen_conditions: Vec<String> = vec!["deleted_at IS NULL".into()];
        let mut bind_values: Vec<String> = Vec::new();

        if let Some(ref grade) = filter.grade {
            seamless_conditions.push("grade = ?".into());
            screen_conditions.push("base_grade = ?".into());
            bind_values.push(grade.clone());
        }
        if let Some(location_id) = filter.location_id {
            seamless_conditions.push("location_id = ?".into());
            screen_conditions.push("location_id = ?".into());
            bind_values.push(location_id.to_string());
        }
        if let Some(ref q) = filter.q {
            if !q.is_empty() {
                seamless_conditions.push("pipe_number LIKE ?".into());
                screen_conditions.push("pipe_number LIKE ?".into());
                bind_values.push(format!("%{}%", q));
            }
        }

        let seamless_where = seamless_conditions.join(" AND ");
        let screen_where = screen_conditions.join(" AND ");
        let pipe_type_filter = filter.pipe_type.clone();
        let is_single_table = pipe_type_filter
            .as_deref()
            .map_or(false, |pt| pt == "seamless" || pt == "casing" || pt == "tubing" || pt == "screen" || pt == "screened");

        let count_sql = match pipe_type_filter.as_deref() {
            Some("seamless" | "casing" | "tubing") => {
                format!(
                    "SELECT COUNT(*) as cnt FROM seamless_pipes WHERE {}",
                    seamless_where
                )
            }
            Some("screen" | "screened") => {
                format!(
                    "SELECT COUNT(*) as cnt FROM screen_pipes WHERE {}",
                    screen_where
                )
            }
            _ => {
                format!(
                    "SELECT (SELECT COUNT(*) FROM seamless_pipes WHERE {}) + \
                     (SELECT COUNT(*) FROM screen_pipes WHERE {}) as cnt",
                    seamless_where, screen_where
                )
            }
        };

        let mut count_q = sqlx::query_as::<_, (i64,)>(&count_sql);
        for val in &bind_values {
            count_q = count_q.bind(val.as_str());
        }
        if !is_single_table {
            // Double-bind for UNION ALL (two subqueries)
            for val in &bind_values {
                count_q = count_q.bind(val.as_str());
            }
        }
        let total: (i64,) = count_q.fetch_one(pool).await.map_err(AppError::from)?;

        let list_sql = match pipe_type_filter.as_deref() {
            Some("seamless" | "casing" | "tubing") => {
                format!(
                    "SELECT id, pipe_number, grade, od, wt, pipe_type, status, location_id, \
                     created_at, updated_at FROM seamless_pipes WHERE {} \
                     ORDER BY created_at DESC LIMIT ? OFFSET ?",
                    seamless_where
                )
            }
            Some("screen" | "screened") => {
                format!(
                    "SELECT id, pipe_number, base_grade as grade, base_od as od, base_wt as wt, \
                     screen_type as pipe_type, status, location_id, created_at, updated_at \
                     FROM screen_pipes WHERE {} \
                     ORDER BY created_at DESC LIMIT ? OFFSET ?",
                    screen_where
                )
            }
            _ => {
                format!(
                    "SELECT id, pipe_number, grade, od, wt, pipe_type, status, location_id, \
                     created_at, updated_at FROM seamless_pipes WHERE {} \
                     UNION ALL \
                     SELECT id, pipe_number, base_grade as grade, base_od as od, base_wt as wt, \
                     screen_type as pipe_type, status, location_id, created_at, updated_at \
                     FROM screen_pipes WHERE {} \
                     ORDER BY created_at DESC LIMIT ? OFFSET ?",
                    seamless_where, screen_where
                )
            }
        };

        let mut list_q = sqlx::query_as::<_, (i64, String, String, f64, f64, String, String, Option<i64>, String, String)>(
            &list_sql,
        );
        for val in &bind_values {
            list_q = list_q.bind(val.as_str());
        }
        if !is_single_table {
            for val in &bind_values {
                list_q = list_q.bind(val.as_str());
            }
        }
        let items: Vec<serde_json::Value> = list_q
            .bind(page_size as i64)
            .bind(offset as i64)
            .fetch_all(pool)
            .await
            .map_err(AppError::from)?
            .into_iter()
            .map(|(id, pipe_number, grade, od, wt, pipe_type, status, location_id, created_at, updated_at)| {
                serde_json::json!({
                    "id": id,
                    "pipe_number": pipe_number,
                    "grade": grade,
                    "od": od,
                    "wt": wt,
                    "pipe_type": pipe_type,
                    "status": status,
                    "location_id": location_id,
                    "created_at": created_at,
                    "updated_at": updated_at,
                })
            })
            .collect();

        Ok((items, total.0 as u64))
    }

    pub async fn list_inventory_logs(
        pool: &SqlitePool,
        filter: &InventoryFilter,
    ) -> Result<(Vec<InventoryLog>, u64), AppError> {
        InventoryLogRepo::list(pool, filter)
            .await
            .map_err(AppError::from)
    }

    pub async fn inventory_statistics(
        pool: &SqlitePool,
    ) -> Result<serde_json::Value, AppError> {
        let total = InventoryRepo::get_total_in_stock(pool)
            .await
            .map_err(AppError::from)?;

        let by_grade = InventoryRepo::get_count_by_grade(pool)
            .await
            .map_err(AppError::from)?;

        let by_location = InventoryRepo::get_count_by_location(pool)
            .await
            .map_err(AppError::from)?;

        Ok(serde_json::json!({
            "total_in_stock": total,
            "by_grade": by_grade,
            "by_location": by_location,
        }))
    }

    pub async fn check_atp(
        pool: &SqlitePool,
        query: &AtpQuery,
    ) -> Result<Vec<AtpItem>, AppError> {
        let rows = InventoryRepo::find_atp(pool, &query.pipe_type, &query.grade, &query.location_id)
            .await
            .map_err(AppError::from)?;
        Ok(rows
            .into_iter()
            .map(|(pipe_type, grade, quantity, location_id)| AtpItem {
                pipe_type,
                grade,
                quantity,
                location_id,
            })
            .collect())
    }
}
