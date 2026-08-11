//! Inventory service — 入库/出库/库存/日志业务规则
//!
//! 关键事务约束：`post_inbound` / `post_outbound` 的所有多表写
//! （inventory 余额更新 + inventory_logs + 订单状态置 posted）必须在**单个 sqlx 事务**内完成。
//! 事务性做法：repo 中 `upsert_inventory_increment` / `upsert_inventory_decrement` /
//! `insert_log` / `update_*_status_tx` / `list_*_items_for_order_tx` 均对 `sqlx::Executor` 泛型化，
//! service 用 `&mut tx` 调用它们；任一步骤抛错则事务在 drop 时自动回滚，订单仍为 'draft'。

use chrono::Utc;
use sqlx::SqlitePool;

use crate::error::{AppError, ErrorCode};
use crate::middleware::auth::AuthUser;
use crate::repos::catalog_repo;
use crate::repos::check_repo;
use crate::repos::check_repo::{CheckDetailRow, CheckSessionRow};
use crate::repos::inventory_repo;
use crate::repos::sales_repo;
use crate::repos::inventory_repo::{
    InboundOrderFilter, InboundOrderItemRow, InboundOrderRow, InventoryLogFilter, InventoryLogRow,
    OutboundOrderFilter, OutboundOrderItemRow, OutboundOrderRow, StockFilter, StockRow,
};

// —— DTOs ——

#[derive(Debug, Clone)]
pub struct CreateInboundItemInput {
    pub item_id: i64,
    pub location_id: i64,
    pub quantity: f64,
    pub notes: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CreateInboundRequest {
    pub inbound_type: String,
    pub order_id: Option<i64>,
    pub supplier_id: Option<i64>,
    pub notes: Option<String>,
    pub items: Vec<CreateInboundItemInput>,
}

#[derive(Debug, Clone)]
pub struct CreateOutboundItemInput {
    pub item_id: i64,
    pub location_id: i64,
    pub quantity: f64,
    pub notes: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CreateOutboundRequest {
    pub outbound_type: String,
    pub order_id: Option<i64>,
    pub customer_id: Option<i64>,
    pub notes: Option<String>,
    pub items: Vec<CreateOutboundItemInput>,
}

#[derive(Debug, Clone)]
pub struct CheckSessionCreateInput {
    pub location_id: i64,
    pub scope: String,
}

// —— Order number helper ——

/// 生成入库单号：`IN{YYYYMMDD}-{rand4hex}`。
/// schema 无 UNIQUE 约束随机后缀足以避免冲突；如需严格序列可后续加 sequence 表。
fn generate_inbound_no() -> String {
    let date = Utc::now().format("%Y%m%d").to_string();
    let rand = uuid::Uuid::new_v4().simple().to_string();
    format!("IN{date}-{}", &rand[..4])
}

/// 生成出库单号：`OUT{YYYYMMDD}-{rand4hex}`
fn generate_outbound_no() -> String {
    let date = Utc::now().format("%Y%m%d").to_string();
    let rand = uuid::Uuid::new_v4().simple().to_string();
    format!("OUT{date}-{}", &rand[..4])
}

// —— Inbound services ——

pub async fn create_inbound(
    pool: &SqlitePool,
    dto: &CreateInboundRequest,
    user: &AuthUser,
) -> Result<InboundOrderRow, AppError> {
    validate_inbound_type(&dto.inbound_type)?;
    if dto.items.is_empty() {
        return Err(AppError::validation("入库明细不能为空"));
    }

    let mut seen: Vec<(i64, i64)> = Vec::with_capacity(dto.items.len());
    for it in &dto.items {
        if it.quantity <= 0.0 {
            return Err(AppError::validation("入库数量必须大于 0"));
        }
        if catalog_repo::find_by_id(pool, it.item_id).await?.is_none() {
            return Err(AppError::new(
                ErrorCode::ItemNotFound,
                format!("商品 {} 不存在", it.item_id),
            ));
        }
        // locations 表没有 is_active 字段判断位，仅校验存在 + 未软删。
        if sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM locations WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(it.location_id)
        .fetch_one(pool)
        .await?
            == 0
        {
            return Err(AppError::new(
                ErrorCode::LocationNotFound,
                format!("库位 {} 不存在", it.location_id),
            ));
        }
        if seen
            .iter()
            .any(|(i, l)| *i == it.item_id && *l == it.location_id)
        {
            return Err(AppError::validation(format!(
                "入库明细中商品 {} 在库位 {} 重复",
                it.item_id, it.location_id
            )));
        }
        seen.push((it.item_id, it.location_id));
    }

    let record_no = generate_inbound_no();
    let order = inventory_repo::create_inbound_order(
        pool,
        &record_no,
        &dto.inbound_type,
        dto.order_id,
        dto.supplier_id,
        Some(user.id),
        dto.notes.as_deref(),
    )
    .await?;

    for it in &dto.items {
        inventory_repo::insert_inbound_item(
            pool,
            order.id,
            it.item_id,
            Some(it.location_id),
            it.quantity,
            it.notes.as_deref(),
        )
        .await?;
    }

    Ok(order)
}

/// 过账入库：在单个事务中：校验状态→对每行明细则增库存+写日志→订单置 posted。
/// 库存增/日志/订单状态写均走同一 `&mut tx`，失败自动回滚。
pub async fn post_inbound(
    pool: &SqlitePool,
    inbound_id: i64,
    _user: &AuthUser,
) -> Result<InboundOrderRow, AppError> {
    let order = inventory_repo::get_inbound_order_by_id(pool, inbound_id)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::OrderNotFound, "入库单未找到"))?;
    if order.status != "draft" {
        return Err(AppError::new(
            ErrorCode::OrderCannotModify,
            format!("入库单当前状态为 {}，不可过账", order.status),
        ));
    }

