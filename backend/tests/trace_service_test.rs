//! Integration tests for TraceService.
//!
//! Covers:
//! - Trace item lifecycle (with/without logs)
//! - Trace by order (inbound + outbound)
//! - Error cases: item not found, invalid order type
//!
//! All tests use a fresh SQLite test pool with migrations applied (tests::common).

mod common;

use erp_server::services::trace_service::TraceService;

/// Seed a generic item (商品) row; returns item id.
async fn seed_item(pool: &sqlx::SqlitePool, sku: &str, name: &str) -> i64 {
    sqlx::query_scalar(
        "INSERT INTO items (sku, name, category, unit, spec, price, status) \
         VALUES (?, ?, '原材料', '件', 'SPC', 10.0, 'active') RETURNING id",
    )
    .bind(sku)
    .bind(name)
    .fetch_one(pool)
    .await
    .unwrap()
}

/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
/// trace_item_lifecycle
/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn trace_item_lifecycle_with_logs() {
    let pool = common::test_pool().await;

    let item_id = seed_item(&pool, "SKU-TRACE-001", "追溯商品").await;

    // Insert inventory log entries directly (matching actual schema)
    sqlx::query(
        "INSERT INTO inventory_logs (item_id, quantity, change_type, ref_type, ref_id, notes, created_at)
         VALUES (?, 10.0, 'inbound', 'purchase', 100, 'received from supplier', datetime('now'))",
    )
    .bind(item_id)
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO inventory_logs (item_id, quantity, change_type, ref_type, ref_id, notes, created_at)
         VALUES (?, 3.0, 'outbound', 'sales', 200, 'shipped to customer', datetime('now'))",
    )
    .bind(item_id)
    .execute(&pool)
    .await
    .unwrap();

    let result = TraceService::trace_item_lifecycle(&pool, item_id)
        .await
        .expect("trace_item_lifecycle must succeed");

    assert_eq!(result["item"]["sku"].as_str(), Some("SKU-TRACE-001"));
    assert_eq!(result["item"]["current_stock"].as_f64(), Some(7.0));

    let events = result["events"].as_array().unwrap();
    assert_eq!(events.len(), 2, "should have 2 events");
    assert_eq!(events[0]["change_type"].as_str(), Some("inbound"));
    assert_eq!(events[1]["change_type"].as_str(), Some("outbound"));
}

#[tokio::test]
async fn trace_item_lifecycle_no_logs() {
    let pool = common::test_pool().await;

    let item_id = seed_item(&pool, "SKU-TRACE-002", "无记录商品").await;

    let result = TraceService::trace_item_lifecycle(&pool, item_id)
        .await
        .expect("trace_item_lifecycle with no logs must succeed");

    assert_eq!(result["item"]["sku"].as_str(), Some("SKU-TRACE-002"));
    assert_eq!(result["item"]["current_stock"].as_f64(), Some(0.0));

    let events = result["events"].as_array().unwrap();
    assert!(events.is_empty(), "should have no events");
}

#[tokio::test]
async fn trace_item_lifecycle_item_not_found() {
    let pool = common::test_pool().await;

    let err = TraceService::trace_item_lifecycle(&pool, 99999)
        .await
        .expect_err("must reject non-existent item");

    assert!(err.to_string().contains("not found"));
}

