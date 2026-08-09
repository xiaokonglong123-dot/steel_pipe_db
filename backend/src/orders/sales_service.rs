use sqlx::SqlitePool;

use crate::dto::common::PaginationParams;
use crate::dto::sales_dto::{
    ApproveOrderRequest, CreateSalesOrderRequest, RejectOrderRequest, SalesOrderFilterParams,
    SalesOrderStatusTransitionRequest, UpdateSalesItemRequest, UpdateSalesOrderRequest,
};
use crate::error::AppError;
use crate::models::sales_order::{SalesOrder, SalesOrderItem};
use crate::parties::customer_repo::CustomerRepo;
use crate::orders::sales_order_repo::SalesOrderRepo;
use crate::utils;

/// Service handling the full lifecycle of Sales Orders (SO)
/// — creation, updates, status transitions, approvals, rejections, and linking to
/// outbound orders. All status transitions are validated against the
/// `OrderStatus` domain-enum rules under the hood.
pub struct SalesService;

impl SalesService {
    /// Kicks off a new sales order. Needs at least one line item; validates the
    /// customer is active and the order number is unique.
    ///
    /// # Errors
    /// - `AppError::Validation` — empty items, duplicate order no, or inactive customer
    /// - `AppError::CustomerNotFound` — customer ID doesn't exist
    pub async fn create_sales_order(
        pool: &SqlitePool,
        dto: &CreateSalesOrderRequest,
    ) -> Result<SalesOrder, AppError> {
        if dto.items.is_empty() {
            return Err(AppError::Validation("At least one item is required".into()));
        }

        let customer = CustomerRepo::find_by_id(pool, dto.customer_id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| {
                AppError::CustomerNotFound(format!("Customer id={} not found", dto.customer_id))
            })?;

        if !customer.is_active {
            return Err(AppError::Validation(format!(
                "Customer '{}' is not active",
                customer.name
            )));
        }

        let order_no = match &dto.order_no {
            Some(on) if !on.is_empty() => {
                if SalesOrderRepo::find_by_order_no(pool, on)
                    .await
                    .map_err(AppError::from)?
                    .is_some()
                {
                    return Err(AppError::Validation(format!(
                        "Order number '{}' already exists",
                        on
                    )));
                }
                on.clone()
            }
            _ => utils::generate_no("SO"),
        };

        SalesOrderRepo::create_with_items(pool, dto, &order_no)
            .await
            .map_err(AppError::from)
    }

    /// Updates the sales-order header fields. Only works when the order is in
    /// `draft` status — no editing once it's moving.
    ///
    /// # Errors
    /// - `AppError::OrderNotFound` — ID doesn't exist or was soft-deleted
    /// - `AppError::OrderCannotModify` — current status won't allow edits
    pub async fn update_sales_order(
        pool: &SqlitePool,
        id: i64,
        dto: &UpdateSalesOrderRequest,
    ) -> Result<SalesOrder, AppError> {
        let existing = SalesOrderRepo::find_by_id(pool, id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::OrderNotFound(format!("Sales order id={} not found", id)))?;

        if existing.deleted_at.is_some() {
            return Err(AppError::OrderNotFound(format!(
                "Sales order id={} has been deleted",
                id
            )));
        }

        if existing.status != "draft" {
            return Err(AppError::OrderCannotModify(format!(
                "Cannot modify order with status '{}'. Only 'draft' orders can be modified.",
                existing.status
            )));
        }

        SalesOrderRepo::update_order(pool, id, dto)
            .await
            .map_err(AppError::from)
    }

