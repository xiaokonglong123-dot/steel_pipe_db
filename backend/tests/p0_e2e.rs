//! P0 端到端测试 — 完整业务闭环
//!
//! 1. PO E2E: create PO (draft) → submit (auto-start workflow @ draft state) →
//!    approve (auto transition: draft→submitted→approved) → receive_purchase_order
//!    (inbound posted + inventory_logs inserted + inventory upsert)
//! 2. SO E2E: PO 先收货补库存 → create SO → submit → approve → ship
//!    (outbound posted + inventory deducted + 释放预留)
//!
//! 串起 catalog → parties → workflow → purchase/sales → inventory → receipt/ship
//! 整个 P0 链路，是 P0 阶段最终验收测试。

mod common;

use erp_v2::auth::bootstrap_admin;
use erp_v2::middleware::auth::AuthUser;
use erp_v2::services::purchase_service::{CreatePurchaseOrderRequest, PurchaseOrderItemInput};
use erp_v2::services::receipt_service::ReceivedItemInput;
use erp_v2::services::sales_service::{CreateSalesOrderItemInput, CreateSalesOrderRequest};
use erp_v2::services::shipment_service::ShippedItemInput;
use erp_v2::services::{
    catalog_service, location_service, parties_service, purchase_service, receipt_service,
    sales_service, shipment_service,
};
use sqlx::SqlitePool;

fn admin_user() -> AuthUser {
    AuthUser {
        id: 1,
        username: "admin".into(),
        display_name: "Administrator".into(),
        permissions: vec![
            "item.read".into(),
            "item.write".into(),
            "stock.read".into(),
            "stock.write".into(),
            "order.read".into(),
            "order.write".into(),
            "order.approve".into(),
            "finance.read".into(),
            "finance.write".into(),
            "report.read".into(),
            "user.manage".into(),
        ],
    }
}

/// 创建 supplier/customer/item/warehouse/location 前置数据。
/// 返回 (supplier_id, customer_id, item_id, warehouse_id, location_id)
async fn seed_prerequisites(pool: &SqlitePool) -> (i64, i64, i64, i64, i64) {
    let supplier =
        parties_service::create_supplier(pool, "S001", "供应商A", None, None, None, None)
            .await
            .unwrap();
    let customer = parties_service::create_customer(pool, "C001", "客户B", None, None, None, None)
        .await
        .unwrap();
    let item =
        catalog_service::create_item(pool, "ITEM-001", "测试商品", Some("测试"), Some("个"), None)
            .await
            .unwrap();
    let wh = location_service::create_warehouse(pool, "W01", "总仓", None)
        .await
        .unwrap();
    let loc = location_service::create_location(pool, Some(wh.id), "W01-A1", "A1 货位")
        .await
        .unwrap();
    (supplier.id, customer.id, item.id, wh.id, loc.id)
}

