use chrono::Utc;
use sqlx::SqlitePool;

use crate::dto::common::PaginationParams;
use crate::dto::inventory_dto::{
    CreateCheckRequest, CreateInboundRecordRequest, CreateLocationRequest,
    CreateOutboundRecordRequest, InboundFilter, InventoryFilter, OutboundFilter,
    SubmitCheckItemRequest, UpdateLocationRequest,
};
use crate::error::AppError;
use crate::models::inventory::{
    InboundItem, InboundRecord, InventoryCheckItem, InventoryCheckRecord, InventoryLog, Location,
    OutboundItem, OutboundRecord,
};
use crate::repositories::inventory_repo::{
    CheckInitItem, CheckRepo, CreateInventoryLog, InboundRepo, InventoryLogRepo, LocationRepo,
    OutboundRepo,
};

pub struct InventoryService;

impl InventoryService {
    fn generate_no(prefix: &str) -> String {
        let now = Utc::now();
        let date_str = now.format("%Y%m%d").to_string();
        let serial = uuid::Uuid::new_v4().to_string();
        let short_serial = &serial[..8];
        format!("{}-{}-{}", prefix, date_str, short_serial)
    }

    fn build_full_code(zone: &str, shelf: &str, level: &str) -> String {
        format!("{}-{}-{}", zone, shelf, level)
    }

    // ━━━ Inbound ━━━

    pub async fn create_inbound(
        pool: &SqlitePool,
        dto: &CreateInboundRecordRequest,
    ) -> Result<InboundRecord, AppError> {
        if dto.pipes.is_empty() {
            return Err(AppError::Validation("At least one pipe is required".into()));
        }

        let inbound_no = Self::generate_no("IN");

        let record = InboundRepo::create_with_items(pool, dto, &inbound_no)
            .await
            .map_err(AppError::from)?;

        if record.approval_status == "auto_approved" {
            for item in &dto.pipes {
                Self::execute_inbound(pool, record.id, item).await?;
            }
        }

        Ok(record)
    }

    async fn execute_inbound(
        pool: &SqlitePool,
        record_id: i64,
        item: &crate::dto::inventory_dto::InboundPipeItem,
    ) -> Result<(), AppError> {
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        match item.pipe_type.as_str() {
            "seamless" | "casing" | "tubing" => {
                sqlx::query(
                    "UPDATE seamless_pipes SET status = 'in_stock', updated_at = ? WHERE id = ? AND deleted_at IS NULL",
                )
                .bind(&now)
                .bind(item.pipe_id)
                .execute(pool)
                .await
                .map_err(AppError::from)?;
            }
            "screen" | "screened" => {
                sqlx::query(
                    "UPDATE screen_pipes SET status = 'in_stock', updated_at = ? WHERE id = ? AND deleted_at IS NULL",
                )
                .bind(&now)
                .bind(item.pipe_id)
                .execute(pool)
                .await
                .map_err(AppError::from)?;
            }
            _ => {
                return Err(AppError::Validation(format!(
                    "Unknown pipe_type: {}",
                    item.pipe_type
                )));
            }
        }

        let _ = InventoryLogRepo::create(
            pool,
            &CreateInventoryLog {
                pipe_type: item.pipe_type.clone(),
                pipe_id: item.pipe_id,
                change_type: "inbound".into(),
                ref_type: Some("inbound".into()),
                ref_id: Some(record_id),
                from_location_id: None,
                to_location_id: None,
                notes: None,
                created_by: None,
            },
        )
        .await;

        Ok(())
    }

    pub async fn approve_inbound(pool: &SqlitePool, id: i64) -> Result<(), AppError> {
        let record = InboundRepo::find_by_id(pool, id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::NotFound(format!("Inbound record id={} not found", id)))?;

        if record.deleted_at.is_some() {
            return Err(AppError::NotFound(format!(
                "Inbound record id={} has been deleted",
                id
            )));
        }

        if record.approval_status != "pending" {
            return Err(AppError::Validation(format!(
                "Cannot approve inbound with status '{}'",
                record.approval_status
            )));
        }

        let items = InboundRepo::find_items(pool, id).await.map_err(AppError::from)?;

        InboundRepo::update_status(pool, id, "approved")
            .await
            .map_err(AppError::from)?;

        for item in &items {
            let pipe_item = crate::dto::inventory_dto::InboundPipeItem {
                pipe_type: item.pipe_type.clone(),
                pipe_id: item.pipe_id,
            };
            Self::execute_inbound(pool, id, &pipe_item).await?;
        }

        Ok(())
    }