#[tokio::test]
async fn trace_item_lifecycle_events_sorted_by_time() {
    let pool = common::test_pool().await;

    let item_id = seed_item(&pool, "SKU-TRACE-003", "排序商品").await;

    // Insert events out of order (older first is correct, but we interleave)
    sqlx::query(
        "INSERT INTO inventory_logs (item_id, quantity, change_type, ref_type, ref_id, notes, created_at)
         VALUES (?, 5.0, 'outbound', 'sales', 300, 'shipped', datetime('now', '+1 days'))",
    )
    .bind(item_id)
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO inventory_logs (item_id, quantity, change_type, ref_type, ref_id, notes, created_at)
         VALUES (?, 8.0, 'inbound', 'purchase', 301, 'received', datetime('now', '-1 days'))",
    )
    .bind(item_id)
    .execute(&pool)
    .await
    .unwrap();

    let result = TraceService::trace_item_lifecycle(&pool, item_id)
        .await
        .expect("trace_item_lifecycle must succeed");

    let events = result["events"].as_array().unwrap();
    assert_eq!(events.len(), 2);
    // First event should be inbound (older)
    assert_eq!(events[0]["change_type"].as_str(), Some("inbound"));
    // Second event should be outbound
    assert_eq!(events[1]["change_type"].as_str(), Some("outbound"));
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// trace_by_order — inbound
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn trace_by_order_inbound() {
    let pool = common::test_pool().await;

    let item_id = seed_item(&pool, "SKU-ORD-IN-001", "入库商品").await;

    // Create inbound record linked to a purchase order
    let inbound_id: i64 = sqlx::query_scalar(
        "INSERT INTO inbound_records (inbound_no, inbound_type, order_id, approval_status, notes, created_at, updated_at)
         VALUES ('IN-ORD-001', 'purchase', 42, 'approved', 'test inbound', datetime('now'), datetime('now'))
         RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    // Create inbound item
    sqlx::query(
        "INSERT INTO inbound_items (inbound_id, item_id, quantity, created_at)
         VALUES (?, ?, 10.0, datetime('now'))",
    )
    .bind(inbound_id)
    .bind(item_id)
    .execute(&pool)
    .await
    .unwrap();

    let result = TraceService::trace_by_order(&pool, "inbound", 42)
        .await
        .expect("trace_by_order inbound must succeed");

    assert_eq!(result["order_type"].as_str(), Some("inbound"));
    assert_eq!(result["order_id"].as_i64(), Some(42));

    let records = result["records"].as_array().unwrap();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0]["approval_status"].as_str(), Some("approved"));

    let related_items = result["related_items"].as_array().unwrap();
    assert_eq!(related_items.len(), 1);
    assert_eq!(related_items[0]["item_id"].as_i64(), Some(item_id));
    assert_eq!(related_items[0]["quantity"].as_f64(), Some(10.0));
}

#[tokio::test]
async fn trace_by_order_inbound_no_records() {
    let pool = common::test_pool().await;

    let result = TraceService::trace_by_order(&pool, "inbound", 999)
        .await
        .expect("trace_by_order with no records must succeed");

    assert_eq!(result["order_id"].as_i64(), Some(999));
    assert!(result["records"].as_array().unwrap().is_empty());
    assert!(result["related_items"].as_array().unwrap().is_empty());
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// trace_by_order — outbound
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn trace_by_order_outbound() {
    let pool = common::test_pool().await;

    let item_id = seed_item(&pool, "SKU-ORD-OUT-001", "出库商品").await;

    let outbound_id: i64 = sqlx::query_scalar(
        "INSERT INTO outbound_records (outbound_no, outbound_type, order_id, approval_status, notes, created_at, updated_at)
         VALUES ('OUT-ORD-001', 'sales', 55, 'approved', 'test outbound', datetime('now'), datetime('now'))
         RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO outbound_items (outbound_id, item_id, quantity, created_at)
         VALUES (?, ?, 4.0, datetime('now'))",
    )
    .bind(outbound_id)
    .bind(item_id)
    .execute(&pool)
    .await
    .unwrap();

    let result = TraceService::trace_by_order(&pool, "outbound", 55)
        .await
        .expect("trace_by_order outbound must succeed");

    assert_eq!(result["order_type"].as_str(), Some("outbound"));
    assert_eq!(result["order_id"].as_i64(), Some(55));

    let records = result["records"].as_array().unwrap();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0]["approval_status"].as_str(), Some("approved"));

    let related_items = result["related_items"].as_array().unwrap();
    assert_eq!(related_items.len(), 1);
    assert_eq!(related_items[0]["item_id"].as_i64(), Some(item_id));
}

#[tokio::test]
async fn trace_by_order_outbound_no_records() {
    let pool = common::test_pool().await;

    let result = TraceService::trace_by_order(&pool, "outbound", 888)
        .await
        .expect("trace_by_order with no records must succeed");

    assert!(result["records"].as_array().unwrap().is_empty());
    assert!(result["related_items"].as_array().unwrap().is_empty());
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// trace_by_order — error cases
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn trace_by_order_invalid_order_type() {
    let pool = common::test_pool().await;

    let err = TraceService::trace_by_order(&pool, "invalid", 1)
        .await
        .expect_err("must reject invalid order type");

    assert!(err.to_string().contains("Unknown order_type"));
}