    let mut tx = pool.begin().await?;

    let items = inventory_repo::list_inbound_items_for_order_tx(&mut *tx, inbound_id).await?;
    if items.is_empty() {
        return Err(AppError::validation("入库明细为空，无法过账"));
    }

    for it in items {
        let location_id = it
            .location_id
            .ok_or_else(|| AppError::validation(format!("入库明细 {} 缺失库位", it.id)))?;
        inventory_repo::upsert_inventory_increment(&mut *tx, it.item_id, location_id, it.quantity)
            .await?;
        let balance_after =
            inventory_repo::get_balance_for_item_at_location_tx(&mut *tx, it.item_id, location_id)
                .await?;
        let notes = format!(
            "balance_after={balance_after}{}",
            it.notes
                .as_deref()
                .map(|n| format!("; note={n}"))
                .unwrap_or_default()
        );
        inventory_repo::insert_log(
            &mut *tx,
            it.item_id,
            Some(location_id),
            "inbound",
            it.quantity,
            Some("inbound"),
            Some(inbound_id),
            Some(&notes),
            order.created_by,
        )
        .await?;
    }

    inventory_repo::update_inbound_status_tx(&mut *tx, inbound_id, "posted").await?;

    tx.commit().await?;

    inventory_repo::get_inbound_order_by_id(pool, inbound_id)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::Internal, "过账后读取入库单失败"))
}

pub async fn list_inbounds(
    pool: &SqlitePool,
    filter: &InboundOrderFilter,
    page: i64,
    page_size: i64,
) -> Result<(Vec<InboundOrderRow>, i64), AppError> {
    inventory_repo::list_inbound_orders(pool, filter, page, page_size).await
}

pub async fn get_inbound(pool: &SqlitePool, id: i64) -> Result<Option<InboundOrderRow>, AppError> {
    inventory_repo::get_inbound_order_by_id(pool, id).await
}

pub async fn get_inbound_with_items(
    pool: &SqlitePool,
    id: i64,
) -> Result<Option<(InboundOrderRow, Vec<InboundOrderItemRow>)>, AppError> {
    let order = inventory_repo::get_inbound_order_by_id(pool, id).await?;
    match order {
        Some(o) => {
            let items = inventory_repo::list_inbound_items_for_order(pool, id).await?;
            Ok(Some((o, items)))
        }
        None => Ok(None),
    }
}

// —— Outbound services ——

