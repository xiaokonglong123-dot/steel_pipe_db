//! Sales service — 销售订单业务规则 + ATP 预留
//!
//! 关键事务约束：
//! - `create_order`：订单头+明细在单个事务内写入（status='draft'）
//! - `submit`：先做 ATP 校验（库存余额 - 已 active 预留 >= 本订单每行 qty），
//!   通过后在单个事务内插入 reservations(active) + 订单置 submitted
//! - `cancel`：若订单原为 submitted（已预留），在事务内释放 reservations
//! 任一步骤抛错则事务回滚，订单状态不变。

use chrono::Utc;
use rust_decimal::Decimal;
use sqlx::SqlitePool;

use crate::error::{AppError, ErrorCode};
use crate::middleware::auth::AuthUser;
use crate::repos::catalog_repo;
use crate::repos::parties_repo;
use crate::repos::sales_repo;
use crate::repos::sales_repo::{
    ReservationRow, SalesOrderFilter, SalesOrderItemRow, SalesOrderRow,
};
use crate::repos::workflow_repo;
use crate::services::workflow_service;

// —— DTOs ——

#[derive(Debug, Clone)]
pub struct CreateSalesOrderItemInput {
    pub item_id: i64,
    pub quantity: f64,
    pub unit_price: String,
    pub notes: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CreateSalesOrderRequest {
    pub customer_id: i64,
    pub order_date: Option<String>,
    pub currency: Option<String>,
    pub notes: Option<String>,
    pub items: Vec<CreateSalesOrderItemInput>,
}

#[derive(Debug, Clone)]
pub struct UpdateSalesOrderRequest {
    pub customer_id: i64,
    pub order_date: Option<String>,
    pub currency: Option<String>,
    pub notes: Option<String>,
    pub items: Vec<CreateSalesOrderItemInput>,
}

// —— Order number helper ——

/// 生成销售订单号：`SO{YYYYMMDD}-{rand4hex}`
fn generate_order_no() -> String {
    let date = Utc::now().format("%Y%m%d").to_string();
    let rand = uuid::Uuid::new_v4().simple().to_string();
    format!("SO{date}-{}", &rand[..4])
}

// —— Status constants ——

const STATUS_DRAFT: &str = "draft";
const STATUS_SUBMITTED: &str = "submitted";
const STATUS_APPROVED: &str = "approved";
const STATUS_REJECTED: &str = "rejected";
const STATUS_CANCELLED: &str = "cancelled";

// doc_status: 0=draft, 1=submitted, 2=approved/rejected, 3=cancelled
const DOC_DRAFT: i64 = 0;
const DOC_SUBMITTED: i64 = 1;
const DOC_APPROVED: i64 = 2;
const DOC_CANCELLED: i64 = 3;

// —— Services ——

pub async fn create_order(
    pool: &SqlitePool,
    dto: &CreateSalesOrderRequest,
    user: &AuthUser,
) -> Result<SalesOrderRow, AppError> {
    if dto.items.is_empty() {
        return Err(AppError::validation("销售订单明细不能为空"));
    }

    // 校验客户存在
    if parties_repo::find_customer_by_id(pool, dto.customer_id)
        .await?
        .is_none()
    {
        return Err(AppError::new(
            ErrorCode::CustomerNotFound,
            format!("客户 {} 不存在", dto.customer_id),
        ));
    }

    // 校验每个明细行：商品存在 + qty>0 + unit_price 合法 Decimal
    let mut seen: Vec<i64> = Vec::with_capacity(dto.items.len());
    let mut total = Decimal::ZERO;
    for it in &dto.items {
        if it.quantity <= 0.0 {
            return Err(AppError::validation("销售数量必须大于 0"));
        }
        if catalog_repo::find_by_id(pool, it.item_id).await?.is_none() {
            return Err(AppError::new(
                ErrorCode::ItemNotFound,
                format!("商品 {} 不存在", it.item_id),
            ));
        }
        let price = crate::domain::money::parse_amount(&it.unit_price)?;
        if price < Decimal::ZERO {
            return Err(AppError::validation("单价不能为负"));
        }
        let line_total = price
            * Decimal::try_from(it.quantity).map_err(|_| {
                AppError::validation(format!("数量 {} 无法转换为金额精度", it.quantity))
            })?;
        total += line_total;
        if seen.iter().any(|i| *i == it.item_id) {
            return Err(AppError::validation(format!(
                "销售明细中商品 {} 重复",
                it.item_id
            )));
        }
        seen.push(it.item_id);
    }

    let order_no = generate_order_no();
    if sales_repo::find_by_order_no(pool, &order_no)
        .await?
        .is_some()
    {
        return Err(AppError::new(ErrorCode::Internal, "订单号生成冲突，请重试"));
    }
    let order_date = dto
        .order_date
        .clone()
        .unwrap_or_else(|| Utc::now().to_rfc3339());
    let currency = dto.currency.clone().unwrap_or_else(|| "CNY".to_string());
    let total_str = total.to_string();

    let mut tx = pool.begin().await?;

    let order_id = sales_repo::insert_order(
        &mut *tx,
        &order_no,
        dto.customer_id,
        &order_date,
        STATUS_DRAFT,
        DOC_DRAFT,
        &total_str,
        &currency,
        dto.notes.as_deref(),
        Some(user.id),
    )
    .await?;

    for it in &dto.items {
        let price = crate::domain::money::parse_amount(&it.unit_price)?;
        let line_total = price
            * Decimal::try_from(it.quantity)
                .map_err(|_| AppError::validation("数量无法转换为金额精度"))?;
        sales_repo::insert_item(
            &mut *tx,
            order_id,
            it.item_id,
            it.quantity,
            Some(&it.unit_price),
            Some(&line_total.to_string()),
            it.notes.as_deref(),
        )
        .await?;
    }

    tx.commit().await?;

    sales_repo::find_by_id(pool, order_id)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::Internal, "销售订单创建后读取失败"))
}

