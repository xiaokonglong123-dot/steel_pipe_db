//! Integration tests for TraceService.
//!
//! Covers:
//! - Trace pipe lifecycle (seamless + screen)
//! - Trace by heat number
//! - Trace by order (inbound + outbound)
//! - Error cases: pipe not found, invalid pipe type, invalid order type
//!
//! All tests use an in-memory SQLite database with fresh migrations.

mod common;

use steel_pipe_db::services::trace_service::TraceService;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// trace_pipe_lifecycle — seamless
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn trace_pipe_lifecycle_seamless_with_logs() {
    let pool = common::test_pool().await;

    let pipe_id = common::seed_seamless_pipe(&pool, "PN-TRACE-001", "in_stock", "L80")
        .await
        .unwrap();

    // Insert inventory log entries directly (matching actual schema)
    sqlx::query(
        "INSERT INTO inventory_logs (pipe_type, pipe_id, change_type, ref_type, ref_id, notes, created_at)
         VALUES ('seamless', $1, 'inbound', 'purchase', 100, 'received from supplier', datetime('now', '-2 days'))",
    )
    .bind(pipe_id)
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO inventory_logs (pipe_type, pipe_id, change_type, ref_type, ref_id, notes, created_at)
         VALUES ('seamless', $1, 'transfer', 'location_change', 200, 'moved to A-01-01', datetime('now', '-1 days'))",
    )
    .bind(pipe_id)
    .execute(&pool)
    .await
    .unwrap();

    let result = TraceService::trace_pipe_lifecycle(&pool, "seamless", pipe_id)
        .await
        .expect("trace_pipe_lifecycle must succeed");

    assert_eq!(result["pipe"]["pipe_number"].as_str(), Some("PN-TRACE-001"));
    assert_eq!(result["pipe"]["current_status"].as_str(), Some("in_stock"));
    assert_eq!(result["pipe"]["pipe_type"].as_str(), Some("seamless"));

    let events = result["events"].as_array().unwrap();
    assert_eq!(events.len(), 2, "should have 2 events");
    assert_eq!(events[0]["change_type"].as_str(), Some("inbound"));
    assert_eq!(events[1]["change_type"].as_str(), Some("transfer"));
}

#[tokio::test]
async fn trace_pipe_lifecycle_seamless_no_logs() {
    let pool = common::test_pool().await;

    let pipe_id = common::seed_seamless_pipe(&pool, "PN-TRACE-002", "scrapped", "J55")
        .await
        .unwrap();

    let result = TraceService::trace_pipe_lifecycle(&pool, "seamless", pipe_id)
        .await
        .expect("trace_pipe_lifecycle with no logs must succeed");

    assert_eq!(result["pipe"]["pipe_number"].as_str(), Some("PN-TRACE-002"));
    assert_eq!(result["pipe"]["current_status"].as_str(), Some("scrapped"));

    let events = result["events"].as_array().unwrap();
    assert!(events.is_empty(), "should have no events");
}

#[tokio::test]
async fn trace_pipe_lifecycle_screen_with_logs() {
    let pool = common::test_pool().await;

    let pipe_id = common::seed_screen_pipe(&pool, "SCR-TRACE-001", "in_stock", "N80")
        .await
        .unwrap();

    sqlx::query(
        "INSERT INTO inventory_logs (pipe_type, pipe_id, change_type, ref_type, ref_id, notes, created_at)
         VALUES ('screen', $1, 'inbound', 'purchase', 101, 'received', datetime('now'))",
    )
    .bind(pipe_id)
    .execute(&pool)
    .await
    .unwrap();

    let result = TraceService::trace_pipe_lifecycle(&pool, "screen", pipe_id)
        .await
        .expect("trace_pipe_lifecycle for screen must succeed");

    assert_eq!(result["pipe"]["pipe_number"].as_str(), Some("SCR-TRACE-001"));
    assert_eq!(result["pipe"]["current_status"].as_str(), Some("in_stock"));

    let events = result["events"].as_array().unwrap();
    assert_eq!(events.len(), 1);
}

#[tokio::test]
async fn trace_pipe_lifecycle_pipe_not_found() {
    let pool = common::test_pool().await;

    let err = TraceService::trace_pipe_lifecycle(&pool, "seamless", 99999)
        .await
        .expect_err("must reject non-existent pipe");

    assert!(err.to_string().contains("not found"));
}

