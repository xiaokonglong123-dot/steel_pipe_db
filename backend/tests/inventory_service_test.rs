//! Integration tests for inventory services (inbound, outbound, location, check, ATP).
//!
//! Tests the core inventory workflows:
//! - Inbound record creation (auto_approved vs pending)
//! - Inbound approval/rejection
//! - Outbound creation with stock validation
//! - Location CRUD
//! - ATP query
//! - Full inbound → outbound pipe lifecycle
//!
//! All tests use an in-memory SQLite database with fresh migrations.

mod common;

use steel_pipe_db::dto::common::PaginationParams;
use steel_pipe_db::dto::inventory_dto::{
    AtpQuery, CreateCheckRequest, CreateInboundRecordRequest, CreateLocationRequest,
    CreateOutboundRecordRequest, InboundPipeItem, OutboundPipeItem, SubmitCheckItemRequest,
};
use steel_pipe_db::services::check_service::CheckService;
use steel_pipe_db::services::inbound_service::InboundService;
use steel_pipe_db::services::inventory_query_service::InventoryQueryService;
use steel_pipe_db::services::location_service::LocationService;
use steel_pipe_db::services::outbound_service::OutboundService;

/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
/// Inbound — auto_approved
/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn create_inbound_auto_approved_sets_pipes_to_in_stock() {
    let pool = common::test_pool().await;

    let pipe_id = common::seed_seamless_pipe(&pool, "PN-001", "scrapped", "J55")
        .await
        .unwrap();

    // Create a "purchase" inbound — auto_approved by default
    let dto = CreateInboundRecordRequest {
        inbound_type: "purchase".into(),
        order_id: None,
        supplier_id: None,
        notes: Some("test inbound".into()),
        pipes: vec![InboundPipeItem {
            pipe_id,
            pipe_type: "seamless".into(),
        }],
    };

    let record = InboundService::create_inbound(&pool, &dto)
        .await
        .expect("create_inbound must succeed");

    assert_eq!(record.approval_status, "auto_approved");

    // Verify pipe status was updated to "in_stock"
    let pipe: (String,) = sqlx::query_as("SELECT status FROM seamless_pipes WHERE id = ?")
        .bind(pipe_id)
        .fetch_one(&pool)
        .await
        .expect("pipe must exist");
    assert_eq!(pipe.0, "in_stock");
}

#[tokio::test]
async fn create_inbound_pending_does_not_update_pipe_status() {
    let pool = common::test_pool().await;

    let pipe_id = common::seed_seamless_pipe(&pool, "PN-002", "scrapped", "J55")
        .await
        .unwrap();

    // "return" inbound — starts as pending (not auto_approved)
    let dto = CreateInboundRecordRequest {
        inbound_type: "return".into(),
        order_id: None,
        supplier_id: None,
        notes: None,
        pipes: vec![InboundPipeItem {
            pipe_id,
            pipe_type: "seamless".into(),
        }],
    };

    let record = InboundService::create_inbound(&pool, &dto)
        .await
        .expect("create_inbound must succeed");
    assert_eq!(record.approval_status, "pending");

    let pipe: (String,) = sqlx::query_as("SELECT status FROM seamless_pipes WHERE id = ?")
        .bind(pipe_id)
        .fetch_one(&pool)
        .await
        .expect("pipe must exist");
    assert_eq!(pipe.0, "scrapped");
}

#[tokio::test]
async fn create_inbound_requires_at_least_one_pipe() {
    let pool = common::test_pool().await;

    let dto = CreateInboundRecordRequest {
        inbound_type: "purchase".into(),
        order_id: None,
        supplier_id: None,
        notes: None,
        pipes: vec![],
    };

    let err = InboundService::create_inbound(&pool, &dto)
        .await
        .expect_err("must fail with no pipes");
    assert!(err.to_string().contains("At least one pipe"));
}

