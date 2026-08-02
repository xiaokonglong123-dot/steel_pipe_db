use chrono::Utc;
use sqlx::{Executor, QueryBuilder, Postgres, PgPool};

use crate::domain::pipe::PipeType;

/// Helper struct for inserting into `inventory_logs` — not a DB model.
#[derive(Debug, Clone)]
pub struct CreateInventoryLog {
    pub pipe_type: String,
    pub pipe_id: i64,
    pub change_type: String,
    pub ref_type: Option<String>,
    pub ref_id: Option<i64>,
    pub from_location_id: Option<i64>,
    pub to_location_id: Option<i64>,
    pub notes: Option<String>,
    pub created_by: Option<i64>,
}

/// Helper struct for seeding inventory check items — not a DB model.
#[derive(Debug, Clone)]
pub struct CheckInitItem {
    pub pipe_type: String,
    pub pipe_id: i64,
    pub expected_status: String,
}

/// Count of pipes grouped by steel grade.
#[derive(Debug, serde::Serialize)]
pub struct GradeCount {
    pub grade: String,
    pub count: i64,
    pub pipe_type: String,
}

/// Count of pipes grouped by location.
#[derive(Debug, serde::Serialize)]
pub struct LocationCount {
    pub location_id: Option<i64>,
    pub count: i64,
    pub pipe_type: String,
}

/// ATP (Available-to-Promise) queries across `seamless_pipes` and `screen_pipes`.
pub struct InventoryRepo;

impl InventoryRepo {
    /// UNION query across both `seamless_pipes` and `screen_pipes` to compute available-to-promise
    /// stock grouped by `pipe_type`, `grade`, and `location_id`. Supports optional filters.
    ///
    /// Accepts any SQLx executor (`&PgPool`, `&mut Transaction`, `&mut Connection`), making it
    /// safe to use inside an `IMMEDIATE` transaction for TOCTOU-free ATP checks.
    pub async fn find_atp<'e, E: Executor<'e, Database = Postgres>>(
        executor: E,
        pipe_type: &Option<String>,
        grade: &Option<String>,
        location_id: &Option<i64>,
    ) -> Result<Vec<(String, String, i64, Option<i64>)>, sqlx::Error> {
        let mut builder: QueryBuilder<Postgres> = QueryBuilder::new(
            "SELECT pipe_type, grade, SUM(cnt)::bigint as quantity, location_id FROM ( \
             SELECT pipe_type, grade, COUNT(*) as cnt, location_id \
             FROM seamless_pipes WHERE status = 'in_stock' AND deleted_at IS NULL",
        );

        if let Some(ref pt) = pipe_type {
            builder.push(" AND pipe_type = ");
            builder.push_bind(pt);
        }
        if let Some(ref g) = grade {
            builder.push(" AND grade = ");
            builder.push_bind(g);
        }
        if let Some(loc) = location_id {
            builder.push(" AND location_id = ");
            builder.push_bind(loc);
        }

        builder.push(
            " GROUP BY pipe_type, grade, location_id \
             UNION ALL \
             SELECT screen_type as pipe_type, base_grade as grade, COUNT(*) as cnt, location_id \
             FROM screen_pipes WHERE status = 'in_stock' AND deleted_at IS NULL",
        );

        if let Some(ref pt) = pipe_type {
            builder.push(" AND screen_type = ");
            builder.push_bind(pt);
        }
        if let Some(ref g) = grade {
            builder.push(" AND base_grade = ");
            builder.push_bind(g);
        }
        if let Some(loc) = location_id {
            builder.push(" AND location_id = ");
            builder.push_bind(loc);
        }

        builder.push(
            " GROUP BY screen_type, base_grade, location_id \
             ) GROUP BY pipe_type, grade, location_id ORDER BY pipe_type, grade",
        );

        builder
            .build_query_as::<(String, String, i64, Option<i64>)>()
            .fetch_all(executor)
            .await
    }

    /// Sums `COUNT(*)` of `in_stock` pipes from `seamless_pipes`, `screen_pipes`, and `welded_pipes`.
    pub async fn get_total_in_stock(pool: &PgPool) -> Result<i64, sqlx::Error> {
        let (seamless,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM seamless_pipes WHERE status = 'in_stock' AND deleted_at IS NULL",
        )
        .fetch_one(pool)
        .await?;

        let (screen,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM screen_pipes WHERE status = 'in_stock' AND deleted_at IS NULL",
        )
        .fetch_one(pool)
        .await?;

        let (welded,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM welded_pipes WHERE status = 'in_stock' AND deleted_at IS NULL",
        )
        .fetch_one(pool)
        .await?;

        Ok(seamless + screen + welded)
    }

