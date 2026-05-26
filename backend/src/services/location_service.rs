use sqlx::SqlitePool;

use crate::dto::common::PaginationParams;
use crate::dto::inventory_dto::{
    AssignLocationRequest, CreateLocationRequest, TransferLocationRequest, UpdateLocationRequest,
};
use crate::error::AppError;
use crate::models::inventory::Location;
use crate::repositories::inventory_repo::{
    CreateInventoryLog, InventoryLogRepo, InventoryRepo, LocationRepo,
};

pub struct LocationService;

impl LocationService {
    fn build_full_code(zone: &str, shelf: &str, level: &str) -> String {
        format!("{}-{}-{}", zone, shelf, level)
    }

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

    pub async fn assign_location(
        pool: &SqlitePool,
        location_id: i64,
        dto: &AssignLocationRequest,
    ) -> Result<serde_json::Value, AppError> {
        let location = LocationRepo::find_by_id(pool, location_id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::LocationNotFound(format!("Location id={} not found", location_id)))?;

        if !location.is_active {
            return Err(AppError::Validation(format!(
                "Location id={} is not active",
                location_id
            )));
        }

        InventoryRepo::update_pipe_location(pool, &dto.pipe_type, dto.pipe_id, location_id)
            .await
            .map_err(AppError::from)?;

        if let Err(e) = InventoryLogRepo::create(
            pool,
            &CreateInventoryLog {
                pipe_type: dto.pipe_type.clone(),
                pipe_id: dto.pipe_id,
                change_type: "location_assign".into(),
                ref_type: None,
                ref_id: None,
                from_location_id: None,
                to_location_id: Some(location_id),
                notes: None,
                created_by: None,
            },
        )
        .await
        {
            tracing::error!(?e, "Failed to create inventory log for location_assign");
        }

        Ok(serde_json::json!({
            "pipe_type": dto.pipe_type,
            "pipe_id": dto.pipe_id,
            "location_id": location_id,
            "location_code": location.full_code,
        }))
    }

    pub async fn transfer_location(
        pool: &SqlitePool,
        pipe_type: &str,
        pipe_id: i64,
        dto: &TransferLocationRequest,
    ) -> Result<serde_json::Value, AppError> {
        let from_location_id = InventoryRepo::get_pipe_location_id(pool, pipe_type, pipe_id)
            .await
            .map_err(AppError::from)?;

        let to_location = LocationRepo::find_by_id(pool, dto.to_location_id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| {
                AppError::LocationNotFound(format!("Location id={} not found", dto.to_location_id))
            })?;

        if !to_location.is_active {
            return Err(AppError::Validation(format!(
                "Target location id={} is not active",
                dto.to_location_id
            )));
        }

        InventoryRepo::update_pipe_location(pool, pipe_type, pipe_id, dto.to_location_id)
            .await
            .map_err(AppError::from)?;

        if let Err(e) = InventoryLogRepo::create(
            pool,
            &CreateInventoryLog {
                pipe_type: pipe_type.into(),
                pipe_id,
                change_type: "location_transfer".into(),
                ref_type: None,
                ref_id: None,
                from_location_id,
                to_location_id: Some(dto.to_location_id),
                notes: dto.notes.clone(),
                created_by: None,
            },
        )
        .await
        {
            tracing::error!(?e, "Failed to create inventory log for location_transfer");
        }

        Ok(serde_json::json!({
            "pipe_type": pipe_type,
            "pipe_id": pipe_id,
            "from_location_id": from_location_id,
            "to_location_id": dto.to_location_id,
            "to_location_code": to_location.full_code,
        }))
    }
}
