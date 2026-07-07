use crate::cache_invalidator::CacheInvalidate;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::dto::common::PaginationParams;
use crate::dto::pipe_dto::{
    BatchCreatePipeRequest, CreateScreenPipeRequest, CreateSeamlessPipeRequest, PipeFilterParams,
    PipeSearchResult, UpdateScreenPipeRequest, UpdateSeamlessPipeRequest,
};
use crate::error::AppError;
use crate::models::screen_pipe::ScreenPipe;
use crate::models::seamless_pipe::SeamlessPipe;
use crate::domain::pipe::PipeModel;
use crate::repositories::generic_pipe_repo::GenericPipeRepo;
use crate::repositories::inventory_repo::InventoryRepo;

/// Pipe master-data service — CRUD and search for seamless and screen pipes.
/// Kicks off with pipe-number uniqueness checks and enforces soft-delete / status gates on mutations.
pub struct PipeService;

impl PipeService {
    fn generate_pipe_number(prefix: &str, grade: &str, od: f64, wt: f64) -> String {
        let serial = Uuid::new_v4().to_string();
        let short_serial = &serial[..8];
        format!("{}-{}-{}x{}-{}", prefix, grade, od, wt, short_serial)
    }

    async fn validate_pipe_number_unique(
        pool: &SqlitePool,
        pipe_number: &str,
    ) -> Result<(), AppError> {
        if !InventoryRepo::check_pipe_number_unique(pool, pipe_number).await.map_err(AppError::from)? {
            return Err(AppError::PipeNumberDuplicate(format!(
                "Pipe number '{}' already exists",
                pipe_number
            )));
        }
        Ok(())
    }

