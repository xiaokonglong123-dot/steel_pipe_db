//! Manufacturing repositories.

use sqlx::SqlitePool;
use crate::models::manufacturing::{Bom, BomItem, Inspection, Ncr, WorkOrder, WorkOrderStep};

pub struct BomRepo;

impl BomRepo {
    pub async fn list(pool: &SqlitePool, tenant_id: i64) -> Result<Vec<Bom>, sqlx::Error> {
        sqlx::query_as::<_, Bom>(
            "SELECT id, tenant_id, name, product_type, version, is_active, notes, \
                    created_at, updated_at, deleted_at \
             FROM mfg_boms WHERE tenant_id = ? AND deleted_at IS NULL ORDER BY id DESC",
        )
        .bind(tenant_id)
        .fetch_all(pool)
        .await
    }

    pub async fn create(
        pool: &SqlitePool,
        tenant_id: i64,
        name: &str,
        product_type: &str,
        notes: Option<&str>,
    ) -> Result<Bom, sqlx::Error> {
        sqlx::query_as::<_, Bom>(
            "INSERT INTO mfg_boms (tenant_id, name, product_type, notes) \
             VALUES (?, ?, ?, ?) \
             RETURNING id, tenant_id, name, product_type, version, is_active, notes, \
                       created_at, updated_at, deleted_at",
        )
        .bind(tenant_id)
        .bind(name)
        .bind(product_type)
        .bind(notes)
        .fetch_one(pool)
        .await
    }

    pub async fn insert_item(
        pool: &SqlitePool,
        bom_id: i64,
        material: &str,
        quantity: f64,
        unit: &str,
        notes: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO mfg_bom_items (bom_id, material, quantity, unit, notes) \
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(bom_id)
        .bind(material)
        .bind(quantity)
        .bind(unit)
        .bind(notes)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn items_for_bom(pool: &SqlitePool, bom_id: i64) -> Result<Vec<BomItem>, sqlx::Error> {
        sqlx::query_as::<_, BomItem>(
            "SELECT id, bom_id, material, quantity, unit, notes \
             FROM mfg_bom_items WHERE bom_id = ? ORDER BY id",
        )
        .bind(bom_id)
        .fetch_all(pool)
        .await
    }

    pub async fn find_by_id(pool: &SqlitePool, tenant_id: i64, id: i64) -> Result<Option<Bom>, sqlx::Error> {
        sqlx::query_as::<_, Bom>(
            "SELECT id, tenant_id, name, product_type, version, is_active, notes, \
                    created_at, updated_at, deleted_at \
             FROM mfg_boms WHERE tenant_id = ? AND id = ? AND deleted_at IS NULL",
        )
        .bind(tenant_id)
        .bind(id)
        .fetch_optional(pool)
        .await
    }
}

pub struct WorkOrderRepo;

impl WorkOrderRepo {
    pub async fn create(
        pool: &SqlitePool,
        tenant_id: i64,
        wo_no: &str,
        bom_id: Option<i64>,
        product_type: &str,
        quantity: f64,
        assigned_to: Option<i64>,
        due_date: Option<chrono::NaiveDate>,
        notes: Option<&str>,
    ) -> Result<WorkOrder, sqlx::Error> {
        sqlx::query_as::<_, WorkOrder>(
            "INSERT INTO mfg_work_orders \
             (tenant_id, wo_no, bom_id, product_type, quantity, assigned_to, due_date, notes) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?) \
             RETURNING id, tenant_id, wo_no, bom_id, product_type, quantity, status, \
                       current_step, assigned_to, due_date, notes, created_at, updated_at, deleted_at",
        )
        .bind(tenant_id)
        .bind(wo_no)
        .bind(bom_id)
        .bind(product_type)
        .bind(quantity)
        .bind(assigned_to)
        .bind(due_date)
        .bind(notes)
        .fetch_one(pool)
        .await
    }