/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
/// Inbound — manual approval / rejection
/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn approve_inbound_updates_pending_record_and_pipes() {
    let pool = common::test_pool().await;

    let pipe_id = common::seed_seamless_pipe(&pool, "PN-003", "scrapped", "J55")
        .await
        .unwrap();

    // "return" inbound starts as pending
    let dto = CreateInboundRecordRequest {
        inbound_type: "return".into(),
        order_id: None,
        supplier_id: None,
        notes: None,
        pipes: vec![InboundPipeItem {
            pipe_id,
            pipe_type: "seamless".into(),
        }],
    };

    let record = InboundService::create_inbound(&pool, &dto).await.unwrap();
    assert_eq!(record.approval_status, "pending");

    // Approve
    InboundService::approve_inbound(&pool, record.id, None)
        .await
        .expect("approve_inbound must succeed");

    // Verify record is now approved
    let updated: (String,) =
        sqlx::query_as("SELECT approval_status FROM inbound_records WHERE id = ?")
            .bind(record.id)
            .fetch_one(&pool)
            .await
            .expect("record must exist");
    assert_eq!(updated.0, "approved");

    // Verify pipe is now in_stock
    let pipe: (String,) = sqlx::query_as("SELECT status FROM seamless_pipes WHERE id = ?")
        .bind(pipe_id)
        .fetch_one(&pool)
        .await
        .expect("pipe must exist");
    assert_eq!(pipe.0, "in_stock");

    // Verify inventory log was created
    let log_count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM inventory_logs WHERE ref_id = ? AND change_type = 'inbound'")
            .bind(record.id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(log_count.0, 1);
}

#[tokio::test]
async fn approve_inbound_fails_for_already_approved() {
    let pool = common::test_pool().await;

    let pipe_id = common::seed_seamless_pipe(&pool, "PN-004", "scrapped", "J55")
        .await
        .unwrap();

    // "purchase" inbound — auto_approved, pipe already in_stock
    let dto = CreateInboundRecordRequest {
        inbound_type: "purchase".into(),
        order_id: None,
        supplier_id: None,
        notes: None,
        pipes: vec![InboundPipeItem {
            pipe_id,
            pipe_type: "seamless".into(),
        }],
    };

    let record = InboundService::create_inbound(&pool, &dto).await.unwrap();

    // Trying to approve it again must fail
    let err = InboundService::approve_inbound(&pool, record.id, None)
        .await
        .expect_err("approve must fail for already approved");
    assert!(
        err.to_string().contains("Cannot approve inbound with status")
            || err.to_string().contains("auto_approved")
    );
}

#[tokio::test]
async fn reject_inbound_only_updates_status() {
    let pool = common::test_pool().await;

    let pipe_id = common::seed_seamless_pipe(&pool, "PN-005", "scrapped", "J55")
        .await
        .unwrap();

    let dto = CreateInboundRecordRequest {
        inbound_type: "return".into(),
        order_id: None,
        supplier_id: None,
        notes: None,
        pipes: vec![InboundPipeItem {
            pipe_id,
            pipe_type: "seamless".into(),
        }],
    };

    let record = InboundService::create_inbound(&pool, &dto).await.unwrap();

    InboundService::reject_inbound(&pool, record.id, "material rejected")
        .await
        .expect("reject_inbound must succeed");

    // Verify record is rejected
    let updated: (String, Option<String>) = sqlx::query_as(
        "SELECT approval_status, rejection_reason FROM inbound_records WHERE id = ?",
    )
    .bind(record.id)
    .fetch_one(&pool)
    .await
    .expect("record must exist");
    assert_eq!(updated.0, "rejected");
    assert_eq!(updated.1.as_deref(), Some("material rejected"));

    let pipe: (String,) = sqlx::query_as("SELECT status FROM seamless_pipes WHERE id = ?")
        .bind(pipe_id)
        .fetch_one(&pool)
        .await
        .expect("pipe must exist");
    assert_eq!(pipe.0, "scrapped");
}

/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
/// Inbound — deletion
/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn delete_inbound_deletes_auto_approved_record() {
    let pool = common::test_pool().await;

    let pipe_id = common::seed_seamless_pipe(&pool, "PN-006", "scrapped", "J55")
        .await
        .unwrap();

    let dto = CreateInboundRecordRequest {
        inbound_type: "purchase".into(),
        order_id: None,
        supplier_id: None,
        notes: None,
        pipes: vec![InboundPipeItem {
            pipe_id,
            pipe_type: "seamless".into(),
        }],
    };

    let record = InboundService::create_inbound(&pool, &dto).await.unwrap();

    InboundService::delete_inbound(&pool, record.id)
        .await
        .expect("delete_inbound must succeed for auto_approved");

    // Record must be soft-deleted
    let deleted: (Option<String>,) =
        sqlx::query_as("SELECT deleted_at FROM inbound_records WHERE id = ?")
            .bind(record.id)
            .fetch_one(&pool)
            .await
            .expect("record must exist");
    assert!(deleted.0.is_some());
}

#[tokio::test]
async fn delete_inbound_fails_for_pending_record() {
    let pool = common::test_pool().await;

    let pipe_id = common::seed_seamless_pipe(&pool, "PN-007", "scrapped", "J55")
        .await
        .unwrap();

    let dto = CreateInboundRecordRequest {
        inbound_type: "return".into(),
        order_id: None,
        supplier_id: None,
        notes: None,
        pipes: vec![InboundPipeItem {
            pipe_id,
            pipe_type: "seamless".into(),
        }],
    };

    let record = InboundService::create_inbound(&pool, &dto).await.unwrap();

    let err = InboundService::delete_inbound(&pool, record.id)
        .await
        .expect_err("delete must fail for pending");
    assert!(
        err.to_string().contains("Cannot delete inbound with status")
            || err.to_string().contains("pending")
    );
}

/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
/// Outbound — stock validation
/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn create_outbound_fails_when_pipe_not_in_stock() {
    let pool = common::test_pool().await;

    let pipe_id = common::seed_seamless_pipe(&pool, "PN-008", "scrapped", "J55")
        .await
        .unwrap();

    let dto = CreateOutboundRecordRequest {
        outbound_type: "sales".into(),
        order_id: None,
        customer_id: None,
        notes: None,
        pipes: vec![OutboundPipeItem {
            pipe_id,
            pipe_type: "seamless".into(),
        }],
    };

    let err = OutboundService::create_outbound(&pool, &dto)
        .await
        .expect_err("outbound must fail for non-in-stock pipe");
    assert!(err.to_string().contains("Insufficient stock"));
}

#[tokio::test]
async fn create_outbound_succeeds_for_in_stock_pipe() {
    let pool = common::test_pool().await;

    // First: inbound to put pipe in stock (purchase = auto_approved)
    let pipe_id = common::seed_seamless_pipe(&pool, "PN-009", "scrapped", "J55")
        .await
        .unwrap();

    let inbound_dto = CreateInboundRecordRequest {
        inbound_type: "purchase".into(),
        order_id: Some(9009),
        supplier_id: None,
        notes: None,
        pipes: vec![InboundPipeItem {
            pipe_id,
            pipe_type: "seamless".into(),
        }],
    };
    InboundService::create_inbound(&pool, &inbound_dto)
        .await
        .unwrap();

    // Now create outbound (sales = auto_approved, executes immediately)
    let outbound_dto = CreateOutboundRecordRequest {
        outbound_type: "sales".into(),
        order_id: None,
        customer_id: None,
        notes: None,
        pipes: vec![OutboundPipeItem {
            pipe_id,
            pipe_type: "seamless".into(),
        }],
    };

    let record = OutboundService::create_outbound(&pool, &outbound_dto)
        .await
        .expect("outbound must succeed for in_stock pipe");
    assert_eq!(record.outbound_type, "sales");
}

#[tokio::test]
async fn create_outbound_requires_at_least_one_pipe() {
    let pool = common::test_pool().await;

    let dto = CreateOutboundRecordRequest {
        outbound_type: "sales".into(),
        order_id: None,
        customer_id: None,
        notes: None,
        pipes: vec![],
    };

    let err = OutboundService::create_outbound(&pool, &dto)
        .await
        .expect_err("must fail with no pipes");
    assert!(err.to_string().contains("At least one pipe"));
}

/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
/// Locations
/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn create_and_list_locations() {
    let pool = common::test_pool().await;

    let dto = CreateLocationRequest {
        zone_code: "A".into(),
        shelf_code: "01".into(),
        level_code: "01".into(),
        description: None,
        capacity: Some(50),
    };

    let location =
        LocationService::create_location(&pool, &dto)
            .await
            .expect("create_location must succeed");

    assert_eq!(location.full_code, "A-01-01");
    assert_eq!(location.zone_code, "A");
    assert_eq!(location.shelf_code, "01");
    assert_eq!(location.level_code, "01");

    // List all locations
    let params = PaginationParams {
        page: None,
        page_size: None,
        sort_by: None,
        sort_order: None,
    };
    let (locations, total) = LocationService::list_locations(&pool, &params, false)
        .await
        .expect("list_locations must succeed");
    assert_eq!(total, 1);
    assert_eq!(locations[0].full_code, "A-01-01");
}

/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
/// Inventory check
/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn create_and_submit_check_record() {
    let pool = common::test_pool().await;

    // Seed a location
    let location_id = common::seed_location(&pool, "B", "02", "03")
        .await
        .unwrap();

    // We need at least one in_stock pipe for the check to create items
    let pipe_id = common::seed_seamless_pipe(&pool, "PN-CHK-001", "scrapped", "J55")
        .await
        .unwrap();
    let inbound_dto = CreateInboundRecordRequest {
        inbound_type: "purchase".into(),
        order_id: None,
        supplier_id: None,
        notes: None,
        pipes: vec![InboundPipeItem {
            pipe_id,
            pipe_type: "seamless".into(),
        }],
    };
    InboundService::create_inbound(&pool, &inbound_dto)
        .await
        .unwrap();

    // Create a check record — auto-scans in_stock pipes
    let dto = CreateCheckRequest {
        location_id: Some(location_id),
        notes: None,
    };

    let check = CheckService::create_check(&pool, &dto)
        .await
        .expect("create_check must succeed");

    assert_eq!(check.status, "in_progress");

    // Get the auto-generated check items to find an item_id
    let (_record, items) = CheckService::get_check_detail(&pool, check.id)
        .await
        .expect("get_check_detail must succeed");
    assert!(!items.is_empty(), "check should have at least one item");

    let item_id = items[0].id;

    // Submit the result for a single check item
    let submit = SubmitCheckItemRequest {
        found_status: "in_stock".into(),
        notes: None,
    };

    CheckService::submit_check_item(&pool, check.id, item_id, &submit)
        .await
        .expect("submit_check_item must succeed");

    // Verify the item was updated
    let updated_items = CheckService::get_check_detail(&pool, check.id)
        .await
        .expect("get_check_detail must succeed")
        .1;
    let submitted = updated_items.iter().find(|i| i.id == item_id).unwrap();
    assert_eq!(submitted.found_status.as_deref(), Some("in_stock"));
}

/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
/// ATP query
/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn atp_query_returns_availability() {
    let pool = common::test_pool().await;

    let pipe_id = common::seed_seamless_pipe(&pool, "PN-ATPC-001", "scrapped", "J55")
        .await
        .unwrap();

    // Purchase inbound = auto_approved → pipe becomes in_stock
    let dto = CreateInboundRecordRequest {
        inbound_type: "purchase".into(),
        order_id: None,
        supplier_id: None,
        notes: None,
        pipes: vec![InboundPipeItem {
            pipe_id,
            pipe_type: "seamless".into(),
        }],
    };
    InboundService::create_inbound(&pool, &dto).await.unwrap();

    let atp_query = AtpQuery {
        pipe_type: Some("casing".into()),
        grade: Some("J55".into()),
        location_id: None,
    };

    let result = InventoryQueryService::check_atp(&pool, &atp_query)
        .await
        .expect("check_atp must succeed");

    assert!(!result.is_empty(), "should have at least 1 available item");
    assert_eq!(result[0].pipe_type, "casing");
    assert_eq!(result[0].grade, "J55");
    assert!(result[0].quantity > 0, "quantity should be positive");
}

/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
/// Pipe status state machine
/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn pipe_status_transitions_correctly_through_inbound_outbound_cycle() {
    let pool = common::test_pool().await;

    let pipe_id = common::seed_seamless_pipe(&pool, "PN-STATE-001", "scrapped", "J55")
        .await
        .unwrap();

    let status: (String,) =
        sqlx::query_as("SELECT status FROM seamless_pipes WHERE id = ?")
            .bind(pipe_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(status.0, "scrapped");

    // 2. Create a pending inbound (non-purchase, e.g. "return")
    let inbound_dto = CreateInboundRecordRequest {
        inbound_type: "return".into(),
        order_id: None,
        supplier_id: None,
        notes: None,
        pipes: vec![InboundPipeItem {
            pipe_id,
            pipe_type: "seamless".into(),
        }],
    };
    let inbound = InboundService::create_inbound(&pool, &inbound_dto)
        .await
        .unwrap();

    // 3. Approve → status becomes "in_stock"
    InboundService::approve_inbound(&pool, inbound.id, None)
        .await
        .expect("approve must succeed");

    let status: (String,) =
        sqlx::query_as("SELECT status FROM seamless_pipes WHERE id = ?")
            .bind(pipe_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(status.0, "in_stock");

    // 4. Create outbound (sales = auto_approved) → status becomes "outbound"
    let outbound_dto = CreateOutboundRecordRequest {
        outbound_type: "sales".into(),
        order_id: None,
        customer_id: None,
        notes: None,
        pipes: vec![OutboundPipeItem {
            pipe_id,
            pipe_type: "seamless".into(),
        }],
    };
    OutboundService::create_outbound(&pool, &outbound_dto)
        .await
        .unwrap();

    let status: (String,) =
        sqlx::query_as("SELECT status FROM seamless_pipes WHERE id = ?")
            .bind(pipe_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(status.0, "outbound");
}
