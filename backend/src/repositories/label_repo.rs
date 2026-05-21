use sqlx::SqlitePool;

use crate::models::quality::QualityCert;
use crate::models::screen_pipe::ScreenPipe;
use crate::models::seamless_pipe::SeamlessPipe;

pub struct LabelRepo;

impl LabelRepo {
    pub async fn find_seamless_pipe(
        pool: &SqlitePool,
        id: i64,
    ) -> Result<Option<SeamlessPipe>, sqlx::Error> {
        sqlx::query_as::<_, SeamlessPipe>(
            "SELECT id, pipe_number, batch_number, pipe_type, grade, od, wt, length, \
             weight_per_unit, end_type, coupling_type, coupling_od, coupling_length, \
             heat_number, serial_number, manufacturer, production_date, cert_number, \
             location_id, status, notes, created_at, updated_at, deleted_at \
             FROM seamless_pipes WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    pub async fn find_screen_pipe(
        pool: &SqlitePool,
        id: i64,
    ) -> Result<Option<ScreenPipe>, sqlx::Error> {
        sqlx::query_as::<_, ScreenPipe>(
            "SELECT id, pipe_number, batch_number, screen_type, slot_size, \
             filtration_grade, base_od, base_wt, base_grade, base_end_type, length, \
             weight_per_unit, heat_number, serial_number, manufacturer, production_date, \
             cert_number, location_id, status, notes, created_at, updated_at, deleted_at \
             FROM screen_pipes WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    pub async fn find_quality_cert(
        pool: &SqlitePool,
        id: i64,
    ) -> Result<Option<QualityCert>, sqlx::Error> {
        sqlx::query_as::<_, QualityCert>(
            "SELECT id, cert_number, pipe_type, pipe_id, cert_date, result, inspector, \
             inspection_body, notes, created_at, updated_at, deleted_at \
             FROM quality_certs WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }
}
