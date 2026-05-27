use sqlx::SqlitePool;

use crate::error::AppError;
use crate::models::inventory::InventoryLog;

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
        pool: &SqlitePool,
        pipe_type: &str,
        pipe_id: i64,
    ) -> Result<serde_json::Value, AppError> {
        let logs: Vec<InventoryLog> = sqlx::query_as::<_, InventoryLog>(
            "SELECT id, pipe_type, pipe_id, change_type, ref_type, ref_id, \
             from_location_id, to_location_id, notes, created_by, created_at \
             FROM inventory_logs WHERE pipe_type = ? AND pipe_id = ? \
             ORDER BY created_at ASC",
        )
        .bind(pipe_type)
        .bind(pipe_id)
        .fetch_all(pool)
        .await
        .map_err(AppError::from)?;

        let pipe_info = match pipe_type {
            "seamless" | "casing" | "tubing" => {
                let row = sqlx::query_as::<_, (String, String, f64, f64, String, Option<i64>)>(
                    "SELECT pipe_number, grade, od, wt, status, location_id \
                     FROM seamless_pipes WHERE id = ? AND deleted_at IS NULL",
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
                     FROM screen_pipes WHERE id = ? AND deleted_at IS NULL",
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
        pool: &SqlitePool,
        heat_number: &str,
    ) -> Result<Vec<serde_json::Value>, AppError> {
        let mut results: Vec<serde_json::Value> = Vec::new();

        let seamless: Vec<(i64, String, String, String, Option<i64>)> = sqlx::query_as(
            "SELECT id, pipe_number, grade, status, location_id \
             FROM seamless_pipes WHERE heat_number = ? AND deleted_at IS NULL",
        )
        .bind(heat_number)
        .fetch_all(pool)
        .await
        .map_err(AppError::from)?;

        for (id, pn, grade, status, loc) in seamless {
            results.push(serde_json::json!({
                "pipe_type": "seamless",
                "pipe_id": id,
                "pipe_number": pn,
                "grade": grade,
                "status": status,
                "location_id": loc,
            }));
        }

        let screen: Vec<(i64, String, String, String, Option<i64>)> = sqlx::query_as(
            "SELECT id, pipe_number, base_grade, status, location_id \
             FROM screen_pipes WHERE heat_number = ? AND deleted_at IS NULL",
        )
        .bind(heat_number)
        .fetch_all(pool)
        .await
        .map_err(AppError::from)?;

        for (id, pn, grade, status, loc) in screen {
            results.push(serde_json::json!({
                "pipe_type": "screen",
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
        pool: &SqlitePool,
        order_type: &str,
        order_id: i64,
    ) -> Result<serde_json::Value, AppError> {
        let (records, items, field_name) = match order_type {
            "inbound" => {
                let records: Vec<(i64, String, String)> = sqlx::query_as(
                    "SELECT id, inbound_no, approval_status \
                     FROM inbound_records WHERE order_id = ? AND deleted_at IS NULL",
                )
                .bind(order_id)
                .fetch_all(pool)
                .await
                .map_err(AppError::from)?;

                let mut pipes: Vec<serde_json::Value> = Vec::new();
                for (rec_id, _no, _status) in &records {
                    let items: Vec<(i64, String, i64)> = sqlx::query_as(
                        "SELECT id, pipe_type, pipe_id FROM inbound_items WHERE inbound_id = ?",
                    )
                    .bind(rec_id)
                    .fetch_all(pool)
                    .await
                    .map_err(AppError::from)?;

                    for (_item_id, pt, pipe_id) in items {
                        let status = match Self::get_pipe_current_status(pool, &pt, pipe_id).await {
                            Ok(s) => s,
                            Err(_) => "unknown".into(),
                        };
                        pipes.push(serde_json::json!({
                            "pipe_type": pt,
                            "pipe_id": pipe_id,
                            "current_status": status,
                        }));
                    }
                }

                (records, pipes, "inbound_no")
            }
            "outbound" => {
                let records: Vec<(i64, String, String)> = sqlx::query_as(
                    "SELECT id, outbound_no, approval_status \
                     FROM outbound_records WHERE order_id = ? AND deleted_at IS NULL",
                )
                .bind(order_id)
                .fetch_all(pool)
                .await
                .map_err(AppError::from)?;

                let mut pipes: Vec<serde_json::Value> = Vec::new();
                for (rec_id, _no, _status) in &records {
                    let items: Vec<(i64, String, i64)> = sqlx::query_as(
                        "SELECT id, pipe_type, pipe_id FROM outbound_items WHERE outbound_id = ?",
                    )
                    .bind(rec_id)
                    .fetch_all(pool)
                    .await
                    .map_err(AppError::from)?;

                    for (_item_id, pt, pipe_id) in items {
                        let status = match Self::get_pipe_current_status(pool, &pt, pipe_id).await {
                            Ok(s) => s,
                            Err(_) => "unknown".into(),
                        };
                        pipes.push(serde_json::json!({
                            "pipe_type": pt,
                            "pipe_id": pipe_id,
                            "current_status": status,
                        }));
                    }
                }

                (records, pipes, "outbound_no")
            }
            _ => {
                return Err(AppError::Validation(format!(
                    "Unknown order_type: {}. Use 'inbound' or 'outbound'.",
                    order_type
                )))
            }
        };

        let records_json: Vec<serde_json::Value> = records
            .into_iter()
            .map(|(id, record_no, approval_status)| {
                serde_json::json!({
                    "id": id,
                    field_name: record_no,
                    "approval_status": approval_status,
                })
            })
            .collect();

        Ok(serde_json::json!({
            "order_type": order_type,
            "order_id": order_id,
            "records": records_json,
            "related_pipes": items,
        }))
    }

    async fn get_pipe_current_status(
        pool: &SqlitePool,
        pipe_type: &str,
        pipe_id: i64,
    ) -> Result<String, AppError> {
        let status: Option<(String,)> = match pipe_type {
            "seamless" | "casing" | "tubing" => {
                sqlx::query_as(
                    "SELECT status FROM seamless_pipes WHERE id = ? AND deleted_at IS NULL",
                )
                .bind(pipe_id)
                .fetch_optional(pool)
                .await
                .map_err(AppError::from)?
            }
            "screen" | "screened" => {
                sqlx::query_as(
                    "SELECT status FROM screen_pipes WHERE id = ? AND deleted_at IS NULL",
                )
                .bind(pipe_id)
                .fetch_optional(pool)
                .await
                .map_err(AppError::from)?
            }
            _ => None,
        };
        Ok(status.map(|s| s.0).unwrap_or_else(|| "deleted".into()))
    }
}