pub async fn get_order(pool: &SqlitePool, id: i64) -> Result<Option<SalesOrderRow>, AppError> {
    sales_repo::find_by_id(pool, id).await
}

pub async fn get_order_with_items(
    pool: &SqlitePool,
    id: i64,
) -> Result<Option<(SalesOrderRow, Vec<SalesOrderItemRow>)>, AppError> {
    let order = sales_repo::find_by_id(pool, id).await?;
    match order {
        Some(o) => {
            let items = sales_repo::list_items_for_order(pool, id).await?;
            Ok(Some((o, items)))
        }
        None => Ok(None),
    }
}

pub async fn list_orders(
    pool: &SqlitePool,
    filter: &SalesOrderFilter,
    page: i64,
    page_size: i64,
) -> Result<(Vec<SalesOrderRow>, i64), AppError> {
    sales_repo::list_orders(pool, filter, page, page_size).await
}

pub async fn update_order(
    pool: &SqlitePool,
    id: i64,
    dto: &UpdateSalesOrderRequest,
    _user: &AuthUser,
) -> Result<SalesOrderRow, AppError> {
    let order = sales_repo::find_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::OrderNotFound, "销售订单未找到"))?;
    if order.status != STATUS_DRAFT {
        return Err(AppError::new(
            ErrorCode::OrderCannotModify,
            format!("销售订单当前状态为 {}，不可编辑", order.status),
        ));
    }
    if dto.items.is_empty() {
        return Err(AppError::validation("销售订单明细不能为空"));
    }
    if parties_repo::find_customer_by_id(pool, dto.customer_id)
        .await?
        .is_none()
    {
        return Err(AppError::new(
            ErrorCode::CustomerNotFound,
            format!("客户 {} 不存在", dto.customer_id),
        ));
    }

    let mut seen: Vec<i64> = Vec::with_capacity(dto.items.len());
    let mut total = Decimal::ZERO;
    for it in &dto.items {
        if it.quantity <= 0.0 {
            return Err(AppError::validation("销售数量必须大于 0"));
        }
        if catalog_repo::find_by_id(pool, it.item_id).await?.is_none() {
            return Err(AppError::new(
                ErrorCode::ItemNotFound,
                format!("商品 {} 不存在", it.item_id),
            ));
        }
        let price = crate::domain::money::parse_amount(&it.unit_price)?;
        if price < Decimal::ZERO {
            return Err(AppError::validation("单价不能为负"));
        }
        let line_total = price
            * Decimal::try_from(it.quantity)
                .map_err(|_| AppError::validation("数量无法转换为金额精度"))?;
        total += line_total;
        if seen.iter().any(|i| *i == it.item_id) {
            return Err(AppError::validation(format!(
                "销售明细中商品 {} 重复",
                it.item_id
            )));
        }
        seen.push(it.item_id);
    }

    let order_date = dto
        .order_date
        .clone()
        .unwrap_or_else(|| Utc::now().to_rfc3339());
    let currency = dto.currency.clone().unwrap_or_else(|| "CNY".to_string());
    let total_str = total.to_string();

    let mut tx = pool.begin().await?;

    // 全量重写明细
    sales_repo::delete_items_for_order_tx(&mut *tx, id).await?;
    for it in &dto.items {
        let price = crate::domain::money::parse_amount(&it.unit_price)?;
        let line_total = price
            * Decimal::try_from(it.quantity)
                .map_err(|_| AppError::validation("数量无法转换为金额精度"))?;
        sales_repo::insert_item(
            &mut *tx,
            id,
            it.item_id,
            it.quantity,
            Some(&it.unit_price),
            Some(&line_total.to_string()),
            it.notes.as_deref(),
        )
        .await?;
    }

    // 更新订单头
    let result = sqlx::query(
        "UPDATE sales_orders SET customer_id = ?, order_date = ?, total_amount = ?,
           currency = ?, notes = ?, updated_at = datetime('now')
         WHERE id = ? AND deleted_at IS NULL AND status = 'draft'",
    )
    .bind(dto.customer_id)
    .bind(&order_date)
    .bind(&total_str)
    .bind(&currency)
    .bind(dto.notes.as_deref())
    .bind(id)
    .execute(&mut *tx)
    .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::new(
            ErrorCode::OrderCannotModify,
            "销售订单不可编辑（状态已变更）",
        ));
    }

    tx.commit().await?;

    sales_repo::find_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::Internal, "销售订单更新后读取失败"))
}

