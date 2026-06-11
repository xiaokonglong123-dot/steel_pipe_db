use sqlx::{QueryBuilder, Sqlite, SqlitePool};

use crate::dto::common::PaginationParams;
use crate::dto::inventory_dto::{CreateInboundRecordRequest, InboundFilter, UpdateInboundRecordRequest};
use crate::models::inventory::{InboundItem, InboundRecord};

/// CRUD for `inbound_records` and `inbound_items`. All queries filter `deleted_at IS NULL`.
pub struct InboundRepo;

impl InboundRepo {
    /// INSERT into `inbound_records` + `inbound_items` in a single transaction.
    /// Purchase-type records start as `auto_approved`; others as `pending`.
    /// Returns the created `InboundRecord`.
    pub async fn create_with_items(
        pool: &SqlitePool,
        dto: &CreateInboundRecordRequest,
        inbound_no: &str,
    ) -> Result<InboundRecord, sqlx::Error> {
        let mut tx = pool.begin().await?;

        let record = sqlx::query_as::<_, InboundRecord>(
            "INSERT INTO inbound_records (inbound_no, inbound_type, order_id, supplier_id, notes, approval_status) \
             VALUES (?, ?, ?, ?, ?, ?) \
             RETURNING id, inbound_no, inbound_type, order_id, supplier_id, notes, approval_status, \
               rejection_reason, approval_reason, handled_by, handled_at, created_at, updated_at, deleted_at",
        )
        .bind(inbound_no)
        .bind(&dto.inbound_type)
        .bind(dto.order_id)
        .bind(dto.supplier_id)
        .bind(&dto.notes)
        .bind(if dto.inbound_type == "purchase" {
            "auto_approved"
        } else {
            "pending"
        })
        .fetch_one(&mut *tx)
        .await?;

        for item in &dto.pipes {
            sqlx::query(
                "INSERT INTO inbound_items (inbound_id, pipe_type, pipe_id) VALUES (?, ?, ?)",
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

    /// SELECT by primary key from `inbound_records`. Returns `None` if not found or soft-deleted.
    pub async fn find_by_id(
        pool: &SqlitePool,
        id: i64,
    ) -> Result<Option<InboundRecord>, sqlx::Error> {
        sqlx::query_as::<_, InboundRecord>(
            "SELECT id, inbound_no, inbound_type, order_id, supplier_id, notes, approval_status, \
             rejection_reason, approval_reason, handled_by, handled_at, created_at, updated_at, deleted_at \
             FROM inbound_records WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    /// SELECT all `InboundItem` rows for a given inbound record.
    pub async fn find_items(
        pool: &SqlitePool,
        inbound_id: i64,
    ) -> Result<Vec<InboundItem>, sqlx::Error> {
        sqlx::query_as::<_, InboundItem>(
            "SELECT id, inbound_id, pipe_type, pipe_id, created_at \
             FROM inbound_items WHERE inbound_id = ? ORDER BY id",
        )
        .bind(inbound_id)
        .fetch_all(pool)
        .await
    }

    /// Paginated SELECT with optional filters (`q`, `inbound_type`, `approval_status`, `order_id`).
    /// Returns `(items, total)`.
    pub async fn list(
        pool: &SqlitePool,
        filter: &InboundFilter,
    ) -> Result<(Vec<InboundRecord>, u64), sqlx::Error> {
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
                conditions.push("inbound_no LIKE ?".into());
                bind_values.push(format!("%{}%", q));
            }
        }
        if let Some(ref inbound_type) = filter.inbound_type {
            conditions.push("inbound_type = ?".into());
            bind_values.push(inbound_type.clone());
        }
        if let Some(ref approval_status) = filter.approval_status {
            conditions.push("approval_status = ?".into());
            bind_values.push(approval_status.clone());
        }
        if let Some(order_id) = filter.order_id {
            conditions.push("order_id = ?".into());
            bind_values.push(order_id.to_string());
        }

        let where_clause = conditions.join(" AND ");

        let sort_by = match pagination.sort_by.as_deref() {
            Some("inbound_no") => "inbound_no",
            Some("inbound_type") => "inbound_type",
            Some("approval_status") => "approval_status",
            _ => "created_at",
        };
        let sort_order = pagination.sort_order_sql();

        let count_sql = format!(
            "SELECT COUNT(*) as cnt FROM inbound_records WHERE {}",
            where_clause
        );
        let mut count_q = sqlx::query_as::<_, (i64,)>(&count_sql);
        for val in &bind_values {
            count_q = count_q.bind(val.as_str());
        }
        let total: (i64,) = count_q.fetch_one(pool).await?;

        let list_sql = format!(
            "SELECT id, inbound_no, inbound_type, order_id, supplier_id, notes, approval_status, \
             rejection_reason, approval_reason, handled_by, handled_at, created_at, updated_at, deleted_at \
             FROM inbound_records WHERE {} \
             ORDER BY {} {} LIMIT ? OFFSET ?",
            where_clause, sort_by, sort_order
        );
        let mut list_q = sqlx::query_as::<_, InboundRecord>(&list_sql);
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

    /// UPDATE `approval_status` on an inbound record. Optionally sets `rejection_reason`.
    pub async fn update_status(
        pool: &SqlitePool,
        id: i64,
        status: &str,
        rejection_reason: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        if let Some(reason) = rejection_reason {
            sqlx::query(
                "UPDATE inbound_records SET approval_status = ?, rejection_reason = ?, \
                 updated_at = datetime('now') WHERE id = ? AND deleted_at IS NULL",
            )
            .bind(status)
            .bind(reason)
            .bind(id)
            .execute(pool)
            .await?;
        } else {
            sqlx::query(
                "UPDATE inbound_records SET approval_status = ?, \
                 rejection_reason = NULL, updated_at = datetime('now') \
                 WHERE id = ? AND deleted_at IS NULL",
            )
            .bind(status)
            .bind(id)
            .execute(pool)
            .await?;
        }
        Ok(())
    }

    /// Sets `order_id` on an inbound record to link it to a purchase order.
    pub async fn link_to_order(
        pool: &SqlitePool,
        inbound_id: i64,
        order_id: i64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE inbound_records SET order_id = ?, updated_at = datetime('now') \
             WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(order_id)
        .bind(inbound_id)
        .execute(pool)
        .await?;
        Ok(())
    }

    /// Soft-delete by setting `deleted_at` timestamp. No-op if already deleted.
    pub async fn delete(pool: &SqlitePool, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE inbound_records SET deleted_at = datetime('now'), \
             updated_at = datetime('now') WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(id)
        .execute(pool)
        .await?;
        Ok(())
    }

    /// UPDATE editable fields (notes, order_id, supplier_id) on an inbound record.
    /// Only records with `auto_approved` or `rejected` status can be updated.
    /// Returns the updated record.
    pub async fn update(
        pool: &SqlitePool,
        id: i64,
        dto: &UpdateInboundRecordRequest,
    ) -> Result<InboundRecord, sqlx::Error> {
        let mut builder: QueryBuilder<Sqlite> =
            QueryBuilder::new("UPDATE inbound_records SET updated_at = datetime('now')");

        if let Some(ref val) = dto.notes {
            builder.push(", notes = ");
            builder.push_bind(val);
        }
        if let Some(val) = dto.order_id {
            builder.push(", order_id = ");
            builder.push_bind(val);
        }
        if let Some(val) = dto.supplier_id {
            builder.push(", supplier_id = ");
            builder.push_bind(val);
        }

        builder.push(" WHERE id = ");
        builder.push_bind(id);
        builder.push(" AND deleted_at IS NULL");
        builder.push(" AND (approval_status = 'auto_approved' OR approval_status = 'rejected')");
        builder.push(
            " RETURNING id, inbound_no, inbound_type, order_id, supplier_id, notes, approval_status, \
             rejection_reason, approval_reason, handled_by, handled_at, created_at, updated_at, deleted_at",
        );

        builder
            .build_query_as::<InboundRecord>()
            .fetch_one(pool)
            .await
    }
}
