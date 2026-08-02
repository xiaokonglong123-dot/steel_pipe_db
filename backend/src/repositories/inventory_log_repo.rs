use sqlx::postgres::Sqlite;
use sqlx::{PgPool, Transaction};

use crate::dto::common::PaginationParams;
use crate::dto::inventory_dto::InventoryFilter;
use crate::models::inventory::InventoryLog;

use super::inventory_repo::CreateInventoryLog;

/// INSERT + paginated SELECT + by-pipe query for `inventory_logs` (pipe movement audit trail).
pub struct InventoryLogRepo;

impl InventoryLogRepo {
    /// INSERT a row into `inventory_logs` using a transaction. Returns the newly created log entry with generated `id`.
    pub async fn create_in_transaction(
        tx: &mut Transaction<'_, Sqlite>,
        log: &CreateInventoryLog,
    ) -> Result<InventoryLog, sqlx::Error> {
        sqlx::query_as::<_, InventoryLog>(
            "INSERT INTO inventory_logs (pipe_type, pipe_id, change_type, ref_type, ref_id, \
             from_location_id, to_location_id, notes, created_by) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) \
             RETURNING id, pipe_type, pipe_id, change_type, ref_type, ref_id, \
               from_location_id, to_location_id, notes, created_by, created_at",
        )
        .bind(&log.pipe_type)
        .bind(log.pipe_id)
        .bind(&log.change_type)
        .bind(&log.ref_type)
        .bind(log.ref_id)
        .bind(log.from_location_id)
        .bind(log.to_location_id)
        .bind(&log.notes)
        .bind(log.created_by)
        .fetch_one(&mut **tx)
        .await
    }

    /// INSERT a row into `inventory_logs`. Returns the newly created log entry with generated `id`.
    pub async fn create(
        pool: &PgPool,
        log: &CreateInventoryLog,
    ) -> Result<InventoryLog, sqlx::Error> {
        sqlx::query_as::<_, InventoryLog>(
            "INSERT INTO inventory_logs (pipe_type, pipe_id, change_type, ref_type, ref_id, \
             from_location_id, to_location_id, notes, created_by) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) \
             RETURNING id, pipe_type, pipe_id, change_type, ref_type, ref_id, \
               from_location_id, to_location_id, notes, created_by, created_at",
        )
        .bind(&log.pipe_type)
        .bind(log.pipe_id)
        .bind(&log.change_type)
        .bind(&log.ref_type)
        .bind(log.ref_id)
        .bind(log.from_location_id)
        .bind(log.to_location_id)
        .bind(&log.notes)
        .bind(log.created_by)
        .fetch_one(pool)
        .await
    }

    /// SELECT inventory logs for a specific pipe, ordered by time ascending.
    /// Used by trace service for full lifecycle audit trail.
    pub async fn find_by_pipe(
        pool: &PgPool,
        pipe_type: &str,
        pipe_id: i64,
    ) -> Result<Vec<InventoryLog>, sqlx::Error> {
        sqlx::query_as::<_, InventoryLog>(
            "SELECT id, pipe_type, pipe_id, change_type, ref_type, ref_id, \
             from_location_id, to_location_id, notes, created_by, created_at \
             FROM inventory_logs WHERE pipe_type = $1 AND pipe_id = $2 \
             ORDER BY created_at ASC",
        )
        .bind(pipe_type)
        .bind(pipe_id)
        .fetch_all(pool)
        .await
    }

    /// Paginated SELECT from `inventory_logs` with optional filters (`pipe_type`, `location_id`).
    /// Returns `(items, total)`.
    pub async fn list(
        pool: &PgPool,
        filter: &InventoryFilter,
    ) -> Result<(Vec<InventoryLog>, u64), sqlx::Error> {
        let pagination = PaginationParams {
            page: filter.page,
            page_size: filter.page_size,
            sort_by: None,
            sort_order: None,
        };
        let page_size = pagination.page_size();
        let offset = pagination.offset();

        let mut conditions: Vec<String> = Vec::new();
        let mut bind_values: Vec<String> = Vec::new();

        if let Some(ref pipe_type) = filter.pipe_type {
            conditions.push(format!("pipe_type = ${}", bind_values.len() + 1));
            bind_values.push(pipe_type.clone());
        }
        if let Some(location_id) = filter.location_id {
            let base = bind_values.len() + 1;
            conditions.push(format!("(from_location_id = ${} OR to_location_id = ${})", base, base + 1));
            bind_values.push(location_id.to_string());
            bind_values.push(location_id.to_string());
        }

        let where_clause = if conditions.is_empty() {
            "1=1".to_string()
        } else {
            conditions.join(" AND ")
        };

        let count_sql = format!(
            "SELECT COUNT(*) as cnt FROM inventory_logs WHERE {}",
            where_clause
        );
        let mut count_q = sqlx::query_as::<_, (i64,)>(&count_sql);
        for val in &bind_values {
            count_q = count_q.bind(val.as_str());
        }
        let total: (i64,) = count_q.fetch_one(pool).await?;

        let list_sql = format!(
            "SELECT id, pipe_type, pipe_id, change_type, ref_type, ref_id, \
             from_location_id, to_location_id, notes, created_by, created_at \
             FROM inventory_logs WHERE {} \
             ORDER BY created_at DESC LIMIT $1 OFFSET $2",
            where_clause
        );
        let mut list_q = sqlx::query_as::<_, InventoryLog>(&list_sql);
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
}