    pub async fn reject_inbound(
        pool: &SqlitePool,
        id: i64,
        _reason: &str,
    ) -> Result<(), AppError> {
        let record = InboundRepo::find_by_id(pool, id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::NotFound(format!("Inbound record id={} not found", id)))?;

        if record.approval_status != "pending" {
            return Err(AppError::Validation(format!(
                "Cannot reject inbound with status '{}'",
                record.approval_status
            )));
        }

        InboundRepo::update_status(pool, id, "rejected")
            .await
            .map_err(AppError::from)?;

        Ok(())
    }

    pub async fn get_inbound_record(
        pool: &SqlitePool,
        id: i64,
    ) -> Result<(InboundRecord, Vec<InboundItem>), AppError> {
        let record = InboundRepo::find_by_id(pool, id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::NotFound(format!("Inbound record id={} not found", id)))?;

        let items = InboundRepo::find_items(pool, id)
            .await
            .map_err(AppError::from)?;

        Ok((record, items))
    }

    pub async fn list_inbound_records(
        pool: &SqlitePool,
        filter: &InboundFilter,
    ) -> Result<(Vec<InboundRecord>, u64), AppError> {
        InboundRepo::list(pool, filter)
            .await
            .map_err(AppError::from)
    }

    pub async fn delete_inbound(pool: &SqlitePool, id: i64) -> Result<(), AppError> {
        let record = InboundRepo::find_by_id(pool, id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::NotFound(format!("Inbound record id={} not found", id)))?;

        if record.approval_status != "auto_approved" && record.approval_status != "rejected" {
            return Err(AppError::Validation(format!(
                "Cannot delete inbound with status '{}'. Only auto-approved or rejected records can be deleted.",
                record.approval_status
            )));
        }

        InboundRepo::delete(pool, id)
            .await
            .map_err(AppError::from)
    }

    // ━━━ Outbound ━━━

    pub async fn create_outbound(
        pool: &SqlitePool,
        dto: &CreateOutboundRecordRequest,
    ) -> Result<OutboundRecord, AppError> {
        if dto.pipes.is_empty() {
            return Err(AppError::Validation("At least one pipe is required".into()));
        }

        for item in &dto.pipes {
            let pipe = Self::find_pipe_by_id(pool, &item.pipe_type, item.pipe_id).await?;
            let status = match pipe.get("status").and_then(|v| v.as_str()) {
                Some(s) => s,
                None => {
                    return Err(AppError::NotFound(format!(
                        "Pipe {} id={} not found",
                        item.pipe_type, item.pipe_id
                    )))
                }
            };
            if status != "in_stock" {
                return Err(AppError::InsufficientStock);
            }
        }

        let outbound_no = Self::generate_no("OUT");

        let record = OutboundRepo::create_with_items(pool, dto, &outbound_no)
            .await
            .map_err(AppError::from)?;

        if record.approval_status == "auto_approved" {
            for item in &dto.pipes {
                Self::execute_outbound(pool, record.id, item).await?;
            }
        }

        Ok(record)
    }

    async fn find_pipe_by_id(
        pool: &SqlitePool,
        pipe_type: &str,
        pipe_id: i64,
    ) -> Result<serde_json::Value, AppError> {
        match pipe_type {
            "seamless" | "casing" | "tubing" => {
                let row = sqlx::query_as::<_, (i64, String)>(
                    "SELECT id, status FROM seamless_pipes WHERE id = ? AND deleted_at IS NULL",
                )
                .bind(pipe_id)
                .fetch_optional(pool)
                .await
                .map_err(AppError::from)?;

                match row {
                    Some((_id, status)) => Ok(serde_json::json!({"status": status})),
                    None => Ok(serde_json::json!({})),
                }
            }
            "screen" | "screened" => {
                let row = sqlx::query_as::<_, (i64, String)>(
                    "SELECT id, status FROM screen_pipes WHERE id = ? AND deleted_at IS NULL",
                )
                .bind(pipe_id)
                .fetch_optional(pool)
                .await
                .map_err(AppError::from)?;

                match row {
                    Some((_id, status)) => Ok(serde_json::json!({"status": status})),
                    None => Ok(serde_json::json!({})),
                }
            }
            _ => Err(AppError::Validation(format!(
                "Unknown pipe_type: {}",
                pipe_type
            ))),
        }
    }

