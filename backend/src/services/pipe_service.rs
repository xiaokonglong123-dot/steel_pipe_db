use sqlx::SqlitePool;
use uuid::Uuid;

use crate::dto::common::PaginationParams;
use crate::dto::pipe_dto::{
    CreateScreenPipeRequest, CreateSeamlessPipeRequest, PipeFilterParams, PipeSearchResult,
    UpdateScreenPipeRequest, UpdateSeamlessPipeRequest,
};
use crate::error::AppError;
use crate::models::screen_pipe::ScreenPipe;
use crate::models::seamless_pipe::SeamlessPipe;
use crate::repositories::pipe_repo::{ScreenPipeRepo, SeamlessPipeRepo};

/// Pipe master-data service — CRUD and search for seamless and screen pipes.
/// Kicks off with pipe-number uniqueness checks and enforces soft-delete / status gates on mutations.
pub struct PipeService;

impl PipeService {
    fn generate_pipe_number(prefix: &str, grade: &str, od: f64, wt: f64) -> String {
        let serial = Uuid::new_v4().to_string();
        let short_serial = &serial[..8];
        format!("{}-{}-{}x{}-{}", prefix, grade, od, wt, short_serial)
    }

    // ━━━ Seamless Pipe ━━━

    /// Creates a seamless pipe record.
    /// Auto-generates a pipe number if none is given; tells you to fuck off if the number's taken.
    ///
    /// # Errors
    /// - `AppError::PipeNumberDuplicate` — the submitted pipe number already exists
    pub async fn create_seamless_pipe(
        pool: &SqlitePool,
        dto: &CreateSeamlessPipeRequest,
    ) -> Result<SeamlessPipe, AppError> {
        let pipe_number = match &dto.pipe_number {
            Some(pn) if !pn.is_empty() => {
                if SeamlessPipeRepo::find_by_pipe_number(pool, pn)
                    .await
                    .map_err(AppError::from)?
                    .is_some()
                {
                    return Err(AppError::PipeNumberDuplicate(format!(
                        "Pipe number '{}' already exists",
                        pn
                    )));
                }
                pn.clone()
            }
            _ => Self::generate_pipe_number("SP", &dto.grade, dto.od, dto.wt),
        };

        let adjusted = CreateSeamlessPipeRequest {
            pipe_number: Some(pipe_number),
            batch_number: dto.batch_number.clone(),
            pipe_type: dto.pipe_type.clone(),
            grade: dto.grade.clone(),
            od: dto.od,
            wt: dto.wt,
            length: dto.length,
            weight_per_unit: dto.weight_per_unit,
            end_type: dto.end_type.clone(),
            coupling_type: dto.coupling_type.clone(),
            coupling_od: dto.coupling_od,
            coupling_length: dto.coupling_length,
            heat_number: dto.heat_number.clone(),
            serial_number: dto.serial_number.clone(),
            manufacturer: dto.manufacturer.clone(),
            production_date: dto.production_date.clone(),
            cert_number: dto.cert_number.clone(),
            notes: dto.notes.clone(),
        };

        SeamlessPipeRepo::create(pool, &adjusted)
            .await
            .map_err(AppError::from)
    }

    /// Updates seamless pipe fields.
    /// Nopes out on soft-deleted records.
    ///
    /// # Errors
    /// - `AppError::PipeNotFound` — pipe ID not found or has been soft-deleted
    pub async fn update_seamless_pipe(
        pool: &SqlitePool,
        id: i64,
        dto: &UpdateSeamlessPipeRequest,
    ) -> Result<SeamlessPipe, AppError> {
        let existing = SeamlessPipeRepo::find_by_id(pool, id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::PipeNotFound(format!("Seamless pipe id={} not found", id)))?;

        if existing.deleted_at.is_some() {
            return Err(AppError::PipeNotFound(format!(
                "Seamless pipe id={} has been deleted",
                id
            )));
        }

        SeamlessPipeRepo::update(pool, id, dto)
            .await
            .map_err(AppError::from)
    }

    /// Soft-deletes a seamless pipe. Only pipes with `in_stock` status get the axe.
    ///
    /// # Errors
    /// - `AppError::PipeNotFound` — ID doesn't exist
    /// - `AppError::PipeStatusConflict` — current status says nope
    pub async fn delete_seamless_pipe(
        pool: &SqlitePool,
        id: i64,
    ) -> Result<(), AppError> {
        let existing = SeamlessPipeRepo::find_by_id(pool, id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::PipeNotFound(format!("Seamless pipe id={} not found", id)))?;

        if existing.status != "in_stock" {
            return Err(AppError::PipeStatusConflict(format!(
                "Cannot delete pipe with status '{}'. Only 'in_stock' pipes can be deleted.",
                existing.status
            )));
        }

        SeamlessPipeRepo::delete(pool, id)
            .await
            .map_err(AppError::from)
    }

    /// Grabs a single seamless pipe by ID.
    ///
    /// # Errors
    /// - `AppError::PipeNotFound` — pipe with the given ID not found
    pub async fn get_seamless_pipe(
        pool: &SqlitePool,
        id: i64,
    ) -> Result<SeamlessPipe, AppError> {
        SeamlessPipeRepo::find_by_id(pool, id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::PipeNotFound(format!("Seamless pipe id={} not found", id)))
    }

    /// Paginated list of seamless pipes — filter by spec, grade, heat number, etc.
    /// Returns `(items, total_count)`.
    pub async fn list_seamless_pipes(
        pool: &SqlitePool,
        filter: &PipeFilterParams,
        params: &PaginationParams,
    ) -> Result<(Vec<SeamlessPipe>, u64), AppError> {
        SeamlessPipeRepo::list(pool, filter, params)
            .await
            .map_err(AppError::from)
    }

