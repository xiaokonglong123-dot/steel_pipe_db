//! Integration tests for `InventoryService`.
//!
//! Tests the core inventory workflows:
//! - Inbound record creation (auto_approved vs pending)
//! - Inbound approval/rejection
//! - Outbound creation with stock validation
//! - Location assignment
//! - ATP query
//!
//! All tests use an in-memory SQLite database with fresh migrations.

mod common;

use steel_pipe_db::dto::inventory_dto::{
    AtpQuery, CreateCheckRequest, CreateInboundRecordRequest, CreateLocationRequest,
    CreateOutboundRecordRequest, InboundFilter, InboundPipeItem, SubmitCheckItemRequest,
};
use steel_pipe_db::repositories::inventory_repo::{CheckInitItem, LocationRepo};
use steel_pipe_db::services::InventoryService;

/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
/// Inbound — auto_approved
/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn create_inbound_auto_approved_sets_pipes_to_in_stock() {
    let pool = common::test_pool().await;

    // Seed a pipe in "available" status
    let pipe_id = common::seed_seamless_pipe(&pool, "PN-001", "available", "J55")
        .await
        .unwrap();

    // Create auto_approved inbound
    let dto = CreateInboundRecordRequest {
        source_type: "purchase".into(),
        approval_status: "auto_approved".into(),
        reference_no: Some("PO-001".into()),
        supplier_id: None,
        customer_id: None,
        expected_date: None,
        notes: Some("test inbound".into()),
        pipes: vec![InboundPipeItem {
            pipe_id,
            pipe_type: "seamless".into(),
            quantity: 1,
            weight: Some(500.0),
            length: Some(9.5),
        }],
    };

    let record = InventoryService::create_inbound(&pool, &dto)
        .await
        .expect("create_inbound must succeed");

    assert_eq!(record.approval_status, "auto_approved");

    // Verify pipe status was updated to "in_stock"
    let pipe: (String,) = sqlx::query_as("SELECT status FROM seamless_pipes WHERE id = $1")
        .bind(pipe_id)
        .fetch_one(&pool)
        .await
        .expect("pipe must exist");
    assert_eq!(pipe.0, "in_stock");
}

#[tokio::test]
async fn create_inbound_pending_does_not_update_pipe_status() {
    let pool = common::test_pool().await;

    let pipe_id = common::seed_seamless_pipe(&pool, "PN-002", "available", "J55")
        .await
        .unwrap();

    let dto = CreateInboundRecordRequest {
        source_type: "purchase".into(),
        approval_status: "pending".into(),
        reference_no: Some("PO-002".into()),
        supplier_id: None,
        customer_id: None,
        expected_date: None,
        notes: None,
        pipes: vec![InboundPipeItem {
            pipe_id,
            pipe_type: "seamless".into(),
            quantity: 1,
            weight: None,
            length: None,
        }],
    };

    let record = InventoryService::create_inbound(&pool, &dto)
        .await
        .expect("create_inbound must succeed");
    assert_eq!(record.approval_status, "pending");

    // Pipe must still be "available" — not touched until approval
    let pipe: (String,) = sqlx::query_as("SELECT status FROM seamless_pipes WHERE id = $1")
        .bind(pipe_id)
        .fetch_one(&pool)
        .await
        .expect("pipe must exist");
    assert_eq!(pipe.0, "available");
}

