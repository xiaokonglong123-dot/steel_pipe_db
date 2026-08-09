use crate::cache::CacheManager;
use sqlx::SqlitePool;

use crate::dto::common::PaginationParams;
use crate::dto::inventory_dto::{CreateLocationRequest, UpdateLocationRequest};
use crate::error::AppError;
use crate::models::inventory::Location;
use crate::inventory::location_repo::LocationRepo;

/// Location service — CRUD for warehouse locations.
/// Location codes follow the `zone-shelf-level` format and are globally unique.
pub struct LocationService;

impl LocationService {
    fn build_full_code(zone: &str, shelf: &str, level: &str) -> String {
        format!("{}-{}-{}", zone, shelf, level)
    }

    /// Creates a location. Auto-concatenates `zone-shelf-level` as the full code; rejects duplicates.
    pub async fn create_location(
        pool: &SqlitePool,
        cache: &CacheManager,
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

        let location = LocationRepo::create(pool, dto, &full_code)
            .await
            .map_err(AppError::from)?;

        cache.invalidate_locations().await;
        Ok(location)
    }

    /// Updates location info. Won't touch soft-deleted locations.
    pub async fn update_location(
        pool: &SqlitePool,
        cache: &CacheManager,
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

        let location = LocationRepo::update(pool, id, dto)
            .await
            .map_err(AppError::from)?;

        cache.invalidate_locations().await;
        Ok(location)
    }

    /// Paginated location list. Pass `active_only=true` to get only active locations.
    pub async fn list_locations(
        pool: &SqlitePool,
        params: &PaginationParams,
        active_only: bool,
    ) -> Result<(Vec<Location>, u64), AppError> {
        LocationRepo::list(pool, params, active_only)
            .await
            .map_err(AppError::from)
    }

    /// Gets a single location by ID.
    pub async fn get_location(pool: &SqlitePool, id: i64) -> Result<Location, AppError> {
        LocationRepo::find_by_id(pool, id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::LocationNotFound(format!("Location id={} not found", id)))
    }

    /// Soft-deletes a location. Only allowed when no stock is recorded there (`used_count == 0`).
    pub async fn delete_location(pool: &SqlitePool, cache: &CacheManager, id: i64) -> Result<(), AppError> {
        let existing = LocationRepo::find_by_id(pool, id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::LocationNotFound(format!("Location id={} not found", id)))?;

        if existing.used_count > 0 {
            return Err(AppError::Validation(format!(
                "Cannot delete location id={} with {} units still stored",
                id, existing.used_count
            )));
        }

        LocationRepo::delete(pool, id).await.map_err(AppError::from)?;
        cache.invalidate_locations().await;
        Ok(())
    }
}
