//! P1 端到端测试 — 财务闭环 + 盘点 + ATP 释放
//!
//! 串起 finance → invoices → payments → trial_balance → check → ATP reserve+release
//! 是 P1 财务闭环最终验收测试。
//!
//! 关键链路：
//! 1. 创建科目 → 创建借贷平衡日记账 → post → 试算平衡（终止状态 posted）
//! 2. 创建 invoice → 创建 payment 关联 invoice → invoice 状态自动转 paid
//! 3. ATP 链路：收货 100 → 销售 40 + submit (reservation 占 40, available 60) → ship → 库存 60, 预留释放, available 60
//! 4. 盘点：库存 60 → 实盘 55 → post → 库存 55, inventory_logs 1 行 check_adjust -5

mod common;

use sqlx::SqlitePool;
use erp_v2::middleware::auth::AuthUser;
use erp_v2::auth::bootstrap_admin;
use erp_v2::services::{
    catalog_service, location_service, parties_service,
    purchase_service, sales_service,
    receipt_service, shipment_service,
    finance_service, inventory_service,
};
use erp_v2::services::purchase_service::{CreatePurchaseOrderRequest, PurchaseOrderItemInput};
use erp_v2::services::sales_service::{CreateSalesOrderRequest, CreateSalesOrderItemInput};
use erp_v2::services::receipt_service::ReceivedItemInput;
use erp_v2::services::shipment_service::ShippedItemInput;
use erp_v2::services::finance_service::{
    CreateAccountRequest, CreateJournalEntryRequest,
    JournalLineInput, CreateInvoiceRequest, CreatePaymentRequest,
};

fn admin_user() -> AuthUser {
    AuthUser {
        id: 1, username: "admin".into(), display_name: "Administrator".into(),
        permissions: vec![
            "item.read".into(), "item.write".into(),
            "stock.read".into(), "stock.write".into(),
            "order.read".into(), "order.write".into(), "order.approve".into(),
            "finance.read".into(), "finance.write".into(),
            "report.read".into(),
            "user.manage".into(),
        ],
    }
}

async fn seed_prerequisites(pool: &SqlitePool) -> (i64, i64, i64, i64, i64) {
    let supplier = parties_service::create_supplier(pool, "S001", "供应商A", None, None, None, None).await.unwrap();
    let customer = parties_service::create_customer(pool, "C001", "客户B", None, None, None, None).await.unwrap();
    let item = catalog_service::create_item(pool, "ITEM-001", "测试商品", Some("测试"), Some("个"), None).await.unwrap();
    let wh = location_service::create_warehouse(pool, "W01", "总仓", None).await.unwrap();
    let loc = location_service::create_location(pool, Some(wh.id), "W01-A1", "A1 货位").await.unwrap();
    (supplier.id, customer.id, item.id, wh.id, loc.id)
}

#[tokio::test]
async fn finance_journal_to_trial_balance_e2e() {
    let (pool, _dir) = common::test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let user = admin_user();

    // 1. 创建资产类科目 "现金" + 收入类科目 "销售收入"
    let cash = finance_service::create_account(&pool, &CreateAccountRequest {
        code: "1001".into(), name: "现金".into(), parent_id: None, account_type: "asset".into(),
    }, &user).await.unwrap();
    let revenue = finance_service::create_account(&pool, &CreateAccountRequest {
        code: "4001".into(), name: "销售收入".into(), parent_id: None, account_type: "income".into(),
    }, &user).await.unwrap();

    // 2. 创建借贷平衡日记账（借现金 100, 贷收入 100）
    let je = finance_service::create_journal_entry(&pool, &CreateJournalEntryRequest {
        entry_date: "2026-08-10".into(),
        description: Some("销售收款".into()),
        ref_type: None, ref_id: None,
        lines: vec![
            JournalLineInput { account_id: cash.id, debit: "100.00".into(), credit: "0".into(), description: Some("现金".into()) },
            JournalLineInput { account_id: revenue.id, debit: "0".into(), credit: "100.00".into(), description: Some("收入".into()) },
        ],
    }, &user).await.unwrap();
    assert_eq!(je.status, "draft");

    // 3. post journal entry
    let posted = finance_service::post_journal_entry(&pool, je.id).await.unwrap();
    assert_eq!(posted.status, "posted");

    // 4. 试算平衡 — 借方/贷方总额应相等（100 = 100）
    let tb = finance_service::trial_balance(&pool).await.unwrap();
    let total_debit: rust_decimal::Decimal = tb.iter()
        .filter_map(|r| rust_decimal::Decimal::from_str(&r.total_debit).ok())
        .sum();
    let total_credit: rust_decimal::Decimal = tb.iter()
        .filter_map(|r| rust_decimal::Decimal::from_str(&r.total_credit).ok())
        .sum();
    use rust_decimal::prelude::FromStr;
    assert_eq!(total_debit.round_dp(2), total_credit.round_dp(2), "试算平衡: 借方应等于贷方");
}

