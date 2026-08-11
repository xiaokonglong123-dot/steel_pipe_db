use sqlx::SqlitePool;

use crate::error::{AppError, ErrorCode};
use crate::middleware::auth::AuthUser;
use crate::repos::{catalog_repo, inventory_repo, purchase_repo, receivable_repo};

#[derive(Debug, Clone)]
pub struct ReceivedItemInput {
    pub item_id: i64,
    pub location_id: i64,
    pub quantity: f64,
}

pub async fn receive_purchase_order(
    pool: &SqlitePool,
    po_id: i64,
    received_items: &[ReceivedItemInput],
    user: &AuthUser,
) -> Result<inventory_repo::InboundOrderRow, AppError> {
    let order = purchase_repo::find_by_id(pool, po_id)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::OrderNotFound, "采购订单未找到"))?;
    if order.status != "approved" {
        return Err(AppError::new(
            ErrorCode::OrderCannotModify,
            "采购订单未审批",
        ));
    }
    if received_items.is_empty() {
        return Err(AppError::validation("收货明细不能为空"));
    }

    for item in received_items {
        if item.quantity <= 0.0 {
            return Err(AppError::validation("收货数量必须大于 0"));
        }
        if catalog_repo::find_by_id(pool, item.item_id)
            .await?
            .is_none()
        {
            return Err(AppError::new(
                ErrorCode::ItemNotFound,
                format!("商品 {} 不存在", item.item_id),
            ));
        }
        let location_exists: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM locations WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(item.location_id)
        .fetch_one(pool)
        .await?;
        if location_exists == 0 {
            return Err(AppError::new(
                ErrorCode::LocationNotFound,
                format!("库位 {} 不存在", item.location_id),
            ));
        }
    }

    let mut tx = pool.begin().await?;
    let record_no = format!("{}-R1", order.order_no);
    let record_id =
        receivable_repo::insert_inbound(&mut *tx, &record_no, po_id, order.supplier_id, user.id)
            .await?;

    for item in received_items {
        receivable_repo::insert_inbound_item(
            &mut *tx,
            record_id,
            item.item_id,
            item.location_id,
            item.quantity,
        )
        .await?;
        inventory_repo::upsert_inventory_increment(
            &mut *tx,
            item.item_id,
            item.location_id,
            item.quantity,
        )
        .await?;
        inventory_repo::insert_log(
            &mut *tx,
            item.item_id,
            Some(item.location_id),
            "inbound",
            item.quantity,
            Some("purchase_order"),
            Some(po_id),
            Some("// TODO P1: post GL entries for received inventory"),
            Some(user.id),
        )
        .await?;
        sqlx::query(
            "UPDATE purchase_order_items
             SET received_qty = received_qty + ?
             WHERE order_id = ? AND item_id = ?",
        )
        .bind(item.quantity)
        .bind(po_id)
        .bind(item.item_id)
        .execute(&mut *tx)
        .await?;
    }

    let remaining: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM purchase_order_items
         WHERE order_id = ? AND received_qty < quantity",
    )
    .bind(po_id)
    .fetch_one(&mut *tx)
    .await?;
    let next_status = if remaining > 0 {
        "partially_received"
    } else {
        "received"
    };
    sqlx::query(
        "UPDATE purchase_orders SET status = ?, updated_at = datetime('now')
         WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(next_status)
    .bind(po_id)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    receivable_repo::find_inbound(pool, record_id)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::Internal, "收货单创建后读取失败"))
}