#[tokio::test]
async fn full_po_e2e_lifecycle() {
    let (pool, _dir) = common::test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let user = admin_user();
    let (supplier_id, _customer_id, item_id, _wh_id, loc_id) = seed_prerequisites(&pool).await;

    // 1. create PO (draft)
    let po = purchase_service::create_order(
        &pool,
        &CreatePurchaseOrderRequest {
            supplier_id,
            order_date: "2026-08-10".into(),
            currency: Some("CNY".into()),
            notes: None,
            items: vec![PurchaseOrderItemInput {
                item_id,
                quantity: 100.0,
                unit_price: Some("12.50".into()),
                notes: None,
            }],
        },
        &user,
    )
    .await
    .unwrap();
    assert_eq!(po.status, "draft", "新 PO 应为 draft");

    // 2. submit (内部自动 start_instance 到 draft state)
    let submitted = purchase_service::submit(&pool, po.id, &user).await.unwrap();
    assert_eq!(submitted.status, "submitted", "submit 后应为 submitted");

    // 3. approve (内部自动 transition: draft→submitted→approved)
    let approved = purchase_service::approve(&pool, po.id, &user)
        .await
        .unwrap();
    assert_eq!(approved.status, "approved", "approve 后应为 approved");

    // 4. receive → 库存增加 + inbound 记录创建
    let inbound = receipt_service::receive_purchase_order(
        &pool,
        po.id,
        &[ReceivedItemInput {
            item_id,
            location_id: loc_id,
            quantity: 100.0,
        }],
        &user,
    )
    .await
    .unwrap();
    assert!(inbound.id > 0, "应生成入库单");

    // 5. 验证库存余额 = 100
    let bal: f64 =
        sqlx::query_scalar("SELECT quantity FROM inventory WHERE item_id = ? AND location_id = ?")
            .bind(item_id)
            .bind(loc_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!((bal - 100.0).abs() < 0.01, "库存余额应为 100，实际 {bal}");

    // 6. 验证 inventory_logs 有 1 条 inbound 流水
    let log_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM inventory_logs WHERE item_id = ? AND change_type = 'inbound'",
    )
    .bind(item_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(log_count, 1, "应有 1 条 inbound 流水");
}

#[tokio::test]
async fn full_so_e2e_with_atp_and_ship() {
    let (pool, _dir) = common::test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let user = admin_user();
    let (supplier_id, customer_id, item_id, _wh_id, loc_id) = seed_prerequisites(&pool).await;

    // 用 PO 收货先补充库存到 100
    let po = purchase_service::create_order(
        &pool,
        &CreatePurchaseOrderRequest {
            supplier_id,
            order_date: "2026-08-10".into(),
            currency: None,
            notes: None,
            items: vec![PurchaseOrderItemInput {
                item_id,
                quantity: 100.0,
                unit_price: Some("10.00".into()),
                notes: None,
            }],
        },
        &user,
    )
    .await
    .unwrap();
    purchase_service::submit(&pool, po.id, &user).await.unwrap();
    purchase_service::approve(&pool, po.id, &user)
        .await
        .unwrap();
    receipt_service::receive_purchase_order(
        &pool,
        po.id,
        &[ReceivedItemInput {
            item_id,
            location_id: loc_id,
            quantity: 100.0,
        }],
        &user,
    )
    .await
    .unwrap();

    // 1. create SO (draft) 销 30 件
    let so = sales_service::create_order(
        &pool,
        &CreateSalesOrderRequest {
            customer_id,
            order_date: Some("2026-08-10".into()),
            currency: None,
            notes: None,
            items: vec![CreateSalesOrderItemInput {
                item_id,
                quantity: 30.0,
                unit_price: "20.00".into(),
                notes: None,
            }],
        },
        &user,
    )
    .await
    .unwrap();
    assert_eq!(so.status, "draft");

    // 2. submit SO
    sales_service::submit(&pool, so.id, &user).await.unwrap();

    // 3. approve SO
    sales_service::approve(&pool, so.id, &user).await.unwrap();

    // 4. ship → 库存 100 - 30 = 70
    let outbound = shipment_service::ship_sales_order(
        &pool,
        so.id,
        &[ShippedItemInput {
            item_id,
            location_id: loc_id,
            quantity: 30.0,
        }],
        &user,
    )
    .await
    .unwrap();
    assert!(outbound.id > 0, "应生成出库单");

    // 5. 库存余额 = 70
    let bal: f64 =
        sqlx::query_scalar("SELECT quantity FROM inventory WHERE item_id = ? AND location_id = ?")
            .bind(item_id)
            .bind(loc_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(
        (bal - 70.0).abs() < 0.01,
        "发货后库存余额应为 70，实际 {bal}"
    );

    // 6. inventory_logs 1 inbound + 1 outbound = 2 条
    let log_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM inventory_logs WHERE item_id = ? AND change_type IN ('inbound','outbound')"
    ).bind(item_id).fetch_one(&pool).await.unwrap();
    assert_eq!(log_count, 2, "应有 1 inbound + 1 outbound 共 2 条流水");
}
