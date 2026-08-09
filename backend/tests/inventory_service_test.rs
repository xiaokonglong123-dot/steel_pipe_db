//! Integration tests for inventory services (inbound, outbound, location, check, ATP).
//!
//! Tests the core inventory workflows on the quantity-based generic ERP schema:
//! - Inbound record creation (auto_approved vs pending)
//! - Inbound approval/rejection
//! - Outbound creation with stock validation
//! - Location CRUD
//! - ATP query
//! - Full inbound → outbound item lifecycle
//!
//! All tests use a fresh SQLite test pool with migrations applied (tests::common).

mod common;

use erp_server::cache::CacheManager;
use erp_server::dto::common::PaginationParams;
use erp_server::dto::inventory_dto::{
    AtpQuery, CreateCheckRequest, CreateInboundRecordRequest, CreateLocationRequest,
    CreateOutboundRecordRequest, InboundItemRequest, OutboundItemRequest, SubmitCheckItemRequest,
};
use erp_server::inventory::check_service::CheckService;
use erp_server::inventory::inbound_service::InboundService;
use erp_server::inventory::inventory_query_service::InventoryQueryService;
use erp_server::inventory::location_service::LocationService;
use erp_server::inventory::outbound_service::OutboundService;

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

/// Signed on-hand stock for an item (mirrors InventoryRepo::stock_on_hand).
async fn on_hand(pool: &sqlx::SqlitePool, item_id: i64) -> f64 {
    let (v,): (f64,) = sqlx::query_as(
        "SELECT CAST(COALESCE(SUM(
             CASE WHEN change_type IN ('inbound', 'check_adjust') THEN quantity
                  ELSE -quantity END), 0.0) AS REAL)
         FROM inventory_logs WHERE item_id = ?",
    )
    .bind(item_id)
    .fetch_one(pool)
    .await
    .unwrap();
    v
}

/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
/// Inbound — auto_approved
/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn create_inbound_auto_approved_adds_stock() {
    let pool = common::test_pool().await;

    let item_id = seed_item(&pool, "ITM-T-001", "测试商品").await;

    // Create a "purchase" inbound — auto_approved by default
    let dto = CreateInboundRecordRequest {
        inbound_type: "purchase".into(),
        order_id: None,
        supplier_id: None,
        notes: Some("test inbound".into()),
        items: vec![InboundItemRequest {
            item_id,
            quantity: 10.0,
        }],
    };

    let record = InboundService::create_inbound(&pool, &dto)
        .await
        .expect("create_inbound must succeed");

    assert_eq!(record.approval_status, "auto_approved");

    // Verify stock was increased
    assert_eq!(on_hand(&pool, item_id).await, 10.0);

    // Verify an inbound inventory log was created
    let log_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM inventory_logs WHERE ref_id = ? AND change_type = 'inbound'",
    )
    .bind(record.id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(log_count.0, 1);
}

#[tokio::test]
async fn create_inbound_pending_does_not_change_stock() {
    let pool = common::test_pool().await;

    let item_id = seed_item(&pool, "ITM-T-002", "测试商品").await;

    // "return" inbound — starts as pending (not auto_approved)
    let dto = CreateInboundRecordRequest {
        inbound_type: "return".into(),
        order_id: None,
        supplier_id: None,
        notes: None,
        items: vec![InboundItemRequest {
            item_id,
            quantity: 5.0,
        }],
    };

    let record = InboundService::create_inbound(&pool, &dto)
        .await
        .expect("create_inbound must succeed");
    assert_eq!(record.approval_status, "pending");

    // Stock unchanged for pending records
    assert_eq!(on_hand(&pool, item_id).await, 0.0);
}

#[tokio::test]
async fn create_inbound_requires_at_least_one_item() {
    let pool = common::test_pool().await;

    let dto = CreateInboundRecordRequest {
        inbound_type: "purchase".into(),
        order_id: None,
        supplier_id: None,
        notes: None,
        items: vec![],
    };

    let err = InboundService::create_inbound(&pool, &dto)
        .await
        .expect_err("must fail with no items");
    assert!(err.to_string().contains("At least one item"));
}