pub async fn create_outbound(
    pool: &SqlitePool,
    dto: &CreateOutboundRequest,
    user: &AuthUser,
) -> Result<OutboundOrderRow, AppError> {
    validate_outbound_type(&dto.outbound_type)?;
    if dto.items.is_empty() {
        return Err(AppError::validation("出库明细不能为空"));
    }

    let mut seen: Vec<(i64, i64)> = Vec::with_capacity(dto.items.len());
    for it in &dto.items {
        if it.quantity <= 0.0 {
            return Err(AppError::validation("出库数量必须大于 0"));
        }
        if catalog_repo::find_by_id(pool, it.item_id).await?.is_none() {
            return Err(AppError::new(
                ErrorCode::ItemNotFound,
                format!("商品 {} 不存在", it.item_id),
            ));
        }
        if sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM locations WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(it.location_id)
        .fetch_one(pool)
        .await?
            == 0
        {
            return Err(AppError::new(
                ErrorCode::LocationNotFound,
                format!("库位 {} 不存在", it.location_id),
            ));
        }
        if seen
            .iter()
            .any(|(i, l)| *i == it.item_id && *l == it.location_id)
        {
            return Err(AppError::validation(format!(
                "出库明细中商品 {} 在库位 {} 重复",
                it.item_id, it.location_id
            )));
        }
        seen.push((it.item_id, it.location_id));
    }

    let record_no = generate_outbound_no();
    let order = inventory_repo::create_outbound_order(
        pool,
        &record_no,
        &dto.outbound_type,
        dto.order_id,
        dto.customer_id,
        Some(user.id),
        dto.notes.as_deref(),
    )
    .await?;

    for it in &dto.items {
        inventory_repo::insert_outbound_item(
            pool,
            order.id,
            it.item_id,
            Some(it.location_id),
            it.quantity,
            it.notes.as_deref(),
        )
        .await?;
    }

    Ok(order)
}

/// 过账出库：单个事务内逐行校验库存足额→减库存→写日志→订单置 posted。
/// 任一行库存不足抛 `InsufficientStock`，事务回滚，订单仍为 draft。
pub async fn post_outbound(
    pool: &SqlitePool,
    outbound_id: i64,
    _user: &AuthUser,
) -> Result<OutboundOrderRow, AppError> {
    let order = inventory_repo::get_outbound_order_by_id(pool, outbound_id)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::OrderNotFound, "出库单未找到"))?;
    if order.status != "draft" {
        return Err(AppError::new(
            ErrorCode::OrderCannotModify,
            format!("出库单当前状态为 {}，不可过账", order.status),
        ));
    }

    let mut tx = pool.begin().await?;

    let items = inventory_repo::list_outbound_items_for_order_tx(&mut *tx, outbound_id).await?;
    if items.is_empty() {
        return Err(AppError::validation("出库明细为空，无法过账"));
    }

    for it in items {
        let location_id = it
            .location_id
            .ok_or_else(|| AppError::validation(format!("出库明细 {} 缺失库位", it.id)))?;
        let balance =
            inventory_repo::get_balance_for_item_at_location_tx(&mut *tx, it.item_id, location_id)
                .await?;
        if balance < it.quantity {
            return Err(AppError::new(
                ErrorCode::InsufficientStock,
                format!(
                    "库存不足：商品 {} 在库位 {} 余量 {}，需出库 {}",
                    it.item_id, location_id, balance, it.quantity
                ),
            ));
        }
        inventory_repo::upsert_inventory_decrement(&mut *tx, it.item_id, location_id, it.quantity)
            .await?;
        let balance_after =
            inventory_repo::get_balance_for_item_at_location_tx(&mut *tx, it.item_id, location_id)
                .await?;
        let notes = format!(
            "balance_after={balance_after}{}",
            it.notes
                .as_deref()
                .map(|n| format!("; note={n}"))
                .unwrap_or_default()
        );
        inventory_repo::insert_log(
            &mut *tx,
            it.item_id,
            Some(location_id),
            "outbound",
            -it.quantity,
            Some("outbound"),
            Some(outbound_id),
            Some(&notes),
            order.created_by,
        )
        .await?;
    }

    inventory_repo::update_outbound_status_tx(&mut *tx, outbound_id, "posted").await?;

    tx.commit().await?;

    inventory_repo::get_outbound_order_by_id(pool, outbound_id)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::Internal, "过账后读取出库单失败"))
}

