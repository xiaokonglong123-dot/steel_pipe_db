use sqlx::PgPool;

use crate::dto::common::PaginationParams;
use crate::dto::purchase_dto::{
    ApproveOrderRequest, CreatePurchaseOrderRequest, PurchaseOrderFilterParams,
    PurchaseOrderStatusTransitionRequest, RejectOrderRequest, UpdatePurchaseItemRequest,
    UpdatePurchaseOrderRequest,
};
use crate::error::AppError;
use crate::models::purchase_order::{PurchaseOrder, PurchaseOrderItem};
use crate::repositories::purchase_order_repo::PurchaseOrderRepo;
use crate::repositories::supplier_repo::SupplierRepo;
use crate::services::utils;

/// Service handling the full lifecycle of Purchase Orders (PO)
/// — creation, updates, status transitions, approvals, rejections, and linking to
/// inbound orders. All status transitions are validated against the
/// `OrderStatus` domain-enum rules under the hood.
pub struct PurchaseService;

impl PurchaseService {
    /// Kicks off a new purchase order. Needs at least one line item; validates the
    /// supplier is active and the order number is unique. Auto-generates a PO-prefixed
    /// number or accepts a custom one.
    ///
    /// # Errors
    /// - `AppError::Validation` — empty items, duplicate order no, or inactive supplier
    /// - `AppError::SupplierNotFound` — supplier ID doesn't exist
    pub async fn create_purchase_order(
        pool: &PgPool,
        dto: &CreatePurchaseOrderRequest,
    ) -> Result<PurchaseOrder, AppError> {
        if dto.items.is_empty() {
            return Err(AppError::Validation("At least one item is required".into()));
        }

        let supplier = SupplierRepo::find_by_id(pool, dto.supplier_id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| {
                AppError::SupplierNotFound(format!("Supplier id={} not found", dto.supplier_id))
            })?;

        if !supplier.is_active {
            return Err(AppError::Validation(format!(
                "Supplier '{}' is not active",
                supplier.name
            )));
        }

        let order_no = match &dto.order_no {
            Some(on) if !on.is_empty() => {
                if PurchaseOrderRepo::find_by_order_no(pool, on)
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
            _ => utils::generate_no("PO"),
        };

        PurchaseOrderRepo::create_with_items(pool, dto, &order_no)
            .await
            .map_err(AppError::from)
    }

    /// Updates the purchase-order header fields and optionally replaces line items.
    /// Only straight-up works when the order is in `draft` status.
    ///
    /// When `dto.items` is `Some(items)`, all line items are replaced within the
    /// same transaction: items with an `id` are updated, new ones are inserted,
    /// removed items are deleted, and `total_amount` is recomputed.
    /// When `dto.items` is `None`, only header fields (`order_date`, `notes`) are
    /// updated — backward compatible with callers that only edit metadata.
    ///
    /// # Errors
    /// - `AppError::OrderNotFound` — ID doesn't exist or was soft-deleted
    /// - `AppError::OrderCannotModify` — current status won't allow edits
    pub async fn update_purchase_order(
        pool: &PgPool,
        id: i64,
        dto: &UpdatePurchaseOrderRequest,
    ) -> Result<PurchaseOrder, AppError> {
        let existing = PurchaseOrderRepo::find_by_id(pool, id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| {
                AppError::OrderNotFound(format!("Purchase order id={} not found", id))
            })?;

        if existing.deleted_at.is_some() {
            return Err(AppError::OrderNotFound(format!(
                "Purchase order id={} has been deleted",
                id
            )));
        }

        if existing.status != "draft" {
            return Err(AppError::OrderCannotModify(format!(
                "Cannot modify order with status '{}'. Only 'draft' orders can be modified.",
                existing.status
            )));
        }

        let mut conn = pool.acquire().await.map_err(AppError::from)?;
        if let Err(e) = sqlx::query("BEGIN").execute(&mut *conn).await {
            return Err(AppError::from(e));
        }

        if let Err(e) = PurchaseOrderRepo::update_order_conn(&mut *conn, id, dto).await {
            sqlx::query("ROLLBACK").execute(&mut *conn).await.ok();
            return Err(AppError::from(e));
        }

        if let Some(ref items) = dto.items {
            if let Err(e) = PurchaseOrderRepo::replace_items_conn(&mut *conn, id, items).await {
                sqlx::query("ROLLBACK").execute(&mut *conn).await.ok();
                return Err(AppError::from(e));
            }
        }

        sqlx::query("COMMIT")
            .execute(&mut *conn)
            .await
            .map_err(AppError::from)?;

        PurchaseOrderRepo::find_by_id(pool, id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::OrderNotFound(format!("Purchase order id={} not found", id)))
    }

    /// Transitions a purchase order's status. Checks the current→target hop against
    /// the `OrderStatus` domain rules — no illegal moves allowed.
    ///
    /// # Errors
    /// - `AppError::OrderNotFound` — ID doesn't exist or was deleted
    /// - `AppError::OrderCannotModify` — status transition isn't valid
    pub async fn transition_purchase_status(
        pool: &PgPool,
        id: i64,
        dto: &PurchaseOrderStatusTransitionRequest,
    ) -> Result<(), AppError> {
        let existing = PurchaseOrderRepo::find_by_id(pool, id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| {
                AppError::OrderNotFound(format!("Purchase order id={} not found", id))
            })?;

        if existing.deleted_at.is_some() {
            return Err(AppError::OrderNotFound(format!(
                "Purchase order id={} has been deleted",
                id
            )));
        }

        utils::validate_status_transition(&existing.status, &dto.status)?;

        let mut conn = pool.acquire().await.map_err(AppError::from)?;
        if let Err(e) = sqlx::query("BEGIN").execute(&mut *conn).await {
            return Err(AppError::from(e));
        }

        let rows_affected = match sqlx::query(
            "UPDATE purchase_orders SET status = ?, updated_at = NOW() \
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

    /// Fetches a purchase order and its line items. Returns a `(order, items)` tuple.
    ///
    /// # Errors
    /// - `AppError::OrderNotFound` — ID doesn't exist
    pub async fn get_purchase_order(
        pool: &PgPool,
        id: i64,
    ) -> Result<(PurchaseOrder, Vec<PurchaseOrderItem>), AppError> {
        let order = PurchaseOrderRepo::find_by_id(pool, id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| {
                AppError::OrderNotFound(format!("Purchase order id={} not found", id))
            })?;

        let items = PurchaseOrderRepo::find_items(pool, id)
            .await
            .map_err(AppError::from)?;

        Ok((order, items))
    }

    /// Paginates purchase orders with filters for supplier, date range, status, etc.
    pub async fn list_purchase_orders(
        pool: &PgPool,
        filter: &PurchaseOrderFilterParams,
        params: &PaginationParams,
    ) -> Result<(Vec<PurchaseOrder>, u64), AppError> {
        PurchaseOrderRepo::list(pool, filter, params)
            .await
            .map_err(AppError::from)
    }

    /// Soft-deletes a purchase order. Only orders in `draft` or `cancelled` status
    /// can be wiped — anything else gets rejected.
    ///
    /// # Errors
    /// - `AppError::OrderNotFound` — ID doesn't exist
    /// - `AppError::OrderCannotModify` — current status doesn't allow deletion
    pub async fn delete_purchase_order(pool: &PgPool, id: i64) -> Result<(), AppError> {
        let existing = PurchaseOrderRepo::find_by_id(pool, id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| {
                AppError::OrderNotFound(format!("Purchase order id={} not found", id))
            })?;

        if existing.status != "draft" && existing.status != "cancelled" {
            return Err(AppError::OrderCannotModify(format!(
                "Cannot delete order with status '{}'. Only 'draft' or 'cancelled' orders can be deleted.",
                existing.status
            )));
        }

        PurchaseOrderRepo::delete(pool, id)
            .await
            .map_err(AppError::from)
    }

    /// Updates a purchase-order line item's specs and quantity. Only works when the
    /// order is still in `draft` status. Returns `(order, updated_item)`.
    ///
    /// # Errors
    /// - `AppError::OrderNotFound` — order doesn't exist
    /// - `AppError::OrderCannotModify` — order ain't in draft
    pub async fn update_purchase_item(
        pool: &PgPool,
        order_id: i64,
        item_id: i64,
        dto: &UpdatePurchaseItemRequest,
    ) -> Result<(PurchaseOrder, PurchaseOrderItem), AppError> {
        let order = PurchaseOrderRepo::find_by_id(pool, order_id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| {
                AppError::OrderNotFound(format!("Purchase order id={} not found", order_id))
            })?;

        if order.status != "draft" {
            return Err(AppError::OrderCannotModify(format!(
                "Cannot modify items on order with status '{}'",
                order.status
            )));
        }

        let item = PurchaseOrderRepo::update_item(pool, item_id, dto)
            .await
            .map_err(AppError::from)?;

        PurchaseOrderRepo::recalculate_total(pool, order_id)
            .await
            .map_err(AppError::from)?;

        let order = PurchaseOrderRepo::find_by_id(pool, order_id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| {
                AppError::OrderNotFound(format!("Purchase order id={} not found", order_id))
            })?;

        Ok((order, item))
    }

    /// Deletes a line item from a purchase order. Only allowed when the order is
    /// still in `draft` — no touching confirmed orders.
    ///
    /// # Errors
    /// - `AppError::OrderNotFound` — order doesn't exist
    /// - `AppError::OrderCannotModify` — order isn't in draft
    pub async fn delete_purchase_item(
        pool: &PgPool,
        order_id: i64,
        item_id: i64,
    ) -> Result<(), AppError> {
        let order = PurchaseOrderRepo::find_by_id(pool, order_id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| {
                AppError::OrderNotFound(format!("Purchase order id={} not found", order_id))
            })?;

        if order.status != "draft" {
            return Err(AppError::OrderCannotModify(format!(
                "Cannot delete items from order with status '{}'",
                order.status
            )));
        }

        PurchaseOrderRepo::delete_item(pool, item_id)
            .await
            .map_err(AppError::from)?;

        PurchaseOrderRepo::recalculate_total(pool, order_id)
            .await
            .map_err(AppError::from)?;

        Ok(())
    }

    /// Approves a purchase order — checks the info and amount, then bumps it to
    /// `approved` status.
    ///
    /// # Errors
    /// - `AppError::OrderNotFound` — ID doesn't exist
    /// - `AppError::OrderCannotModify` — current status won't allow approval
    /// - `AppError::Validation` — approval info is incomplete
    pub async fn approve_purchase_order(
        pool: &PgPool,
        id: i64,
        _dto: &ApproveOrderRequest,
    ) -> Result<(), AppError> {
        let existing = PurchaseOrderRepo::find_by_id(pool, id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| {
                AppError::OrderNotFound(format!("Purchase order id={} not found", id))
            })?;

        if existing.deleted_at.is_some() {
            return Err(AppError::OrderNotFound(format!(
                "Purchase order id={} has been deleted",
                id
            )));
        }

        if existing.status != "pending" {
            return Err(AppError::OrderCannotModify(format!(
                "Cannot approve order with status '{}'. Only 'pending' orders can be approved.",
                existing.status
            )));
        }

        let mut conn = pool.acquire().await.map_err(AppError::from)?;
        if let Err(e) = sqlx::query("BEGIN").execute(&mut *conn).await {
            return Err(AppError::from(e));
        }

        let rows_affected = match sqlx::query(
            "UPDATE purchase_orders SET status = 'approved', updated_at = NOW() \
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

    /// Rejects a purchase order. Requires a rejection reason and rolls the status
    /// back to `draft`.
    ///
    /// # Errors
    /// - `AppError::OrderNotFound` — ID doesn't exist
    /// - `AppError::OrderCannotModify` — current status won't allow rejection
    pub async fn reject_purchase_order(
        pool: &PgPool,
        id: i64,
        dto: &RejectOrderRequest,
    ) -> Result<(), AppError> {
        let existing = PurchaseOrderRepo::find_by_id(pool, id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| {
                AppError::OrderNotFound(format!("Purchase order id={} not found", id))
            })?;

        if existing.deleted_at.is_some() {
            return Err(AppError::OrderNotFound(format!(
                "Purchase order id={} has been deleted",
                id
            )));
        }

        if existing.status != "pending" {
            return Err(AppError::OrderCannotModify(format!(
                "Cannot reject order with status '{}'. Only 'pending' orders can be rejected.",
                existing.status
            )));
        }

        let mut conn = pool.acquire().await.map_err(AppError::from)?;
        if let Err(e) = sqlx::query("BEGIN").execute(&mut *conn).await {
            return Err(AppError::from(e));
        }

        let rows_affected = match sqlx::query(
            "UPDATE purchase_orders SET status = 'rejected', notes = ?, \
             updated_at = NOW() \
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

    /// Links an inbound order to a purchase order. Records the inbound ID and,
    /// if every item's `received_quantity >= quantity` (fully received), bumps
    /// the PO status to `completed`.
    ///
    /// # Errors
    /// - `AppError::OrderNotFound` — purchase order doesn't exist
    /// - `AppError::OrderCannotModify` — can't link (bad status or already linked)
    pub async fn link_inbound_to_order(
        pool: &PgPool,
        order_id: i64,
        inbound_id: i64,
    ) -> Result<(), AppError> {
        let existing = PurchaseOrderRepo::find_by_id(pool, order_id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| {
                AppError::OrderNotFound(format!("Purchase order id={} not found", order_id))
            })?;

        if existing.deleted_at.is_some() {
            return Err(AppError::OrderNotFound(format!(
                "Purchase order id={} has been deleted",
                order_id
            )));
        }

        let mut conn = pool.acquire().await.map_err(AppError::from)?;
        if let Err(e) = sqlx::query("BEGIN").execute(&mut *conn).await {
            return Err(AppError::from(e));
        }

        // Link the inbound record to this purchase order
        if let Err(e) = sqlx::query(
            "UPDATE inbound_records SET order_id = ?, updated_at = NOW() \
             WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(order_id)
        .bind(inbound_id)
        .execute(&mut *conn)
        .await
        {
            sqlx::query("ROLLBACK").execute(&mut *conn).await.ok();
            return Err(AppError::from(e));
        }

        // Check whether the purchase order is fully received
        let items = match sqlx::query_as::<_, PurchaseOrderItem>(
            "SELECT id, order_id, pipe_type, grade, od, wt, quantity, received_quantity, \
             unit_price, total_price, notes, created_at \
             FROM purchase_order_items WHERE order_id = ? ORDER BY id ASC",
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

        let all_received = items.iter().all(|item| item.received_quantity >= item.quantity);
        if all_received {
            if let Err(e) = sqlx::query(
                "UPDATE purchase_orders SET status = 'completed', updated_at = NOW() \
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
