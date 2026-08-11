//! Purchase service — 采购订单业务规则
//!
//! 状态机（detailed-design §4.5）：
//!   draft → submitted → (approved | rejected)
//!   draft | submitted → cancelled
//! 金额：rust_decimal 全链路；total_amount 以 TEXT 存储（to_string()），不在 SQL 上做 SUM（ADR-002）。
//! 事务：create_order 在单事务内插单头 + 单体行；submit/approve/reject/cancel 走单表 update_status。

use chrono::Utc;
use rust_decimal::Decimal;
use sqlx::SqlitePool;
use std::str::FromStr;

use crate::domain::money::parse_amount;
use crate::domain::order::{DOC_CANCELLED, DOC_DRAFT, DOC_SUBMITTED};
use crate::error::{AppError, ErrorCode};
use crate::middleware::auth::AuthUser;
use crate::repos::purchase_repo::{PurchaseOrderFilter, PurchaseOrderItemRow, PurchaseOrderRow};
use crate::repos::{catalog_repo, parties_repo, purchase_repo, workflow_repo};
use crate::services::workflow_service;

// —— DTOs ——
// 注：purchase_repo 直接复用这里的输入类型（crate 内兄弟模块），避免重复定义。

#[derive(Debug, Clone)]
pub struct PurchaseOrderItemInput {
    pub item_id: i64,
    pub quantity: f64,
    pub unit_price: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CreatePurchaseOrderRequest {
    pub supplier_id: i64,
    pub order_date: String,
    pub currency: Option<String>,
    pub notes: Option<String>,
    pub items: Vec<PurchaseOrderItemInput>,
}

#[derive(Debug, Clone)]
pub struct UpdatePurchaseOrderRequest {
    pub supplier_id: i64,
    pub order_date: String,
    pub currency: Option<String>,
    pub notes: Option<String>,
    pub items: Vec<PurchaseOrderItemInput>,
}

// —— Helpers ——

/// 生成采购订单号：`PO{YYYYMMDD}-{rand4hex}`。schema 有 UNIQUE，service 层在冲突时重试。
fn generate_order_no() -> String {
    let date = Utc::now().format("%Y%m%d").to_string();
    let rand = uuid::Uuid::new_v4().simple().to_string();
    format!("PO{date}-{}", &rand[..4])
}

/// 校验 supplier_id 存在
async fn validate_supplier(pool: &SqlitePool, supplier_id: i64) -> Result<(), AppError> {
    if parties_repo::find_supplier_by_id(pool, supplier_id)
        .await?
        .is_none()
    {
        return Err(AppError::new(
            ErrorCode::SupplierNotFound,
            format!("供应商 {supplier_id} 不存在"),
        ));
    }
    Ok(())
}

/// 校验单体行：item 存在 + quantity > 0 + unit_price（若提供）是合法 Decimal 字符串
async fn validate_items(
    pool: &SqlitePool,
    items: &[PurchaseOrderItemInput],
) -> Result<(), AppError> {
    if items.is_empty() {
        return Err(AppError::validation("采购明细不能为空"));
    }
    for it in items {
        if it.quantity <= 0.0 {
            return Err(AppError::validation("采购数量必须大于 0"));
        }
        if catalog_repo::find_by_id(pool, it.item_id).await?.is_none() {
            return Err(AppError::new(
                ErrorCode::ItemNotFound,
                format!("商品 {} 不存在", it.item_id),
            ));
        }
        if let Some(p) = &it.unit_price {
            // 校验可解析（不强制非负——0 价格允许，仅拒绝非法字符串）
            let _ = parse_amount(p)?;
        }
    }
    Ok(())
}

/// 按 rust_decimal 计算单体行 total_price 与单头 total_amount。
/// `line_total = qty_decimal * unit_price_decimal`；未提供 unit_price 视为 0。
fn compute_totals(items: &[PurchaseOrderItemInput]) -> Result<(Decimal, Vec<Decimal>), AppError> {
    let mut line_totals: Vec<Decimal> = Vec::with_capacity(items.len());
    let mut total = Decimal::ZERO;
    for it in items {
        let qty = Decimal::from_f64_retain(it.quantity)
            .ok_or_else(|| AppError::validation(format!("无效的数量: {}", it.quantity)))?;
        let unit = match &it.unit_price {
            Some(p) => Decimal::from_str(p)
                .map_err(|_| AppError::validation(format!("无效的单价: {p}")))?,
            None => Decimal::ZERO,
        };
        let line = qty * unit;
        line_totals.push(line);
        total += line;
    }
    Ok((total, line_totals))
}

// —— Services ——

pub async fn create_order(
    pool: &SqlitePool,
    dto: &CreatePurchaseOrderRequest,
    user: &AuthUser,
) -> Result<PurchaseOrderRow, AppError> {
    validate_supplier(pool, dto.supplier_id).await?;
    validate_items(pool, &dto.items).await?;
    let (total_amount, line_totals) = compute_totals(&dto.items)?;
    let total_text = total_amount.to_string();

    // 事务：插单头 + 所有单体行。order_no 由 service 生成，UNIQUE 冲突时重试（最多 8 次）。
    let currency = dto.currency.as_deref().unwrap_or("CNY");
    let mut order_id: Option<i64> = None;
    for _attempt in 0..8 {
        let order_no = generate_order_no();
        let mut tx = pool.begin().await?;
        let inserted = sqlx::query(
            "INSERT INTO purchase_orders
                (order_no, supplier_id, order_date, status, doc_status, total_amount,
                 currency, notes, created_by)
             VALUES (?, ?, ?, 'draft', ?, ?, ?, ?, ?)",
        )
        .bind(&order_no)
        .bind(dto.supplier_id)
        .bind(&dto.order_date)
        .bind(DOC_DRAFT)
        .bind(&total_text)
        .bind(currency)
        .bind(dto.notes.as_deref())
        .bind(user.id)
        .execute(&mut *tx)
        .await;

        match inserted {
            Ok(_) => {
                let id: i64 = sqlx::query_scalar("SELECT last_insert_rowid()")
                    .fetch_one(&mut *tx)
                    .await?;
                for (it, line_total) in dto.items.iter().zip(line_totals.iter()) {
                    sqlx::query(
                        "INSERT INTO purchase_order_items
                        (order_id, item_id, quantity, received_qty, unit_price, total_price, notes)
                     VALUES (?, ?, ?, 0, ?, ?, ?)",
                    )
                    .bind(id)
                    .bind(it.item_id)
                    .bind(it.quantity)
                    .bind(it.unit_price.as_deref())
                    .bind(line_total.to_string())
                    .bind(it.notes.as_deref())
                    .execute(&mut *tx)
                    .await?;
                }
                tx.commit().await?;
                order_id = Some(id);
                break;
            }
            Err(sqlx::Error::Database(ref db_err))
                if db_err
                    .message()
                    .contains("UNIQUE constraint failed: purchase_orders.order_no") =>
            {
                // 冲突：回滚（drop）后重试新 order_no
                continue;
            }
            Err(e) => {
                return Err(AppError::from(e));
            }
        }
    }
    let order_id = order_id
        .ok_or_else(|| AppError::new(ErrorCode::Internal, "采购单号生成冲突超过重试上限"))?;

    purchase_repo::find_by_id(pool, order_id)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::Internal, "采购订单创建后读取失败"))
}

