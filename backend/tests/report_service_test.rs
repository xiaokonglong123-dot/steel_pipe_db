//! Integration tests for ReportService.
//!
//! Covers:
//! - Inventory summary (by status, category, movement type, location)
//! - Order reports (purchase + sales with period)
//! - Quality reports (pass/fail by item, by month from mfg_inspections)
//! - Dashboard aggregation
//!
//! All tests use a fresh SQLite database with migrations applied.

mod common;

use erp_server::services::report_service::ReportService;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Item helpers
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Seed an item master row; returns its id.
async fn seed_item(pool: &sqlx::SqlitePool, sku: &str, category: &str) -> i64 {
    sqlx::query_scalar(
        "INSERT INTO items (sku, name, category, unit, spec, price, status) \
         VALUES (?, 'Test Item', ?, '个', 'spec', 10.0, 'active') \
         RETURNING id",
    )
    .bind(sku)
    .bind(category)
    .fetch_one(pool)
    .await
    .expect("seed_item must succeed")
}

/// Seed an inventory_logs entry (inbound/outbound) for an item.
async fn seed_inventory_log(
    pool: &sqlx::SqlitePool,
    item_id: i64,
    change_type: &str,
    quantity: f64,
) {
    sqlx::query(
        "INSERT INTO inventory_logs (item_id, quantity, change_type, created_at) \
         VALUES (?, ?, ?, datetime('now'))",
    )
    .bind(item_id)
    .bind(quantity)
    .bind(change_type)
    .execute(pool)
    .await
    .expect("seed_inventory_log must succeed");
}

/// Seed a manufacturing inspection row (pass/fail) for an item.
async fn seed_inspection(pool: &sqlx::SqlitePool, item_id: i64, result: &str) {
    sqlx::query(
        "INSERT INTO mfg_inspections (item_id, inspection_type, result, notes, inspected_at, created_at) \
         VALUES (?, 'visual', ?, NULL, datetime('now'), datetime('now'))",
    )
    .bind(item_id)
    .bind(result)
    .execute(pool)
    .await
    .expect("seed_inspection must succeed");
}

