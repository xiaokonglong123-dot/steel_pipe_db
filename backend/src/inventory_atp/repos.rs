//! Inventory ATP repositories — reservations, transfers, count templates.

use sqlx::SqlitePool;
use crate::models::inventory_atp::{
    AtpOverviewRow, AtpSlot, CountSession, CountTemplate, InternalTransfer,
};

pub struct AtpSlotRepo;

impl AtpSlotRepo {
    pub async fn reserve(
        pool: &SqlitePool,
        tenant_id: i64,
        item_id: i64,
        sku: Option<&str>,
        quantity: f64,
        sales_order_id: Option<i64>,
    ) -> Result<AtpSlot, sqlx::Error> {
        sqlx::query_as::<_, AtpSlot>(
            "INSERT INTO atp_slots \
             (tenant_id, item_id, sku, quantity_reserved, sales_order_id) \
             VALUES (?, ?, ?, ?, ?) \
             RETURNING id, tenant_id, item_id, sku, quantity_reserved, \
                       sales_order_id, status, created_at, released_at",
        )
        .bind(tenant_id)
        .bind(item_id)
        .bind(sku)
        .bind(quantity)
        .bind(sales_order_id)
        .fetch_one(pool)
        .await
    }

    pub async fn release(
        pool: &SqlitePool,
        tenant_id: i64,
        id: i64,
    ) -> Result<Option<AtpSlot>, sqlx::Error> {
        sqlx::query_as::<_, AtpSlot>(
            "UPDATE atp_slots SET status = 'released', released_at = datetime('now') \
             WHERE tenant_id = ? AND id = ? AND status = 'reserved' \
             RETURNING id, tenant_id, item_id, sku, quantity_reserved, \
                       sales_order_id, status, created_at, released_at",
        )
        .bind(tenant_id)
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    pub async fn reserved_total(
        pool: &SqlitePool,
        tenant_id: i64,
        item_id: i64,
    ) -> Result<f64, sqlx::Error> {
        let v: f64 = sqlx::query_scalar(
            "SELECT CAST(COALESCE(SUM(quantity_reserved), 0.0) AS REAL) FROM atp_slots \
             WHERE tenant_id = ? AND item_id = ? AND status = 'reserved'",
        )
        .bind(tenant_id)
        .bind(item_id)
        .fetch_one(pool)
        .await?;
        Ok(v)
    }

    /// On-hand per item from inventory_logs minus active reservations.
    pub async fn overview(pool: &SqlitePool, tenant_id: i64) -> Result<Vec<AtpOverviewRow>, sqlx::Error> {
        sqlx::query_as::<_, AtpOverviewRow>(
            "SELECT i.id AS item_id, i.sku,
                    CAST(COALESCE((SELECT SUM(
                        CASE WHEN l.change_type IN ('inbound', 'check_adjust')
                             THEN l.quantity ELSE -l.quantity END)
                     FROM inventory_logs l WHERE l.item_id = i.id) , 0.0) AS REAL) AS on_hand,
                    CAST(COALESCE((SELECT SUM(a.quantity_reserved) FROM atp_slots a
                              WHERE a.item_id = i.id AND a.status = 'reserved'
                                AND a.tenant_id = ?) , 0.0) AS REAL) AS reserved,
                    CAST(COALESCE((SELECT SUM(
                        CASE WHEN l.change_type IN ('inbound', 'check_adjust')
                             THEN l.quantity ELSE -l.quantity END)
                     FROM inventory_logs l WHERE l.item_id = i.id) , 0.0) AS REAL)
                    - CAST(COALESCE((SELECT SUM(a.quantity_reserved) FROM atp_slots a
                              WHERE a.item_id = i.id AND a.status = 'reserved'
                                AND a.tenant_id = ?) , 0.0) AS REAL) AS available
             FROM items i
             WHERE i.deleted_at IS NULL
             ORDER BY i.sku",
        )
        .bind(tenant_id)
        .bind(tenant_id)
        .fetch_all(pool)
        .await
    }

    /// Per-item ATP: on-hand minus reserved for a specific item.
    pub async fn item_atp(
        pool: &SqlitePool,
        tenant_id: i64,
        item_id: i64,
    ) -> Result<AtpOverviewRow, sqlx::Error> {
        sqlx::query_as::<_, AtpOverviewRow>(
            "SELECT i.id AS item_id, i.sku,
                    CAST(COALESCE((SELECT SUM(
                        CASE WHEN l.change_type IN ('inbound', 'check_adjust')
                             THEN l.quantity ELSE -l.quantity END)
                     FROM inventory_logs l WHERE l.item_id = i.id) , 0.0) AS REAL) AS on_hand,
                    CAST(COALESCE((SELECT SUM(a.quantity_reserved) FROM atp_slots a
                              WHERE a.item_id = i.id AND a.status = 'reserved'
                                AND a.tenant_id = ?) , 0.0) AS REAL) AS reserved,
                    CAST(COALESCE((SELECT SUM(
                        CASE WHEN l.change_type IN ('inbound', 'check_adjust')
                             THEN l.quantity ELSE -l.quantity END)
                     FROM inventory_logs l WHERE l.item_id = i.id) , 0.0) AS REAL)
                    - CAST(COALESCE((SELECT SUM(a.quantity_reserved) FROM atp_slots a
                              WHERE a.item_id = i.id AND a.status = 'reserved'
                                AND a.tenant_id = ?) , 0.0) AS REAL) AS available
             FROM items i
             WHERE i.id = ? AND i.deleted_at IS NULL",
        )
        .bind(tenant_id)
        .bind(tenant_id)
        .bind(item_id)
        .fetch_one(pool)
        .await
    }
}

pub struct TransferRepo;

impl TransferRepo {
    pub async fn create(
        pool: &SqlitePool,
        tenant_id: i64,
        transfer_no: &str,
        from_location_id: i64,
        to_location_id: i64,
        item_id: Option<i64>,
        sku: Option<&str>,
        quantity: f64,
        created_by: Option<i64>,
        notes: Option<&str>,
    ) -> Result<InternalTransfer, sqlx::Error> {
        sqlx::query_as::<_, InternalTransfer>(
            "INSERT INTO internal_transfers \
             (tenant_id, transfer_no, from_location_id, to_location_id, item_id, sku, \
              quantity, created_by, notes) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?) \
             RETURNING id, tenant_id, transfer_no, from_location_id, to_location_id, item_id, \
                       sku, quantity, transferred_at, status, created_by, notes, created_at",
        )
        .bind(tenant_id)
        .bind(transfer_no)
        .bind(from_location_id)
        .bind(to_location_id)
        .bind(item_id)
        .bind(sku)
        .bind(quantity)
        .bind(created_by)
        .bind(notes)
        .fetch_one(pool)
        .await
    }

