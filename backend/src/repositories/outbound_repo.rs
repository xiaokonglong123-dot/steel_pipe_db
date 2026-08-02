use sqlx::{QueryBuilder, Sqlite, PgPool, Transaction};

use crate::dto::common::PaginationParams;
use crate::dto::inventory_dto::{CreateOutboundRecordRequest, OutboundFilter, UpdateOutboundRecordRequest};
use crate::models::inventory::{OutboundItem, OutboundRecord};

/// CRUD for `outbound_records` and `outbound_items`. All queries filter `deleted_at IS NULL`.
pub struct OutboundRepo;

impl OutboundRepo {
    /// INSERT into `outbound_records` + `outbound_items` in a single transaction.
    /// Sales-type records start as `auto_approved`; others as `pending`.
    /// Returns the created `OutboundRecord`.
    pub async fn create_with_items(
        pool: &PgPool,
        dto: &CreateOutboundRecordRequest,
        outbound_no: &str,
    ) -> Result<OutboundRecord, sqlx::Error> {
        let mut tx = pool.begin().await?;

        let record = sqlx::query_as::<_, OutboundRecord>(
            "INSERT INTO outbound_records (outbound_no, outbound_type, order_id, customer_id, notes, approval_status) \
             VALUES ($1, $2, $3, $4, $5, $6) \
             RETURNING id, outbound_no, outbound_type, order_id, customer_id, notes, approval_status, \
               rejection_reason, approval_reason, handled_by, handled_at, created_at, updated_at, deleted_at",
        )
        .bind(outbound_no)
        .bind(&dto.outbound_type)
        .bind(dto.order_id)
        .bind(dto.customer_id)
        .bind(&dto.notes)
        .bind(if dto.outbound_type == "sales" {
            "auto_approved"
        } else {
            "pending"
        })
        .fetch_one(&mut *tx)
        .await?;

        for item in &dto.pipes {
            sqlx::query(
                "INSERT INTO outbound_items (outbound_id, pipe_type, pipe_id) VALUES ($1, $2, $3)",
            )
            .bind(record.id)
            .bind(&item.pipe_type)
            .bind(item.pipe_id)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(record)
    }

    /// SELECT by primary key from `outbound_records`. Returns `None` if not found or soft-deleted.
    pub async fn find_by_id(
        pool: &PgPool,
        id: i64,
    ) -> Result<Option<OutboundRecord>, sqlx::Error> {
        sqlx::query_as::<_, OutboundRecord>(
            "SELECT id, outbound_no, outbound_type, order_id, customer_id, notes, approval_status, \
             rejection_reason, approval_reason, handled_by, handled_at, created_at, updated_at, deleted_at \
             FROM outbound_records WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    /// SELECT all `OutboundItem` rows for a given outbound record.
    pub async fn find_items(
        pool: &PgPool,
        outbound_id: i64,
    ) -> Result<Vec<OutboundItem>, sqlx::Error> {
        sqlx::query_as::<_, OutboundItem>(
            "SELECT id, outbound_id, pipe_type, pipe_id, created_at \
             FROM outbound_items WHERE outbound_id = $1 ORDER BY id",
        )
        .bind(outbound_id)
        .fetch_all(pool)
        .await
    }

    /// Paginated SELECT with optional filters (`q`, `outbound_type`, `approval_status`, `order_id`).
    /// Returns `(items, total)`.
    pub async fn list(
        pool: &PgPool,
        filter: &OutboundFilter,
    ) -> Result<(Vec<OutboundRecord>, u64), sqlx::Error> {
        let pagination = PaginationParams {
            page: filter.page,
            page_size: filter.page_size,
            sort_by: filter.sort_by.clone(),
            sort_order: filter.sort_order.clone(),
        };
        let page_size = pagination.page_size();
        let offset = pagination.offset();

        let mut conditions: Vec<String> = vec!["deleted_at IS NULL".into()];
        let mut bind_values: Vec<String> = Vec::new();

        if let Some(ref q) = filter.q {
            if !q.is_empty() {
                conditions.push(format!("outbound_no LIKE ${}", bind_values.len() + 1));
                bind_values.push(format!("%{}%", q));
            }
        }
        if let Some(ref outbound_type) = filter.outbound_type {
            conditions.push(format!("outbound_type = ${}", bind_values.len() + 1));
            bind_values.push(outbound_type.clone());
        }
        if let Some(ref approval_status) = filter.approval_status {
            conditions.push(format!("approval_status = ${}", bind_values.len() + 1));
            bind_values.push(approval_status.clone());
        }
        if let Some(order_id) = filter.order_id {
            conditions.push(format!("order_id = ${}", bind_values.len() + 1));
            bind_values.push(order_id.to_string());
        }

        let where_clause = conditions.join(" AND ");

        let sort_by = match pagination.sort_by.as_deref() {
            Some("outbound_no") => "outbound_no",
            Some("outbound_type") => "outbound_type",
            Some("approval_status") => "approval_status",
            _ => "created_at",
        };
        let sort_order = pagination.sort_order_sql();

        let count_sql = format!(
            "SELECT COUNT(*) as cnt FROM outbound_records WHERE {}",
            where_clause
        );
        let mut count_q = sqlx::query_as::<_, (i64,)>(&count_sql);
        for val in &bind_values {
            count_q = count_q.bind(val.as_str());
        }
        let total: (i64,) = count_q.fetch_one(pool).await?;

        let list_sql = format!(
            "SELECT id, outbound_no, outbound_type, order_id, customer_id, notes, approval_status, \
             rejection_reason, approval_reason, handled_by, handled_at, created_at, updated_at, deleted_at \
             FROM outbound_records WHERE {} \
             ORDER BY {} {} LIMIT $1 OFFSET $2",
            where_clause, sort_by, sort_order
        );
        let mut list_q = sqlx::query_as::<_, OutboundRecord>(&list_sql);
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

    /// UPDATE `approval_status` on an outbound record. Optionally sets `rejection_reason`.
    pub async fn update_status(
        pool: &PgPool,
        id: i64,
        status: &str,
        rejection_reason: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        if let Some(reason) = rejection_reason {
            sqlx::query(
                "UPDATE outbound_records SET approval_status = $1, rejection_reason = $2, \
                 updated_at = NOW() WHERE id = $3 AND deleted_at IS NULL",
            )
            .bind(status)
            .bind(reason)
            .bind(id)
            .execute(pool)
            .await?;
        } else {
            sqlx::query(
                "UPDATE outbound_records SET approval_status = $1, \
                 rejection_reason = NULL, updated_at = NOW() \
                 WHERE id = $2 AND deleted_at IS NULL",
            )
            .bind(status)
            .bind(id)
            .execute(pool)
            .await?;
        }
        Ok(())
    }

    /// UPDATE outbound_records status to 'approved' inside an existing transaction.
    /// Returns the number of rows affected (0 if record was already processed or deleted).
    pub async fn approve(
        tx: &mut Transaction<'_, Sqlite>,
        id: i64,
        approval_reason: Option<&str>,
        handled_by: Option<i64>,
    ) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            "UPDATE outbound_records SET approval_status = 'approved', \
             rejection_reason = NULL, approval_reason = $1, handled_by = $2, handled_at = NOW(), updated_at = NOW() \
             WHERE id = $3 AND deleted_at IS NULL AND approval_status = 'pending'",
        )
        .bind(approval_reason)
        .bind(handled_by)
        .bind(id)
        .execute(&mut **tx)
        .await?;
        Ok(result.rows_affected())
    }

    /// Sets `order_id` on an outbound record to link it to a sales order.
    pub async fn link_to_order(
        pool: &PgPool,
        outbound_id: i64,
        order_id: i64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE outbound_records SET order_id = $1, updated_at = NOW() \
             WHERE id = $2 AND deleted_at IS NULL",
        )
        .bind(order_id)
        .bind(outbound_id)
        .execute(pool)
        .await?;
        Ok(())
    }

    /// SELECT outbound records by order_id. Returns only non-deleted records.
    pub async fn find_by_order_id(
        pool: &PgPool,
        order_id: i64,
    ) -> Result<Vec<OutboundRecord>, sqlx::Error> {
        sqlx::query_as::<_, OutboundRecord>(
            "SELECT id, outbound_no, outbound_type, order_id, customer_id, notes, approval_status, \
             rejection_reason, approval_reason, handled_by, handled_at, created_at, updated_at, deleted_at \
             FROM outbound_records WHERE order_id = $1 AND deleted_at IS NULL",
        )
        .bind(order_id)
        .fetch_all(pool)
        .await
    }

    /// Soft-delete by setting `deleted_at` timestamp. No-op if already deleted.
    pub async fn delete(pool: &PgPool, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE outbound_records SET deleted_at = NOW(), \
             updated_at = NOW() WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(id)
        .execute(pool)
        .await?;
        Ok(())
    }

    /// UPDATE editable fields (notes, order_id, customer_id) on an outbound record.
    /// Only records with `auto_approved` or `rejected` status can be updated.
    /// Returns the updated record.
    pub async fn update(
        pool: &PgPool,
        id: i64,
        dto: &UpdateOutboundRecordRequest,
    ) -> Result<OutboundRecord, sqlx::Error> {
        let mut builder: QueryBuilder<Sqlite> =
            QueryBuilder::new("UPDATE outbound_records SET updated_at = NOW()");

        if let Some(ref val) = dto.notes {
            builder.push(", notes = ");
            builder.push_bind(val);
        }
        if let Some(val) = dto.order_id {
            builder.push(", order_id = ");
            builder.push_bind(val);
        }
        if let Some(val) = dto.customer_id {
            builder.push(", customer_id = ");
            builder.push_bind(val);
        }

        builder.push(" WHERE id = ");
        builder.push_bind(id);
        builder.push(" AND deleted_at IS NULL");
        builder.push(" AND (approval_status = 'auto_approved' OR approval_status = 'rejected')");
        builder.push(
            " RETURNING id, outbound_no, outbound_type, order_id, customer_id, notes, approval_status, \
             rejection_reason, approval_reason, handled_by, handled_at, created_at, updated_at, deleted_at",
        );

        builder
            .build_query_as::<OutboundRecord>()
            .fetch_one(pool)
            .await
    }
}
