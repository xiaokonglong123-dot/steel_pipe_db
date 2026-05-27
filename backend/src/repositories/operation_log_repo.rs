use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};

use crate::dto::common::PaginationParams;

/// Audit log row from `operation_logs` table.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OperationLog {
    pub id: i64,
    pub user_id: Option<i64>,
    pub username: Option<String>,
    pub action: String,
    pub entity_type: String,
    pub entity_id: Option<i64>,
    pub details: Option<String>,
    pub ip_address: Option<String>,
    pub created_at: String,
}

/// Input struct for inserting a new operation log entry.
#[derive(Debug, Clone)]
pub struct CreateOperationLog {
    pub user_id: Option<i64>,
    pub username: Option<String>,
    pub action: String,
    pub entity_type: String,
    pub entity_id: Option<i64>,
    pub details: Option<String>,
    pub ip_address: Option<String>,
}

/// Filter parameters for querying operation logs (all optional).
#[derive(Debug, Clone, Default)]
pub struct OperationLogFilter {
    pub user_id: Option<i64>,
    pub username: Option<String>,
    pub action: Option<String>,
    pub entity_type: Option<String>,
    pub entity_id: Option<i64>,
}

/// Audit log queries (no soft-delete — logs are never deleted).
pub struct OperationLogRepo;

impl OperationLogRepo {
    /// INSERT a log entry and return the created `OperationLog`.
    pub async fn create(
        pool: &SqlitePool,
        log: &CreateOperationLog,
    ) -> Result<OperationLog, sqlx::Error> {
        sqlx::query_as::<_, OperationLog>(
            "INSERT INTO operation_logs (user_id, username, action, entity_type, entity_id, details, ip_address)
             VALUES (?, ?, ?, ?, ?, ?, ?)
             RETURNING id, user_id, username, action, entity_type, entity_id, details, ip_address, created_at",
        )
        .bind(log.user_id)
        .bind(&log.username)
        .bind(&log.action)
        .bind(&log.entity_type)
        .bind(log.entity_id)
        .bind(&log.details)
        .bind(&log.ip_address)
        .fetch_one(pool)
        .await
    }

    /// Paginated log list with dynamic filters (user_id, username, action, entity_type, entity_id).
    /// Ordered by `created_at DESC`. Returns `(items, total)`.
    pub async fn list(
        pool: &SqlitePool,
        params: &PaginationParams,
        filter: &OperationLogFilter,
    ) -> Result<(Vec<OperationLog>, u64), sqlx::Error> {
        let page_size = params.page_size();
        let offset = params.offset();

        let mut where_clauses: Vec<String> = Vec::new();
        let mut bind_values: Vec<String> = Vec::new();

        if let Some(ref user_id) = filter.user_id {
            bind_values.push(user_id.to_string());
            where_clauses.push(format!("user_id = ?{}", bind_values.len()));
        }
        if let Some(ref username) = filter.username {
            bind_values.push(username.clone());
            where_clauses.push(format!("username = ?{}", bind_values.len()));
        }
        if let Some(ref action) = filter.action {
            bind_values.push(action.clone());
            where_clauses.push(format!("action = ?{}", bind_values.len()));
        }
        if let Some(ref entity_type) = filter.entity_type {
            bind_values.push(entity_type.clone());
            where_clauses.push(format!("entity_type = ?{}", bind_values.len()));
        }
        if let Some(ref entity_id) = filter.entity_id {
            bind_values.push(entity_id.to_string());
            where_clauses.push(format!("entity_id = ?{}", bind_values.len()));
        }

        let where_sql = if where_clauses.is_empty() {
            String::from("1=1")
        } else {
            where_clauses.join(" AND ")
        };

        let count_sql = format!(
            "SELECT COUNT(*) as cnt FROM operation_logs WHERE {}",
            where_sql
        );
        let list_sql = format!(
            "SELECT id, user_id, username, action, entity_type, entity_id, details, ip_address, created_at
             FROM operation_logs WHERE {}
             ORDER BY created_at DESC LIMIT ?{} OFFSET ?{}",
            where_sql,
            bind_values.len() + 1,
            bind_values.len() + 2,
        );

        let mut count_query = sqlx::query_as::<_, (i64,)>(&count_sql);
        for v in &bind_values {
            count_query = count_query.bind(v);
        }
        let total: (i64,) = count_query.fetch_one(pool).await?;

        let mut list_query = sqlx::query_as::<_, OperationLog>(&list_sql);
        for v in &bind_values {
            list_query = list_query.bind(v);
        }
        list_query = list_query
            .bind(page_size as i64)
            .bind(offset as i64);

        let items = list_query.fetch_all(pool).await?;
        Ok((items, total.0 as u64))
    }
}
