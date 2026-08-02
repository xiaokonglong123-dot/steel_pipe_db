use sqlx::PgPool;

use crate::error::AppError;
use crate::repositories::inbound_repo::InboundRepo;
use crate::repositories::inventory_log_repo::InventoryLogRepo;
use crate::repositories::inventory_repo::InventoryRepo;
use crate::repositories::outbound_repo::OutboundRepo;
use crate::repositories::pipe_repo::ScreenPipeRepo;
use crate::repositories::pipe_repo::SeamlessPipeRepo;

/// Trace service — full lifecycle tracking for pipes: inbound/outbound events by pipe ID,
/// pipe distribution by heat number, and related inventory records by order number.
pub struct TraceService;

impl TraceService {
    /// Trace a single pipe's full lifecycle — returns current pipe info and
    /// all inventory change logs (inbound/outbound/transfer) sorted by time ascending.
    ///
    /// # Errors
    /// - `AppError::NotFound` — pipe ID does not exist or was deleted
    /// - `AppError::Validation` — invalid pipe_type
    pub async fn trace_pipe_lifecycle(
        pool: &PgPool,
        pipe_type: &str,
        pipe_id: i64,
    ) -> Result<serde_json::Value, AppError> {
        let logs = InventoryLogRepo::find_by_pipe(pool, pipe_type, pipe_id)
            .await
            .map_err(AppError::from)?;

        // Cross-table pipe info query: assembles trace data from seamless_pipes or screen_pipes
        // with different column names (grade vs base_grade, od vs base_od) depending on pipe type.
        // Stays in service because it's a traceability concern combining two table schemas into one
        // JSON response structure — not a reusable CRUD operation.
        let pipe_info = match pipe_type {
            "seamless" | "casing" | "tubing" => {
                let row = sqlx::query_as::<_, (String, String, f64, f64, String, Option<i64>)>(
                    "SELECT pipe_number, grade, od, wt, status, location_id \
                     FROM seamless_pipes WHERE id = $1 AND deleted_at IS NULL",
                )
                .bind(pipe_id)
                .fetch_optional(pool)
                .await
                .map_err(AppError::from)?;
                match row {
                    Some((pn, grade, od, wt, status, loc)) => serde_json::json!({
                        "pipe_type": pipe_type,
                        "pipe_number": pn,
                        "grade": grade,
                        "od": od,
                        "wt": wt,
                        "current_status": status,
                        "current_location_id": loc,
                    }),
                    None => {
                        return Err(AppError::NotFound(format!(
                            "Pipe {} id={} not found",
                            pipe_type, pipe_id
                        )))
                    }
                }
            }
            "screen" | "screened" => {
                let row = sqlx::query_as::<_, (String, String, f64, f64, String, Option<i64>)>(
                    "SELECT pipe_number, base_grade, base_od, base_wt, status, location_id \
                     FROM screen_pipes WHERE id = $1 AND deleted_at IS NULL",
                )
                .bind(pipe_id)
                .fetch_optional(pool)
                .await
                .map_err(AppError::from)?;
                match row {
                    Some((pn, grade, od, wt, status, loc)) => serde_json::json!({
                        "pipe_type": pipe_type,
                        "pipe_number": pn,
                        "grade": grade,
                        "od": od,
                        "wt": wt,
                        "current_status": status,
                        "current_location_id": loc,
                    }),
                    None => {
                        return Err(AppError::NotFound(format!(
                            "Pipe {} id={} not found",
                            pipe_type, pipe_id
                        )))
                    }
                }
            }
            "welded" => {
                let row = sqlx::query_as::<_, (String, String, f64, f64, String, Option<i64>)>(
                    "SELECT pipe_number, grade, od, wt, status, location_id \
                     FROM welded_pipes WHERE id = $1 AND deleted_at IS NULL",
                )
                .bind(pipe_id)
                .fetch_optional(pool)
                .await
                .map_err(AppError::from)?;
                match row {
                    Some((pn, grade, od, wt, status, loc)) => serde_json::json!({
                        "pipe_type": pipe_type,
                        "pipe_number": pn,
                        "grade": grade,
                        "od": od,
                        "wt": wt,
                        "current_status": status,
                        "current_location_id": loc,
                    }),
                    None => {
                        return Err(AppError::NotFound(format!(
                            "Pipe {} id={} not found",
                            pipe_type, pipe_id
                        )))
                    }
                }
            }
            _ => {
                return Err(AppError::Validation(format!(
                    "Unknown pipe_type: {}",
                    pipe_type
                )))
            }
        };

        let events: Vec<serde_json::Value> = logs
            .into_iter()
            .map(|log| {
                serde_json::json!({
                    "id": log.id,
                    "change_type": log.change_type,
                    "ref_type": log.ref_type,
                    "ref_id": log.ref_id,
                    "from_location_id": log.from_location_id,
                    "to_location_id": log.to_location_id,
                    "notes": log.notes,
                    "created_by": log.created_by,
                    "created_at": log.created_at,
                })
            })
            .collect();

        Ok(serde_json::json!({
            "pipe": pipe_info,
            "events": events,
        }))
    }

