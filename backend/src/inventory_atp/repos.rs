//! Inventory ATP repositories — reservations, transfers, count templates.

use sqlx::PgPool;
use crate::models::inventory_atp::{
    AtpOverviewRow, AtpSlot, CountSession, CountTemplate, InternalTransfer,
};

pub struct AtpSlotRepo;

impl AtpSlotRepo {
    pub async fn reserve(
        pool: &PgPool,
        tenant_id: i64,
        pipe_type: &str,
        pipe_number: Option<&str>,
        quantity: rust_decimal::Decimal,
        sales_order_id: Option<i64>,
    ) -> Result<AtpSlot, sqlx::Error> {
        sqlx::query_as::<_, AtpSlot>(
            "INSERT INTO atp_slots \
             (tenant_id, pipe_type, pipe_number, quantity_reserved, sales_order_id) \
             VALUES ($1, $2, $3, $4, $5) \
             RETURNING id, tenant_id, pipe_type, pipe_number, quantity_reserved, \
                       sales_order_id, status, created_at, released_at",
        )
        .bind(tenant_id)
        .bind(pipe_type)
        .bind(pipe_number)
        .bind(quantity)
        .bind(sales_order_id)
        .fetch_one(pool)
        .await
    }

    pub async fn release(
        pool: &PgPool,
        tenant_id: i64,
        id: i64,
    ) -> Result<Option<AtpSlot>, sqlx::Error> {
        sqlx::query_as::<_, AtpSlot>(
            "UPDATE atp_slots SET status = 'released', released_at = NOW() \
             WHERE tenant_id = $1 AND id = $2 AND status = 'reserved' \
             RETURNING id, tenant_id, pipe_type, pipe_number, quantity_reserved, \
                       sales_order_id, status, created_at, released_at",
        )
        .bind(tenant_id)
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    pub async fn reserved_total(
        pool: &PgPool,
        tenant_id: i64,
        pipe_type: &str,
        pipe_number: Option<&str>,
    ) -> Result<rust_decimal::Decimal, sqlx::Error> {
        let v: rust_decimal::Decimal = sqlx::query_scalar(
            "SELECT COALESCE(SUM(quantity_reserved), 0) FROM atp_slots \
             WHERE tenant_id = $1 AND pipe_type = $2 AND status = 'reserved' \
             AND ($3::text IS NULL OR pipe_number = $3)",
        )
        .bind(tenant_id)
        .bind(pipe_type)
        .bind(pipe_number)
        .fetch_one(pool)
        .await?;
        Ok(v)
    }

    /// On-hand per pipe type from the pipes tables (one row per pipe,
    /// status='in_stock'), minus active reservations. Inventory is not a
    /// materialized table in this codebase — pipes ARE the stock.
    pub async fn overview(pool: &PgPool, tenant_id: i64) -> Result<Vec<AtpOverviewRow>, sqlx::Error> {
        sqlx::query_as::<_, AtpOverviewRow>(
            "SELECT pipe_type, \
                    COALESCE(SUM(cnt), 0)::NUMERIC AS on_hand, \
                    COALESCE((SELECT SUM(a.quantity_reserved) FROM atp_slots a \
                              WHERE a.pipe_type = t.pipe_type AND a.status = 'reserved'), 0) AS reserved, \
                    (COALESCE(SUM(cnt), 0) - COALESCE((SELECT SUM(a.quantity_reserved) FROM atp_slots a \
                              WHERE a.pipe_type = t.pipe_type AND a.status = 'reserved'), 0))::NUMERIC AS available \
             FROM ( \
                SELECT 'seamless' AS pipe_type, COUNT(*)::BIGINT AS cnt FROM seamless_pipes WHERE status = 'in_stock' \
                UNION ALL \
                SELECT 'screen' AS pipe_type, COUNT(*)::BIGINT AS cnt FROM screen_pipes WHERE status = 'in_stock' \
                UNION ALL \
                SELECT 'welded' AS pipe_type, COUNT(*)::BIGINT AS cnt FROM welded_pipes WHERE status = 'in_stock' \
             ) t \
             WHERE cnt > 0 GROUP BY t.pipe_type ORDER BY t.pipe_type",
        )
        .bind(tenant_id)
        .fetch_all(pool)
        .await
    }

    /// Per-pipe-number ATP: on-hand minus reserved for a specific pipe.
    pub async fn pipe_atp(
        pool: &PgPool,
        tenant_id: i64,
        pipe_type: &str,
        pipe_number: &str,
    ) -> Result<AtpOverviewRow, sqlx::Error> {
        let table = pipe_table(pipe_type);
        let sql = format!(
            "SELECT $2::text AS pipe_type, \
                    (SELECT COUNT(*) FROM {table} WHERE pipe_number = $3 AND status = 'in_stock')::NUMERIC AS on_hand, \
                    COALESCE((SELECT SUM(a.quantity_reserved) FROM atp_slots a \
                              WHERE a.pipe_type = $2 AND a.pipe_number = $3 AND a.status = 'reserved'), 0) AS reserved, \
                    ((SELECT COUNT(*) FROM {table} WHERE pipe_number = $3 AND status = 'in_stock') \
                     - COALESCE((SELECT SUM(a.quantity_reserved) FROM atp_slots a \
                              WHERE a.pipe_type = $2 AND a.pipe_number = $3 AND a.status = 'reserved'), 0))::NUMERIC AS available"
        );
        sqlx::query_as::<_, AtpOverviewRow>(&sql)
            .bind(tenant_id)
            .bind(pipe_type)
            .bind(pipe_number)
            .fetch_one(pool)
            .await
    }
}

/// Map pipe_type string to its pipes table name.
fn pipe_table(pipe_type: &str) -> &'static str {
    match pipe_type {
        "screen" => "screen_pipes",
        "welded" => "welded_pipes",
        _ => "seamless_pipes",
    }
}