pub async fn get_order(
    pool: &SqlitePool,
    id: i64,
) -> Result<(PurchaseOrderRow, Vec<PurchaseOrderItemRow>), AppError> {
    let order = purchase_repo::find_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::OrderNotFound, "采购订单未找到"))?;
    let items = purchase_repo::list_items_for_order(pool, id).await?;
    Ok((order, items))
}

pub async fn list_orders(
    pool: &SqlitePool,
    filter: &PurchaseOrderFilter,
    page: i64,
    page_size: i64,
) -> Result<(Vec<PurchaseOrderRow>, i64), AppError> {
    purchase_repo::list_orders(pool, filter, page, page_size).await
}

pub async fn update_order(
    pool: &SqlitePool,
    id: i64,
    dto: &UpdatePurchaseOrderRequest,
    _user: &AuthUser,
) -> Result<(), AppError> {
    let order = purchase_repo::find_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::OrderNotFound, "采购订单未找到"))?;
    if order.status != "draft" {
        return Err(AppError::new(
            ErrorCode::OrderCannotModify,
            format!("采购订单当前状态为 {}，不可修改", order.status),
        ));
    }
    validate_supplier(pool, dto.supplier_id).await?;
    validate_items(pool, &dto.items).await?;
    let (total_amount, line_totals) = compute_totals(&dto.items)?;
    let total_text = total_amount.to_string();

    let mut tx = pool.begin().await?;

    sqlx::query(
        "UPDATE purchase_orders SET supplier_id = ?, order_date = ?, total_amount = ?,
             currency = ?, notes = ?, updated_at = datetime('now')
         WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(dto.supplier_id)
    .bind(&dto.order_date)
    .bind(&total_text)
    .bind(dto.currency.as_deref().unwrap_or("CNY"))
    .bind(dto.notes.as_deref())
    .bind(id)
    .execute(&mut *tx)
    .await?;

    sqlx::query("DELETE FROM purchase_order_items WHERE order_id = ?")
        .bind(id)
        .execute(&mut *tx)
        .await?;

    for (it, line_total) in dto.items.iter().zip(line_totals.iter()) {
        sqlx::query(
            "INSERT INTO purchase_order_items
                (order_id, item_id, quantity, received_qty, unit_price, total_price, notes)
             VALUES (?, ?, ?, 0, ?, ?, ?)",
        )
        .bind(id)
        .bind(it.item_id)
        .bind(it.quantity)
        .bind(it.unit_price.as_deref())
        .bind(line_total.to_string())
        .bind(it.notes.as_deref())
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

/// draft → submitted（doc_status 0 → 1）
pub async fn submit(
    pool: &SqlitePool,
    id: i64,
    _user: &AuthUser,
) -> Result<PurchaseOrderRow, AppError> {
    let order = purchase_repo::find_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::OrderNotFound, "采购订单未找到"))?;
    if order.status != "draft" {
        return Err(AppError::new(
            ErrorCode::OrderCannotModify,
            format!("采购订单当前状态为 {}，不可提交", order.status),
        ));
    }
    let workflow = workflow_repo::find_active_workflow_by_type(pool, "purchase_order").await?;
    let initial = match &workflow {
        Some(workflow) => workflow_repo::find_initial_state(pool, workflow.id).await?,
        None => None,
    };
    let mut tx = pool.begin().await?;
    sqlx::query(
        "UPDATE purchase_orders SET status = ?, doc_status = ?, updated_at = datetime('now')
         WHERE id = ? AND deleted_at IS NULL",
    )
    .bind("submitted")
    .bind(DOC_SUBMITTED)
    .bind(id)
    .execute(&mut *tx)
    .await?;
    if let (Some(workflow), Some(initial)) = (&workflow, &initial) {
        workflow_service::start_instance_in_tx(&mut tx, workflow, initial, "purchase_order", id)
            .await?;
    }
    tx.commit().await?;
    purchase_repo::find_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::Internal, "提交后读取采购订单失败"))
}