    pub async fn list(pool: &SqlitePool, tenant_id: i64) -> Result<Vec<InternalTransfer>, sqlx::Error> {
        sqlx::query_as::<_, InternalTransfer>(
            "SELECT id, tenant_id, transfer_no, from_location_id, to_location_id, item_id, \
                    sku, quantity, transferred_at, status, created_by, notes, created_at \
             FROM internal_transfers WHERE tenant_id = ? ORDER BY id DESC LIMIT 500",
        )
        .bind(tenant_id)
        .fetch_all(pool)
        .await
    }
}

pub struct CountRepo;

impl CountRepo {
    pub async fn create_template(
        pool: &SqlitePool,
        tenant_id: i64,
        name: &str,
        description: Option<&str>,
        location_ids: &serde_json::Value,
    ) -> Result<CountTemplate, sqlx::Error> {
        sqlx::query_as::<_, CountTemplate>(
            "INSERT INTO count_templates (tenant_id, name, description, location_ids) \
             VALUES (?, ?, ?, ?) \
             RETURNING id, tenant_id, name, description, location_ids, is_active, created_at, updated_at",
        )
        .bind(tenant_id)
        .bind(name)
        .bind(description)
        .bind(location_ids)
        .fetch_one(pool)
        .await
    }

    pub async fn list_templates(pool: &SqlitePool, tenant_id: i64) -> Result<Vec<CountTemplate>, sqlx::Error> {
        sqlx::query_as::<_, CountTemplate>(
            "SELECT id, tenant_id, name, description, location_ids, is_active, created_at, updated_at \
             FROM count_templates WHERE tenant_id = ? ORDER BY id DESC",
        )
        .bind(tenant_id)
        .fetch_all(pool)
        .await
    }

    pub async fn create_session(
        pool: &SqlitePool,
        tenant_id: i64,
        template_id: i64,
        session_no: &str,
    ) -> Result<CountSession, sqlx::Error> {
        sqlx::query_as::<_, CountSession>(
            "INSERT INTO count_sessions (tenant_id, template_id, session_no) \
             VALUES (?, ?, ?) \
             RETURNING id, tenant_id, template_id, session_no, status, started_at, completed_at, result_json",
        )
        .bind(tenant_id)
        .bind(template_id)
        .bind(session_no)
        .fetch_one(pool)
        .await
    }

    pub async fn complete_session(
        pool: &SqlitePool,
        tenant_id: i64,
        session_id: i64,
        result: &serde_json::Value,
    ) -> Result<Option<CountSession>, sqlx::Error> {
        sqlx::query_as::<_, CountSession>(
            "UPDATE count_sessions SET status = 'completed', completed_at = datetime('now'), result_json = ? \
             WHERE tenant_id = ? AND id = ? AND status = 'inprogress' \
             RETURNING id, tenant_id, template_id, session_no, status, started_at, completed_at, result_json",
        )
        .bind(result)
        .bind(tenant_id)
        .bind(session_id)
        .fetch_optional(pool)
        .await
    }

    pub async fn list_sessions(pool: &SqlitePool, tenant_id: i64) -> Result<Vec<CountSession>, sqlx::Error> {
        sqlx::query_as::<_, CountSession>(
            "SELECT id, tenant_id, template_id, session_no, status, started_at, completed_at, result_json \
             FROM count_sessions WHERE tenant_id = ? ORDER BY id DESC LIMIT 500",
        )
        .bind(tenant_id)
        .fetch_all(pool)
        .await
    }
}