    async fn execute_outbound(
        pool: &SqlitePool,
        record_id: i64,
        item: &crate::dto::inventory_dto::OutboundPipeItem,
    ) -> Result<(), AppError> {
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        match item.pipe_type.as_str() {
            "seamless" | "casing" | "tubing" => {
                sqlx::query(
                    "UPDATE seamless_pipes SET status = 'outbound', updated_at = ? WHERE id = ? AND deleted_at IS NULL",
                )
                .bind(&now)
                .bind(item.pipe_id)
                .execute(pool)
                .await
                .map_err(AppError::from)?;
            }
            "screen" | "screened" => {
                sqlx::query(
                    "UPDATE screen_pipes SET status = 'outbound', updated_at = ? WHERE id = ? AND deleted_at IS NULL",
                )
                .bind(&now)
                .bind(item.pipe_id)
                .execute(pool)
                .await
                .map_err(AppError::from)?;
            }
            _ => {
                return Err(AppError::Validation(format!(
                    "Unknown pipe_type: {}",
                    item.pipe_type
                )));
            }
        }

        let _ = InventoryLogRepo::create(
            pool,
            &CreateInventoryLog {
                pipe_type: item.pipe_type.clone(),
                pipe_id: item.pipe_id,
                change_type: "outbound".into(),
                ref_type: Some("outbound".into()),
                ref_id: Some(record_id),
                from_location_id: None,
                to_location_id: None,
                notes: None,
                created_by: None,
            },
        )
        .await;

        Ok(())
    }

    pub async fn approve_outbound(pool: &SqlitePool, id: i64) -> Result<(), AppError> {
        let record = OutboundRepo::find_by_id(pool, id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::NotFound(format!("Outbound record id={} not found", id)))?;

        if record.deleted_at.is_some() {
            return Err(AppError::NotFound(format!(
                "Outbound record id={} has been deleted",
                id
            )));
        }

        if record.approval_status != "pending" {
            return Err(AppError::Validation(format!(
                "Cannot approve outbound with status '{}'",
                record.approval_status
            )));
        }

        let items = OutboundRepo::find_items(pool, id)
            .await
            .map_err(AppError::from)?;

        OutboundRepo::update_status(pool, id, "approved")
            .await
            .map_err(AppError::from)?;

        for item in &items {
            let pipe_item = crate::dto::inventory_dto::OutboundPipeItem {
                pipe_type: item.pipe_type.clone(),
                pipe_id: item.pipe_id,
            };
            Self::execute_outbound(pool, id, &pipe_item).await?;
        }

        Ok(())
    }

    pub async fn reject_outbound(
        pool: &SqlitePool,
        id: i64,
        _reason: &str,
    ) -> Result<(), AppError> {
        let record = OutboundRepo::find_by_id(pool, id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::NotFound(format!("Outbound record id={} not found", id)))?;

        if record.approval_status != "pending" {
            return Err(AppError::Validation(format!(
                "Cannot reject outbound with status '{}'",
                record.approval_status
            )));
        }

        OutboundRepo::update_status(pool, id, "rejected")
            .await
            .map_err(AppError::from)?;

        Ok(())
    }

    pub async fn get_outbound_record(
        pool: &SqlitePool,
        id: i64,
    ) -> Result<(OutboundRecord, Vec<OutboundItem>), AppError> {
        let record = OutboundRepo::find_by_id(pool, id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::NotFound(format!("Outbound record id={} not found", id)))?;

        let items = OutboundRepo::find_items(pool, id)
            .await
            .map_err(AppError::from)?;

        Ok((record, items))
    }

    pub async fn list_outbound_records(
        pool: &SqlitePool,
        filter: &OutboundFilter,
    ) -> Result<(Vec<OutboundRecord>, u64), AppError> {
        OutboundRepo::list(pool, filter)
            .await
            .map_err(AppError::from)
    }