#[tokio::test]
async fn create_inbound_requires_at_least_one_pipe() {
    let pool = common::test_pool().await;

    let dto = CreateInboundRecordRequest {
        source_type: "purchase".into(),
        approval_status: "auto_approved".into(),
        reference_no: None,
        supplier_id: None,
        customer_id: None,
        expected_date: None,
        notes: None,
        pipes: vec![],
    };

    let err = InventoryService::create_inbound(&pool, &dto)
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

    let pipe_id = common::seed_seamless_pipe(&pool, "PN-003", "available", "J55")
        .await
        .unwrap();

    // Create pending inbound (not auto_approved)
    let dto = CreateInboundRecordRequest {
        source_type: "purchase".into(),
        approval_status: "pending".into(),
        reference_no: Some("PO-003".into()),
        supplier_id: None,
        customer_id: None,
        expected_date: None,
        notes: None,
        pipes: vec![InboundPipeItem {
            pipe_id,
            pipe_type: "seamless".into(),
            quantity: 1,
            weight: None,
            length: None,
        }],
    };

    let record = InventoryService::create_inbound(&pool, &dto).await.unwrap();
    assert_eq!(record.approval_status, "pending");

    // Approve
    InventoryService::approve_inbound(&pool, record.id)
        .await
        .expect("approve_inbound must succeed");

    // Verify record is now approved
    let updated: (String,) =
        sqlx::query_as("SELECT approval_status FROM inbound_records WHERE id = $1")
            .bind(record.id)
            .fetch_one(&pool)
            .await
            .expect("record must exist");
    assert_eq!(updated.0, "approved");

    // Verify pipe is now in_stock
    let pipe: (String,) = sqlx::query_as("SELECT status FROM seamless_pipes WHERE id = $1")
        .bind(pipe_id)
        .fetch_one(&pool)
        .await
        .expect("pipe must exist");
    assert_eq!(pipe.0, "in_stock");

    // Verify inventory log was created
    let log_count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM inventory_logs WHERE ref_id = $1 AND change_type = 'inbound'")
            .bind(record.id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(log_count.0, 1);
}

#[tokio::test]
async fn approve_inbound_fails_for_already_approved() {
    let pool = common::test_pool().await;

    let pipe_id = common::seed_seamless_pipe(&pool, "PN-004", "available", "J55")
        .await
        .unwrap();

    // Auto-approved inbound — pipe already in_stock
    let dto = CreateInboundRecordRequest {
        source_type: "purchase".into(),
        approval_status: "auto_approved".into(),
        reference_no: None,
        supplier_id: None,
        customer_id: None,
        expected_date: None,
        notes: None,
        pipes: vec![InboundPipeItem {
            pipe_id,
            pipe_type: "seamless".into(),
            quantity: 1,
            weight: None,
            length: None,
        }],
    };

    let record = InventoryService::create_inbound(&pool, &dto).await.unwrap();

    // Trying to approve it again must fail
    let err = InventoryService::approve_inbound(&pool, record.id)
        .await
        .expect_err("approve must fail for already approved");
    assert!(
        err.to_string().contains("Cannot approve inbound with status")
            || err.to_string().contains("'approved'")
    );
}

#[tokio::test]
async fn reject_inbound_only_updates_status() {
    let pool = common::test_pool().await;

    let pipe_id = common::seed_seamless_pipe(&pool, "PN-005", "available", "J55")
        .await
        .unwrap();

    let dto = CreateInboundRecordRequest {
        source_type: "purchase".into(),
        approval_status: "pending".into(),
        reference_no: None,
        supplier_id: None,
        customer_id: None,
        expected_date: None,
        notes: None,
        pipes: vec![InboundPipeItem {
            pipe_id,
            pipe_type: "seamless".into(),
            quantity: 1,
            weight: None,
            length: None,
        }],
    };

    let record = InventoryService::create_inbound(&pool, &dto).await.unwrap();

    InventoryService::reject_inbound(&pool, record.id, "material rejected")
        .await
        .expect("reject_inbound must succeed");

    // Verify record is rejected
    let updated: (String, Option<String>) = sqlx::query_as(
        "SELECT approval_status, rejection_reason FROM inbound_records WHERE id = $1",
    )
    .bind(record.id)
    .fetch_one(&pool)
    .await
    .expect("record must exist");
    assert_eq!(updated.0, "rejected");
    assert_eq!(updated.1.as_deref(), Some("material rejected"));

    // Pipe must still be "available" — rejection does NOT touch it
    let pipe: (String,) = sqlx::query_as("SELECT status FROM seamless_pipes WHERE id = $1")
        .bind(pipe_id)
        .fetch_one(&pool)
        .await
        .expect("pipe must exist");
    assert_eq!(pipe.0, "available");
}

/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
/// Inbound — deletion
/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn delete_inbound_deletes_auto_approved_record() {
    let pool = common::test_pool().await;

    let pipe_id = common::seed_seamless_pipe(&pool, "PN-006", "available", "J55")
        .await
        .unwrap();

    let dto = CreateInboundRecordRequest {
        source_type: "purchase".into(),
        approval_status: "auto_approved".into(),
        reference_no: None,
        supplier_id: None,
        customer_id: None,
        expected_date: None,
        notes: None,
        pipes: vec![InboundPipeItem {
            pipe_id,
            pipe_type: "seamless".into(),
            quantity: 1,
            weight: None,
            length: None,
        }],
    };

    let record = InventoryService::create_inbound(&pool, &dto).await.unwrap();

    InventoryService::delete_inbound(&pool, record.id)
        .await
        .expect("delete_inbound must succeed for auto_approved");

    // Record must be soft-deleted
    let deleted: (Option<String>,) =
        sqlx::query_as("SELECT deleted_at FROM inbound_records WHERE id = $1")
            .bind(record.id)
            .fetch_one(&pool)
            .await
            .expect("record must exist");
    assert!(deleted.0.is_some());
}

#[tokio::test]
async fn delete_inbound_fails_for_pending_record() {
    let pool = common::test_pool().await;

    let pipe_id = common::seed_seamless_pipe(&pool, "PN-007", "available", "J55")
        .await
        .unwrap();

    let dto = CreateInboundRecordRequest {
        source_type: "purchase".into(),
        approval_status: "pending".into(),
        reference_no: None,
        supplier_id: None,
        customer_id: None,
        expected_date: None,
        notes: None,
        pipes: vec![InboundPipeItem {
            pipe_id,
            pipe_type: "seamless".into(),
            quantity: 1,
            weight: None,
            length: None,
        }],
    };

    let record = InventoryService::create_inbound(&pool, &dto).await.unwrap();

    let err = InventoryService::delete_inbound(&pool, record.id)
        .await
        .expect_err("delete must fail for pending");
    assert!(
        err.to_string().contains("Cannot delete inbound with status")
            || err.to_string().contains("'pending'")
    );
}

/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
/// Outbound — stock validation
/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn create_outbound_fails_when_pipe_not_in_stock() {
    let pool = common::test_pool().await;

    // Pipe is "available" — not yet in stock
    let pipe_id = common::seed_seamless_pipe(&pool, "PN-008", "available", "J55")
        .await
        .unwrap();

    let dto = CreateOutboundRecordRequest {
        outbound_type: "sales".into(),
        customer_id: None,
        reference_no: Some("SO-001".into()),
        notes: None,
        pipes: vec![InboundPipeItem {
            pipe_id,
            pipe_type: "seamless".into(),
            quantity: 1,
            weight: None,
            length: None,
        }],
    };

    let err = InventoryService::create_outbound(&pool, &dto)
        .await
        .expect_err("outbound must fail for non-in-stock pipe");
    assert!(err.to_string().contains("not in stock"));
}

#[tokio::test]
async fn create_outbound_succeeds_for_in_stock_pipe() {
    let pool = common::test_pool().await;

    // First: inbound + approve to put pipe in stock
    let pipe_id = common::seed_seamless_pipe(&pool, "PN-009", "available", "J55")
        .await
        .unwrap();

    let inbound_dto = CreateInboundRecordRequest {
        source_type: "purchase".into(),
        approval_status: "pending".into(),
        reference_no: Some("PO-009".into()),
        supplier_id: None,
        customer_id: None,
        expected_date: None,
        notes: None,
        pipes: vec![InboundPipeItem {
            pipe_id,
            pipe_type: "seamless".into(),
            quantity: 1,
            weight: None,
            length: None,
        }],
    };
    let inbound = InventoryService::create_inbound(&pool, &inbound_dto)
        .await
        .unwrap();
    InventoryService::approve_inbound(&pool, inbound.id)
        .await
        .expect("approve must succeed");

    // Now create outbound
    let outbound_dto = CreateOutboundRecordRequest {
        outbound_type: "sales".into(),
        customer_id: None,
        reference_no: Some("SO-009".into()),
        notes: None,
        pipes: vec![InboundPipeItem {
            pipe_id,
            pipe_type: "seamless".into(),
            quantity: 1,
            weight: None,
            length: None,
        }],
    };

    let record = InventoryService::create_outbound(&pool, &outbound_dto)
        .await
        .expect("outbound must succeed for in_stock pipe");
    assert_eq!(record.outbound_type, "sales");
}

#[tokio::test]
async fn create_outbound_requires_at_least_one_pipe() {
    let pool = common::test_pool().await;

    let dto = CreateOutboundRecordRequest {
        outbound_type: "sales".into(),
        customer_id: None,
        reference_no: None,
        notes: None,
        pipes: vec![],
    };

    let err = InventoryService::create_outbound(&pool, &dto)
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
        zone: "A".into(),
        shelf: "01".into(),
        level: "01".into(),
        location_type: "storage".into(),
        max_capacity: Some(50),
        notes: None,
    };

    let location =
        InventoryService::create_location(&pool, &dto)
            .await
            .expect("create_location must succeed");

    assert_eq!(location.code, "A-01-01");
    assert_eq!(location.zone, "A");
    assert_eq!(location.shelf, "01");
    assert_eq!(location.level, "01");

    // List all locations
    let (locations, total) = InventoryService::list_locations(&pool, None)
        .await
        .expect("list_locations must succeed");
    assert_eq!(total, 1);
    assert_eq!(locations[0].code, "A-01-01");
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

    // Create check record
    let dto = CreateCheckRequest {
        check_type: "annual".into(),
        location_id,
        scheduled_date: None,
        notes: None,
    };

    let check = InventoryService::create_check_record(&pool, &dto, 1)
        .await
        .expect("create_check_record must succeed");

    assert_eq!(check.check_type, "annual");

    // Submit check items
    let submit = SubmitCheckItemRequest {
        check_id: check.id,
        items: vec![CheckInitItem {
            pipe_id: 9999, // non-existent pipe for test
            pipe_type: "seamless".into(),
            expected_status: "in_stock".into(),
            actual_status: Some("in_stock".into()),
            notes: None,
        }],
    };

    InventoryService::submit_check(&pool, &submit, 1)
        .await
        .expect("submit_check must succeed");
}