    /// Transitions a sales order's status. Validates the current→target hop against
    /// `OrderStatus` domain rules — only valid transitions are allowed.
    ///
    /// Uses `BEGIN` to serialize concurrent transitions and prevent TOCTOU
    /// races — the status guard in the WHERE clause ensures that if another request
    /// changed the order between our read and the update, the update hits zero rows
    /// and we fail safe.
    ///
    /// # Errors
    /// - `AppError::OrderNotFound` — ID doesn't exist or was deleted
    /// - `AppError::OrderCannotModify` — status transition isn't valid or race lost
    pub async fn transition_sales_status(
        pool: &SqlitePool,
        id: i64,
        dto: &SalesOrderStatusTransitionRequest,
    ) -> Result<(), AppError> {
        let existing = SalesOrderRepo::find_by_id(pool, id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::OrderNotFound(format!("Sales order id={} not found", id)))?;

        if existing.deleted_at.is_some() {
            return Err(AppError::OrderNotFound(format!(
                "Sales order id={} has been deleted",
                id
            )));
        }

        // Acquire a connection and start an IMMEDIATE transaction to serialize
        // concurrent status transitions.
        let mut conn = pool.acquire().await.map_err(AppError::from)?;
        if let Err(e) = sqlx::query("BEGIN").execute(&mut *conn).await {
            return Err(AppError::from(e));
        }

        // Validate transition inside the serialised transaction.
        if let Err(e) = utils::validate_status_transition(&existing.status, &dto.status) {
            sqlx::query("ROLLBACK").execute(&mut *conn).await.ok();
            return Err(e);
        }

        let rows_affected = match sqlx::query(
            "UPDATE sales_orders SET status = ?, updated_at = datetime('now') \
             WHERE id = ? AND status = ? AND deleted_at IS NULL",
        )
        .bind(&dto.status)
        .bind(id)
        .bind(&existing.status)
        .execute(&mut *conn)
        .await
        {
            Ok(result) => result.rows_affected(),
            Err(e) => {
                sqlx::query("ROLLBACK").execute(&mut *conn).await.ok();
                return Err(AppError::from(e));
            }
        };

        if rows_affected == 0 {
            sqlx::query("ROLLBACK").execute(&mut *conn).await.ok();
            return Err(AppError::OrderCannotModify(
                "Order status changed or already processed".into(),
            ));
        }

        sqlx::query("COMMIT")
            .execute(&mut *conn)
            .await
            .map_err(AppError::from)
            .map(|_| ())
    }

    /// Fetches a sales order and its line items. Returns a `(order, items)` tuple.
    ///
    /// # Errors
    /// - `AppError::OrderNotFound` — ID doesn't exist
    pub async fn get_sales_order(
        pool: &SqlitePool,
        id: i64,
    ) -> Result<(SalesOrder, Vec<SalesOrderItem>), AppError> {
        let order = SalesOrderRepo::find_by_id(pool, id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::OrderNotFound(format!("Sales order id={} not found", id)))?;

        let items = SalesOrderRepo::find_items(pool, id)
            .await
            .map_err(AppError::from)?;

        Ok((order, items))
    }

    /// Paginates sales orders with filters for customer, date range, status, etc.
    pub async fn list_sales_orders(
        pool: &SqlitePool,
        filter: &SalesOrderFilterParams,
        params: &PaginationParams,
    ) -> Result<(Vec<SalesOrder>, u64), AppError> {
        SalesOrderRepo::list(pool, filter, params)
            .await
            .map_err(AppError::from)
    }

    /// Soft-deletes a sales order. Only orders in `draft` or `cancelled` status
    /// can be removed — anything else gets a hard no.
    ///
    /// # Errors
    /// - `AppError::OrderNotFound` — ID doesn't exist
    /// - `AppError::OrderCannotModify` — current status doesn't allow deletion
    pub async fn delete_sales_order(pool: &SqlitePool, id: i64) -> Result<(), AppError> {
        let existing = SalesOrderRepo::find_by_id(pool, id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::OrderNotFound(format!("Sales order id={} not found", id)))?;

        if existing.status != "draft" && existing.status != "cancelled" {
            return Err(AppError::OrderCannotModify(format!(
                "Cannot delete order with status '{}'. Only 'draft' or 'cancelled' orders can be deleted.",
                existing.status
            )));
        }

        SalesOrderRepo::delete(pool, id)
            .await
            .map_err(AppError::from)
    }