pub async fn delete_order(pool: &SqlitePool, id: i64) -> Result<(), AppError> {
    sales_repo::soft_delete_order(pool, id).await
}

/// 提交销售订单：ATP 校验 → 单事务内插入 reservations + 置 submitted
pub async fn submit(
    pool: &SqlitePool,
    id: i64,
    user: &AuthUser,
) -> Result<SalesOrderRow, AppError> {
    let order = sales_repo::find_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::OrderNotFound, "销售订单未找到"))?;
    if order.status != STATUS_DRAFT {
        return Err(AppError::new(
            ErrorCode::OrderCannotModify,
            format!("销售订单当前状态为 {}，不可提交", order.status),
        ));
    }

    let items = sales_repo::list_items_for_order(pool, id).await?;
    if items.is_empty() {
        return Err(AppError::validation("销售订单明细为空，无法提交"));
    }

    // ATP 校验：每行 qty <= 库存余额 - 已 active 预留
    for it in &items {
        let balance = sales_repo::sum_balance_for_item(pool, it.item_id).await?;
        let reserved = sales_repo::sum_active_reserved_quantity_for_item(pool, it.item_id).await?;
        let available = balance - reserved;
        if it.quantity > available {
            return Err(AppError::new(
                ErrorCode::InsufficientStock,
                format!(
                    "库存不足：商品 {} 可用 {}（余额 {} - 已预留 {}），本次申请 {}",
                    it.item_id, available, balance, reserved, it.quantity
                ),
            ));
        }
    }

    let workflow = workflow_repo::find_active_workflow_by_type(pool, "sales_order").await?;
    let initial = match &workflow {
        Some(workflow) => workflow_repo::find_initial_state(pool, workflow.id).await?,
        None => None,
    };
    let mut tx = pool.begin().await?;

    for it in &items {
        sales_repo::insert_reservation(&mut *tx, it.item_id, it.quantity, id, Some(user.id))
            .await?;
    }
    sales_repo::update_status_tx(&mut *tx, id, STATUS_SUBMITTED, DOC_SUBMITTED).await?;
    if let (Some(workflow), Some(initial)) = (&workflow, &initial) {
        workflow_service::start_instance_in_tx(&mut tx, workflow, initial, "sales_order", id)
            .await?;
    }

    tx.commit().await?;

    sales_repo::find_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::Internal, "销售订单提交后读取失败"))
}