pub async fn list_outbounds(
    pool: &SqlitePool,
    filter: &OutboundOrderFilter,
    page: i64,
    page_size: i64,
) -> Result<(Vec<OutboundOrderRow>, i64), AppError> {
    inventory_repo::list_outbound_orders(pool, filter, page, page_size).await
}

pub async fn get_outbound(
    pool: &SqlitePool,
    id: i64,
) -> Result<Option<OutboundOrderRow>, AppError> {
    inventory_repo::get_outbound_order_by_id(pool, id).await
}

pub async fn get_outbound_with_items(
    pool: &SqlitePool,
    id: i64,
) -> Result<Option<(OutboundOrderRow, Vec<OutboundOrderItemRow>)>, AppError> {
    let order = inventory_repo::get_outbound_order_by_id(pool, id).await?;
    match order {
        Some(o) => {
            let items = inventory_repo::list_outbound_items_for_order(pool, id).await?;
            Ok(Some((o, items)))
        }
        None => Ok(None),
    }
}

// —— Stock / logs services ——

pub async fn get_stock_at(
    pool: &SqlitePool,
    item_id: i64,
    location_id: i64,
) -> Result<f64, AppError> {
    inventory_repo::get_balance_for_item_at_location(pool, item_id, location_id).await
}

pub async fn list_stock(
    pool: &SqlitePool,
    filter: &StockFilter,
    page: i64,
    page_size: i64,
) -> Result<(Vec<StockRow>, i64), AppError> {
    inventory_repo::list_stock(pool, filter, page, page_size).await
}

pub async fn list_logs(
    pool: &SqlitePool,
    filter: &InventoryLogFilter,
    page: i64,
    page_size: i64,
) -> Result<(Vec<InventoryLogRow>, i64), AppError> {
    inventory_repo::list_logs(pool, filter, page, page_size).await
}

fn generate_check_no() -> String {
    let date = Utc::now().format("%Y%m%d").to_string();
    let rand = uuid::Uuid::new_v4().simple().to_string();
    format!("CHK{date}-{}", &rand[..4])
}

pub async fn create_check_session(
    pool: &SqlitePool,
    input: &CheckSessionCreateInput,
    user: &AuthUser,
) -> Result<CheckSessionRow, AppError> {
    if input.scope.trim().is_empty() {
        return Err(AppError::validation("盘点范围不能为空"));
    }
    let location_exists: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM locations WHERE id = ? AND deleted_at IS NULL")
            .bind(input.location_id)
            .fetch_one(pool)
            .await?;
    if location_exists == 0 {
        return Err(AppError::new(ErrorCode::LocationNotFound, "库位未找到"));
    }

    let mut tx = pool.begin().await?;
    let session_id =
        check_repo::insert_session_tx(&mut *tx, &generate_check_no(), input.location_id, user.id)
            .await?;
    let snapshot = check_repo::system_snapshot_for_location_tx(&mut *tx, input.location_id).await?;
    for (item_id, _, system_qty) in snapshot {
        check_repo::insert_detail(&mut *tx, session_id, item_id, system_qty).await?;
    }
    tx.commit().await?;

    check_repo::find_session_by_id(pool, session_id)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::Internal, "盘点单创建后读取失败"))
}

pub async fn record_actual_qty(
    pool: &SqlitePool,
    session_id: i64,
    detail_id: i64,
    actual_qty: f64,
    _user: &AuthUser,
) -> Result<(), AppError> {
    if !actual_qty.is_finite() || actual_qty < 0.0 {
        return Err(AppError::validation("实盘数量必须为非负有限数"));
    }
    let session = check_repo::find_session_by_id(pool, session_id)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::CheckNotFound, "盘点单未找到"))?;
    if !matches!(session.status.as_str(), "draft" | "counted") {
        return Err(AppError::new(
            ErrorCode::CheckNotDraft,
            format!("盘点单当前状态为 {}，不可录入", session.status),
        ));
    }
    let detail = check_repo::find_detail_by_id(pool, detail_id)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::CheckNotFound, "盘点明细未找到"))?;
    if detail.session_id != session_id {
        return Err(AppError::new(
            ErrorCode::CheckNotFound,
            "盘点明细不属于该盘点单",
        ));
    }
    check_repo::update_actual_qty(pool, detail_id, actual_qty).await?;
    check_repo::update_session_status(pool, session_id, "counted").await
}

