use sqlx::SqlitePool;

use crate::error::AppError;
use crate::repositories::inbound_repo::InboundRepo;
use crate::repositories::inventory_log_repo::InventoryLogRepo;
use crate::repositories::inventory_repo::InventoryRepo;
use crate::repositories::outbound_repo::OutboundRepo;

/// Trace service — full lifecycle tracking for items: inbound/outbound events by item ID,
/// and related inventory records by order number.
pub struct TraceService;

impl TraceService {
    /// Trace a single item's full lifecycle — returns current item info and
    /// all inventory change logs (inbound/outbound/transfer) sorted by time ascending.
    ///
    /// # Errors
    /// - `AppError::NotFound` — item ID does not exist or was deleted
    pub async fn trace_item_lifecycle(
        pool: &SqlitePool,
        item_id: i64,
    ) -> Result<serde_json::Value, AppError> {
        let logs = InventoryLogRepo::find_by_item(pool, item_id)
            .await
            .map_err(AppError::from)?;

        let item_info: Option<(String, String, Option<String>, Option<String>, Option<String>, f64)> =
            sqlx::query_as(
                "SELECT i.sku, i.name, i.category, i.unit, i.spec,
                        CAST(COALESCE((SELECT SUM(
                            CASE WHEN l.change_type IN ('inbound', 'check_adjust')
                                 THEN l.quantity ELSE -l.quantity END)
                         FROM inventory_logs l WHERE l.item_id = i.id) , 0.0) AS REAL) AS on_hand
                 FROM items i WHERE i.id = ? AND i.deleted_at IS NULL",
            )
            .bind(item_id)
            .fetch_optional(pool)
            .await
            .map_err(AppError::from)?;

        let item_info = match item_info {
            Some((sku, name, category, unit, spec, on_hand)) => serde_json::json!({
                "item_id": item_id,
                "sku": sku,
                "name": name,
                "category": category,
                "unit": unit,
                "spec": spec,
                "current_stock": on_hand,
            }),
            None => {
                return Err(AppError::NotFound(format!(
                    "Item id={} not found",
                    item_id
                )))
            }
        };

        let events: Vec<serde_json::Value> = logs
            .into_iter()
            .map(|log| {
                serde_json::json!({
                    "id": log.id,
                    "change_type": log.change_type,
                    "quantity": log.quantity,
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
            "item": item_info,
            "events": events,
        }))
    }

    /// Trace by order — queries inbound/outbound records for a purchase/sales order,
    /// along with the list of items in each record and their current stock.
    ///
    /// # Errors
    /// - `AppError::Validation` — order_type is not `inbound` or `outbound`
    pub async fn trace_by_order(
        pool: &SqlitePool,
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

                let mut items_out: Vec<serde_json::Value> = Vec::new();
                // Collect all (item_id, quantity) pairs first, then resolve
                // current stock in ONE aggregate query (avoids N+1 per item).
                let mut pairs: Vec<(i64, f64)> = Vec::new();
                for rec in &records {
                    let record_items = InboundRepo::find_items(pool, rec.id)
                        .await
                        .map_err(AppError::from)?;
                    for item in &record_items {
                        pairs.push((item.item_id, item.quantity));
                    }
                }
                let item_ids: Vec<i64> = pairs.iter().map(|(id, _)| *id).collect();
                let stock_map = InventoryRepo::stock_on_hand_map(pool, &item_ids)
                    .await
                    .map_err(AppError::from)?;
                for (item_id, quantity) in pairs {
                    items_out.push(serde_json::json!({
                        "item_id": item_id,
                        "quantity": quantity,
                        "current_stock": stock_map.get(&item_id).copied().unwrap_or(0.0),
                    }));
                }

                (records_json, items_out)
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

                let mut items_out: Vec<serde_json::Value> = Vec::new();
                // Same batching as the inbound branch — one stock query total.
                let mut pairs: Vec<(i64, f64)> = Vec::new();
                for rec in &records {
                    let record_items = OutboundRepo::find_items(pool, rec.id)
                        .await
                        .map_err(AppError::from)?;
                    for item in &record_items {
                        pairs.push((item.item_id, item.quantity));
                    }
                }
                let item_ids: Vec<i64> = pairs.iter().map(|(id, _)| *id).collect();
                let stock_map = InventoryRepo::stock_on_hand_map(pool, &item_ids)
                    .await
                    .map_err(AppError::from)?;
                for (item_id, quantity) in pairs {
                    items_out.push(serde_json::json!({
                        "item_id": item_id,
                        "quantity": quantity,
                        "current_stock": stock_map.get(&item_id).copied().unwrap_or(0.0),
                    }));
                }

                (records_json, items_out)
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
            "related_items": items,
        }))
    }
}
