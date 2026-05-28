//! Integration tests for ReportService.
//!
//! Covers:
//! - Inventory summary (by status, grade, type, location)
//! - Order reports (purchase + sales with period)
//! - Quality reports (pass/fail by grade, by month)
//! - Dashboard aggregation
//!
//! All tests use an in-memory SQLite database with fresh migrations.

mod common;

use steel_pipe_db::services::report_service::ReportService;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// inventory_summary
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn inventory_summary_empty_database() {
    let pool = common::test_pool().await;

    let summary = ReportService::inventory_summary(&pool)
        .await
        .expect("inventory_summary must succeed");

    assert_eq!(
        summary["by_status"].as_array().unwrap().len(),
        2,
        "by_status returns total_seamless and total_screen even when empty"
    );
    assert_eq!(
        summary["by_grade"].as_array().unwrap().len(),
        0,
        "no pipes means no grade aggregates"
    );
    assert!(summary["location_occupancy"].is_array());
}

#[tokio::test]
async fn inventory_summary_with_pipes_shows_counts() {
    let pool = common::test_pool().await;

    common::seed_seamless_pipe(&pool, "PN-RPT-001", "in_stock", "L80")
        .await
        .unwrap();
    common::seed_seamless_pipe(&pool, "PN-RPT-002", "in_stock", "J55")
        .await
        .unwrap();
    common::seed_seamless_pipe(&pool, "PN-RPT-003", "scrapped", "L80")
        .await
        .unwrap();

    let summary = ReportService::inventory_summary(&pool)
        .await
        .expect("inventory_summary must succeed");

    let by_status = summary["by_status"].as_array().unwrap();
    let by_grade = summary["by_grade"].as_array().unwrap();

    assert!(!by_status.is_empty(), "should have status aggregates");

    let in_stock = by_status
        .iter()
        .find(|v| v["status"].as_str() == Some("in_stock"));
    assert!(in_stock.is_some(), "should have in_stock entry");
    assert_eq!(
        in_stock.unwrap()["count"].as_i64(),
        Some(2),
        "should count 2 in_stock pipes"
    );

    let l80_grade = by_grade
        .iter()
        .find(|v| v["grade"].as_str() == Some("L80"));
    assert!(l80_grade.is_some(), "should have L80 grade entry");
}

#[tokio::test]
async fn inventory_summary_by_type_shows_pipe_type_counts() {
    let pool = common::test_pool().await;

    common::seed_seamless_pipe(&pool, "PN-RPT-010", "in_stock", "L80")
        .await
        .unwrap();
    common::seed_screen_pipe(&pool, "SCR-RPT-001", "in_stock", "N80")
        .await
        .unwrap();

    let summary = ReportService::inventory_summary(&pool)
        .await
        .expect("inventory_summary must succeed");

    let by_type = summary["by_type"].as_array().unwrap();
    assert!(!by_type.is_empty(), "should have type aggregates");
}

#[tokio::test]
async fn inventory_summary_with_location_shows_occupancy() {
    let pool = common::test_pool().await;

    let loc_id = common::seed_location(&pool, "A", "01", "01").await.unwrap();
    let pipe_id = common::seed_seamless_pipe(&pool, "PN-RPT-020", "in_stock", "L80")
        .await
        .unwrap();

    // Assign pipe to location
    sqlx::query("UPDATE seamless_pipes SET location_id = ? WHERE id = ?")
        .bind(loc_id)
        .bind(pipe_id)
        .execute(&pool)
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

    assert!(report["by_grade"].as_array().unwrap().is_empty());
    assert!(report["by_month"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn quality_report_with_certs_shows_aggregates() {
    let pool = common::test_pool().await;

    let pipe_id = common::seed_seamless_pipe(&pool, "PN-QR-001", "in_stock", "L80")
        .await
        .unwrap();

    common::seed_quality_cert(&pool, "QC-QR-001", "seamless", pipe_id, "pass")
        .await
        .unwrap();
    common::seed_quality_cert(&pool, "QC-QR-002", "seamless", pipe_id, "fail")
        .await
        .unwrap();

    let report = ReportService::quality_report(&pool)
        .await
        .expect("quality_report must succeed");

    let by_grade = report["by_grade"].as_array().unwrap();
    assert!(!by_grade.is_empty(), "should have grade aggregates");

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

    assert_eq!(dash["total_stock"].as_i64(), Some(0));
    assert_eq!(dash["inbound_30d"].as_i64(), Some(0));
    assert_eq!(dash["outbound_30d"].as_i64(), Some(0));
    assert_eq!(dash["pending_approvals"].as_i64(), Some(0));
    assert!(dash["recent_inbound"].as_array().unwrap().is_empty());
    assert!(dash["recent_outbound"].as_array().unwrap().is_empty());
    assert!(dash["pending_approval_list"].as_array().unwrap().is_empty());
    assert!(dash["recent_quality_failures"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn dashboard_with_pipes_shows_stock_count() {
    let pool = common::test_pool().await;

    common::seed_seamless_pipe(&pool, "PN-DASH-001", "in_stock", "L80")
        .await
        .unwrap();
    common::seed_seamless_pipe(&pool, "PN-DASH-002", "in_stock", "J55")
        .await
        .unwrap();

    let dash = ReportService::dashboard(&pool)
        .await
        .expect("dashboard must succeed");

    assert!(
        dash["total_stock"].as_i64().unwrap_or(0) >= 2,
        "should count at least 2 in-stock pipes"
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

    let pipe_id = common::seed_seamless_pipe(&pool, "PN-DASH-QC", "in_stock", "L80")
        .await
        .unwrap();
    common::seed_quality_cert(&pool, "QC-DASH-001", "seamless", pipe_id, "fail")
        .await
        .unwrap();

    let dash = ReportService::dashboard(&pool)
        .await
        .expect("dashboard must succeed");

    let failures = dash["recent_quality_failures"].as_array().unwrap();
    assert!(!failures.is_empty(), "should show quality failures");
}