pub async fn post_check_session(
    pool: &SqlitePool,
    session_id: i64,
    user: &AuthUser,
) -> Result<CheckSessionRow, AppError> {
    let session = check_repo::find_session_by_id(pool, session_id)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::CheckNotFound, "盘点单未找到"))?;
    if session.status == "draft" {
        return Err(AppError::validation("需先录入实盘数量"));
    }
    if session.status != "counted" {
        return Err(AppError::new(
            ErrorCode::CheckNotDraft,
            "盘点单需先录入实盘数量后再过账",
        ));
    }
    let details = check_repo::list_details_for_session(pool, session_id).await?;
    if !details.iter().any(|detail| detail.actual_qty.is_some()) {
        return Err(AppError::validation("需先录入实盘数量"));
    }

    let mut tx = pool.begin().await?;
    for detail in details {
        let Some(actual_qty) = detail.actual_qty else {
            continue;
        };
        let diff = actual_qty - detail.system_qty;
        if diff.abs() <= 0.0001 {
            continue;
        }
        let location_id = detail
            .location_id
            .ok_or_else(|| AppError::validation("盘点明细缺少库位"))?;
        inventory_repo::upsert_inventory_increment(&mut *tx, detail.item_id, location_id, diff)
            .await?;
        inventory_repo::insert_log(
            &mut *tx,
            detail.item_id,
            Some(location_id),
            "check_adjust",
            diff,
            Some("check"),
            Some(session_id),
            None,
            Some(user.id),
        )
        .await?;
    }
    check_repo::update_session_status_tx(&mut *tx, session_id, "posted").await?;
    tx.commit().await?;

    check_repo::find_session_by_id(pool, session_id)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::Internal, "盘点单过账后读取失败"))
}

pub async fn list_check_sessions(
    pool: &SqlitePool,
    page: i64,
    page_size: i64,
) -> Result<(Vec<CheckSessionRow>, i64), AppError> {
    check_repo::list_sessions(pool, page, page_size).await
}

pub async fn get_check_session(
    pool: &SqlitePool,
    id: i64,
) -> Result<(CheckSessionRow, Vec<CheckDetailRow>), AppError> {
    let session = check_repo::find_session_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::CheckNotFound, "盘点单未找到"))?;
    let details = check_repo::list_details_for_session(pool, id).await?;
    Ok((session, details))
}

// —— Type validation ——

fn validate_inbound_type(t: &str) -> Result<(), AppError> {
    if matches!(t, "purchase" | "production" | "return" | "other") {
        Ok(())
    } else {
        Err(AppError::validation(
            "inbound_type 取值必须为 purchase/production/return/other",
        ))
    }
}

fn validate_outbound_type(t: &str) -> Result<(), AppError> {
    if matches!(t, "sales" | "requisition" | "other") {
        Ok(())
    } else {
        Err(AppError::validation(
            "outbound_type 取值必须为 sales/requisition/other",
        ))
    }
}

/// ATP 可用量查询：库存余额 - 已 active 预留
///
/// 公式：available = inventory.quantity - SUM(reservations.quantity WHERE status='active')
pub async fn get_available_qty(
    pool: &SqlitePool,
    item_id: i64,
    location_id: Option<i64>,
) -> Result<f64, AppError> {
    let balance: f64 = match location_id {
        Some(loc) => {
            inventory_repo::get_balance_for_item_at_location(pool, item_id, loc).await?
        }
        None => {
            inventory_repo::get_balance_for_item(pool, item_id).await?
        }
    };
    let reserved = sales_repo::sum_active_reservations_for_item(pool, item_id).await?;
    Ok(balance - reserved)
}