#[tokio::test]
async fn trace_pipe_lifecycle_invalid_pipe_type() {
    let pool = common::test_pool().await;

    let err = TraceService::trace_pipe_lifecycle(&pool, "bogus", 1)
        .await
        .expect_err("must reject invalid pipe type");

    assert!(err.to_string().contains("Unknown pipe_type"));
}

#[tokio::test]
async fn trace_pipe_lifecycle_events_sorted_by_time() {
    let pool = common::test_pool().await;

    let pipe_id = common::seed_seamless_pipe(&pool, "PN-TRACE-003", "in_stock", "L80")
        .await
        .unwrap();

    // Insert events out of order (older first is correct, but we interleave)
    sqlx::query(
        "INSERT INTO inventory_logs (pipe_type, pipe_id, change_type, ref_type, ref_id, notes, created_at)
         VALUES ('seamless', $1, 'outbound', 'sales', 300, 'shipped to customer', datetime('now', '+1 days'))",
    )
    .bind(pipe_id)
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO inventory_logs (pipe_type, pipe_id, change_type, ref_type, ref_id, notes, created_at)
         VALUES ('seamless', $1, 'inbound', 'purchase', 301, 'received', datetime('now', '-1 days'))",
    )
    .bind(pipe_id)
    .execute(&pool)
    .await
    .unwrap();

    let result = TraceService::trace_pipe_lifecycle(&pool, "seamless", pipe_id)
        .await
        .expect("trace_pipe_lifecycle must succeed");

    let events = result["events"].as_array().unwrap();
    assert_eq!(events.len(), 2);
    // First event should be inbound (older)
    assert_eq!(events[0]["change_type"].as_str(), Some("inbound"));
    // Second event should be outbound
    assert_eq!(events[1]["change_type"].as_str(), Some("outbound"));
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// trace_by_heat_number
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn trace_by_heat_number_finds_pipes() {
    let pool = common::test_pool().await;

    let pid1 = common::seed_seamless_pipe(&pool, "PN-HEAT-001", "in_stock", "L80")
        .await
        .unwrap();

    // Update heat numbers via SQL (seed doesn't set custom heat_number)
    sqlx::query("UPDATE seamless_pipes SET heat_number = 'HEAT-TEST-001' WHERE id = $1")
        .bind(pid1)
        .execute(&pool)
        .await
        .unwrap();

    let pid2 = common::seed_seamless_pipe(&pool, "PN-HEAT-002", "in_stock", "J55")
        .await
        .unwrap();
    sqlx::query("UPDATE seamless_pipes SET heat_number = 'HEAT-TEST-001' WHERE id = $1")
        .bind(pid2)
        .execute(&pool)
        .await
        .unwrap();

    let results = TraceService::trace_by_heat_number(&pool, "HEAT-TEST-001")
        .await
        .expect("trace_by_heat_number must succeed");

    assert_eq!(results.len(), 2);
    let nums: Vec<&str> = results.iter()
        .filter_map(|v| v["pipe_number"].as_str())
        .collect();
    assert!(nums.contains(&"PN-HEAT-001"));
    assert!(nums.contains(&"PN-HEAT-002"));
}

#[tokio::test]
async fn trace_by_heat_number_finds_screen_pipes() {
    let pool = common::test_pool().await;

    let pid = common::seed_screen_pipe(&pool, "SCR-HEAT-001", "in_stock", "N80")
        .await
        .unwrap();
    sqlx::query("UPDATE screen_pipes SET heat_number = 'HEAT-SCR-TEST' WHERE id = $1")
        .bind(pid)
        .execute(&pool)
        .await
        .unwrap();

    let results = TraceService::trace_by_heat_number(&pool, "HEAT-SCR-TEST")
        .await
        .expect("trace_by_heat_number must succeed");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["pipe_number"].as_str(), Some("SCR-HEAT-001"));
    assert_eq!(results[0]["pipe_type"].as_str(), Some("screen"));
}

#[tokio::test]
async fn trace_by_heat_number_no_matches() {
    let pool = common::test_pool().await;

    let results = TraceService::trace_by_heat_number(&pool, "HEAT-NONEXISTENT")
        .await
        .expect("trace_by_heat_number with no matches must succeed");

    assert!(results.is_empty());
}