    // ━━━ Screen Pipe ━━━

    /// Creates a screen pipe. Auto-generates the pipe number or checks the submitted one is unique.
    ///
    /// # Errors
    /// - `AppError::PipeNumberDuplicate` — pipe number's already taken
    pub async fn create_screen_pipe(
        pool: &SqlitePool,
        dto: &CreateScreenPipeRequest,
    ) -> Result<ScreenPipe, AppError> {
        let pipe_number = match &dto.pipe_number {
            Some(pn) if !pn.is_empty() => {
                if ScreenPipeRepo::find_by_pipe_number(pool, pn)
                    .await
                    .map_err(AppError::from)?
                    .is_some()
                {
                    return Err(AppError::PipeNumberDuplicate(format!(
                        "Pipe number '{}' already exists",
                        pn
                    )));
                }
                pn.clone()
            }
            _ => Self::generate_pipe_number("SCP", &dto.base_grade, dto.base_od, dto.base_wt),
        };

        let adjusted = CreateScreenPipeRequest {
            pipe_number: Some(pipe_number),
            batch_number: dto.batch_number.clone(),
            screen_type: dto.screen_type.clone(),
            slot_size: dto.slot_size,
            filtration_grade: dto.filtration_grade.clone(),
            base_od: dto.base_od,
            base_wt: dto.base_wt,
            base_grade: dto.base_grade.clone(),
            base_end_type: dto.base_end_type.clone(),
            length: dto.length,
            weight_per_unit: dto.weight_per_unit,
            heat_number: dto.heat_number.clone(),
            serial_number: dto.serial_number.clone(),
            manufacturer: dto.manufacturer.clone(),
            production_date: dto.production_date.clone(),
            cert_number: dto.cert_number.clone(),
            notes: dto.notes.clone(),
        };

        ScreenPipeRepo::create(pool, &adjusted)
            .await
            .map_err(AppError::from)
    }

    /// Updates screen pipe fields. Won't touch soft-deleted records.
    ///
    /// # Errors
    /// - `AppError::PipeNotFound` — ID not found or already deleted
    pub async fn update_screen_pipe(
        pool: &SqlitePool,
        id: i64,
        dto: &UpdateScreenPipeRequest,
    ) -> Result<ScreenPipe, AppError> {
        let existing = ScreenPipeRepo::find_by_id(pool, id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::PipeNotFound(format!("Screen pipe id={} not found", id)))?;

        if existing.deleted_at.is_some() {
            return Err(AppError::PipeNotFound(format!(
                "Screen pipe id={} has been deleted",
                id
            )));
        }

        ScreenPipeRepo::update(pool, id, dto)
            .await
            .map_err(AppError::from)
    }

    /// Soft-deletes a screen pipe. Only `in_stock` pipes are fair game.
    ///
    /// # Errors
    /// - `AppError::PipeNotFound` — ID doesn't exist
    /// - `AppError::PipeStatusConflict` — status won't allow it
    pub async fn delete_screen_pipe(
        pool: &SqlitePool,
        id: i64,
    ) -> Result<(), AppError> {
        let existing = ScreenPipeRepo::find_by_id(pool, id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::PipeNotFound(format!("Screen pipe id={} not found", id)))?;

        if existing.status != "in_stock" {
            return Err(AppError::PipeStatusConflict(format!(
                "Cannot delete pipe with status '{}'. Only 'in_stock' pipes can be deleted.",
                existing.status
            )));
        }

        ScreenPipeRepo::delete(pool, id)
            .await
            .map_err(AppError::from)
    }

    /// Gets a single screen pipe by ID.
    ///
    /// # Errors
    /// - `AppError::PipeNotFound` — ID doesn't exist
    pub async fn get_screen_pipe(
        pool: &SqlitePool,
        id: i64,
    ) -> Result<ScreenPipe, AppError> {
        ScreenPipeRepo::find_by_id(pool, id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::PipeNotFound(format!("Screen pipe id={} not found", id)))
    }

    /// Paginated list of screen pipes — filter by spec, grade, whatever.
    /// Returns `(items, total_count)`.
    pub async fn list_screen_pipes(
        pool: &SqlitePool,
        filter: &PipeFilterParams,
        params: &PaginationParams,
    ) -> Result<(Vec<ScreenPipe>, u64), AppError> {
        ScreenPipeRepo::list(pool, filter, params)
            .await
            .map_err(AppError::from)
    }

    // ━━━ Search ━━━

    /// Searches across both pipe types and smashes the results together.
    /// Each hit is tagged `pipe_type: "seamless"` or `"screen"`.
    ///
    /// # Errors
    /// - `AppError::Internal` — JSON serialization failure for a matched pipe
    pub async fn search_pipes(
        pool: &SqlitePool,
        query: &str,
    ) -> Result<Vec<PipeSearchResult>, AppError> {
        let seamless = SeamlessPipeRepo::search(pool, query)
            .await
            .map_err(AppError::from)?;

        let screen = ScreenPipeRepo::search(pool, query)
            .await
            .map_err(AppError::from)?;

        let mut results = Vec::new();

        for pipe in seamless {
            results.push(PipeSearchResult {
                pipe_type: "seamless".into(),
                pipe: serde_json::to_value(pipe)
                    .map_err(|e| AppError::Internal(e.to_string()))?,
            });
        }

        for pipe in screen {
            results.push(PipeSearchResult {
                pipe_type: "screen".into(),
                pipe: serde_json::to_value(pipe)
                    .map_err(|e| AppError::Internal(e.to_string()))?,
            });
        }

        Ok(results)
    }
}