    /// GROUP BY `grade`/`base_grade` across both pipe tables. Returns typed structs.
    pub async fn get_count_by_grade(pool: &PgPool) -> Result<Vec<GradeCount>, sqlx::Error> {
        let seamless: Vec<(String, i64)> = sqlx::query_as(
            "SELECT grade, COUNT(*) as cnt FROM seamless_pipes \
             WHERE status = 'in_stock' AND deleted_at IS NULL GROUP BY grade ORDER BY grade",
        )
        .fetch_all(pool)
        .await?;

        let screen: Vec<(String, i64)> = sqlx::query_as(
            "SELECT base_grade as grade, COUNT(*) as cnt FROM screen_pipes \
             WHERE status = 'in_stock' AND deleted_at IS NULL GROUP BY base_grade ORDER BY base_grade",
        )
        .fetch_all(pool)
        .await?;

        let welded: Vec<(String, i64)> = sqlx::query_as(
            "SELECT grade, COUNT(*) as cnt FROM welded_pipes \
             WHERE status = 'in_stock' AND deleted_at IS NULL GROUP BY grade ORDER BY grade",
        )
        .fetch_all(pool)
        .await?;

        let mut result = Vec::new();
        for (grade, count) in seamless {
            result.push(GradeCount {
                grade,
                count,
                pipe_type: "seamless".to_string(),
            });
        }
        for (grade, count) in screen {
            result.push(GradeCount {
                grade,
                count,
                pipe_type: "screen".to_string(),
            });
        }
        for (grade, count) in welded {
            result.push(GradeCount {
                grade,
                count,
                pipe_type: "welded".to_string(),
            });
        }
        Ok(result)
    }

    /// GROUP BY `location_id` across both pipe tables. Returns typed structs.
    pub async fn get_count_by_location(
        pool: &PgPool,
    ) -> Result<Vec<LocationCount>, sqlx::Error> {
        let seamless: Vec<(Option<i64>, i64)> = sqlx::query_as(
            "SELECT location_id, COUNT(*) as cnt FROM seamless_pipes \
             WHERE status = 'in_stock' AND deleted_at IS NULL GROUP BY location_id",
        )
        .fetch_all(pool)
        .await?;

        let screen: Vec<(Option<i64>, i64)> = sqlx::query_as(
            "SELECT location_id, COUNT(*) as cnt FROM screen_pipes \
             WHERE status = 'in_stock' AND deleted_at IS NULL GROUP BY location_id",
        )
        .fetch_all(pool)
        .await?;

        let welded: Vec<(Option<i64>, i64)> = sqlx::query_as(
            "SELECT location_id, COUNT(*) as cnt FROM welded_pipes \
             WHERE status = 'in_stock' AND deleted_at IS NULL GROUP BY location_id",
        )
        .fetch_all(pool)
        .await?;

        let mut result = Vec::new();
        for (location_id, count) in seamless {
            result.push(LocationCount {
                location_id,
                count,
                pipe_type: "seamless".to_string(),
            });
        }
        for (location_id, count) in screen {
            result.push(LocationCount {
                location_id,
                count,
                pipe_type: "screen".to_string(),
            });
        }
        for (location_id, count) in welded {
            result.push(LocationCount {
                location_id,
                count,
                pipe_type: "welded".to_string(),
            });
        }
        Ok(result)
    }

    /// Updates `location_id` on either `seamless_pipes` or `screen_pipes` depending on
    /// `pipe_type`. No-op if `pipe_type` is neither seamless nor screen.
    pub async fn update_pipe_location(
        pool: &PgPool,
        pipe_type: &str,
        pipe_id: i64,
        location_id: i64,
    ) -> Result<(), sqlx::Error> {
        match PipeType::from_pipe_type_str(pipe_type) {
            Some(PipeType::Seamless) => {
                sqlx::query(
                    "UPDATE seamless_pipes SET location_id = $1, updated_at = NOW() \
                     WHERE id = $2 AND deleted_at IS NULL",
                )
                .bind(location_id)
                .bind(pipe_id)
                .execute(pool)
                .await?;
            }
            Some(PipeType::Screen) => {
                sqlx::query(
                    "UPDATE screen_pipes SET location_id = $1, updated_at = NOW() \
                     WHERE id = $2 AND deleted_at IS NULL",
                )
                .bind(location_id)
                .bind(pipe_id)
                .execute(pool)
                .await?;
            }
            Some(PipeType::Welded) => {
                sqlx::query(
                    "UPDATE welded_pipes SET location_id = $1, updated_at = NOW() \
                     WHERE id = $2 AND deleted_at IS NULL",
                )
                .bind(location_id)
                .bind(pipe_id)
                .execute(pool)
                .await?;
            }
            None => {}
        }
        Ok(())
    }

