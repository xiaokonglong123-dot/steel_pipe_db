use chrono::Utc;
use sqlx::SqlitePool;

use crate::domain::order::OrderStatus;
use crate::dto::common::PaginationParams;
use crate::dto::purchase_dto::{
    CreatePurchaseOrderRequest, PurchaseOrderFilterParams, PurchaseOrderStatusTransitionRequest,
    UpdatePurchaseItemRequest, UpdatePurchaseOrderRequest,
};
use crate::dto::purchase_dto::{
    ApproveOrderRequest as PurchaseApproveReq, LinkInboundRequest, RejectOrderRequest as PurchaseRejectReq,
};
use crate::dto::sales_dto::{
    ApproveOrderRequest as SalesApproveReq, CreateSalesOrderRequest, LinkOutboundRequest,
    RejectOrderRequest as SalesRejectReq, SalesOrderFilterParams, SalesOrderStatusTransitionRequest,
    UpdateSalesItemRequest, UpdateSalesOrderRequest,
};
use crate::error::AppError;
use crate::models::purchase_order::{PurchaseOrder, PurchaseOrderItem};
use crate::models::sales_order::{SalesOrder, SalesOrderItem};
use crate::repositories::customer_repo::CustomerRepo;
use crate::repositories::inventory_repo::{InboundRepo, InventoryRepo, OutboundRepo};
use crate::repositories::purchase_order_repo::PurchaseOrderRepo;
use crate::repositories::sales_order_repo::SalesOrderRepo;
use crate::repositories::supplier_repo::SupplierRepo;

pub struct PurchaseSalesService;

impl PurchaseSalesService {
    fn generate_order_no(prefix: &str) -> String {
        let now = Utc::now();
        let date_str = now.format("%Y%m%d").to_string();
        let timestamp = now.format("%H%M%S").to_string();
        let serial: String = (now.timestamp_subsec_millis() % 1000).to_string();
        format!("{}-{}-{}{}", prefix, date_str, timestamp, serial)
    }

    fn validate_status_transition(
        current: &str,
        target: &str,
    ) -> Result<(), AppError> {
        let current_status = OrderStatus::from_str(current).ok_or_else(|| {
            AppError::Validation(format!("Invalid current status: {}", current))
        })?;
        let target_status = OrderStatus::from_str(target).ok_or_else(|| {
            AppError::Validation(format!("Invalid target status: {}", target))
        })?;

        if !current_status.valid_transition(&target_status) {
            return Err(AppError::OrderCannotModify(format!(
                "Cannot transition from '{}' to '{}'",
                current, target
            )));
        }
        Ok(())
    }

    // ━━━ Purchase Orders ━━━

    pub async fn create_purchase_order(
        pool: &SqlitePool,
        dto: &CreatePurchaseOrderRequest,
    ) -> Result<PurchaseOrder, AppError> {
        if dto.items.is_empty() {
            return Err(AppError::Validation(
                "At least one item is required".into(),
            ));
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
            _ => Self::generate_order_no("PO"),
        };

        PurchaseOrderRepo::create_with_items(pool, dto, &order_no)
            .await
            .map_err(AppError::from)
    }

    pub async fn update_purchase_order(
        pool: &SqlitePool,
        id: i64,
        dto: &UpdatePurchaseOrderRequest,
    ) -> Result<PurchaseOrder, AppError> {
        let existing = PurchaseOrderRepo::find_by_id(pool, id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::OrderNotFound(format!("Purchase order id={} not found", id)))?;

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

        PurchaseOrderRepo::update_order(pool, id, dto)
            .await
            .map_err(AppError::from)
    }

    pub async fn transition_purchase_status(
        pool: &SqlitePool,
        id: i64,
        dto: &PurchaseOrderStatusTransitionRequest,
    ) -> Result<(), AppError> {
        let existing = PurchaseOrderRepo::find_by_id(pool, id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::OrderNotFound(format!("Purchase order id={} not found", id)))?;

        if existing.deleted_at.is_some() {
            return Err(AppError::OrderNotFound(format!(
                "Purchase order id={} has been deleted",
                id
            )));
        }

        Self::validate_status_transition(&existing.status, &dto.status)?;

        PurchaseOrderRepo::update_status(pool, id, &dto.status)
            .await
            .map_err(AppError::from)
    }