    pub async fn insert_step(
        pool: &SqlitePool,
        work_order_id: i64,
        step_index: i32,
        step_name: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO mfg_work_order_steps (work_order_id, step_index, step_name) \
             VALUES (?, ?, ?)",
        )
        .bind(work_order_id)
        .bind(step_index)
        .bind(step_name)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn list(pool: &SqlitePool, tenant_id: i64, status: Option<&str>) -> Result<Vec<WorkOrder>, sqlx::Error> {
        sqlx::query_as::<_, WorkOrder>(
            "SELECT id, tenant_id, wo_no, bom_id, product_type, quantity, status, \
                    current_step, assigned_to, due_date, notes, created_at, updated_at, deleted_at \
             FROM mfg_work_orders WHERE tenant_id = ? AND deleted_at IS NULL \
             AND (? IS NULL OR status = ?) ORDER BY id DESC LIMIT 500",
        )
        .bind(tenant_id)
        .bind(status)
        .bind(status)
        .fetch_all(pool)
        .await
    }

    pub async fn find_by_id(pool: &SqlitePool, tenant_id: i64, id: i64) -> Result<Option<WorkOrder>, sqlx::Error> {
        sqlx::query_as::<_, WorkOrder>(
            "SELECT id, tenant_id, wo_no, bom_id, product_type, quantity, status, \
                    current_step, assigned_to, due_date, notes, created_at, updated_at, deleted_at \
             FROM mfg_work_orders WHERE tenant_id = ? AND id = ? AND deleted_at IS NULL",
        )
        .bind(tenant_id)
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    pub async fn update_status(
        pool: &SqlitePool,
        tenant_id: i64,
        id: i64,
        status: &str,
    ) -> Result<Option<WorkOrder>, sqlx::Error> {
        sqlx::query_as::<_, WorkOrder>(
            "UPDATE mfg_work_orders SET status = ?, updated_at = datetime('now') \
             WHERE tenant_id = ? AND id = ? AND deleted_at IS NULL \
             RETURNING id, tenant_id, wo_no, bom_id, product_type, quantity, status, \
                       current_step, assigned_to, due_date, notes, created_at, updated_at, deleted_at",
        )
        .bind(status)
        .bind(tenant_id)
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    pub async fn advance_step(
        pool: &SqlitePool,
        tenant_id: i64,
        id: i64,
        step: i32,
    ) -> Result<Option<WorkOrder>, sqlx::Error> {
        sqlx::query_as::<_, WorkOrder>(
            "UPDATE mfg_work_orders SET current_step = ?, updated_at = datetime('now') \
             WHERE tenant_id = ? AND id = ? AND deleted_at IS NULL \
             RETURNING id, tenant_id, wo_no, bom_id, product_type, quantity, status, \
                       current_step, assigned_to, due_date, notes, created_at, updated_at, deleted_at",
        )
        .bind(step)
        .bind(tenant_id)
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    pub async fn steps_for_wo(pool: &SqlitePool, work_order_id: i64) -> Result<Vec<WorkOrderStep>, sqlx::Error> {
        sqlx::query_as::<_, WorkOrderStep>(
            "SELECT id, work_order_id, step_index, step_name, status, started_at, completed_at, notes \
             FROM mfg_work_order_steps WHERE work_order_id = ? ORDER BY step_index",
        )
        .bind(work_order_id)
        .fetch_all(pool)
        .await
    }

    pub async fn complete_step(
        pool: &SqlitePool,
        work_order_id: i64,
        step_index: i32,
    ) -> Result<Option<WorkOrderStep>, sqlx::Error> {
        sqlx::query_as::<_, WorkOrderStep>(
            "UPDATE mfg_work_order_steps SET status = 'done', completed_at = datetime('now') \
             WHERE work_order_id = ? AND step_index = ? AND status = 'pending' \
             RETURNING id, work_order_id, step_index, step_name, status, started_at, completed_at, notes",
        )
        .bind(work_order_id)
        .bind(step_index)
        .fetch_optional(pool)
        .await
    }
}

pub struct InspectionRepo;

impl InspectionRepo {
    pub async fn create(
        pool: &SqlitePool,
        tenant_id: i64,
        work_order_id: Option<i64>,
        item_id: Option<i64>,
        inspection_type: &str,
        result: &str,
        inspector: Option<i64>,
        notes: Option<&str>,
    ) -> Result<Inspection, sqlx::Error> {
        sqlx::query_as::<_, Inspection>(
            "INSERT INTO mfg_inspections \
             (tenant_id, work_order_id, item_id, inspection_type, result, inspector, notes) \
             VALUES (?, ?, ?, ?, ?, ?, ?) \
             RETURNING id, tenant_id, work_order_id, item_id, inspection_type, result, \
                       inspector, notes, inspected_at, created_at",
        )
        .bind(tenant_id)
        .bind(work_order_id)
        .bind(item_id)
        .bind(inspection_type)
        .bind(result)
        .bind(inspector)
        .bind(notes)
        .fetch_one(pool)
        .await
    }

    pub async fn list(pool: &SqlitePool, tenant_id: i64, work_order_id: Option<i64>) -> Result<Vec<Inspection>, sqlx::Error> {
        sqlx::query_as::<_, Inspection>(
            "SELECT id, tenant_id, work_order_id, item_id, inspection_type, result, \
                    inspector, notes, inspected_at, created_at \
             FROM mfg_inspections WHERE tenant_id = ? \
             AND (? IS NULL OR work_order_id = ?) ORDER BY id DESC LIMIT 500",
        )
        .bind(tenant_id)
        .bind(work_order_id)
        .bind(work_order_id)
        .fetch_all(pool)
        .await
    }
}

pub struct NcrRepo;

impl NcrRepo {
    pub async fn create(
        pool: &SqlitePool,
        tenant_id: i64,
        ncr_no: &str,
        work_order_id: Option<i64>,
        item_id: Option<i64>,
        description: &str,
        severity: &str,
        created_by: Option<i64>,
    ) -> Result<Ncr, sqlx::Error> {
        sqlx::query_as::<_, Ncr>(
            "INSERT INTO mfg_ncrs \
             (tenant_id, ncr_no, work_order_id, item_id, description, severity, created_by) \
             VALUES (?, ?, ?, ?, ?, ?, ?) \
             RETURNING id, tenant_id, ncr_no, work_order_id, item_id, description, severity, \
                       disposition, status, created_by, created_at, resolved_at",
        )
        .bind(tenant_id)
        .bind(ncr_no)
        .bind(work_order_id)
        .bind(item_id)
        .bind(description)
        .bind(severity)
        .bind(created_by)
        .fetch_one(pool)
        .await
    }

    pub async fn list(pool: &SqlitePool, tenant_id: i64, status: Option<&str>) -> Result<Vec<Ncr>, sqlx::Error> {
        sqlx::query_as::<_, Ncr>(
            "SELECT id, tenant_id, ncr_no, work_order_id, item_id, description, severity, \
                    disposition, status, created_by, created_at, resolved_at \
             FROM mfg_ncrs WHERE tenant_id = ? \
             AND (? IS NULL OR status = ?) ORDER BY id DESC LIMIT 500",
        )
        .bind(tenant_id)
        .bind(status)
        .bind(status)
        .fetch_all(pool)
        .await
    }

    pub async fn resolve(
        pool: &SqlitePool,
        tenant_id: i64,
        id: i64,
        disposition: &str,
    ) -> Result<Option<Ncr>, sqlx::Error> {
        sqlx::query_as::<_, Ncr>(
            "UPDATE mfg_ncrs SET disposition = ?, status = 'resolved', resolved_at = datetime('now') \
             WHERE tenant_id = ? AND id = ? AND status = 'open' \
             RETURNING id, tenant_id, ncr_no, work_order_id, item_id, description, severity, \
                       disposition, status, created_by, created_at, resolved_at",
        )
        .bind(disposition)
        .bind(tenant_id)
        .bind(id)
        .fetch_optional(pool)
        .await
    }
}