/// submitted → approved（doc_status 保持 1）
pub async fn approve(
    pool: &SqlitePool,
    id: i64,
    user: &AuthUser,
) -> Result<PurchaseOrderRow, AppError> {
    let order = purchase_repo::find_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::OrderNotFound, "采购订单未找到"))?;
    if order.status != "submitted" {
        return Err(AppError::new(
            ErrorCode::OrderCannotModify,
            format!("采购订单当前状态为 {}，不可审批通过", order.status),
        ));
    }
    purchase_repo::update_status(pool, id, "approved", Some(DOC_SUBMITTED)).await?;
    if let Some(instance) =
        workflow_service::find_active_instance_for(pool, "purchase_order", id).await?
    {
        let amount = Decimal::from_str_radix(&order.total_amount, 10).ok();
        if instance.current_state == "draft" {
            workflow_service::transition_with_amount(pool, instance.id, "submit", user, None, amount).await?;
        }
        workflow_service::transition_with_amount(pool, instance.id, "approve", user, None, amount).await?;
    }
    purchase_repo::find_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::Internal, "审批后读取采购订单失败"))
}

/// submitted → rejected（doc_status 保持 1：审批流程已完成，仅是被驳回）
pub async fn reject(
    pool: &SqlitePool,
    id: i64,
    _user: &AuthUser,
) -> Result<PurchaseOrderRow, AppError> {
    let order = purchase_repo::find_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::OrderNotFound, "采购订单未找到"))?;
    if order.status != "submitted" {
        return Err(AppError::new(
            ErrorCode::OrderCannotModify,
            format!("采购订单当前状态为 {}，不可驳回", order.status),
        ));
    }
    purchase_repo::update_status(pool, id, "rejected", Some(DOC_SUBMITTED)).await?;
    purchase_repo::find_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::Internal, "驳回后读取采购订单失败"))
}

/// draft | submitted → cancelled（doc_status → 2）
pub async fn cancel(
    pool: &SqlitePool,
    id: i64,
    _user: &AuthUser,
) -> Result<PurchaseOrderRow, AppError> {
    let order = purchase_repo::find_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::OrderNotFound, "采购订单未找到"))?;
    if order.status != "draft" && order.status != "submitted" {
        return Err(AppError::new(
            ErrorCode::OrderCannotModify,
            format!("采购订单当前状态为 {}，不可取消", order.status),
        ));
    }
    purchase_repo::update_status(pool, id, "cancelled", Some(DOC_CANCELLED)).await?;
    purchase_repo::find_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::Internal, "取消后读取采购订单失败"))
}

pub async fn delete_order(pool: &SqlitePool, id: i64, _user: &AuthUser) -> Result<(), AppError> {
    let order = purchase_repo::find_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::OrderNotFound, "采购订单未找到"))?;
    if order.status != "draft" {
        return Err(AppError::new(
            ErrorCode::OrderCannotModify,
            format!("采购订单当前状态为 {}，不可删除", order.status),
        ));
    }
    purchase_repo::soft_delete_order(pool, id).await?;
    Ok(())
}