    /// Query pipes by heat number — searches both seamless and screen pipes,
    /// returning type, ID, number, grade, status, and location.
    pub async fn trace_by_heat_number(
        pool: &PgPool,
        heat_number: &str,
    ) -> Result<Vec<serde_json::Value>, AppError> {
        let mut results: Vec<serde_json::Value> = Vec::new();

        let seamless = SeamlessPipeRepo::find_by_heat_number(pool, heat_number)
            .await
            .map_err(AppError::from)?;

        for p in &seamless {
            results.push(serde_json::json!({
                "pipe_type": "seamless",
                "pipe_id": p.id,
                "pipe_number": p.pipe_number,
                "grade": p.grade,
                "status": p.status,
                "location_id": p.location_id,
            }));
        }

        let screen = ScreenPipeRepo::find_by_heat_number(pool, heat_number)
            .await
            .map_err(AppError::from)?;

        for p in &screen {
            results.push(serde_json::json!({
                "pipe_type": "screen",
                "pipe_id": p.id,
                "pipe_number": p.pipe_number,
                "grade": p.base_grade,
                "status": p.status,
                "location_id": p.location_id,
            }));
        }

        let welded: Vec<(i64, String, String, String, Option<i64>)> = sqlx::query_as(
            "SELECT id, pipe_number, grade, status, location_id \
             FROM welded_pipes WHERE heat_number = $1 AND deleted_at IS NULL",
        )
        .bind(heat_number)
        .fetch_all(pool)
        .await
        .map_err(AppError::from)?;

        for (id, pn, grade, status, loc) in welded {
            results.push(serde_json::json!({
                "pipe_type": "welded",
                "pipe_id": id,
                "pipe_number": pn,
                "grade": grade,
                "status": status,
                "location_id": loc,
            }));
        }

        Ok(results)
    }

    /// Trace by order — queries inbound/outbound records for a purchase/sales order,
    /// along with the list of pipes in each record and their current status.
    ///
    /// # Errors
    /// - `AppError::Validation` — order_type is not `inbound` or `outbound`
    pub async fn trace_by_order(
        pool: &PgPool,
        order_type: &str,
        order_id: i64,
    ) -> Result<serde_json::Value, AppError> {
        let (records_json, items) = match order_type {
            "inbound" => {
                let records = InboundRepo::find_by_order_id(pool, order_id)
                    .await
                    .map_err(AppError::from)?;

                let records_json: Vec<serde_json::Value> = records
                    .iter()
                    .map(|rec| {
                        serde_json::json!({
                            "id": rec.id,
                            "inbound_no": rec.inbound_no,
                            "approval_status": rec.approval_status,
                        })
                    })
                    .collect();

                let mut pipes: Vec<serde_json::Value> = Vec::new();
                for rec in &records {
                    let record_items = InboundRepo::find_items(pool, rec.id)
                        .await
                        .map_err(AppError::from)?;

                    for item in &record_items {
                        let status = match Self::get_pipe_current_status(pool, &item.pipe_type, item.pipe_id).await {
                            Ok(s) => s,
                            Err(_) => "unknown".into(),
                        };
                        pipes.push(serde_json::json!({
                            "pipe_type": item.pipe_type,
                            "pipe_id": item.pipe_id,
                            "current_status": status,
                        }));
                    }
                }

                (records_json, pipes)
            }
            "outbound" => {
                let records = OutboundRepo::find_by_order_id(pool, order_id)
                    .await
                    .map_err(AppError::from)?;

                let records_json: Vec<serde_json::Value> = records
                    .iter()
                    .map(|rec| {
                        serde_json::json!({
                            "id": rec.id,
                            "outbound_no": rec.outbound_no,
                            "approval_status": rec.approval_status,
                        })
                    })
                    .collect();

                let mut pipes: Vec<serde_json::Value> = Vec::new();
                for rec in &records {
                    let record_items = OutboundRepo::find_items(pool, rec.id)
                        .await
                        .map_err(AppError::from)?;

                    for item in &record_items {
                        let status = match Self::get_pipe_current_status(pool, &item.pipe_type, item.pipe_id).await {
                            Ok(s) => s,
                            Err(_) => "unknown".into(),
                        };
                        pipes.push(serde_json::json!({
                            "pipe_type": item.pipe_type,
                            "pipe_id": item.pipe_id,
                            "current_status": status,
                        }));
                    }
                }

                (records_json, pipes)
            }
            _ => {
                return Err(AppError::Validation(format!(
                    "Unknown order_type: {}. Use 'inbound' or 'outbound'.",
                    order_type
                )))
            }
        };

        Ok(serde_json::json!({
            "order_type": order_type,
            "order_id": order_id,
            "records": records_json,
            "related_pipes": items,
        }))
    }

    async fn get_pipe_current_status(
        pool: &PgPool,
        pipe_type: &str,
        pipe_id: i64,
    ) -> Result<String, AppError> {
        Ok(
            match InventoryRepo::get_pipe_status(pool, pipe_type, pipe_id)
                .await
                .map_err(AppError::from)?
            {
                Some(s) => s,
                None => "deleted".into(),
            },
        )
    }
}