#[tokio::test]
async fn create_inbound_rejects_unknown_item() {
    let pool = common::test_pool().await;

    let dto = CreateInboundRecordRequest {
        inbound_type: "purchase".into(),
        order_id: None,
        supplier_id: None,
        notes: None,
        items: vec![InboundItemRequest {
            item_id: 99999,
            quantity: 1.0,
        }],
    };

    let err = InboundService::create_inbound(&pool, &dto)
        .await
        .expect_err("must fail for unknown item");
    assert!(err.to_string().contains("not found"));
}

/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
/// Inbound — manual approval / rejection
/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn approve_inbound_updates_pending_record_and_stock() {
    let pool = common::test_pool().await;

    let item_id = seed_item(&pool, "ITM-T-003", "测试商品").await;

    // "return" inbound starts as pending
    let dto = CreateInboundRecordRequest {
        inbound_type: "return".into(),
        order_id: None,
        supplier_id: None,
        notes: None,
        items: vec![InboundItemRequest {
            item_id,
            quantity: 7.0,
        }],
    };

    let record = InboundService::create_inbound(&pool, &dto).await.unwrap();
    assert_eq!(record.approval_status, "pending");

    // Approve
    InboundService::approve_inbound(&pool, record.id, None, None)
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

    // Verify stock was added after approval
    assert_eq!(on_hand(&pool, item_id).await, 7.0);

    // Verify inventory log was created
    let log_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM inventory_logs WHERE ref_id = ? AND change_type = 'inbound'",
    )
    .bind(record.id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(log_count.0, 1);
}

#[tokio::test]
async fn approve_inbound_fails_for_already_approved() {
    let pool = common::test_pool().await;

    let item_id = seed_item(&pool, "ITM-T-004", "测试商品").await;

    // "purchase" inbound — auto_approved
    let dto = CreateInboundRecordRequest {
        inbound_type: "purchase".into(),
        order_id: None,
        supplier_id: None,
        notes: None,
        items: vec![InboundItemRequest {
            item_id,
            quantity: 1.0,
        }],
    };

    let record = InboundService::create_inbound(&pool, &dto).await.unwrap();

    // Trying to approve it again must fail
    let err = InboundService::approve_inbound(&pool, record.id, None, None)
        .await
        .expect_err("approve must fail for already approved");
    assert!(
        err.to_string()
            .contains("Cannot approve inbound with status")
            || err.to_string().contains("auto_approved")
    );
}

