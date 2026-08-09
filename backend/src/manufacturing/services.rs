//! Manufacturing services — BOMs, work orders (step state machine), NCRs.

use sqlx::SqlitePool;

use crate::dto::manufacturing_dto::{
    CreateBomRequest, CreateInspectionRequest, CreateNcrRequest, CreateWorkOrderRequest,
    ResolveNcrRequest,
};
use crate::error::AppError;
use crate::manufacturing::repos::{BomRepo, InspectionRepo, NcrRepo, WorkOrderRepo};
use crate::models::manufacturing::{Bom, BomItem, Inspection, Ncr, WorkOrder, WorkOrderStep};

pub struct ManufacturingService;

impl ManufacturingService {
    // -----------------------------------------------------------------------
    // BOMs
    // -----------------------------------------------------------------------

    pub async fn create_bom(pool: &SqlitePool, tenant_id: i64, dto: &CreateBomRequest) -> Result<Bom, AppError> {
        if dto.name.trim().is_empty() {
            return Err(AppError::Validation("BOM name is required".into()));
        }
        if dto.items.is_empty() {
            return Err(AppError::Validation("BOM needs at least one item".into()));
        }
        let bom = BomRepo::create(pool, tenant_id, dto.name.trim(), &dto.product_type, dto.notes.as_deref())
            .await
            .map_err(AppError::from)?;
        for item in &dto.items {
            if item.material.trim().is_empty() {
                return Err(AppError::Validation("BOM item material is required".into()));
            }
            BomRepo::insert_item(
                pool,
                bom.id,
                item.material.trim(),
                item.quantity,
                item.unit.as_deref().unwrap_or("pcs"),
                item.notes.as_deref(),
            )
            .await
            .map_err(AppError::from)?;
        }
        Ok(bom)
    }

    pub async fn list_boms(pool: &SqlitePool, tenant_id: i64) -> Result<Vec<Bom>, AppError> {
        BomRepo::list(pool, tenant_id).await.map_err(AppError::from)
    }

