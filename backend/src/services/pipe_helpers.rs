use chrono::Utc;
use sqlx::sqlite::Sqlite;
use sqlx::{SqlitePool, Transaction};

use crate::domain::pipe::PipeType;
use crate::error::AppError;

/// Common pipe operations — deduplicated helper functions for pipe status updates and validations.
pub struct PipeHelpers;

impl PipeHelpers {
    /// Validates that a pipe exists and is not soft-deleted.
    /// Returns the pipe's current status.
    pub async fn validate_pipe_exists(
        pool: &SqlitePool,
        pipe_type: &PipeType,
        pipe_id: i64,
    ) -> Result<String, AppError> {
        let table = match pipe_type {
            PipeType::Seamless => "seamless_pipes",
            PipeType::Screen => "screen_pipes",
        };

        let query = format!(
            "SELECT status FROM {} WHERE id = ? AND deleted_at IS NULL",
            table
        );

        let result: Option<(String,)> = sqlx::query_as(&query)
            .bind(pipe_id)
            .fetch_optional(pool)
            .await
            .map_err(AppError::from)?;

        match result {
            Some((status,)) => Ok(status),
            None => Err(AppError::NotFound(format!(
                "{:?} pipe id={} not found or has been deleted",
                pipe_type, pipe_id
            ))),
        }
    }

    /// Updates pipe status to the target status within a transaction.
    /// Returns the number of rows affected.
    pub async fn update_pipe_status(
        tx: &mut Transaction<'_, Sqlite>,
        pipe_type: &PipeType,
        pipe_id: i64,
        target_status: &str,
    ) -> Result<u64, AppError> {
        let table = match pipe_type {
            PipeType::Seamless => "seamless_pipes",
            PipeType::Screen => "screen_pipes",
        };

        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        let query = format!(
            "UPDATE {} SET status = ?, updated_at = ? WHERE id = ? AND deleted_at IS NULL",
            table
        );

        let result = sqlx::query(&query)
            .bind(target_status)
            .bind(&now)
            .bind(pipe_id)
            .execute(&mut **tx)
            .await
            .map_err(AppError::from)?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(format!(
                "{:?} pipe id={} not found during status update",
                pipe_type, pipe_id
            )));
        }

        Ok(result.rows_affected())
    }

    /// Creates an inventory log entry for a pipe status change.
    pub async fn create_inventory_log(
        tx: &mut Transaction<'_, Sqlite>,
        pipe_type: &str,
        pipe_id: i64,
        change_type: &str,
        ref_type: Option<&str>,
        ref_id: Option<i64>,
        from_location_id: Option<i64>,
        to_location_id: Option<i64>,
        notes: Option<&str>,
        created_by: Option<i64>,
    ) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO inventory_logs (pipe_type, pipe_id, change_type, ref_type, ref_id, \
             from_location_id, to_location_id, notes, created_by) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(pipe_type)
        .bind(pipe_id)
        .bind(change_type)
        .bind(ref_type)
        .bind(ref_id)
        .bind(from_location_id)
        .bind(to_location_id)
        .bind(notes)
        .bind(created_by)
        .execute(&mut **tx)
        .await
        .map_err(AppError::from)?;

        Ok(())
    }

    /// Validates pipe status transition is allowed.
    /// Returns Ok(()) if valid, Err if not.
    pub fn validate_status_transition(
        current_status: &str,
        target_status: &str,
    ) -> Result<(), AppError> {
        let valid_transitions = [
            // From "new" (just created, not yet in stock)
            ("new", "in_stock"),
            // From "in_stock" (in warehouse)
            ("in_stock", "outbound"),
            ("in_stock", "scrapped"),
            // From "outbound" (shipped out)
            ("outbound", "in_stock"),
            // From "scrapped" (end of life) - no transitions allowed
        ];

        let is_valid = valid_transitions
            .iter()
            .any(|(from, to)| *from == current_status && *to == target_status);

        if is_valid {
            Ok(())
        } else {
            Err(AppError::Validation(format!(
                "Invalid status transition from '{}' to '{}'",
                current_status, target_status
            )))
        }
    }

    /// Validates that a pipe is not already in the target status.
    pub fn validate_not_already_in_status(
        current_status: &str,
        target_status: &str,
    ) -> Result<(), AppError> {
        if current_status == target_status {
            Err(AppError::Validation(format!(
                "Pipe is already in '{}' status",
                target_status
            )))
        } else {
            Ok(())
        }
    }

    /// Checks if a pipe number already exists across both seamless and screen pipes.
    pub async fn check_pipe_number_unique(
        pool: &SqlitePool,
        pipe_number: &str,
    ) -> Result<bool, AppError> {
        let seamless_exists: Option<(i64,)> = sqlx::query_as(
            "SELECT id FROM seamless_pipes WHERE pipe_number = ? AND deleted_at IS NULL",
        )
        .bind(pipe_number)
        .fetch_optional(pool)
        .await
        .map_err(AppError::from)?;

        if seamless_exists.is_some() {
            return Ok(false);
        }

        let screen_exists: Option<(i64,)> = sqlx::query_as(
            "SELECT id FROM screen_pipes WHERE pipe_number = ? AND deleted_at IS NULL",
        )
        .bind(pipe_number)
        .fetch_optional(pool)
        .await
        .map_err(AppError::from)?;

        Ok(screen_exists.is_none())
    }

    /// Gets the pipe's current location ID.
    pub async fn get_pipe_location(
        pool: &SqlitePool,
        pipe_type: &PipeType,
        pipe_id: i64,
    ) -> Result<Option<i64>, AppError> {
        let table = match pipe_type {
            PipeType::Seamless => "seamless_pipes",
            PipeType::Screen => "screen_pipes",
        };

        let query = format!(
            "SELECT location_id FROM {} WHERE id = ? AND deleted_at IS NULL",
            table
        );

        let result: Option<(Option<i64>,)> = sqlx::query_as(&query)
            .bind(pipe_id)
            .fetch_optional(pool)
            .await
            .map_err(AppError::from)?;

        match result {
            Some((location_id,)) => Ok(location_id),
            None => Err(AppError::NotFound(format!(
                "{:?} pipe id={} not found",
                pipe_type, pipe_id
            ))),
        }
    }
}