pub async fn approve(
    pool: &SqlitePool,
    id: i64,
    _user: &AuthUser,
) -> Result<SalesOrderRow, AppError> {
    let order = sales_repo::find_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::OrderNotFound, "销售订单未找到"))?;
    if order.status != STATUS_SUBMITTED {
        return Err(AppError::new(
            ErrorCode::OrderCannotModify,
            format!("销售订单当前状态为 {}，不可审批", order.status),
        ));
    }

    let mut tx = pool.begin().await?;
    sales_repo::update_status_tx(&mut *tx, id, STATUS_APPROVED, DOC_APPROVED).await?;
    tx.commit().await?;

    if let Some(instance) =
        workflow_service::find_active_instance_for(pool, "sales_order", id).await?
    {
        let amount = Decimal::from_str_radix(&order.total_amount, 10).ok();
        if instance.current_state == STATUS_DRAFT {
            workflow_service::transition_with_amount(pool, instance.id, "submit", _user, None, amount).await?;
        }
        workflow_service::transition_with_amount(pool, instance.id, "approve", _user, None, amount).await?;
    }
    sales_repo::find_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::Internal, "销售订单审批后读取失败"))
}

pub async fn reject(
    pool: &SqlitePool,
    id: i64,
    _user: &AuthUser,
) -> Result<SalesOrderRow, AppError> {
    let order = sales_repo::find_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::OrderNotFound, "销售订单未找到"))?;
    if order.status != STATUS_SUBMITTED {
        return Err(AppError::new(
            ErrorCode::OrderCannotModify,
            format!("销售订单当前状态为 {}，不可驳回", order.status),
        ));
    }

    let mut tx = pool.begin().await?;
    sales_repo::update_status_tx(&mut *tx, id, STATUS_REJECTED, DOC_APPROVED).await?;
    // 驳回不释放预留（预留仍保留，待后续 cancel 或重审）
    tx.commit().await?;

    sales_repo::find_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::Internal, "销售订单驳回后读取失败"))
}

pub async fn cancel(
    pool: &SqlitePool,
    id: i64,
    _user: &AuthUser,
) -> Result<SalesOrderRow, AppError> {
    let order = sales_repo::find_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::OrderNotFound, "销售订单未找到"))?;
    if !matches!(order.status.as_str(), "submitted" | "approved") {
        return Err(AppError::new(
            ErrorCode::OrderCannotModify,
            format!("销售订单当前状态为 {}，不可取消", order.status),
        ));
    }

    let mut tx = pool.begin().await?;

    // 若原为 submitted，释放该订单的 active 预留
    if order.status == STATUS_SUBMITTED {
        sales_repo::release_reservations_for_order_tx(&mut *tx, id).await?;
    }
    sales_repo::update_status_tx(&mut *tx, id, STATUS_CANCELLED, DOC_CANCELLED).await?;

    tx.commit().await?;

    sales_repo::find_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::Internal, "销售订单取消后读取失败"))
}

// —— Reservation query ——

pub async fn list_active_reservations_for_item(
    pool: &SqlitePool,
    item_id: i64,
) -> Result<Vec<ReservationRow>, AppError> {
    sales_repo::list_active_reservations_for_item(pool, item_id).await
}