#[tokio::test]
async fn reject_inbound_only_updates_status() {
    let pool = common::test_pool().await;

    let item_id = seed_item(&pool, "ITM-T-005", "测试商品").await;

    let dto = CreateInboundRecordRequest {
        inbound_type: "return".into(),
        order_id: None,
        supplier_id: None,
        notes: None,
        items: vec![InboundItemRequest {
            item_id,
            quantity: 3.0,
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

    // No stock change for rejected record
    assert_eq!(on_hand(&pool, item_id).await, 0.0);
}

/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
/// Inbound — deletion
/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn delete_inbound_deletes_auto_approved_record() {
    let pool = common::test_pool().await;

    let item_id = seed_item(&pool, "ITM-T-006", "测试商品").await;

    let dto = CreateInboundRecordRequest {
        inbound_type: "purchase".into(),
        order_id: None,
        supplier_id: None,
        notes: None,
        items: vec![InboundItemRequest {
            item_id,
            quantity: 2.0,
        }],
    };

    let record = InboundService::create_inbound(&pool, &dto).await.unwrap();

    InboundService::delete_inbound(&pool, record.id)
        .await
        .expect("delete_inbound must succeed for auto_approved");

    // Record must be soft-deleted
    let deleted: (Option<chrono::DateTime<chrono::Utc>>,) =
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

    let item_id = seed_item(&pool, "ITM-T-007", "测试商品").await;

    let dto = CreateInboundRecordRequest {
        inbound_type: "return".into(),
        order_id: None,
        supplier_id: None,
        notes: None,
        items: vec![InboundItemRequest {
            item_id,
            quantity: 1.0,
        }],
    };

    let record = InboundService::create_inbound(&pool, &dto).await.unwrap();

    let err = InboundService::delete_inbound(&pool, record.id)
        .await
        .expect_err("delete must fail for pending");
    assert!(
        err.to_string()
            .contains("Cannot delete inbound with status")
            || err.to_string().contains("pending")
    );
}

/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
/// Outbound — stock validation
/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn create_outbound_fails_when_stock_insufficient() {
    let pool = common::test_pool().await;

    let item_id = seed_item(&pool, "ITM-T-008", "测试商品").await;

    let dto = CreateOutboundRecordRequest {
        outbound_type: "sales".into(),
        order_id: None,
        customer_id: None,
        notes: None,
        items: vec![OutboundItemRequest {
            item_id,
            quantity: 5.0,
        }],
    };

    let err = OutboundService::create_outbound(&pool, &dto)
        .await
        .expect_err("outbound must fail without stock");
    assert!(err.to_string().contains("Insufficient stock"));
}

#[tokio::test]
async fn create_outbound_succeeds_for_in_stock_item() {
    let pool = common::test_pool().await;

    // First: inbound to put item in stock (purchase = auto_approved)
    let item_id = seed_item(&pool, "ITM-T-009", "测试商品").await;

    let inbound_dto = CreateInboundRecordRequest {
        inbound_type: "purchase".into(),
        order_id: Some(9009),
        supplier_id: None,
        notes: None,
        items: vec![InboundItemRequest {
            item_id,
            quantity: 20.0,
        }],
    };
    InboundService::create_inbound(&pool, &inbound_dto).await.unwrap();

    // Now create outbound (sales = auto_approved, executes immediately)
    let outbound_dto = CreateOutboundRecordRequest {
        outbound_type: "sales".into(),
        order_id: None,
        customer_id: None,
        notes: None,
        items: vec![OutboundItemRequest {
            item_id,
            quantity: 5.0,
        }],
    };

    let record = OutboundService::create_outbound(&pool, &outbound_dto)
        .await
        .expect("outbound must succeed for in_stock item");
    assert_eq!(record.outbound_type, "sales");
    assert_eq!(on_hand(&pool, item_id).await, 15.0);
}

#[tokio::test]
async fn create_outbound_requires_at_least_one_item() {
    let pool = common::test_pool().await;

    let dto = CreateOutboundRecordRequest {
        outbound_type: "sales".into(),
        order_id: None,
        customer_id: None,
        notes: None,
        items: vec![],
    };

    let err = OutboundService::create_outbound(&pool, &dto)
        .await
        .expect_err("must fail with no items");
    assert!(err.to_string().contains("At least one item"));
}

/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
/// Locations
/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn create_and_list_locations() {
    let pool = common::test_pool().await;
    let cache = CacheManager::new();

    let dto = CreateLocationRequest {
        zone_code: "A".into(),
        shelf_code: "01".into(),
        level_code: "01".into(),
        description: None,
        capacity: Some(50),
    };

    let location = LocationService::create_location(&pool, &cache, &dto)
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
    assert!(total >= 1, "should have at least 1 location, got {}", total);
    assert!(
        locations.iter().any(|l| l.full_code == "A-01-01"),
        "should contain A-01-01"
    );
}

/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
/// Inventory check
/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn create_and_submit_check_record() {
    let pool = common::test_pool().await;

    // Stock one item so the check initializer finds it.
    let item_id = seed_item(&pool, "ITM-CHK-001", "盘点商品").await;
    let inbound_dto = CreateInboundRecordRequest {
        inbound_type: "purchase".into(),
        order_id: None,
        supplier_id: None,
        notes: None,
        items: vec![InboundItemRequest {
            item_id,
            quantity: 12.0,
        }],
    };
    InboundService::create_inbound(&pool, &inbound_dto).await.unwrap();

    // Create a check record — auto-scans stocked items
    let dto = CreateCheckRequest {
        location_id: None,
        notes: None,
    };

    let check = CheckService::create_check(&pool, &dto)
        .await
        .expect("create_check must succeed");

    assert_eq!(check.status, "in_progress");

    // Get the auto-generated check items
    let (_record, items) = CheckService::get_check_detail(&pool, check.id)
        .await
        .expect("get_check_detail must succeed");
    assert!(!items.is_empty(), "check should have at least one item");
    assert_eq!(items[0].item_id, item_id);
    assert_eq!(items[0].expected_quantity, Some(12.0));

    let check_item_id = items[0].id;

    // Submit the result for a single check item
    let submit = SubmitCheckItemRequest {
        found_quantity: 12.0,
        notes: None,
    };

    CheckService::submit_check_item(&pool, check.id, check_item_id, &submit)
        .await
        .expect("submit_check_item must succeed");

    // Verify the item was updated and matched
    let updated_items = CheckService::get_check_detail(&pool, check.id)
        .await
        .expect("get_check_detail must succeed")
        .1;
    let submitted = updated_items.iter().find(|i| i.id == check_item_id).unwrap();
    assert_eq!(submitted.found_quantity, Some(12.0));
    assert_eq!(submitted.is_match, Some(true));
}

/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
/// ATP query
/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn atp_query_returns_availability() {
    let pool = common::test_pool().await;

    let item_id = seed_item(&pool, "ITM-ATPC-001", "ATP商品").await;

    // Purchase inbound = auto_approved → stock available
    let dto = CreateInboundRecordRequest {
        inbound_type: "purchase".into(),
        order_id: None,
        supplier_id: None,
        notes: None,
        items: vec![InboundItemRequest {
            item_id,
            quantity: 100.0,
        }],
    };
    InboundService::create_inbound(&pool, &dto).await.unwrap();

    let atp_query = AtpQuery {
        item_id: Some(item_id),
        location_id: None,
    };

    let result = InventoryQueryService::check_atp(&pool, &atp_query)
        .await
        .expect("check_atp must succeed");

    assert!(!result.is_empty(), "should have at least 1 available item");
    assert_eq!(result[0].item_id, item_id);
    assert_eq!(result[0].quantity, 100.0);
}

/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
/// Item lifecycle through inbound → outbound
/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn stock_transitions_correctly_through_inbound_outbound_cycle() {
    let pool = common::test_pool().await;

    let item_id = seed_item(&pool, "ITM-STATE-001", "生命周期商品").await;
    assert_eq!(on_hand(&pool, item_id).await, 0.0);

    // 1. Pending inbound ("return") — no stock yet
    let inbound_dto = CreateInboundRecordRequest {
        inbound_type: "return".into(),
        order_id: None,
        supplier_id: None,
        notes: None,
        items: vec![InboundItemRequest {
            item_id,
            quantity: 30.0,
        }],
    };
    let inbound = InboundService::create_inbound(&pool, &inbound_dto).await.unwrap();
    assert_eq!(on_hand(&pool, item_id).await, 0.0);

    // 2. Approve → stock becomes 30
    InboundService::approve_inbound(&pool, inbound.id, None, None)
        .await
        .expect("approve must succeed");
    assert_eq!(on_hand(&pool, item_id).await, 30.0);

    // 3. Outbound (sales = auto_approved) → stock becomes 20
    let outbound_dto = CreateOutboundRecordRequest {
        outbound_type: "sales".into(),
        order_id: None,
        customer_id: None,
        notes: None,
        items: vec![OutboundItemRequest {
            item_id,
            quantity: 10.0,
        }],
    };
    OutboundService::create_outbound(&pool, &outbound_dto).await.unwrap();
    assert_eq!(on_hand(&pool, item_id).await, 20.0);

    // 4. Over-deduction must be rejected
    let over_dto = CreateOutboundRecordRequest {
        outbound_type: "sales".into(),
        order_id: None,
        customer_id: None,
        notes: None,
        items: vec![OutboundItemRequest {
            item_id,
            quantity: 100.0,
        }],
    };
    let err = OutboundService::create_outbound(&pool, &over_dto)
        .await
        .expect_err("outbound exceeding stock must fail");
    assert!(err.to_string().contains("Insufficient stock"));
}