    pub async fn delete_outbound(pool: &SqlitePool, id: i64) -> Result<(), AppError> {
        let record = OutboundRepo::find_by_id(pool, id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::NotFound(format!("Outbound record id={} not found", id)))?;

        if record.approval_status != "auto_approved" && record.approval_status != "rejected" {
            return Err(AppError::Validation(format!(
                "Cannot delete outbound with status '{}'. Only auto-approved or rejected records can be deleted.",
                record.approval_status
            )));
        }

        OutboundRepo::delete(pool, id)
            .await
            .map_err(AppError::from)
    }

    // ━━━ Inventory ━━━

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

        let mut conditions: Vec<String> = vec!["deleted_at IS NULL".into()];
        if let Some(ref grade) = filter.grade {
            conditions.push(format!("grade = '{}'", grade.replace('\'', "''")));
        }
        if let Some(location_id) = filter.location_id {
            conditions.push(format!("location_id = {}", location_id));
        }
        if let Some(ref q) = filter.q {
            if !q.is_empty() {
                conditions.push(format!(
                    "pipe_number LIKE '%{}%'",
                    q.replace('\'', "''")
                ));
            }
        }
        let where_clause = conditions.join(" AND ");

        let pipe_type_filter = filter.pipe_type.clone();

        let count_sql = match &pipe_type_filter {
            Some(pt) if pt == "seamless" || pt == "casing" || pt == "tubing" => {
                format!(
                    "SELECT COUNT(*) as cnt FROM seamless_pipes WHERE {}",
                    where_clause
                )
            }
            Some(pt) if pt == "screen" || pt == "screened" => {
                format!(
                    "SELECT COUNT(*) as cnt FROM screen_pipes WHERE {}",
                    where_clause
                )
            }
            _ => {
                format!(
                    "SELECT (SELECT COUNT(*) FROM seamless_pipes WHERE {}) + \
                     (SELECT COUNT(*) FROM screen_pipes WHERE {}) as cnt",
                    where_clause,
                    where_clause.replace("grade", "base_grade")
                )
            }
        };

        let total: (i64,) = sqlx::query_as(&count_sql).fetch_one(pool).await.map_err(AppError::from)?;

        let list_sql = match &pipe_type_filter {
            Some(pt) if pt == "seamless" || pt == "casing" || pt == "tubing" => {
                format!(
                    "SELECT id, pipe_number, grade, od, wt, pipe_type, status, location_id, \
                     created_at, updated_at FROM seamless_pipes WHERE {} \
                     ORDER BY created_at DESC LIMIT {} OFFSET {}",
                    where_clause, page_size, offset
                )
            }
            Some(pt) if pt == "screen" || pt == "screened" => {
                format!(
                    "SELECT id, pipe_number, base_grade as grade, base_od as od, base_wt as wt, \
                     screen_type as pipe_type, status, location_id, created_at, updated_at \
                     FROM screen_pipes WHERE {} \
                     ORDER BY created_at DESC LIMIT {} OFFSET {}",
                    where_clause, page_size, offset
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
                     ORDER BY created_at DESC LIMIT {} OFFSET {}",
                    where_clause,
                    where_clause.replace("grade", "base_grade"),
                    page_size,
                    offset
                )
            }
        };

        let items: Vec<serde_json::Value> = sqlx::query_as::<_, (i64, String, String, f64, f64, String, String, Option<i64>, String, String)>(
            &list_sql,
        )
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

    // ━━━ Locations ━━━

    pub async fn create_location(
        pool: &SqlitePool,
        dto: &CreateLocationRequest,
    ) -> Result<Location, AppError> {
        let full_code = Self::build_full_code(&dto.zone_code, &dto.shelf_code, &dto.level_code);

        if LocationRepo::find_by_full_code(pool, &full_code)
            .await
            .map_err(AppError::from)?
            .is_some()
        {
            return Err(AppError::Validation(format!(
                "Location '{}' already exists",
                full_code
            )));
        }

        LocationRepo::create(pool, dto, &full_code)
            .await
            .map_err(AppError::from)
    }