    pub async fn get_bom(pool: &SqlitePool, tenant_id: i64, id: i64) -> Result<(Bom, Vec<BomItem>), AppError> {
        let bom = BomRepo::find_by_id(pool, tenant_id, id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("BOM not found: {}", id)))?;
        let items = BomRepo::items_for_bom(pool, id).await.map_err(AppError::from)?;
        Ok((bom, items))
    }

    // -----------------------------------------------------------------------
    // Work orders
    // -----------------------------------------------------------------------

    /// Create a work order; if a BOM is referenced, its items become the
    /// default step sequence (one step per material).
    pub async fn create_work_order(
        pool: &SqlitePool,
        tenant_id: i64,
        dto: &CreateWorkOrderRequest,
    ) -> Result<WorkOrder, AppError> {
        if dto.quantity <= 0.0 {
            return Err(AppError::Validation("Quantity must be positive".into()));
        }
        let wo_no = format!("WO-{}-{}", chrono::Utc::now().format("%Y%m%d"), seq(pool, "mfg_work_orders").await?);
        let wo = WorkOrderRepo::create(
            pool, tenant_id, &wo_no, dto.bom_id, &dto.product_type, dto.quantity,
            dto.assigned_to, dto.due_date, dto.notes.as_deref(),
        )
        .await
        .map_err(AppError::from)?;

        // Default step sequence from BOM items (or a generic 3-step default).
        if let Some(bom_id) = dto.bom_id {
            let items = BomRepo::items_for_bom(pool, bom_id).await.map_err(AppError::from)?;
            for (i, item) in items.iter().enumerate() {
                WorkOrderRepo::insert_step(pool, wo.id, i as i32, &format!("加工 {}", item.material))
                    .await
                    .map_err(AppError::from)?;
            }
        } else {
            for (i, name) in ["备料", "加工", "质检"].iter().enumerate() {
                WorkOrderRepo::insert_step(pool, wo.id, i as i32, name)
                    .await
                    .map_err(AppError::from)?;
            }
        }
        Ok(wo)
    }

    pub async fn list_work_orders(pool: &SqlitePool, tenant_id: i64, status: Option<&str>) -> Result<Vec<WorkOrder>, AppError> {
        WorkOrderRepo::list(pool, tenant_id, status).await.map_err(AppError::from)
    }

    pub async fn get_work_order(
        pool: &SqlitePool,
        tenant_id: i64,
        id: i64,
    ) -> Result<(WorkOrder, Vec<WorkOrderStep>), AppError> {
        let wo = WorkOrderRepo::find_by_id(pool, tenant_id, id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Work order not found: {}", id)))?;
        let steps = WorkOrderRepo::steps_for_wo(pool, id).await.map_err(AppError::from)?;
        Ok((wo, steps))
    }

    /// Start a pending work order → in_progress.
    pub async fn start_work_order(pool: &SqlitePool, tenant_id: i64, id: i64) -> Result<WorkOrder, AppError> {
        let wo = WorkOrderRepo::find_by_id(pool, tenant_id, id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Work order not found: {}", id)))?;
        if wo.status != "pending" {
            return Err(AppError::Validation(format!(
                "Only pending work orders can be started (status: {})",
                wo.status
            )));
        }
        WorkOrderRepo::update_status(pool, tenant_id, id, "in_progress")
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Work order not found: {}", id)))
    }

    /// Complete the current step; advances current_step. When the last step
    /// completes, the work order is marked completed.
    pub async fn complete_step(pool: &SqlitePool, tenant_id: i64, id: i64) -> Result<WorkOrder, AppError> {
        let wo = WorkOrderRepo::find_by_id(pool, tenant_id, id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Work order not found: {}", id)))?;
        if wo.status != "in_progress" {
            return Err(AppError::Validation(format!(
                "Only in-progress work orders can advance (status: {})",
                wo.status
            )));
        }
        let steps = WorkOrderRepo::steps_for_wo(pool, id).await.map_err(AppError::from)?;
        let current = steps.iter().find(|s| s.step_index == wo.current_step);
        match current {
            Some(step) => {
                if step.status == "done" {
                    return Err(AppError::Validation(format!(
                        "Step {} already completed",
                        step.step_index
                    )));
                }
                WorkOrderRepo::complete_step(pool, id, step.step_index)
                    .await
                    .map_err(AppError::from)?;
                let next_step = steps.iter().find(|s| s.step_index > step.step_index);
                match next_step {
                    Some(next) => WorkOrderRepo::advance_step(pool, tenant_id, id, next.step_index)
                        .await
                        .map_err(AppError::from),
                    None => WorkOrderRepo::update_status(pool, tenant_id, id, "completed")
                        .await
                        .map_err(AppError::from),
                }
            }
            None => Err(AppError::Validation(format!(
                "No step at index {}",
                wo.current_step
            ))),
        }
        .map_err(|e: AppError| e)
        .and_then(|opt| opt.ok_or_else(|| AppError::NotFound(format!("Work order not found: {}", id))))
    }

    // -----------------------------------------------------------------------
    // Inspections & NCRs
    // -----------------------------------------------------------------------

    pub async fn create_inspection(
        pool: &SqlitePool,
        tenant_id: i64,
        dto: &CreateInspectionRequest,
        inspector: Option<i64>,
    ) -> Result<Inspection, AppError> {
        if !matches!(dto.result.as_str(), "pass" | "fail") {
            return Err(AppError::Validation(format!("Invalid inspection result: {}", dto.result)));
        }
        InspectionRepo::create(
            pool, tenant_id, dto.work_order_id, dto.item_id, &dto.inspection_type,
            &dto.result, inspector, dto.notes.as_deref(),
        )
        .await
        .map_err(AppError::from)
    }

    pub async fn list_inspections(
        pool: &SqlitePool,
        tenant_id: i64,
        work_order_id: Option<i64>,
    ) -> Result<Vec<Inspection>, AppError> {
        InspectionRepo::list(pool, tenant_id, work_order_id).await.map_err(AppError::from)
    }

    pub async fn create_ncr(
        pool: &SqlitePool,
        tenant_id: i64,
        dto: &CreateNcrRequest,
        created_by: Option<i64>,
    ) -> Result<Ncr, AppError> {
        if dto.description.trim().is_empty() {
            return Err(AppError::Validation("NCR description is required".into()));
        }
        let severity = dto.severity.clone().unwrap_or_else(|| "minor".into());
        if !matches!(severity.as_str(), "minor" | "major" | "critical") {
            return Err(AppError::Validation(format!("Invalid severity: {}", severity)));
        }
        let ncr_no = format!("NCR-{}-{}", chrono::Utc::now().format("%Y%m%d"), seq(pool, "mfg_ncrs").await?);
        NcrRepo::create(
            pool, tenant_id, &ncr_no, dto.work_order_id, dto.item_id,
            dto.description.trim(), &severity, created_by,
        )
        .await
        .map_err(AppError::from)
    }

    pub async fn list_ncrs(pool: &SqlitePool, tenant_id: i64, status: Option<&str>) -> Result<Vec<Ncr>, AppError> {
        NcrRepo::list(pool, tenant_id, status).await.map_err(AppError::from)
    }

    pub async fn resolve_ncr(
        pool: &SqlitePool,
        tenant_id: i64,
        id: i64,
        dto: &ResolveNcrRequest,
    ) -> Result<Ncr, AppError> {
        if !matches!(dto.disposition.as_str(), "rework" | "scrap" | "use_as_is") {
            return Err(AppError::Validation(format!("Invalid disposition: {}", dto.disposition)));
        }
        NcrRepo::resolve(pool, tenant_id, id, &dto.disposition)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Open NCR not found: {}", id)))
    }
}

/// Per-table sequence helper for document numbers.
async fn seq(pool: &SqlitePool, table: &str) -> Result<i64, AppError> {
    let n: i64 = sqlx::query_scalar(&format!("SELECT COALESCE(MAX(id), 0) + 1 FROM {}", table))
        .fetch_one(pool)
        .await
        .map_err(AppError::from)?;
    Ok(n)
}