#[tokio::test]
async fn finance_invoice_to_payment_e2e() {
    let (pool, _dir) = common::test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let user = admin_user();

    // supplier 先创建（payment 关联）
    let supplier = parties_service::create_supplier(&pool, "S002", "供应商PM", None, None, None, None).await.unwrap();

    // 1. 创建发票 100.00 (supplier 端 AP)
    let invoice = finance_service::create_invoice(&pool, &CreateInvoiceRequest {
        invoice_no: "INV-001".into(), invoice_date: "2026-08-10".into(),
        party_type: "supplier".into(), party_id: supplier.id, amount: "100.00".into(),
        ref_type: None, ref_id: None,
    }, &user).await.unwrap();
    assert_eq!(invoice.status, "unpaid");

    // 2. 创建 payment 100.00 关联 invoice
    let payment = finance_service::create_payment(&pool, &CreatePaymentRequest {
        payment_no: "PMT-001".into(), payment_date: "2026-08-10".into(),
        supplier_id: Some(supplier.id), amount: "100.00".into(),
        invoice_id: Some(invoice.id), method: Some("bank_transfer".into()), notes: None,
    }, &user).await.unwrap();
    assert!(payment.id > 0);

    // 3. 检查 invoice.status 已转为 paid
    let updated_invoice = finance_service::list_invoices(&pool, 1, 20).await.unwrap().0
        .into_iter().find(|i| i.id == invoice.id).unwrap();
    assert_eq!(updated_invoice.status, "paid", "付款完成发票状态应自动转 paid");

    // 4. 部分付款场景 — 不同 invoice 90.00，payment 30.00 → partially_paid
    let invoice2 = finance_service::create_invoice(&pool, &CreateInvoiceRequest {
        invoice_no: "INV-002".into(), invoice_date: "2026-08-10".into(),
        party_type: "supplier".into(), party_id: supplier.id, amount: "90.00".into(),
        ref_type: None, ref_id: None,
    }, &user).await.unwrap();

    finance_service::create_payment(&pool, &CreatePaymentRequest {
        payment_no: "PMT-002".into(), payment_date: "2026-08-10".into(),
        supplier_id: Some(supplier.id), amount: "30.00".into(),
        invoice_id: Some(invoice2.id), method: None, notes: None,
    }, &user).await.unwrap();

    let u2 = finance_service::list_invoices(&pool, 1, 20).await.unwrap().0
        .into_iter().find(|i| i.id == invoice2.id).unwrap();
    assert_eq!(u2.status, "partially_paid", "部分付款发票应为 partially_paid");
}