    pub async fn update_location(
        pool: &SqlitePool,
        id: i64,
        dto: &UpdateLocationRequest,
    ) -> Result<Location, AppError> {
        let existing = LocationRepo::find_by_id(pool, id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::LocationNotFound(format!("Location id={} not found", id)))?;

        if existing.deleted_at.is_some() {
            return Err(AppError::LocationNotFound(format!(
                "Location id={} has been deleted",
                id
            )));
        }

        LocationRepo::update(pool, id, dto)
            .await
            .map_err(AppError::from)
    }

    pub async fn list_locations(
        pool: &SqlitePool,
        params: &PaginationParams,
        active_only: bool,
    ) -> Result<(Vec<Location>, u64), AppError> {
        LocationRepo::list(pool, params, active_only)
            .await
            .map_err(AppError::from)
    }

    pub async fn get_location(pool: &SqlitePool, id: i64) -> Result<Location, AppError> {
        LocationRepo::find_by_id(pool, id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::LocationNotFound(format!("Location id={} not found", id)))
    }

    pub async fn delete_location(pool: &SqlitePool, id: i64) -> Result<(), AppError> {
        let existing = LocationRepo::find_by_id(pool, id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::LocationNotFound(format!("Location id={} not found", id)))?;

        if existing.used_count > 0 {
            return Err(AppError::Validation(format!(
                "Cannot delete location id={} with {} pipes still stored",
                id, existing.used_count
            )));
        }

        LocationRepo::delete(pool, id)
            .await
            .map_err(AppError::from)
    }

    // ━━━ Checks ━━━

    pub async fn create_check(
        pool: &SqlitePool,
        dto: &CreateCheckRequest,
    ) -> Result<InventoryCheckRecord, AppError> {
        let check_no = Self::generate_no("CHK");

        let mut items: Vec<CheckInitItem> = Vec::new();

        let seamless_pipes: Vec<(i64,)> = sqlx::query_as(
            "SELECT id FROM seamless_pipes WHERE status = 'in_stock' AND deleted_at IS NULL",
        )
        .fetch_all(pool)
        .await
        .map_err(AppError::from)?;

        for (id,) in seamless_pipes {
            items.push(CheckInitItem {
                pipe_type: "seamless".into(),
                pipe_id: id,
                expected_status: "in_stock".into(),
            });
        }

        let screen_pipes: Vec<(i64,)> = sqlx::query_as(
            "SELECT id FROM screen_pipes WHERE status = 'in_stock' AND deleted_at IS NULL",
        )
        .fetch_all(pool)
        .await
        .map_err(AppError::from)?;

        for (id,) in screen_pipes {
            items.push(CheckInitItem {
                pipe_type: "screen".into(),
                pipe_id: id,
                expected_status: "in_stock".into(),
            });
        }

        CheckRepo::create(pool, dto, &check_no, &items)
            .await
            .map_err(AppError::from)
    }

    pub async fn get_check_detail(
        pool: &SqlitePool,
        id: i64,
    ) -> Result<(InventoryCheckRecord, Vec<InventoryCheckItem>), AppError> {
        let record = CheckRepo::find_by_id(pool, id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::NotFound(format!("Check record id={} not found", id)))?;

        let items = CheckRepo::get_check_items(pool, id)
            .await
            .map_err(AppError::from)?;

        Ok((record, items))
    }

    pub async fn list_checks(
        pool: &SqlitePool,
        params: &PaginationParams,
    ) -> Result<(Vec<InventoryCheckRecord>, u64), AppError> {
        CheckRepo::list(pool, params)
            .await
            .map_err(AppError::from)
    }

    pub async fn submit_check_item(
        pool: &SqlitePool,
        check_id: i64,
        item_id: i64,
        dto: &SubmitCheckItemRequest,
    ) -> Result<InventoryCheckItem, AppError> {
        let record = CheckRepo::find_by_id(pool, check_id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::NotFound(format!("Check record id={} not found", check_id)))?;

        if record.status != "in_progress" {
            return Err(AppError::Validation(format!(
                "Check id={} is not in progress (status: {})",
                check_id, record.status
            )));
        }

        CheckRepo::update_item_result(pool, check_id, item_id, &dto.found_status, &dto.notes)
            .await
            .map_err(AppError::from)
    }
}
