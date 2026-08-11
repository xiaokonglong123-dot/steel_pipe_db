use sqlx::SqlitePool;

use crate::error::{AppError, ErrorCode};
use crate::middleware::auth::AuthUser;
use crate::repos::{inventory_repo, receivable_repo, sales_repo};

#[derive(Debug, Clone)]
pub struct ShippedItemInput {
    pub item_id: i64,
    pub location_id: i64,
    pub quantity: f64,
}

pub async fn ship_sales_order(
    pool: &SqlitePool,
    so_id: i64,
    shipped_items: &[ShippedItemInput],
    user: &AuthUser,
) -> Result<inventory_repo::OutboundOrderRow, AppError> {
    let order = sales_repo::find_by_id(pool, so_id)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::OrderNotFound, "销售订单未找到"))?;
    if order.status != "approved" {
        return Err(AppError::new(
            ErrorCode::OrderCannotModify,
            "销售订单未审批",
        ));
    }
    if shipped_items.is_empty() {
        return Err(AppError::validation("发货明细不能为空"));
    }

    let mut tx = pool.begin().await?;
    let record_no = format!("{}-S1", order.order_no);
    let record_id =
        receivable_repo::insert_outbound(&mut *tx, &record_no, so_id, order.customer_id, user.id)
            .await?;
    for item in shipped_items {
        if item.quantity <= 0.0 {
            return Err(AppError::validation("发货数量必须大于 0"));
        }
        let balance = inventory_repo::get_balance_for_item_at_location_tx(
            &mut *tx,
            item.item_id,
            item.location_id,
        )
        .await?;
        if balance < item.quantity {
            return Err(AppError::new(
                ErrorCode::InsufficientStock,
                format!(
                    "库存不足：商品 {} 在库位 {} 余量 {}，需发货 {}",
                    item.item_id, item.location_id, balance, item.quantity
                ),
            ));
        }
        receivable_repo::insert_outbound_item(
            &mut *tx,
            record_id,
            item.item_id,
            item.location_id,
            item.quantity,
        )
        .await?;
        inventory_repo::upsert_inventory_decrement(
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
            "outbound",
            -item.quantity,
            Some("sales_order"),
            Some(so_id),
            Some("// TODO P1: post GL entries for shipped inventory"),
            Some(user.id),
        )
        .await?;
        sqlx::query(
            "UPDATE sales_order_items
             SET shipped_qty = shipped_qty + ?
             WHERE order_id = ? AND item_id = ?",
        )
        .bind(item.quantity)
        .bind(so_id)
        .bind(item.item_id)
        .execute(&mut *tx)
        .await?;
    }
    sales_repo::release_reservations_for_order_tx(&mut *tx, so_id).await?;
    sqlx::query("UPDATE sales_orders SET status = 'shipped', doc_status = 3, updated_at = datetime('now') WHERE id = ?")
        .bind(so_id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    receivable_repo::find_outbound(pool, record_id)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::Internal, "发货单创建后读取失败"))
}