    pub async fn get_purchase_order(
        pool: &SqlitePool,
        id: i64,
    ) -> Result<(PurchaseOrder, Vec<PurchaseOrderItem>), AppError> {
        let order = PurchaseOrderRepo::find_by_id(pool, id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::OrderNotFound(format!("Purchase order id={} not found", id)))?;

        let items = PurchaseOrderRepo::find_items(pool, id)
            .await
            .map_err(AppError::from)?;

        Ok((order, items))
    }

    pub async fn list_purchase_orders(
        pool: &SqlitePool,
        filter: &PurchaseOrderFilterParams,
        params: &PaginationParams,
    ) -> Result<(Vec<PurchaseOrder>, u64), AppError> {
        PurchaseOrderRepo::list(pool, filter, params)
            .await
            .map_err(AppError::from)
    }

    pub async fn delete_purchase_order(
        pool: &SqlitePool,
        id: i64,
    ) -> Result<(), AppError> {
        let existing = PurchaseOrderRepo::find_by_id(pool, id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::OrderNotFound(format!("Purchase order id={} not found", id)))?;

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

    pub async fn update_purchase_item(
        pool: &SqlitePool,
        order_id: i64,
        item_id: i64,
        dto: &UpdatePurchaseItemRequest,
    ) -> Result<(PurchaseOrder, PurchaseOrderItem), AppError> {
        let order = PurchaseOrderRepo::find_by_id(pool, order_id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::OrderNotFound(format!("Purchase order id={} not found", order_id)))?;

        if order.status != "draft" {
            return Err(AppError::OrderCannotModify(format!(
                "Cannot modify items on order with status '{}'",
                order.status
            )));
        }

        let item = PurchaseOrderRepo::update_item(pool, item_id, dto)
            .await
            .map_err(AppError::from)?;

        Ok((order, item))
    }

    pub async fn delete_purchase_item(
        pool: &SqlitePool,
        order_id: i64,
        item_id: i64,
    ) -> Result<(), AppError> {
        let order = PurchaseOrderRepo::find_by_id(pool, order_id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::OrderNotFound(format!("Purchase order id={} not found", order_id)))?;

        if order.status != "draft" {
            return Err(AppError::OrderCannotModify(format!(
                "Cannot delete items from order with status '{}'",
                order.status
            )));
        }

        PurchaseOrderRepo::delete_item(pool, item_id)
            .await
            .map_err(AppError::from)
    }

    // ━━━ Sales Orders ━━━

    pub async fn create_sales_order(
        pool: &SqlitePool,
        dto: &CreateSalesOrderRequest,
    ) -> Result<SalesOrder, AppError> {
        if dto.items.is_empty() {
            return Err(AppError::Validation(
                "At least one item is required".into(),
            ));
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

    pub async fn list_sales_orders(
        pool: &SqlitePool,
        filter: &SalesOrderFilterParams,
        params: &PaginationParams,
    ) -> Result<(Vec<SalesOrder>, u64), AppError> {
        SalesOrderRepo::list(pool, filter, params)
            .await
            .map_err(AppError::from)
    }

    pub async fn delete_sales_order(
        pool: &SqlitePool,
        id: i64,
    ) -> Result<(), AppError> {
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

    pub async fn update_sales_item(
        pool: &SqlitePool,
        order_id: i64,
        item_id: i64,
        dto: &UpdateSalesItemRequest,
    ) -> Result<(SalesOrder, SalesOrderItem), AppError> {
        let order = SalesOrderRepo::find_by_id(pool, order_id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::OrderNotFound(format!("Sales order id={} not found", order_id)))?;

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

    pub async fn delete_sales_item(
        pool: &SqlitePool,
        order_id: i64,
        item_id: i64,
    ) -> Result<(), AppError> {
        let order = SalesOrderRepo::find_by_id(pool, order_id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::OrderNotFound(format!("Sales order id={} not found", order_id)))?;

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

    // ━━━ Purchase Order Approve / Reject / Link ━━━

    pub async fn approve_purchase_order(
        pool: &SqlitePool,
        id: i64,
        _dto: &PurchaseApproveReq,
    ) -> Result<(), AppError> {
        let existing = PurchaseOrderRepo::find_by_id(pool, id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::OrderNotFound(format!("Purchase order id={} not found", id)))?;

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

        PurchaseOrderRepo::update_status(pool, id, "approved")
            .await
            .map_err(AppError::from)
    }

    pub async fn reject_purchase_order(
        pool: &SqlitePool,
        id: i64,
        dto: &PurchaseRejectReq,
    ) -> Result<(), AppError> {
        let existing = PurchaseOrderRepo::find_by_id(pool, id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::OrderNotFound(format!("Purchase order id={} not found", id)))?;

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

        PurchaseOrderRepo::reject(pool, id, &dto.reason)
            .await
            .map_err(AppError::from)
    }

    pub async fn link_inbound_to_order(
        pool: &SqlitePool,
        purchase_order_id: i64,
        dto: &LinkInboundRequest,
    ) -> Result<(), AppError> {
        let existing = PurchaseOrderRepo::find_by_id(pool, purchase_order_id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::OrderNotFound(format!("Purchase order id={} not found", purchase_order_id)))?;

        if existing.deleted_at.is_some() {
            return Err(AppError::OrderNotFound(format!(
                "Purchase order id={} has been deleted",
                purchase_order_id
            )));
        }

        InboundRepo::link_to_order(pool, dto.inbound_record_id, purchase_order_id)
            .await
            .map_err(AppError::from)
    }

    // ━━━ Sales Order Approve / Reject / Link ━━━

    pub async fn approve_sales_order(
        pool: &SqlitePool,
        id: i64,
        _dto: &SalesApproveReq,
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

        for item in &items {
            let atp_rows = InventoryRepo::find_atp(
                pool,
                &Some(item.pipe_type.clone()),
                &Some(item.grade.clone()),
                &None,
            )
            .await
            .map_err(AppError::from)?;

            let available: i64 = atp_rows.iter().map(|(_, _, cnt, _)| cnt).sum();

            if available < item.quantity {
                return Err(AppError::InsufficientStock);
            }
        }

        let mut tx = pool.begin().await.map_err(AppError::from)?;
        sqlx::query("BEGIN IMMEDIATE")
            .execute(&mut *tx)
            .await
            .map_err(AppError::from)?;

        let rows_affected = sqlx::query(
            "UPDATE sales_orders SET status = 'approved', updated_at = datetime('now') \
             WHERE id = ? AND status = 'pending' AND deleted_at IS NULL",
        )
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(AppError::from)?
        .rows_affected();

        if rows_affected == 0 {
            sqlx::query("ROLLBACK")
                .execute(&mut *tx)
                .await
                .map_err(AppError::from)?;
            return Err(AppError::OrderCannotModify(
                "Order status changed or already processed".into(),
            ));
        }

        tx.commit().await.map_err(AppError::from)
    }

    pub async fn reject_sales_order(
        pool: &SqlitePool,
        id: i64,
        dto: &SalesRejectReq,
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

    pub async fn link_outbound_to_order(
        pool: &SqlitePool,
        sales_order_id: i64,
        dto: &LinkOutboundRequest,
    ) -> Result<(), AppError> {
        let existing = SalesOrderRepo::find_by_id(pool, sales_order_id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::OrderNotFound(format!("Sales order id={} not found", sales_order_id)))?;

        if existing.deleted_at.is_some() {
            return Err(AppError::OrderNotFound(format!(
                "Sales order id={} has been deleted",
                sales_order_id
            )));
        }

        OutboundRepo::link_to_order(pool, dto.outbound_record_id, sales_order_id)
            .await
            .map_err(AppError::from)
    }
}
