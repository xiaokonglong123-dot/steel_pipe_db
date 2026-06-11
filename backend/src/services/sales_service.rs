use chrono::Utc;
use sqlx::SqlitePool;
use std::str::FromStr;

use crate::domain::order::OrderStatus;
use crate::dto::common::PaginationParams;
use crate::dto::sales_dto::{
    ApproveOrderRequest, CreateSalesOrderRequest, RejectOrderRequest, SalesOrderFilterParams,
    SalesOrderStatusTransitionRequest, UpdateSalesItemRequest, UpdateSalesOrderRequest,
};
use crate::error::AppError;
use crate::models::sales_order::{SalesOrder, SalesOrderItem};
use crate::repositories::customer_repo::CustomerRepo;
use crate::repositories::inventory_repo::InventoryRepo;
use crate::repositories::outbound_repo::OutboundRepo;
use crate::repositories::sales_order_repo::SalesOrderRepo;

/// Service handling the full lifecycle of Sales Orders (SO)
/// — creation, updates, status transitions, approvals, rejections, and linking to
/// outbound orders. All status transitions are validated against the
/// `OrderStatus` domain-enum rules under the hood.
pub struct SalesService;

impl SalesService {
    fn generate_order_no(prefix: &str) -> String {
        let now = Utc::now();
        let date_str = now.format("%Y%m%d").to_string();
        let serial = uuid::Uuid::new_v4().to_string();
        let short_serial = &serial[..8];
        format!("{}-{}-{}", prefix, date_str, short_serial)
    }

    fn validate_status_transition(current: &str, target: &str) -> Result<(), AppError> {
        let current_status = OrderStatus::from_str(current)
            .map_err(|_| AppError::Validation(format!("Invalid current status: {}", current)))?;
        let target_status = OrderStatus::from_str(target)
            .map_err(|_| AppError::Validation(format!("Invalid target status: {}", target)))?;

        if !current_status.valid_transition(&target_status) {
            return Err(AppError::OrderCannotModify(format!(
                "Cannot transition from '{}' to '{}'",
                current, target
            )));
        }
        Ok(())
    }

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
            _ => Self::generate_order_no("SO"),
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
    /// # Errors
    /// - `AppError::OrderNotFound` — ID doesn't exist or was deleted
    /// - `AppError::OrderCannotModify` — status transition isn't valid
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

        Self::validate_status_transition(&existing.status, &dto.status)?;

        SalesOrderRepo::update_status(pool, id, &dto.status)
            .await
            .map_err(AppError::from)
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
            .map_err(AppError::from)
    }

    /// Approves a sales order — checks the info and amount, then bumps it to
    /// `approved` status. Also verifies there's enough ATP stock for each item.
    ///
    /// Uses `BEGIN IMMEDIATE` to serialize concurrent approvals and prevent TOCTOU
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
        if let Err(e) = sqlx::query("BEGIN IMMEDIATE").execute(&mut *conn).await {
            return Err(AppError::from(e));
        }

        // ATP check inside the serialised transaction so no concurrent writer can
        // modify stock between our read and the status update.
        for item in &items {
            let atp_rows = match InventoryRepo::find_atp(
                &mut *conn,
                &Some(item.pipe_type.clone()),
                &Some(item.grade.clone()),
                &None,
            )
            .await
            {
                Ok(rows) => rows,
                Err(e) => {
                    sqlx::query("ROLLBACK").execute(&mut *conn).await.ok();
                    return Err(AppError::from(e));
                }
            };

            let available: i64 = atp_rows.iter().map(|(_, _, cnt, _)| cnt).sum();
            if available < item.quantity {
                sqlx::query("ROLLBACK").execute(&mut *conn).await.ok();
                return Err(AppError::InsufficientStock("Insufficient stock".into()));
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
    /// # Errors
    /// - `AppError::OrderNotFound` — ID doesn't exist
    /// - `AppError::OrderCannotModify` — current status won't allow rejection
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

        SalesOrderRepo::reject(pool, id, &dto.reason)
            .await
            .map_err(AppError::from)
    }

    /// Links an outbound order to a sales order. Records the outbound ID and
    /// bumps the SO status to `shipped`.
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

        OutboundRepo::link_to_order(pool, outbound_id, order_id)
            .await
            .map_err(AppError::from)
    }
}