#[tokio::test]
async fn trace_by_heat_number_returns_both_types() {
    let pool = common::test_pool().await;

    let seamless_id = common::seed_seamless_pipe(&pool, "PN-HEAT-BOTH", "in_stock", "L80")
        .await
        .unwrap();
    sqlx::query("UPDATE seamless_pipes SET heat_number = 'HEAT-BOTH' WHERE id = $1")
        .bind(seamless_id)
        .execute(&pool)
        .await
        .unwrap();

    let screen_id = common::seed_screen_pipe(&pool, "SCR-HEAT-BOTH", "in_stock", "N80")
        .await
        .unwrap();
    sqlx::query("UPDATE screen_pipes SET heat_number = 'HEAT-BOTH' WHERE id = $1")
        .bind(screen_id)
        .execute(&pool)
        .await
        .unwrap();

    let results = TraceService::trace_by_heat_number(&pool, "HEAT-BOTH")
        .await
        .expect("trace_by_heat_number must succeed");

    assert_eq!(results.len(), 2);
    let types: Vec<&str> = results.iter()
        .filter_map(|v| v["pipe_type"].as_str())
        .collect();
    assert!(types.contains(&"seamless"));
    assert!(types.contains(&"screen"));
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// trace_by_order — inbound
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn trace_by_order_inbound() {
    let pool = common::test_pool().await;

    let pipe_id = common::seed_seamless_pipe(&pool, "PN-ORD-IN-001", "in_stock", "L80")
        .await
        .unwrap();

    // Create inbound record linked to a purchase order
    let inbound_id: i64 = sqlx::query(
        "INSERT INTO inbound_records (inbound_no, inbound_type, order_id, approval_status, notes, created_at, updated_at)
         VALUES ('IN-ORD-001', 'purchase', 42, 'approved', 'test inbound', datetime('now'), datetime('now'))",
    )
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_rowid();

    // Create inbound item
    sqlx::query(
        "INSERT INTO inbound_items (inbound_id, pipe_type, pipe_id, created_at)
         VALUES ($1, 'seamless', $2, datetime('now'))",
    )
    .bind(inbound_id)
    .bind(pipe_id)
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

    let related_pipes = result["related_pipes"].as_array().unwrap();
    assert_eq!(related_pipes.len(), 1);
    assert_eq!(related_pipes[0]["pipe_id"].as_i64(), Some(pipe_id));
}

#[tokio::test]
async fn trace_by_order_inbound_no_records() {
    let pool = common::test_pool().await;

    let result = TraceService::trace_by_order(&pool, "inbound", 999)
        .await
        .expect("trace_by_order with no records must succeed");

    assert_eq!(result["order_id"].as_i64(), Some(999));
    assert!(result["records"].as_array().unwrap().is_empty());
    assert!(result["related_pipes"].as_array().unwrap().is_empty());
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// trace_by_order — outbound
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn trace_by_order_outbound() {
    let pool = common::test_pool().await;

    let pipe_id = common::seed_seamless_pipe(&pool, "PN-ORD-OUT-001", "outbound", "J55")
        .await
        .unwrap();

    let outbound_id: i64 = sqlx::query(
        "INSERT INTO outbound_records (outbound_no, outbound_type, order_id, approval_status, notes, created_at, updated_at)
         VALUES ('OUT-ORD-001', 'sales', 55, 'approved', 'test outbound', datetime('now'), datetime('now'))",
    )
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_rowid();

    sqlx::query(
        "INSERT INTO outbound_items (outbound_id, pipe_type, pipe_id, created_at)
         VALUES ($1, 'seamless', $2, datetime('now'))",
    )
    .bind(outbound_id)
    .bind(pipe_id)
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

    let related_pipes = result["related_pipes"].as_array().unwrap();
    assert_eq!(related_pipes.len(), 1);
    assert_eq!(related_pipes[0]["pipe_id"].as_i64(), Some(pipe_id));
}

#[tokio::test]
async fn trace_by_order_outbound_no_records() {
    let pool = common::test_pool().await;

    let result = TraceService::trace_by_order(&pool, "outbound", 888)
        .await
        .expect("trace_by_order with no records must succeed");

    assert!(result["records"].as_array().unwrap().is_empty());
    assert!(result["related_pipes"].as_array().unwrap().is_empty());
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