/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
/// ATP query
/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn atp_query_returns_availability() {
    let pool = common::test_pool().await;

    let pipe_id = common::seed_seamless_pipe(&pool, "PN-ATPC-001", "available", "J55")
        .await
        .unwrap();

    let dto = CreateInboundRecordRequest {
        source_type: "purchase".into(),
        approval_status: "pending".into(),
        reference_no: None,
        supplier_id: None,
        customer_id: None,
        expected_date: None,
        notes: None,
        pipes: vec![InboundPipeItem {
            pipe_id,
            pipe_type: "seamless".into(),
            quantity: 1,
            weight: None,
            length: None,
        }],
    };
    let inbound = InventoryService::create_inbound(&pool, &dto).await.unwrap();
    InventoryService::approve_inbound(&pool, inbound.id)
        .await
        .expect("approve must succeed");

    let atp_query = AtpQuery {
        pipe_type: "seamless".into(),
        grade: Some("J55".into()),
        od_min: Some(170.0),
        od_max: Some(180.0),
        wt_min: None,
        wt_max: None,
        length_min: None,
        length_max: None,
        location_id: None,
    };

    let result = InventoryService::atp_query(&pool, &atp_query, None)
        .await
        .expect("atp_query must succeed");

    assert!(result.total_quantity > 0, "should have at least 1 available");
}