    /// Updates pipe status with a stock guard (`AND status = 'in_stock'`).
    /// Used by outbound operations to prevent double-deduction.
    /// Returns rows affected (0 means pipe was not in_stock).
    /// Accepts any SQLx executor, safe to use inside transactions.
    pub async fn update_pipe_status_with_stock_check<'e, E: Executor<'e, Database = Postgres>>(
        executor: E,
        pipe_type: &str,
        pipe_id: i64,
        new_status: &str,
    ) -> Result<u64, sqlx::Error> {
        let now = Utc::now();
        let (table, _) = match PipeType::from_pipe_type_str(pipe_type) {
            Some(PipeType::Seamless) => ("seamless_pipes", "status"),
            Some(PipeType::Screen) => ("screen_pipes", "status"),
            Some(PipeType::Welded) => ("welded_pipes", "status"),
            None => return Ok(0),
        };
        // Safety: table name is validated above via PipeType enum
        let query = format!(
            "UPDATE {} SET status = $1, updated_at = $2 \
             WHERE id = $3 AND deleted_at IS NULL AND status = 'in_stock'",
            table
        );
        let result = sqlx::query(&query)
            .bind(new_status)
            .bind(&now)
            .bind(pipe_id)
            .execute(executor)
            .await?;
        Ok(result.rows_affected())
    }

    /// Returns `status` for a given pipe (seamless, screen, or welded). Returns `None` if not found.
    pub async fn get_pipe_status<'e, E: Executor<'e, Database = Postgres>>(
        executor: E,
        pipe_type: &str,
        pipe_id: i64,
    ) -> Result<Option<String>, sqlx::Error> {
        match PipeType::from_pipe_type_str(pipe_type) {
            Some(PipeType::Seamless) => {
                let row: Option<(String,)> = sqlx::query_as(
                    "SELECT status FROM seamless_pipes WHERE id = $1 AND deleted_at IS NULL",
                )
                .bind(pipe_id)
                .fetch_optional(executor)
                .await?;
                Ok(row.map(|r| r.0))
            }
            Some(PipeType::Screen) => {
                let row: Option<(String,)> = sqlx::query_as(
                    "SELECT status FROM screen_pipes WHERE id = $1 AND deleted_at IS NULL",
                )
                .bind(pipe_id)
                .fetch_optional(executor)
                .await?;
                Ok(row.map(|r| r.0))
            }
            Some(PipeType::Welded) => {
                let row: Option<(String,)> = sqlx::query_as(
                    "SELECT status FROM welded_pipes WHERE id = $1 AND deleted_at IS NULL",
                )
                .bind(pipe_id)
                .fetch_optional(executor)
                .await?;
                Ok(row.map(|r| r.0))
            }
            None => Ok(None),
        }
    }

    /// Returns IDs of all `in_stock` pipes (seamless, screen, and welded).
    /// Returns a tuple `(seamless_ids, screen_ids, welded_ids)`.
    /// When `location_id` is `Some`, only pipes at that location are returned.
    pub async fn find_in_stock_pipe_ids(
        pool: &PgPool,
        location_id: Option<i64>,
    ) -> Result<(Vec<i64>, Vec<i64>, Vec<i64>), sqlx::Error> {
        let seamless: Vec<(i64,)> = sqlx::query_as(
            "SELECT id FROM seamless_pipes \
             WHERE status = 'in_stock' AND deleted_at IS NULL \
             AND ($1 IS NULL OR location_id = $2)",
        )
        .bind(location_id)
        .bind(location_id)
        .fetch_all(pool)
        .await?;

        let screen: Vec<(i64,)> = sqlx::query_as(
            "SELECT id FROM screen_pipes \
             WHERE status = 'in_stock' AND deleted_at IS NULL \
             AND ($1 IS NULL OR location_id = $2)",
        )
        .bind(location_id)
        .bind(location_id)
        .fetch_all(pool)
        .await?;

        let welded: Vec<(i64,)> = sqlx::query_as(
            "SELECT id FROM welded_pipes \
             WHERE status = 'in_stock' AND deleted_at IS NULL \
             AND ($1 IS NULL OR location_id = $2)",
        )
        .bind(location_id)
        .bind(location_id)
        .fetch_all(pool)
        .await?;

        let seamless_ids: Vec<i64> = seamless.into_iter().map(|(id,)| id).collect();
        let screen_ids: Vec<i64> = screen.into_iter().map(|(id,)| id).collect();
        let welded_ids: Vec<i64> = welded.into_iter().map(|(id,)| id).collect();
        Ok((seamless_ids, screen_ids, welded_ids))
    }