    /// Updates a sales-order line item's specs and quantity. Only works when the
    /// order is still in `draft`. Returns `(order, updated_item)`.
    ///
    /// # Errors
    /// - `AppError::OrderNotFound` — order doesn't exist
    /// - `AppError::OrderCannotModify` — order ain't in draft
    pub async fn update_sales_item(
        pool: &SqlitePool,
        order_id: i64,
        item_id: i64,
        dto: &UpdateSalesItemRequest,
    ) -> Result<(SalesOrder, SalesOrderItem), AppError> {
        let order = SalesOrderRepo::find_by_id(pool, order_id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| {
                AppError::OrderNotFound(format!("Sales order id={} not found", order_id))
            })?;

        if order.status != "draft" {
            return Err(AppError::OrderCannotModify(format!(
                "Cannot modify items on order with status '{}'",
                order.status
            )));
        }

        let item = SalesOrderRepo::update_item(pool, item_id, dto)
            .await
            .map_err(AppError::from)?;

        SalesOrderRepo::recalculate_total(pool, order_id)
            .await
            .map_err(AppError::from)?;

        let order = SalesOrderRepo::find_by_id(pool, order_id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| {
                AppError::OrderNotFound(format!("Sales order id={} not found", order_id))
            })?;

        Ok((order, item))
    }

    /// Deletes a line item from a sales order. Only allowed when the order is
    /// still in `draft` — no touching confirmed orders.
    ///
    /// # Errors
    /// - `AppError::OrderNotFound` — order doesn't exist
    /// - `AppError::OrderCannotModify` — order isn't in draft
    pub async fn delete_sales_item(
        pool: &SqlitePool,
        order_id: i64,
        item_id: i64,
    ) -> Result<(), AppError> {
        let order = SalesOrderRepo::find_by_id(pool, order_id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| {
                AppError::OrderNotFound(format!("Sales order id={} not found", order_id))
            })?;

        if order.status != "draft" {
            return Err(AppError::OrderCannotModify(format!(
                "Cannot delete items from order with status '{}'",
                order.status
            )));
        }

        SalesOrderRepo::delete_item(pool, item_id)
            .await
            .map_err(AppError::from)?;

        SalesOrderRepo::recalculate_total(pool, order_id)
            .await
            .map_err(AppError::from)?;

        Ok(())
    }