/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
/// Pipe status state machine
/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn pipe_status_transitions_correctly_through_inbound_outbound_cycle() {
    let pool = common::test_pool().await;

    let pipe_id = common::seed_seamless_pipe(&pool, "PN-STATE-001", "available", "J55")
        .await
        .unwrap();

    // 1. Initial state: available
    let status: (String,) =
        sqlx::query_as("SELECT status FROM seamless_pipes WHERE id = $1")
            .bind(pipe_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(status.0, "available");

    // 2. Create pending inbound (no state change yet)
    let inbound_dto = CreateInboundRecordRequest {
        source_type: "purchase".into(),
        approval_status: "pending".into(),
        reference_no: None,
        supplier_id: None,
        customer_id: None,
        expected_date: None,
        notes: None,
        pipes: vec![InboundPipeItem {
            pipe_id,
            pipe_type: "seamless".into(),
            quantity: 1,
            weight: None,
            length: None,
        }],
    };
    let inbound = InventoryService::create_inbound(&pool, &inbound_dto)
        .await
        .unwrap();

    // 3. Approve → status becomes "in_stock"
    InventoryService::approve_inbound(&pool, inbound.id)
        .await
        .expect("approve must succeed");

    let status: (String,) =
        sqlx::query_as("SELECT status FROM seamless_pipes WHERE id = $1")
            .bind(pipe_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(status.0, "in_stock");

    // 4. Create outbound → status becomes "sold"
    let outbound_dto = CreateOutboundRecordRequest {
        outbound_type: "sales".into(),
        customer_id: None,
        reference_no: Some("SO-STATE-001".into()),
        notes: None,
        pipes: vec![InboundPipeItem {
            pipe_id,
            pipe_type: "seamless".into(),
            quantity: 1,
            weight: None,
            length: None,
        }],
    };
    let outbound = InventoryService::create_outbound(&pool, &outbound_dto)
        .await
        .unwrap();

    let status: (String,) =
        sqlx::query_as("SELECT status FROM seamless_pipes WHERE id = $1")
            .bind(pipe_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(status.0, "sold");
}