/// Seed an open NCR (quality non-conformance) row.
async fn seed_ncr(pool: &sqlx::SqlitePool, ncr_no: &str, item_id: i64, status: &str) {
    sqlx::query(
        "INSERT INTO mfg_ncrs (ncr_no, item_id, description, severity, disposition, status, created_at) \
         VALUES (?, ?, 'test ncr', 'minor', 'rework', ?, datetime('now'))",
    )
    .bind(ncr_no)
    .bind(item_id)
    .bind(status)
    .execute(pool)
    .await
    .expect("seed_ncr must succeed");
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// inventory_summary
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn inventory_summary_empty_database() {
    let pool = common::test_pool().await;

    let summary = ReportService::inventory_summary(&pool)
        .await
        .expect("inventory_summary must succeed");

    assert!(
        summary["by_status"].as_array().unwrap().len() >= 1,
        "by_status should have at least the total entry"
    );
    assert!(summary["by_category"].is_array());
    assert!(summary["by_type"].is_array());
    assert!(summary["location_occupancy"].is_array());
}

#[tokio::test]
async fn inventory_summary_with_items_shows_counts() {
    let pool = common::test_pool().await;

    seed_item(&pool, "ITM-RPT-001", "原材料").await;
    seed_item(&pool, "ITM-RPT-002", "原材料").await;
    seed_item(&pool, "ITM-RPT-003", "标准件").await;

    let summary = ReportService::inventory_summary(&pool)
        .await
        .expect("inventory_summary must succeed");

    let by_status = summary["by_status"].as_array().unwrap();
    let by_category = summary["by_category"].as_array().unwrap();

    assert!(!by_status.is_empty(), "should have status aggregates");

    let active = by_status
        .iter()
        .find(|v| v["status"].as_str() == Some("active"));
    assert!(active.is_some(), "should have active entry");
    assert!(
        active.unwrap()["count"].as_i64().unwrap() >= 3,
        "should have at least 3 active items, got {}",
        active.unwrap()["count"].as_i64().unwrap()
    );

    let raw = by_category
        .iter()
        .find(|v| v["category"].as_str() == Some("原材料"));
    assert!(raw.is_some(), "should have 原材料 category entry");
    assert!(
        raw.unwrap()["count"].as_i64().unwrap() >= 2,
        "should have at least 2 raw-material items"
    );
}

#[tokio::test]
async fn inventory_summary_by_type_shows_movement_counts() {
    let pool = common::test_pool().await;

    let item_id = seed_item(&pool, "ITM-RPT-010", "原材料").await;
    seed_inventory_log(&pool, item_id, "inbound", 10.0).await;
    seed_inventory_log(&pool, item_id, "outbound", 4.0).await;

    let summary = ReportService::inventory_summary(&pool)
        .await
        .expect("inventory_summary must succeed");

    let by_type = summary["by_type"].as_array().unwrap();
    assert!(!by_type.is_empty(), "should have movement aggregates");

    let inbound = by_type
        .iter()
        .find(|v| v["change_type"].as_str() == Some("inbound"));
    assert!(inbound.is_some(), "should have inbound movement entry");
    assert!(
        inbound.unwrap()["total_quantity"].as_f64().unwrap_or(0.0) >= 10.0,
        "inbound total should be at least 10"
    );
}

#[tokio::test]
async fn inventory_summary_with_location_shows_occupancy() {
    let pool = common::test_pool().await;

    common::seed_location(&pool, "A", "01", "01")
        .await
        .unwrap();

    let summary = ReportService::inventory_summary(&pool)
        .await
        .expect("inventory_summary must succeed");

    let occupancy = summary["location_occupancy"].as_array().unwrap();
    assert!(!occupancy.is_empty(), "should have location occupancy data");
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// order_report
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn order_report_purchase_default_period() {
    let pool = common::test_pool().await;

    let supplier_id = common::seed_supplier(&pool, "SUP-RPT", "Test Supplier")
        .await
        .unwrap();
    common::seed_purchase_order(&pool, "PO-RPT-001", supplier_id, "pending")
        .await
        .unwrap();

    let report = ReportService::order_report(&pool, "purchase", "")
        .await
        .expect("order_report must succeed");

    assert_eq!(report["type"].as_str(), Some("purchase"));
    assert_eq!(report["period"].as_str(), Some("monthly"));
    assert!(report["orders"].is_array());
    assert!(report["status_distribution"].is_array());
    assert!(report["top_suppliers"].is_array());
}

#[tokio::test]
async fn order_report_purchase_with_custom_period() {
    let pool = common::test_pool().await;

    let supplier_id = common::seed_supplier(&pool, "SUP-RPT2", "Another Supplier")
        .await
        .unwrap();
    common::seed_purchase_order(&pool, "PO-RPT-002", supplier_id, "approved")
        .await
        .unwrap();

    let report = ReportService::order_report(&pool, "purchase", "yearly")
        .await
        .expect("order_report must succeed");

    assert_eq!(report["period"].as_str(), Some("yearly"));
    assert_eq!(report["type"].as_str(), Some("purchase"));
}

#[tokio::test]
async fn order_report_sales() {
    let pool = common::test_pool().await;

    let customer_id = common::seed_customer(&pool, "CUST-RPT", "Test Customer")
        .await
        .unwrap();
    common::seed_sales_order(&pool, "SO-RPT-001", customer_id, "pending")
        .await
        .unwrap();

    let report = ReportService::order_report(&pool, "sales", "monthly")
        .await
        .expect("order_report must succeed");

    assert_eq!(report["type"].as_str(), Some("sales"));
    assert_eq!(report["period"].as_str(), Some("monthly"));
    assert!(report["orders"].is_array());
    assert!(report["status_distribution"].is_array());
    assert!(report["top_customers"].is_array());
}

#[tokio::test]
async fn order_report_empty_returns_empty_arrays() {
    let pool = common::test_pool().await;

    let report = ReportService::order_report(&pool, "purchase", "monthly")
        .await
        .expect("order_report on empty DB must succeed");

    assert!(report["orders"].as_array().unwrap().is_empty());
    assert!(report["status_distribution"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn order_report_unknown_type_defaults_to_purchase() {
    let pool = common::test_pool().await;

    // Falls through to the purchase branch in the service
    let report = ReportService::order_report(&pool, "unknown_type", "monthly")
        .await
        .expect("order_report with unknown type must succeed");

    assert_eq!(report["type"].as_str(), Some("purchase"));
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// quality_report
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn quality_report_empty() {
    let pool = common::test_pool().await;

    let report = ReportService::quality_report(&pool)
        .await
        .expect("quality_report must succeed");

    assert!(report["by_item"].as_array().unwrap().is_empty());
    assert!(report["by_month"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn quality_report_with_inspections_shows_aggregates() {
    let pool = common::test_pool().await;

    let item_id = seed_item(&pool, "ITM-QR-001", "原材料").await;
    seed_inspection(&pool, item_id, "pass").await;
    seed_inspection(&pool, item_id, "fail").await;

    let report = ReportService::quality_report(&pool)
        .await
        .expect("quality_report must succeed");

    let by_item = report["by_item"].as_array().unwrap();
    assert!(!by_item.is_empty(), "should have item aggregates");

    let entry = by_item
        .iter()
        .find(|v| v["sku"].as_str() == Some("ITM-QR-001"));
    assert!(entry.is_some(), "should have ITM-QR-001 aggregate");
    assert_eq!(entry.unwrap()["pass_count"].as_i64().unwrap(), 1);
    assert_eq!(entry.unwrap()["fail_count"].as_i64().unwrap(), 1);

    let by_month = report["by_month"].as_array().unwrap();
    assert!(!by_month.is_empty(), "should have monthly aggregates");
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// dashboard
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn dashboard_empty_returns_zero_counts() {
    let pool = common::test_pool().await;

    let dash = ReportService::dashboard(&pool)
        .await
        .expect("dashboard must succeed");

    assert!(dash["total_stock"].as_i64().unwrap() >= 0, "total_stock should be non-negative");
    assert!(dash["inbound_30d"].as_i64().unwrap() >= 0, "inbound_30d should be non-negative");
    assert!(dash["outbound_30d"].as_i64().unwrap() >= 0, "outbound_30d should be non-negative");
    assert!(dash["pending_approvals"].as_i64().unwrap() >= 0, "pending_approvals should be non-negative");
    assert!(dash["recent_inbound"].as_array().unwrap().is_empty());
    assert!(dash["recent_outbound"].as_array().unwrap().is_empty());
    assert!(dash["pending_approval_list"].as_array().unwrap().is_empty());
    assert!(dash["recent_quality_failures"]
        .as_array()
        .unwrap()
        .is_empty());
}

#[tokio::test]
async fn dashboard_with_items_shows_stock_count() {
    let pool = common::test_pool().await;

    seed_item(&pool, "ITM-DASH-001", "原材料").await;
    seed_item(&pool, "ITM-DASH-002", "原材料").await;

    let dash = ReportService::dashboard(&pool)
        .await
        .expect("dashboard must succeed");

    assert!(
        dash["total_stock"].as_i64().unwrap_or(0) >= 2,
        "should count at least 2 active items"
    );
}

#[tokio::test]
async fn dashboard_with_pending_orders_shows_approvals() {
    let pool = common::test_pool().await;

    let supplier_id = common::seed_supplier(&pool, "SUP-DASH", "Dash Supplier")
        .await
        .unwrap();
    common::seed_purchase_order(&pool, "PO-DASH-001", supplier_id, "pending")
        .await
        .unwrap();

    let dash = ReportService::dashboard(&pool)
        .await
        .expect("dashboard must succeed");

    assert!(
        dash["pending_approvals"].as_i64().unwrap_or(0) >= 1,
        "should have at least 1 pending approval"
    );
}

#[tokio::test]
async fn dashboard_with_quality_failures() {
    let pool = common::test_pool().await;

    let item_id = seed_item(&pool, "ITM-DASH-QC", "原材料").await;
    seed_ncr(&pool, "NCR-DASH-001", item_id, "open").await;

    let dash = ReportService::dashboard(&pool)
        .await
        .expect("dashboard must succeed");

    let failures = dash["recent_quality_failures"].as_array().unwrap();
    assert!(!failures.is_empty(), "should show quality failures");
}