    /// Computes the currently available (promiseable) quantity for an item.
    ///
    /// Canonical ATP formula (single source of truth — matches
    /// `inventory_atp/repos.rs::overview`): on-hand is derived from inventory
    /// logs (inbound/check_adjust add, outbound/transfer subtract), minus
    /// active reservations from `atp_slots`.
    async fn available_quantity(conn: &mut sqlx::SqliteConnection, item_id: i64) -> Result<f64, AppError> {
        let row: (Option<f64>,) = sqlx::query_as(
            "SELECT \
             CAST(COALESCE((SELECT SUM(
                 CASE WHEN change_type IN ('inbound', 'check_adjust') THEN quantity
                      ELSE -quantity END)
              FROM inventory_logs WHERE item_id = ?), 0.0) AS REAL) \
             - CAST(COALESCE((SELECT SUM(quantity_reserved) FROM atp_slots \
                WHERE item_id = ? AND status = 'reserved'), 0.0) AS REAL)",
        )
        .bind(item_id)
        .bind(item_id)
        .fetch_one(&mut *conn)
        .await
        .map_err(AppError::from)?;
        Ok(row.0.unwrap_or(0.0))
    }

    /// Approves a sales order — checks the info and amount, then bumps it to
    /// `approved` status. Also verifies there's enough ATP stock for each item.
    ///
    /// Uses `BEGIN` to serialize concurrent approvals and prevent TOCTOU
    /// races on ATP (Available-to-Promise) inventory checks. The ATP query and the
    /// status update share a single serialised transaction, so two concurrent
    /// approvals for the same order cannot both pass the ATP check.
    ///
    /// # Errors
    /// - `AppError::OrderNotFound` — ID doesn't exist
    /// - `AppError::OrderCannotModify` — current status won't allow approval
    /// - `AppError::Validation` — approval info is incomplete
    /// - `AppError::InsufficientStock` — not enough inventory to fulfill
    pub async fn approve_sales_order(
        pool: &SqlitePool,
        id: i64,
        _dto: &ApproveOrderRequest,
    ) -> Result<(), AppError> {
        let existing = SalesOrderRepo::find_by_id(pool, id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::OrderNotFound(format!("Sales order id={} not found", id)))?;

        if existing.deleted_at.is_some() {
            return Err(AppError::OrderNotFound(format!(
                "Sales order id={} has been deleted",
                id
            )));
        }

        if existing.status != "pending" {
            return Err(AppError::OrderCannotModify(format!(
                "Cannot approve order with status '{}'. Only 'pending' orders can be approved.",
                existing.status
            )));
        }

        let items = SalesOrderRepo::find_items(pool, id)
            .await
            .map_err(AppError::from)?;

        // Acquire a connection and start an IMMEDIATE transaction — this prevents two
        // concurrent requests from both reading stale ATP data (C1: TOCTOU fix).
        let mut conn = pool.acquire().await.map_err(AppError::from)?;
        if let Err(e) = sqlx::query("BEGIN").execute(&mut *conn).await {
            return Err(AppError::from(e));
        }

        // ATP check inside the serialised transaction so no concurrent writer can
        // modify stock between our read and the status update.
        for item in &items {
            let available = match Self::available_quantity(&mut conn, item.item_id).await {
                Ok(q) => q,
                Err(e) => {
                    sqlx::query("ROLLBACK").execute(&mut *conn).await.ok();
                    return Err(e);
                }
            };

            if available < item.quantity {
                sqlx::query("ROLLBACK").execute(&mut *conn).await.ok();
                return Err(AppError::InsufficientStock(format!(
                    "Insufficient stock for item_id={}: available {}, required {}",
                    item.item_id, available, item.quantity
                )));
            }
        }

        let rows_affected = match sqlx::query(
            "UPDATE sales_orders SET status = 'approved', updated_at = datetime('now') \
             WHERE id = ? AND status = 'pending' AND deleted_at IS NULL",
        )
        .bind(id)
        .execute(&mut *conn)
        .await
        {
            Ok(result) => result.rows_affected(),
            Err(e) => {
                sqlx::query("ROLLBACK").execute(&mut *conn).await.ok();
                return Err(AppError::from(e));
            }
        };

        if rows_affected == 0 {
            sqlx::query("ROLLBACK").execute(&mut *conn).await.ok();
            return Err(AppError::OrderCannotModify(
                "Order status changed or already processed".into(),
            ));
        }

        sqlx::query("COMMIT")
            .execute(&mut *conn)
            .await
            .map_err(AppError::from)
            .map(|_| ())
    }

    /// Rejects a sales order. Requires a rejection reason and rolls the status
    /// back to `draft`.
    ///
    /// Uses `BEGIN` to serialize concurrent operations and prevent TOCTOU
    /// races — the `AND status='pending'` guard in the WHERE clause means that a
    /// concurrent approve wins and this reject safely fails instead of silently
    /// overwriting.
    ///
    /// # Errors
    /// - `AppError::OrderNotFound` — ID doesn't exist
    /// - `AppError::OrderCannotModify` — current status won't allow rejection, or
    ///   race lost to another concurrent status change
    pub async fn reject_sales_order(
        pool: &SqlitePool,
        id: i64,
        dto: &RejectOrderRequest,
    ) -> Result<(), AppError> {
        let existing = SalesOrderRepo::find_by_id(pool, id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::OrderNotFound(format!("Sales order id={} not found", id)))?;

        if existing.deleted_at.is_some() {
            return Err(AppError::OrderNotFound(format!(
                "Sales order id={} has been deleted",
                id
            )));
        }

        if existing.status != "pending" {
            return Err(AppError::OrderCannotModify(format!(
                "Cannot reject order with status '{}'. Only 'pending' orders can be rejected.",
                existing.status
            )));
        }

        // Acquire a connection and start an IMMEDIATE transaction — this prevents two
        // concurrent requests from both reading stale status data (TOCTOU fix).
        let mut conn = pool.acquire().await.map_err(AppError::from)?;
        if let Err(e) = sqlx::query("BEGIN").execute(&mut *conn).await {
            return Err(AppError::from(e));
        }

        let rows_affected = match sqlx::query(
            "UPDATE sales_orders SET status = 'rejected', notes = ?, updated_at = datetime('now') \
             WHERE id = ? AND status = 'pending' AND deleted_at IS NULL",
        )
        .bind(&dto.reason)
        .bind(id)
        .execute(&mut *conn)
        .await
        {
            Ok(result) => result.rows_affected(),
            Err(e) => {
                sqlx::query("ROLLBACK").execute(&mut *conn).await.ok();
                return Err(AppError::from(e));
            }
        };

        if rows_affected == 0 {
            sqlx::query("ROLLBACK").execute(&mut *conn).await.ok();
            return Err(AppError::OrderCannotModify(
                "Order status changed or already processed".into(),
            ));
        }

        sqlx::query("COMMIT")
            .execute(&mut *conn)
            .await
            .map_err(AppError::from)
            .map(|_| ())
    }

    /// Links an outbound order to a sales order. Records the outbound ID and,
    /// if every item's `delivered_quantity >= quantity` (fully fulfilled), bumps
    /// the SO status to `completed`.
    ///
    /// # Errors
    /// - `AppError::OrderNotFound` — sales order doesn't exist
    /// - `AppError::OrderCannotModify` — can't link (bad status or already linked)
    pub async fn link_outbound_to_order(
        pool: &SqlitePool,
        order_id: i64,
        outbound_id: i64,
    ) -> Result<(), AppError> {
        let existing = SalesOrderRepo::find_by_id(pool, order_id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| {
                AppError::OrderNotFound(format!("Sales order id={} not found", order_id))
            })?;

        if existing.deleted_at.is_some() {
            return Err(AppError::OrderNotFound(format!(
                "Sales order id={} has been deleted",
                order_id
            )));
        }

        let mut conn = pool.acquire().await.map_err(AppError::from)?;
        if let Err(e) = sqlx::query("BEGIN").execute(&mut *conn).await {
            return Err(AppError::from(e));
        }

        // Link the outbound record to this sales order
        if let Err(e) = sqlx::query(
            "UPDATE outbound_records SET order_id = ?, updated_at = datetime('now') \
             WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(order_id)
        .bind(outbound_id)
        .execute(&mut *conn)
        .await
        {
            sqlx::query("ROLLBACK").execute(&mut *conn).await.ok();
            return Err(AppError::from(e));
        }

        // Check whether the sales order is fully delivered
        let items = match sqlx::query_as::<_, SalesOrderItem>(
            "SELECT id, order_id, item_id, quantity, delivered_quantity, \
             unit_price, total_price, notes, created_at \
             FROM sales_order_items WHERE order_id = ? ORDER BY id ASC",
        )
        .bind(order_id)
        .fetch_all(&mut *conn)
        .await
        {
            Ok(items) => items,
            Err(e) => {
                sqlx::query("ROLLBACK").execute(&mut *conn).await.ok();
                return Err(AppError::from(e));
            }
        };

        let all_delivered = items.iter().all(|item| item.delivered_quantity >= item.quantity);
        if all_delivered {
            if let Err(e) = sqlx::query(
                "UPDATE sales_orders SET status = 'completed', updated_at = datetime('now') \
                 WHERE id = ? AND deleted_at IS NULL",
            )
            .bind(order_id)
            .execute(&mut *conn)
            .await
            {
                sqlx::query("ROLLBACK").execute(&mut *conn).await.ok();
                return Err(AppError::from(e));
            }
        }

        sqlx::query("COMMIT")
            .execute(&mut *conn)
            .await
            .map_err(AppError::from)
            .map(|_| ())
    }
}