    pub async fn create_seamless_pipe<C: CacheInvalidate>(
        pool: &SqlitePool,
        cache: &C,
        dto: &CreateSeamlessPipeRequest,
    ) -> Result<SeamlessPipe, AppError> {
        let pipe_number = match &dto.pipe_number {
            Some(pn) if !pn.is_empty() => {
                Self::validate_pipe_number_unique(pool, pn).await?;
                pn.clone()
            }
            _ => {
                let mut pn = Self::generate_pipe_number("SP", &dto.grade, dto.od, dto.wt);
                while !InventoryRepo::check_pipe_number_unique(pool, &pn).await.map_err(AppError::from)? {
                    pn = Self::generate_pipe_number("SP", &dto.grade, dto.od, dto.wt);
                }
                pn
            }
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

        let pipe = GenericPipeRepo::<SeamlessPipe>::create(pool, &adjusted)
            .await
            .map_err(AppError::from)?;
        
        cache.invalidate_pipes()?;
        Ok(pipe)
    }

    /// Updates seamless pipe fields.
    /// Nopes out on soft-deleted records.
    ///
    /// # Errors
    /// - `AppError::PipeNotFound` — pipe ID not found or has been soft-deleted
    pub async fn update_seamless_pipe<C: CacheInvalidate>(
        pool: &SqlitePool,
        cache: &C,
        id: i64,
        dto: &UpdateSeamlessPipeRequest,
    ) -> Result<SeamlessPipe, AppError> {
        let existing = GenericPipeRepo::<SeamlessPipe>::find_by_id(pool, id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::PipeNotFound(format!("Seamless pipe id={} not found", id)))?;

        if existing.is_deleted() {
            return Err(AppError::PipeNotFound(format!(
                "Seamless pipe id={} has been deleted",
                id
            )));
        }

        let pipe = GenericPipeRepo::<SeamlessPipe>::update(pool, id, dto)
            .await
            .map_err(AppError::from)?;
        
        cache.invalidate_pipes()?;
        Ok(pipe)
    }

    /// Soft-deletes a seamless pipe. Only pipes with `in_stock` status get the axe.
    ///
    /// # Errors
    /// - `AppError::PipeNotFound` — ID doesn't exist
    /// - `AppError::PipeStatusConflict` — current status says nope
    pub async fn delete_seamless_pipe<C: CacheInvalidate>(pool: &SqlitePool, cache: &C, id: i64) -> Result<(), AppError> {
        let existing = GenericPipeRepo::<SeamlessPipe>::find_by_id(pool, id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::PipeNotFound(format!("Seamless pipe id={} not found", id)))?;

        if existing.status() != "in_stock" {
            return Err(AppError::PipeStatusConflict(format!(
                "Cannot delete pipe with status '{}'. Only 'in_stock' pipes can be deleted.",
                existing.status()
            )));
        }

        GenericPipeRepo::<SeamlessPipe>::delete(pool, id)
            .await
            .map_err(AppError::from)?;
        
        cache.invalidate_pipes()?;
        Ok(())
    }

    /// Grabs a single seamless pipe by ID.
    ///
    /// # Errors
    /// - `AppError::PipeNotFound` — pipe with the given ID not found
    pub async fn get_seamless_pipe(pool: &SqlitePool, id: i64) -> Result<SeamlessPipe, AppError> {
        GenericPipeRepo::<SeamlessPipe>::find_by_id(pool, id)
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
        GenericPipeRepo::<SeamlessPipe>::list(pool, filter, params)
            .await
            .map_err(AppError::from)
    }

    // ━━━ Screen Pipe ━━━

    pub async fn create_screen_pipe<C: CacheInvalidate>(
        pool: &SqlitePool,
        cache: &C,
        dto: &CreateScreenPipeRequest,
    ) -> Result<ScreenPipe, AppError> {
        let pipe_number = match &dto.pipe_number {
            Some(pn) if !pn.is_empty() => {
                Self::validate_pipe_number_unique(pool, pn).await?;
                pn.clone()
            }
            _ => {
                let mut pn = Self::generate_pipe_number("SCP", &dto.base_grade, dto.base_od, dto.base_wt);
                while !InventoryRepo::check_pipe_number_unique(pool, &pn).await.map_err(AppError::from)? {
                    pn = Self::generate_pipe_number("SCP", &dto.base_grade, dto.base_od, dto.base_wt);
                }
                pn
            }
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

        let pipe = GenericPipeRepo::<ScreenPipe>::create(pool, &adjusted)
            .await
            .map_err(AppError::from)?;
        
        cache.invalidate_pipes()?;
        Ok(pipe)
    }

    /// Updates screen pipe fields. Won't touch soft-deleted records.
    ///
    /// # Errors
    /// - `AppError::PipeNotFound` — ID not found or already deleted
    pub async fn update_screen_pipe<C: CacheInvalidate>(
        pool: &SqlitePool,
        cache: &C,
        id: i64,
        dto: &UpdateScreenPipeRequest,
    ) -> Result<ScreenPipe, AppError> {
        let existing = GenericPipeRepo::<ScreenPipe>::find_by_id(pool, id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::PipeNotFound(format!("Screen pipe id={} not found", id)))?;

        if existing.is_deleted() {
            return Err(AppError::PipeNotFound(format!(
                "Screen pipe id={} has been deleted",
                id
            )));
        }

        let pipe = GenericPipeRepo::<ScreenPipe>::update(pool, id, dto)
            .await
            .map_err(AppError::from)?;
        
        cache.invalidate_pipes()?;
        Ok(pipe)
    }

    /// Soft-deletes a screen pipe. Only `in_stock` pipes are fair game.
    ///
    /// # Errors
    /// - `AppError::PipeNotFound` — ID doesn't exist
    /// - `AppError::PipeStatusConflict` — status won't allow it
    pub async fn delete_screen_pipe<C: CacheInvalidate>(pool: &SqlitePool, cache: &C, id: i64) -> Result<(), AppError> {
        let existing = GenericPipeRepo::<ScreenPipe>::find_by_id(pool, id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::PipeNotFound(format!("Screen pipe id={} not found", id)))?;

        if existing.status() != "in_stock" {
            return Err(AppError::PipeStatusConflict(format!(
                "Cannot delete pipe with status '{}'. Only 'in_stock' pipes can be deleted.",
                existing.status()
            )));
        }

        GenericPipeRepo::<ScreenPipe>::delete(pool, id)
            .await
            .map_err(AppError::from)?;
        
        cache.invalidate_pipes()?;
        Ok(())
    }

    /// Gets a single screen pipe by ID.
    ///
    /// # Errors
    /// - `AppError::PipeNotFound` — ID doesn't exist
    pub async fn get_screen_pipe(pool: &SqlitePool, id: i64) -> Result<ScreenPipe, AppError> {
        GenericPipeRepo::<ScreenPipe>::find_by_id(pool, id)
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
        GenericPipeRepo::<ScreenPipe>::list(pool, filter, params)
            .await
            .map_err(AppError::from)
    }

    pub async fn batch_create_pipes<C: CacheInvalidate>(
        pool: &SqlitePool,
        cache: &C,
        dto: &BatchCreatePipeRequest,
    ) -> Result<Vec<i64>, AppError> {
        let mut pipe_ids = Vec::with_capacity(dto.quantity as usize);

        for _ in 0..dto.quantity {
            let prefix = match dto.pipe_type.as_str() {
                "seamless" | "casing" | "tubing" => "SP",
                "screen" => "SCP",
                _ => "P",
            };

            let mut pipe_number = Self::generate_pipe_number(
                prefix,
                &dto.grade,
                dto.od,
                dto.wt,
            );
            while !InventoryRepo::check_pipe_number_unique(pool, &pipe_number).await.map_err(AppError::from)? {
                pipe_number = Self::generate_pipe_number(prefix, &dto.grade, dto.od, dto.wt);
            }

            match dto.pipe_type.as_str() {
                "seamless" | "casing" | "tubing" => {
                    let req = CreateSeamlessPipeRequest {
                        pipe_number: Some(pipe_number),
                        batch_number: dto.batch_number.clone(),
                        pipe_type: Some(dto.pipe_type.clone()),
                        grade: dto.grade.clone(),
                        od: dto.od,
                        wt: dto.wt,
                        length: dto.length,
                        weight_per_unit: None,
                        end_type: dto.end_type.clone(),
                        coupling_type: None,
                        coupling_od: None,
                        coupling_length: None,
                        heat_number: dto.heat_number.clone(),
                        serial_number: None,
                        manufacturer: dto.manufacturer.clone(),
                        production_date: None,
                        cert_number: None,
                        notes: dto.notes.clone(),
                    };
                    let pipe = Self::create_seamless_pipe(pool, cache, &req).await?;
                    pipe_ids.push(pipe.id);
                }
                "screen" => {
                    let req = CreateScreenPipeRequest {
                        pipe_number: Some(pipe_number),
                        batch_number: dto.batch_number.clone(),
                        screen_type: None,
                        slot_size: None,
                        filtration_grade: None,
                        base_od: dto.od,
                        base_wt: dto.wt,
                        base_grade: dto.grade.clone(),
                        base_end_type: dto.end_type.clone(),
                        length: dto.length,
                        weight_per_unit: None,
                        heat_number: dto.heat_number.clone(),
                        serial_number: None,
                        manufacturer: dto.manufacturer.clone(),
                        production_date: None,
                        cert_number: None,
                        notes: dto.notes.clone(),
                    };
                    let pipe = Self::create_screen_pipe(pool, cache, &req).await?;
                    pipe_ids.push(pipe.id);
                }
                _ => {
                    return Err(AppError::Validation(format!(
                        "Unknown pipe_type: {}",
                        dto.pipe_type
                    )));
                }
            }
        }

        Ok(pipe_ids)
    }

        // ━━━ Search ━━━

        pub(crate) async fn search_pipes_generic(
            pool: &SqlitePool,
            query: &str,
            marker: PipeMarker,
        ) -> Result<Vec<PipeSearchResult>, AppError> {
            let seamless = GenericPipeRepo::<SeamlessPipe>::search(pool, query)
                .await
                .map_err(AppError::from)?;
            
            let screen = GenericPipeRepo::<ScreenPipe>::search(pool, query)
                .await
                .map_err(AppError::from)?;
            
            let mut results = Vec::new();
            
            if let PipeMarker::Seamless | PipeMarker::All = marker {
                for pipe in seamless {
                    results.push(PipeSearchResult {
                        id: pipe.id,
                        pipe_type: "seamless".into(),
                        pipe_number: pipe.pipe_number,
                        grade: pipe.grade,
                        od: pipe.od,
                        wt: pipe.wt,
                        status: pipe.status,
                        location_id: pipe.location_id,
                    });
                }
            }
            
            if let PipeMarker::Screen | PipeMarker::All = marker {
                for pipe in screen {
                    results.push(PipeSearchResult {
                        id: pipe.id,
                        pipe_type: "screen".into(),
                        pipe_number: pipe.pipe_number,
                        grade: pipe.base_grade,
                        od: pipe.base_od,
                        wt: pipe.base_wt,
                        status: pipe.status,
                        location_id: pipe.location_id,
                    });
                }
            }
            
            Ok(results)
        }

        /// Searches across both pipe types and combines the results.
        ///
        /// Each hit is tagged `pipe_type: "seamless"` or `"screen"`.
        pub async fn search_pipes(
            pool: &SqlitePool,
            query: &str,
        ) -> Result<Vec<PipeSearchResult>, AppError> {
            Self::search_pipes_generic(pool, query, PipeMarker::All).await
        }

        // ━━━ Generic Pipe CRUD ━━━

        pub async fn list_pipes<P>(
            pool: &SqlitePool,
            filter: &PipeFilterParams,
            params: &PaginationParams,
        ) -> Result<(Vec<P>, u64), AppError>
        where
            P: PipeModel + Send + Sync + 'static,
        {
            GenericPipeRepo::<P>::list(pool, filter, params)
                .await
                .map_err(AppError::from)
        }

        pub async fn create_pipe<P, C: CacheInvalidate>(
            pool: &SqlitePool,
            cache: &C,
            dto: &P::CreateDto,
        ) -> Result<P, AppError>
        where
            P: PipeModel + Send + Sync + 'static,
        {
            let pipe = GenericPipeRepo::<P>::create(pool, dto)
                .await
                .map_err(AppError::from)?;
            cache.invalidate_pipes()?;
            Ok(pipe)
        }

        pub async fn get_pipe<P>(
            pool: &SqlitePool,
            id: i64,
        ) -> Result<P, AppError>
        where
            P: PipeModel + Send + Sync + 'static,
        {
            GenericPipeRepo::<P>::find_by_id(pool, id)
                .await
                .map_err(AppError::from)?
                .ok_or_else(|| AppError::PipeNotFound(format!(
                    "Pipe id={} not found", id
                )))
        }

        pub async fn update_pipe<P, C: CacheInvalidate>(
            pool: &SqlitePool,
            cache: &C,
            id: i64,
            dto: &P::UpdateDto,
        ) -> Result<P, AppError>
        where
            P: PipeModel + Send + Sync + 'static,
        {
            let existing = GenericPipeRepo::<P>::find_by_id(pool, id)
                .await
                .map_err(AppError::from)?
                .ok_or_else(|| AppError::PipeNotFound(format!(
                    "Pipe id={} not found", id
                )))?;

            if existing.is_deleted() {
                return Err(AppError::PipeNotFound(format!(
                    "Pipe id={} has been deleted", id
                )));
            }

            let pipe = GenericPipeRepo::<P>::update(pool, id, dto)
                .await
                .map_err(AppError::from)?;

            cache.invalidate_pipes()?;
            Ok(pipe)
        }

        pub async fn delete_pipe<P, C: CacheInvalidate>(
            pool: &SqlitePool,
            cache: &C,
            id: i64,
        ) -> Result<(), AppError>
        where
            P: PipeModel + Send + Sync + 'static,
        {
            let existing = GenericPipeRepo::<P>::find_by_id(pool, id)
                .await
                .map_err(AppError::from)?
                .ok_or_else(|| AppError::PipeNotFound(format!(
                    "Pipe id={} not found", id
                )))?;

            if existing.is_deleted() {
                return Err(AppError::PipeNotFound(format!(
                    "Pipe id={} has been deleted", id
                )));
            }

            GenericPipeRepo::<P>::delete(pool, id)
                .await
                .map_err(AppError::from)?;

            cache.invalidate_pipes()?;
            Ok(())
        }
    }

// Helper enum to specify which pipe types to include in search
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PipeMarker {
    /// Include only seamless pipes
    Seamless,
    /// Include only screen pipes
    Screen,
    /// Include all pipe types
    All,
}

impl From<String> for PipeMarker {
    fn from(s: String) -> Self {
        match s.as_str() {
            "seamless" => PipeMarker::Seamless,
            "screen" => PipeMarker::Screen,
            _ => PipeMarker::All,
        }
    }
}