    /// Updates pipe status WITHOUT a stock guard.
    /// Used by inbound operations where pipes can be in any valid pre-inbound status.
    /// Returns rows affected (0 means pipe was not found or was deleted).
    /// Accepts any SQLx executor, safe to use inside transactions.
    pub async fn update_pipe_status<'e, E: Executor<'e, Database = Postgres>>(
        executor: E,
        pipe_type: &str,
        pipe_id: i64,
        new_status: &str,
    ) -> Result<u64, sqlx::Error> {
        let now = Utc::now();
        let table = match PipeType::from_pipe_type_str(pipe_type) {
            Some(PipeType::Seamless) => "seamless_pipes",
            Some(PipeType::Screen) => "screen_pipes",
            Some(PipeType::Welded) => "welded_pipes",
            None => return Ok(0),
        };
        // Safety: table name is validated above via PipeType enum
        let query = format!(
            "UPDATE {} SET status = $1, updated_at = $2 \
             WHERE id = $3 AND deleted_at IS NULL",
            table
        );
        let result = sqlx::query(&query)
            .bind(new_status)
            .bind(&now)
            .bind(pipe_id)
            .execute(executor)
            .await?;
        Ok(result.rows_affected())
    }

    /// Checks whether a pipe number is unique across seamless, screen, and welded pipes.
    /// Returns `true` if the pipe number does NOT exist in any table (i.e., it's unique).
    pub async fn check_pipe_number_unique(
        pool: &PgPool,
        pipe_number: &str,
    ) -> Result<bool, sqlx::Error> {
        let row: Option<(i64,)> = sqlx::query_as(
            "SELECT id FROM (
                SELECT id FROM seamless_pipes WHERE pipe_number = $1 AND deleted_at IS NULL
                UNION ALL
                SELECT id FROM screen_pipes WHERE pipe_number = $2 AND deleted_at IS NULL
                UNION ALL
                SELECT id FROM welded_pipes WHERE pipe_number = $3 AND deleted_at IS NULL
            ) LIMIT 1",
        )
        .bind(pipe_number)
        .bind(pipe_number)
        .bind(pipe_number)
        .fetch_optional(pool)
        .await?;

        Ok(row.is_none())
    }

    /// Returns `location_id` for a given pipe (seamless, screen, or welded). Returns `None` if not found.
    pub async fn get_pipe_location_id(
        pool: &PgPool,
        pipe_type: &str,
        pipe_id: i64,
    ) -> Result<Option<i64>, sqlx::Error> {
        match PipeType::from_pipe_type_str(pipe_type) {
            Some(PipeType::Seamless) => {
                let row: Option<(Option<i64>,)> = sqlx::query_as(
                    "SELECT location_id FROM seamless_pipes WHERE id = $1 AND deleted_at IS NULL",
                )
                .bind(pipe_id)
                .fetch_optional(pool)
                .await?;
                Ok(row.and_then(|r| r.0))
            }
            Some(PipeType::Screen) => {
                let row: Option<(Option<i64>,)> = sqlx::query_as(
                    "SELECT location_id FROM screen_pipes WHERE id = $1 AND deleted_at IS NULL",
                )
                .bind(pipe_id)
                .fetch_optional(pool)
                .await?;
                Ok(row.and_then(|r| r.0))
            }
            Some(PipeType::Welded) => {
                let row: Option<(Option<i64>,)> = sqlx::query_as(
                    "SELECT location_id FROM welded_pipes WHERE id = $1 AND deleted_at IS NULL",
                )
                .bind(pipe_id)
                .fetch_optional(pool)
                .await?;
                Ok(row.and_then(|r| r.0))
            }
            None => Ok(None),
        }
    }
}