#[tokio::test]
async fn inventory_check_post_updates_balance_and_logs() {
    let (pool, _dir) = common::test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let user = admin_user();
    let (supplier_id, _cust, item_id, _wh, loc_id) = seed_prerequisites(&pool).await;

    // 先收货 100 到 loc_id
    let po = purchase_service::create_order(&pool, &CreatePurchaseOrderRequest {
        supplier_id, order_date: "2026-08-10".into(), currency: None, notes: None,
        items: vec![PurchaseOrderItemInput {
            item_id, quantity: 100.0, unit_price: Some("10.00".into()), notes: None,
        }],
    }, &user).await.unwrap();
    purchase_service::submit(&pool, po.id, &user).await.unwrap();
    purchase_service::approve(&pool, po.id, &user).await.unwrap();
    receipt_service::receive_purchase_order(&pool, po.id, &[
        ReceivedItemInput { item_id, location_id: loc_id, quantity: 100.0 },
    ], &user).await.unwrap();

    // 1. 创建盘点 session (location_id=loc_id)
    use erp_v2::services::inventory_service::CheckSessionCreateInput;
    let session = inventory_service::create_check_session(&pool, &CheckSessionCreateInput {
        location_id: loc_id, scope: "all".into(),
    }, &user).await.unwrap();
    assert_eq!(session.status, "draft");

    // 2. 录入实盘 95 (diff -5)
    let details = inventory_service::get_check_session(&pool, session.id).await.unwrap().1;
    assert_eq!(details.len(), 1, "应有 1 个 detail（item_id × location_id）");
    assert_eq!(details[0].system_qty, 100.0, "快照系统数量应是 100");

    inventory_service::record_actual_qty(&pool, session.id, details[0].id, 95.0, &user).await.unwrap();
    let updated_detail = inventory_service::get_check_session(&pool, session.id).await.unwrap().1[0].clone();
    assert_eq!(updated_detail.actual_qty, Some(95.0));
    assert_eq!(updated_detail.diff_qty, Some(-5.0));

    // 3. 过账 → 应调整库存到 95 + 写 check_adjust 流水 -5
    inventory_service::post_check_session(&pool, session.id, &user).await.unwrap();
    let bal: f64 = sqlx::query_scalar(
        "SELECT quantity FROM inventory WHERE item_id = ? AND location_id = ?"
    ).bind(item_id).bind(loc_id).fetch_one(&pool).await.unwrap();
    assert!((bal - 95.0).abs() < 0.01, "盘点后库存应为 95, 实际 {bal}");

    let log_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM inventory_logs WHERE item_id = ? AND change_type = 'check_adjust'"
    ).bind(item_id).fetch_one(&pool).await.unwrap();
    assert_eq!(log_count, 1, "应有 1 条 check_adjust 流水");
}

#[tokio::test]
async fn atp_reservation_releases_after_ship() {
    let (pool, _dir) = common::test_pool().await;
    bootstrap_admin(&pool, "admin", "admin123").await.unwrap();
    let user = admin_user();
    let (supplier_id, customer_id, item_id, _wh, loc_id) = seed_prerequisites(&pool).await;

    // 收货 100
    let po = purchase_service::create_order(&pool, &CreatePurchaseOrderRequest {
        supplier_id, order_date: "2026-08-10".into(), currency: None, notes: None,
        items: vec![PurchaseOrderItemInput {
            item_id, quantity: 100.0, unit_price: Some("10.00".into()), notes: None,
        }],
    }, &user).await.unwrap();
    purchase_service::submit(&pool, po.id, &user).await.unwrap();
    purchase_service::approve(&pool, po.id, &user).await.unwrap();
    receipt_service::receive_purchase_order(&pool, po.id, &[
        ReceivedItemInput { item_id, location_id: loc_id, quantity: 100.0 },
    ], &user).await.unwrap();

    // create SO 销 40 + submit (reservation 占用)
    let so = sales_service::create_order(&pool, &CreateSalesOrderRequest {
        customer_id, order_date: Some("2026-08-10".into()), currency: None, notes: None,
        items: vec![CreateSalesOrderItemInput {
            item_id, quantity: 40.0, unit_price: "20.00".into(), notes: None,
        }],
    }, &user).await.unwrap();
    sales_service::submit(&pool, so.id, &user).await.unwrap();

    // avail = 100 - 40 = 60
    let avail1 = inventory_service::get_available_qty(&pool, item_id, Some(loc_id)).await.unwrap();
    assert!((avail1 - 60.0).abs() < 0.01, "submit 后可用量应为 60");

    // approve SO + ship 40
    sales_service::approve(&pool, so.id, &user).await.unwrap();
    shipment_service::ship_sales_order(&pool, so.id, &[
        ShippedItemInput { item_id, location_id: loc_id, quantity: 40.0 },
    ], &user).await.unwrap();

    // 库存 = 60, 预留应为 0 (已释放), available = 60 - 0 = 60
    let bal: f64 = sqlx::query_scalar(
        "SELECT quantity FROM inventory WHERE item_id = ? AND location_id = ?"
    ).bind(item_id).bind(loc_id).fetch_one(&pool).await.unwrap();
    assert!((bal - 60.0).abs() < 0.01, "ship 后库存应为 60, 实际 {bal}");

    let reserved: f64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(quantity), 0.0) FROM reservations WHERE item_id = ? AND status = 'active'"
    ).bind(item_id).fetch_one(&pool).await.unwrap();
    assert!((reserved - 0.0).abs() < 0.01, "ship 后 active 预留应为 0, 实际 {reserved}");

    let avail2 = inventory_service::get_available_qty(&pool, item_id, Some(loc_id)).await.unwrap();
    assert!((avail2 - 60.0).abs() < 0.01, "ship 后 available 应为 60 (余额60 - 预留0)");
}