pub struct TransferRepo;

impl TransferRepo {
    pub async fn create(
        pool: &PgPool,
        tenant_id: i64,
        transfer_no: &str,
        from_location_id: i64,
        to_location_id: i64,
        pipe_id: Option<i64>,
        pipe_number: Option<&str>,
        quantity: rust_decimal::Decimal,
        created_by: Option<i64>,
        notes: Option<&str>,
    ) -> Result<InternalTransfer, sqlx::Error> {
        sqlx::query_as::<_, InternalTransfer>(
            "INSERT INTO internal_transfers \
             (tenant_id, transfer_no, from_location_id, to_location_id, pipe_id, pipe_number, \
              quantity, created_by, notes) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) \
             RETURNING id, tenant_id, transfer_no, from_location_id, to_location_id, pipe_id, \
                       pipe_number, quantity, transferred_at, status, created_by, notes, created_at",
        )
        .bind(tenant_id)
        .bind(transfer_no)
        .bind(from_location_id)
        .bind(to_location_id)
        .bind(pipe_id)
        .bind(pipe_number)
        .bind(quantity)
        .bind(created_by)
        .bind(notes)
        .fetch_one(pool)
        .await
    }

    pub async fn list(pool: &PgPool, tenant_id: i64) -> Result<Vec<InternalTransfer>, sqlx::Error> {
        sqlx::query_as::<_, InternalTransfer>(
            "SELECT id, tenant_id, transfer_no, from_location_id, to_location_id, pipe_id, \
                    pipe_number, quantity, transferred_at, status, created_by, notes, created_at \
             FROM internal_transfers WHERE tenant_id = $1 ORDER BY id DESC LIMIT 500",
        )
        .bind(tenant_id)
        .fetch_all(pool)
        .await
    }
}

pub struct CountRepo;

impl CountRepo {
    pub async fn create_template(
        pool: &PgPool,
        tenant_id: i64,
        name: &str,
        description: Option<&str>,
        location_ids: &serde_json::Value,
    ) -> Result<CountTemplate, sqlx::Error> {
        sqlx::query_as::<_, CountTemplate>(
            "INSERT INTO count_templates (tenant_id, name, description, location_ids) \
             VALUES ($1, $2, $3, $4) \
             RETURNING id, tenant_id, name, description, location_ids, is_active, created_at, updated_at",
        )
        .bind(tenant_id)
        .bind(name)
        .bind(description)
        .bind(location_ids)
        .fetch_one(pool)
        .await
    }

    pub async fn list_templates(pool: &PgPool, tenant_id: i64) -> Result<Vec<CountTemplate>, sqlx::Error> {
        sqlx::query_as::<_, CountTemplate>(
            "SELECT id, tenant_id, name, description, location_ids, is_active, created_at, updated_at \
             FROM count_templates WHERE tenant_id = $1 ORDER BY id DESC",
        )
        .bind(tenant_id)
        .fetch_all(pool)
        .await
    }

    pub async fn create_session(
        pool: &PgPool,
        tenant_id: i64,
        template_id: i64,
        session_no: &str,
    ) -> Result<CountSession, sqlx::Error> {
        sqlx::query_as::<_, CountSession>(
            "INSERT INTO count_sessions (tenant_id, template_id, session_no) \
             VALUES ($1, $2, $3) \
             RETURNING id, tenant_id, template_id, session_no, status, started_at, completed_at, result_json",
        )
        .bind(tenant_id)
        .bind(template_id)
        .bind(session_no)
        .fetch_one(pool)
        .await
    }

    pub async fn complete_session(
        pool: &PgPool,
        tenant_id: i64,
        session_id: i64,
        result: &serde_json::Value,
    ) -> Result<Option<CountSession>, sqlx::Error> {
        sqlx::query_as::<_, CountSession>(
            "UPDATE count_sessions SET status = 'completed', completed_at = NOW(), result_json = $3 \
             WHERE tenant_id = $1 AND id = $2 AND status = 'inprogress' \
             RETURNING id, tenant_id, template_id, session_no, status, started_at, completed_at, result_json",
        )
        .bind(tenant_id)
        .bind(session_id)
        .bind(result)
        .fetch_optional(pool)
        .await
    }

    pub async fn list_sessions(pool: &PgPool, tenant_id: i64) -> Result<Vec<CountSession>, sqlx::Error> {
        sqlx::query_as::<_, CountSession>(
            "SELECT id, tenant_id, template_id, session_no, status, started_at, completed_at, result_json \
             FROM count_sessions WHERE tenant_id = $1 ORDER BY id DESC LIMIT 500",
        )
        .bind(tenant_id)
        .fetch_all(pool)
        .await
    }
}
