//! Integration tests for `PurchaseService` and `SalesService` — purchase and sales order lifecycle.
//!
//! Covers PO/SO creation, status transitions, approval/rejection, item management,
//! soft-delete, linking to inbound/outbound, and ATP validation for sales orders.
//!
//! All tests use an in-memory SQLite database with fresh migrations.

mod common;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Item helpers (generic ERP) — item master + stock seeding
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Seed an item master row; returns its id.
async fn seed_item(pool: &sqlx::SqlitePool, sku: &str) -> i64 {
    sqlx::query_scalar(
        "INSERT INTO items (sku, name, category, unit, spec, price, status) \
         VALUES (?, 'Test Item', '原材料', '个', 'spec', 10.0, 'active') \
         RETURNING id",
    )
    .bind(sku)
    .fetch_one(pool)
    .await
    .expect("seed_item must succeed")
}

/// Seed an inventory_logs entry for an item (inbound/outbound) to simulate stock.
async fn seed_inventory_log(pool: &sqlx::SqlitePool, item_id: i64, change_type: &str, quantity: f64) {
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

use chrono::{DateTime, Utc};
use rust_decimal_macros::dec;
use erp_server::dto::common::PaginationParams;
use erp_server::dto::purchase_dto::{
    ApproveOrderRequest as PurchaseApproveReq, CreatePurchaseItemRequest,
    CreatePurchaseOrderRequest, PurchaseOrderFilterParams, PurchaseOrderStatusTransitionRequest,
    RejectOrderRequest as PurchaseRejectReq, UpdatePurchaseItemRequest, UpdatePurchaseOrderRequest,
};
use erp_server::dto::sales_dto::{
    ApproveOrderRequest as SalesApproveReq, CreateSalesItemRequest, CreateSalesOrderRequest,
    RejectOrderRequest as SalesRejectReq, SalesOrderFilterParams,
    SalesOrderStatusTransitionRequest, UpdateSalesItemRequest, UpdateSalesOrderRequest,
};
use erp_server::services::purchase_service::PurchaseService;
use erp_server::services::sales_service::SalesService;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Purchase Order — create
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn create_purchase_order_with_items() {
    let pool = common::test_pool().await;
    let item_a = seed_item(&pool, "ITM-CREATE_PURCHASE_ORDE-A").await;

    let supplier_id = common::seed_supplier(&pool, "SUP-001", "Test Supplier")
        .await
        .unwrap();

    let dto = CreatePurchaseOrderRequest {
        order_no: None,
        supplier_id,
        order_date: "2025-06-01".into(),
        notes: Some("initial PO".into()),
        items: vec![CreatePurchaseItemRequest {
            item_id: item_a,
            quantity: 100.0,
            unit_price: Some(dec!(150.0)),
            total_price: Some(dec!(15000.0)),
            notes: None,
        }],
    };

    let order = PurchaseService::create_purchase_order(&pool, &dto)
        .await
        .expect("create_purchase_order must succeed");

    assert!(order.id > 0);
    assert!(order.order_no.starts_with("PO-"));
    assert_eq!(order.status, "draft");
    assert_eq!(order.supplier_id, supplier_id);
    assert_eq!(order.total_amount, Some(15000.0));
}

#[tokio::test]
async fn create_purchase_order_fails_empty_items() {
    let pool = common::test_pool().await;

    let supplier_id = common::seed_supplier(&pool, "SUP-002", "Empty Items Supplier")
        .await
        .unwrap();

    let dto = CreatePurchaseOrderRequest {
        order_no: None,
        supplier_id,
        order_date: "2025-06-01".into(),
        notes: None,
        items: vec![],
    };

    let err = PurchaseService::create_purchase_order(&pool, &dto)
        .await
        .expect_err("must fail with empty items");
    assert!(err.to_string().contains("At least one item"));
}

#[tokio::test]
async fn create_purchase_order_fails_inactive_supplier() {
    let pool = common::test_pool().await;
    let item_a = seed_item(&pool, "ITM-CREATE_PURCHASE_ORDE-A").await;

    // Manually insert an inactive supplier
    let supplier_id: i64 = sqlx::query_scalar(
        "INSERT INTO suppliers (supplier_code, name, contact_person, phone, email, address, \
         is_active, notes, created_at, updated_at) \
         VALUES (?, ?, 'Contact', '13800138000', ?, 'Addr', 0, 'inactive', \
         datetime('now'), datetime('now')) RETURNING id",
    )
    .bind("SUP-003")
    .bind("Inactive Supplier")
    .bind("sup003@test.local")
    .fetch_one(&pool)
    .await
    .unwrap();

    let dto = CreatePurchaseOrderRequest {
        order_no: None,
        supplier_id,
        order_date: "2025-06-01".into(),
        notes: None,
        items: vec![CreatePurchaseItemRequest {
            item_id: item_a,
            quantity: 10.0,
            unit_price: Some(dec!(100.0)),
            total_price: Some(dec!(1000.0)),
            notes: None,
        }],
    };

    let err = PurchaseService::create_purchase_order(&pool, &dto)
        .await
        .expect_err("must fail for inactive supplier");
    assert!(err.to_string().contains("not active"));
}

#[tokio::test]
async fn create_purchase_order_fails_duplicate_order_no() {
    let pool = common::test_pool().await;
    let item_a = seed_item(&pool, "ITM-CREATE_PURCHASE_ORDE-A").await;

    let supplier_id = common::seed_supplier(&pool, "SUP-004", "Supplier Dup")
        .await
        .unwrap();

    let dto = CreatePurchaseOrderRequest {
        order_no: Some("PO-DUP-001".into()),
        supplier_id,
        order_date: "2025-06-01".into(),
        notes: None,
        items: vec![CreatePurchaseItemRequest {
            item_id: item_a,
            quantity: 10.0,
            unit_price: Some(dec!(100.0)),
            total_price: Some(dec!(1000.0)),
            notes: None,
        }],
    };

    // First creation should succeed
    PurchaseService::create_purchase_order(&pool, &dto)
        .await
        .expect("first create must succeed");

    // Second creation with same order_no should fail
    let err = PurchaseService::create_purchase_order(&pool, &dto)
        .await
        .expect_err("duplicate order_no must fail");
    assert!(err.to_string().contains("already exists"));
}

#[tokio::test]
async fn create_purchase_order_fails_nonexistent_supplier() {
    let pool = common::test_pool().await;
    let item_a = seed_item(&pool, "ITM-CREATE_PURCHASE_ORDE-A").await;

    let dto = CreatePurchaseOrderRequest {
        order_no: None,
        supplier_id: 99999,
        order_date: "2025-06-01".into(),
        notes: None,
        items: vec![CreatePurchaseItemRequest {
            item_id: item_a,
            quantity: 10.0,
            unit_price: Some(dec!(100.0)),
            total_price: Some(dec!(1000.0)),
            notes: None,
        }],
    };

    let err = PurchaseService::create_purchase_order(&pool, &dto)
        .await
        .expect_err("must fail for nonexistent supplier");
    assert!(err.to_string().contains("Supplier"));
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Purchase Order — update header
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn update_purchase_order_updates_header() {
    let pool = common::test_pool().await;
    let item_a = seed_item(&pool, "ITM-UPDATE_PURCHASE_ORDE-A").await;

    let supplier_id = common::seed_supplier(&pool, "SUP-UPD", "Update Supplier")
        .await
        .unwrap();

    let dto = CreatePurchaseOrderRequest {
        order_no: Some("PO-UPD-001".into()),
        supplier_id,
        order_date: "2025-06-01".into(),
        notes: Some("original".into()),
        items: vec![CreatePurchaseItemRequest {
            item_id: item_a,
            quantity: 50.0,
            unit_price: Some(dec!(100.0)),
            total_price: Some(dec!(5000.0)),
            notes: None,
        }],
    };

    let order = PurchaseService::create_purchase_order(&pool, &dto)
        .await
        .unwrap();

    let update = UpdatePurchaseOrderRequest {
        order_date: Some("2025-07-01".into()),
        notes: Some("updated notes".into()),
        items: None,
    };

    let updated = PurchaseService::update_purchase_order(&pool, order.id, &update)
        .await
        .expect("update_purchase_order must succeed");

    assert_eq!(updated.notes.as_deref(), Some("updated notes"));
}

#[tokio::test]
async fn update_purchase_order_fails_non_draft() {
    let pool = common::test_pool().await;
    let item_a = seed_item(&pool, "ITM-UPDATE_PURCHASE_ORDE-A").await;

    let supplier_id = common::seed_supplier(&pool, "SUP-UPD2", "Supplier")
        .await
        .unwrap();

    let dto = CreatePurchaseOrderRequest {
        order_no: Some("PO-UPD-002".into()),
        supplier_id,
        order_date: "2025-06-01".into(),
        notes: None,
        items: vec![CreatePurchaseItemRequest {
            item_id: item_a,
            quantity: 10.0,
            unit_price: Some(dec!(100.0)),
            total_price: Some(dec!(1000.0)),
            notes: None,
        }],
    };

    let order = PurchaseService::create_purchase_order(&pool, &dto)
        .await
        .unwrap();

    // Transition to pending
    let trans = PurchaseOrderStatusTransitionRequest {
        status: "pending".into(),
    };
    PurchaseService::transition_purchase_status(&pool, order.id, &trans)
        .await
        .unwrap();

    // Now try to update — must fail
    let update = UpdatePurchaseOrderRequest {
        order_date: None,
        notes: Some("should fail".into()),
        items: None,
    };
    let err = PurchaseService::update_purchase_order(&pool, order.id, &update)
        .await
        .expect_err("update must fail for non-draft order");
    assert!(err.to_string().contains("Cannot modify"));
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Purchase Order — status transitions
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn transition_purchase_status_draft_to_pending() {
    let pool = common::test_pool().await;

    let supplier_id = common::seed_supplier(&pool, "SUP-TRN", "Trans Supplier")
        .await
        .unwrap();
    let order = create_dummy_po(&pool, supplier_id).await;

    let trans = PurchaseOrderStatusTransitionRequest {
        status: "pending".into(),
    };
    PurchaseService::transition_purchase_status(&pool, order.id, &trans)
        .await
        .expect("draft -> pending must succeed");

    let (fetched, _items) = PurchaseService::get_purchase_order(&pool, order.id)
        .await
        .unwrap();
    assert_eq!(fetched.status, "pending");
}

#[tokio::test]
async fn transition_purchase_status_draft_to_cancelled() {
    let pool = common::test_pool().await;

    let supplier_id = common::seed_supplier(&pool, "SUP-TRN2", "Supplier")
        .await
        .unwrap();
    let order = create_dummy_po(&pool, supplier_id).await;

    let trans = PurchaseOrderStatusTransitionRequest {
        status: "cancelled".into(),
    };
    PurchaseService::transition_purchase_status(&pool, order.id, &trans)
        .await
        .expect("draft -> cancelled must succeed");

    let (fetched, _) = PurchaseService::get_purchase_order(&pool, order.id)
        .await
        .unwrap();
    assert_eq!(fetched.status, "cancelled");
}

#[tokio::test]
async fn transition_purchase_status_invalid_hop_fails() {
    let pool = common::test_pool().await;

    let supplier_id = common::seed_supplier(&pool, "SUP-TRN3", "Supplier")
        .await
        .unwrap();
    let order = create_dummy_po(&pool, supplier_id).await;

    // draft -> approved is invalid
    let trans = PurchaseOrderStatusTransitionRequest {
        status: "approved".into(),
    };
    let err = PurchaseService::transition_purchase_status(&pool, order.id, &trans)
        .await
        .expect_err("draft -> approved must fail");
    assert!(err.to_string().contains("Cannot transition"));
}

#[tokio::test]
async fn transition_purchase_status_full_flow() {
    let pool = common::test_pool().await;

    let supplier_id = common::seed_supplier(&pool, "SUP-FLOW", "Flow Supplier")
        .await
        .unwrap();
    let order = create_dummy_po(&pool, supplier_id).await;

    // draft -> pending
    let t1 = PurchaseOrderStatusTransitionRequest {
        status: "pending".into(),
    };
    PurchaseService::transition_purchase_status(&pool, order.id, &t1)
        .await
        .unwrap();

    // pending -> approved
    let t2 = PurchaseOrderStatusTransitionRequest {
        status: "approved".into(),
    };
    PurchaseService::transition_purchase_status(&pool, order.id, &t2)
        .await
        .unwrap();

    let (fetched, _) = PurchaseService::get_purchase_order(&pool, order.id)
        .await
        .unwrap();
    assert_eq!(fetched.status, "approved");
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Purchase Order — get / list
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn get_purchase_order_with_items() {
    let pool = common::test_pool().await;
    let item_a = seed_item(&pool, "ITM-GET_PURCHASE_ORDER_W-A").await;
    let item_b = seed_item(&pool, "ITM-GET_PURCHASE_ORDER_W-B").await;

    let supplier_id = common::seed_supplier(&pool, "SUP-GET", "Get Supplier")
        .await
        .unwrap();
    let dto = CreatePurchaseOrderRequest {
        order_no: Some("PO-GET-001".into()),
        supplier_id,
        order_date: "2025-06-01".into(),
        notes: None,
        items: vec![
            CreatePurchaseItemRequest {
            item_id: item_a,
            quantity: 20.0,
                unit_price: Some(dec!(100.0)),
                total_price: Some(dec!(2000.0)),
                notes: None,
            },
            CreatePurchaseItemRequest {
            item_id: item_b,
            quantity: 30.0,
                unit_price: Some(dec!(80.0)),
                total_price: Some(dec!(2400.0)),
                notes: None,
            },
        ],
    };

    let order = PurchaseService::create_purchase_order(&pool, &dto)
        .await
        .unwrap();

    let (fetched, items) = PurchaseService::get_purchase_order(&pool, order.id)
        .await
        .expect("get_purchase_order must succeed");

    assert_eq!(fetched.id, order.id);
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].order_id, order.id);
}

#[tokio::test]
async fn get_purchase_order_fails_not_found() {
    let pool = common::test_pool().await;

    let err = PurchaseService::get_purchase_order(&pool, 99999)
        .await
        .expect_err("must fail for nonexistent order");
    assert!(err.to_string().contains("not found"));
}

#[tokio::test]
async fn list_purchase_orders_pagination() {
    let pool = common::test_pool().await;
    let item_a = seed_item(&pool, "ITM-LIST_PURCHASE_ORDERS-A").await;

    let supplier_id = common::seed_supplier(&pool, "SUP-LST", "List Supplier")
        .await
        .unwrap();

    // Create 3 POs
    for i in 1..=3 {
        let dto = CreatePurchaseOrderRequest {
            order_no: Some(format!("PO-LST-{:03}", i)),
            supplier_id,
            order_date: "2025-06-01".into(),
            notes: None,
            items: vec![CreatePurchaseItemRequest {
            item_id: item_a,
            quantity: 10.0,
                unit_price: Some(dec!(100.0)),
                total_price: Some(dec!(1000.0)),
                notes: None,
            }],
        };
        PurchaseService::create_purchase_order(&pool, &dto)
            .await
            .unwrap();
    }

    let filter = PurchaseOrderFilterParams {
        q: None,
        status: None,
        supplier_id: None,
        order_date_from: None,
        order_date_to: None,
        page: None,
        page_size: None,
        sort_by: None,
        sort_order: None,
    };
    let params = PaginationParams {
        page: Some(1),
        page_size: Some(2),
        sort_by: None,
        sort_order: None,
    };

    let (orders, total) = PurchaseService::list_purchase_orders(&pool, &filter, &params)
        .await
        .expect("list must succeed");

    assert_eq!(orders.len(), 2);
    assert_eq!(total, 3);
}

#[tokio::test]
async fn list_purchase_orders_status_filter() {
    let pool = common::test_pool().await;
    let item_a = seed_item(&pool, "ITM-LIST_PURCHASE_ORDERS-A").await;
    let item_b = seed_item(&pool, "ITM-LIST_PURCHASE_ORDERS-B").await;

    let supplier_id = common::seed_supplier(&pool, "SUP-FLT", "Filter Supplier")
        .await
        .unwrap();

    // Create 2 POs — one draft, one pending
    let order1 = {
        let dto = CreatePurchaseOrderRequest {
            order_no: Some("PO-FLT-001".into()),
            supplier_id,
            order_date: "2025-06-01".into(),
            notes: None,
            items: vec![CreatePurchaseItemRequest {
            item_id: item_a,
            quantity: 10.0,
                unit_price: Some(dec!(100.0)),
                total_price: Some(dec!(1000.0)),
                notes: None,
            }],
        };
        PurchaseService::create_purchase_order(&pool, &dto)
            .await
            .unwrap()
    };

    let order2 = {
        let dto = CreatePurchaseOrderRequest {
            order_no: Some("PO-FLT-002".into()),
            supplier_id,
            order_date: "2025-06-01".into(),
            notes: None,
            items: vec![CreatePurchaseItemRequest {
            item_id: item_b,
            quantity: 10.0,
                unit_price: Some(dec!(100.0)),
                total_price: Some(dec!(1000.0)),
                notes: None,
            }],
        };
        let o = PurchaseService::create_purchase_order(&pool, &dto)
            .await
            .unwrap();
        let trans = PurchaseOrderStatusTransitionRequest {
            status: "pending".into(),
        };
        PurchaseService::transition_purchase_status(&pool, o.id, &trans)
            .await
            .unwrap();
        o
    };

    // Filter by status = "draft"
    let filter = PurchaseOrderFilterParams {
        q: None,
        status: Some("draft".into()),
        supplier_id: None,
        order_date_from: None,
        order_date_to: None,
        page: None,
        page_size: None,
        sort_by: None,
        sort_order: None,
    };
    let params = PaginationParams {
        page: None,
        page_size: None,
        sort_by: None,
        sort_order: None,
    };

    let (orders, total) = PurchaseService::list_purchase_orders(&pool, &filter, &params)
        .await
        .expect("list with status filter must succeed");
    assert_eq!(total, 1);
    assert_eq!(orders[0].id, order1.id);
    assert_eq!(orders[0].status, "draft");
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Purchase Order — delete (soft)
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn delete_purchase_order_draft() {
    let pool = common::test_pool().await;

    let supplier_id = common::seed_supplier(&pool, "SUP-DEL", "Del Supplier")
        .await
        .unwrap();
    let order = create_dummy_po(&pool, supplier_id).await;

    PurchaseService::delete_purchase_order(&pool, order.id)
        .await
        .expect("delete draft PO must succeed");

    // Verify soft-deleted
    let deleted_at: (Option<DateTime<Utc>>,) =
        sqlx::query_as("SELECT deleted_at FROM purchase_orders WHERE id = ?")
            .bind(order.id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(deleted_at.0.is_some());

    // get should fail
    let err = PurchaseService::get_purchase_order(&pool, order.id)
        .await
        .expect_err("deleted order should not be findable");
    assert!(err.to_string().contains("not found"));
}

#[tokio::test]
async fn delete_purchase_order_fails_approved() {
    let pool = common::test_pool().await;

    let supplier_id = common::seed_supplier(&pool, "SUP-DEL2", "Supplier")
        .await
        .unwrap();

    // Seed a PO directly with "approved" status to bypass the transition
    let order_id = common::seed_purchase_order(&pool, "PO-DEL-002", supplier_id, "approved")
        .await
        .unwrap();

    let err = PurchaseService::delete_purchase_order(&pool, order_id)
        .await
        .expect_err("deleting approved PO must fail");
    assert!(err.to_string().contains("Cannot delete"));
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Purchase Order — update item
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn update_purchase_item_changes_qty() {
    let pool = common::test_pool().await;
    let item_a = seed_item(&pool, "ITM-UPDATE_PURCHASE_ITEM-A").await;

    let supplier_id = common::seed_supplier(&pool, "SUP-ITM", "Item Supplier")
        .await
        .unwrap();

    // Create PO with one item
    let dto = CreatePurchaseOrderRequest {
        order_no: Some("PO-ITM-001".into()),
        supplier_id,
        order_date: "2025-06-01".into(),
        notes: None,
        items: vec![CreatePurchaseItemRequest {
            item_id: item_a,
            quantity: 50.0,
            unit_price: Some(dec!(100.0)),
            total_price: Some(dec!(5000.0)),
            notes: None,
        }],
    };
    let order = PurchaseService::create_purchase_order(&pool, &dto)
        .await
        .unwrap();

    let (_order, items) = PurchaseService::get_purchase_order(&pool, order.id)
        .await
        .unwrap();
    let item_id = items[0].id;

    let update = UpdatePurchaseItemRequest {
        item_id: None,
        id: None,
        quantity: Some(75.0),
        unit_price: Some(dec!(90.0)),
        notes: None,
    };

    let (_order, updated_item) =
        PurchaseService::update_purchase_item(&pool, order.id, item_id, &update)
            .await
            .expect("update_purchase_item must succeed");

    assert_eq!(updated_item.quantity, 75.0);
    assert_eq!(updated_item.unit_price, Some(90.0));
}

#[tokio::test]
async fn update_purchase_item_fails_non_draft() {
    let pool = common::test_pool().await;

    let supplier_id = common::seed_supplier(&pool, "SUP-ITM2", "Supplier")
        .await
        .unwrap();
    let order = create_dummy_po(&pool, supplier_id).await;

    // Transition to pending
    let trans = PurchaseOrderStatusTransitionRequest {
        status: "pending".into(),
    };
    PurchaseService::transition_purchase_status(&pool, order.id, &trans)
        .await
        .unwrap();

    let update = UpdatePurchaseItemRequest {
        item_id: None,
        id: None,
        quantity: Some(99.0),
        unit_price: None,
        notes: None,
    };

    let err = PurchaseService::update_purchase_item(&pool, order.id, 0, &update)
        .await
        .expect_err("update must fail for non-draft");
    assert!(err.to_string().contains("Cannot modify"));
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Purchase Order — delete item
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn delete_purchase_item_removes_item() {
    let pool = common::test_pool().await;
    let item_a = seed_item(&pool, "ITM-DELETE_PURCHASE_ITEM-A").await;
    let item_b = seed_item(&pool, "ITM-DELETE_PURCHASE_ITEM-B").await;

    let supplier_id = common::seed_supplier(&pool, "SUP-DLI", "DelItem Supplier")
        .await
        .unwrap();

    let dto = CreatePurchaseOrderRequest {
        order_no: Some("PO-DLI-001".into()),
        supplier_id,
        order_date: "2025-06-01".into(),
        notes: None,
        items: vec![
            CreatePurchaseItemRequest {
            item_id: item_a,
            quantity: 10.0,
                unit_price: Some(dec!(100.0)),
                total_price: Some(dec!(1000.0)),
                notes: None,
            },
            CreatePurchaseItemRequest {
            item_id: item_b,
            quantity: 20.0,
                unit_price: Some(dec!(100.0)),
                total_price: Some(dec!(1000.0)),
                notes: None,
            },
        ],
    };
    let order = PurchaseService::create_purchase_order(&pool, &dto)
        .await
        .unwrap();

    let (_order, items) = PurchaseService::get_purchase_order(&pool, order.id)
        .await
        .unwrap();
    assert_eq!(items.len(), 2);

    let item_id = items[0].id;
    PurchaseService::delete_purchase_item(&pool, order.id, item_id)
        .await
        .expect("delete_purchase_item must succeed");

    let (_order, remaining) = PurchaseService::get_purchase_order(&pool, order.id)
        .await
        .unwrap();
    assert_eq!(remaining.len(), 1);
    assert_ne!(remaining[0].id, item_id);
}

#[tokio::test]
async fn delete_purchase_item_fails_non_draft() {
    let pool = common::test_pool().await;

    let supplier_id = common::seed_supplier(&pool, "SUP-DLI2", "Supplier")
        .await
        .unwrap();
    let order = create_dummy_po(&pool, supplier_id).await;

    let trans = PurchaseOrderStatusTransitionRequest {
        status: "pending".into(),
    };
    PurchaseService::transition_purchase_status(&pool, order.id, &trans)
        .await
        .unwrap();

    let err = PurchaseService::delete_purchase_item(&pool, order.id, 0)
        .await
        .expect_err("delete item must fail for non-draft");
    assert!(err.to_string().contains("Cannot delete"));
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Purchase Order — approve / reject / link
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn approve_purchase_order_approves() {
    let pool = common::test_pool().await;

    let supplier_id = common::seed_supplier(&pool, "SUP-APR", "Approve Supplier")
        .await
        .unwrap();
    let order = create_dummy_po(&pool, supplier_id).await;

    // draft -> pending
    let trans = PurchaseOrderStatusTransitionRequest {
        status: "pending".into(),
    };
    PurchaseService::transition_purchase_status(&pool, order.id, &trans)
        .await
        .unwrap();

    // approve
    let req = PurchaseApproveReq { notes: None };
    PurchaseService::approve_purchase_order(&pool, order.id, &req)
        .await
        .expect("approve must succeed");

    let (fetched, _) = PurchaseService::get_purchase_order(&pool, order.id)
        .await
        .unwrap();
    assert_eq!(fetched.status, "approved");
}

#[tokio::test]
async fn approve_purchase_order_fails_non_pending() {
    let pool = common::test_pool().await;

    let supplier_id = common::seed_supplier(&pool, "SUP-APR2", "Supplier")
        .await
        .unwrap();
    let order = create_dummy_po(&pool, supplier_id).await;

    // Still in draft — approval must fail
    let req = PurchaseApproveReq { notes: None };
    let err = PurchaseService::approve_purchase_order(&pool, order.id, &req)
        .await
        .expect_err("approve from draft must fail");
    assert!(err.to_string().contains("Cannot approve"));
}

#[tokio::test]
async fn reject_purchase_order_rejects() {
    let pool = common::test_pool().await;

    let supplier_id = common::seed_supplier(&pool, "SUP-REJ", "Reject Supplier")
        .await
        .unwrap();
    let order = create_dummy_po(&pool, supplier_id).await;

    // draft -> pending
    let trans = PurchaseOrderStatusTransitionRequest {
        status: "pending".into(),
    };
    PurchaseService::transition_purchase_status(&pool, order.id, &trans)
        .await
        .unwrap();

    // reject
    let req = PurchaseRejectReq {
        reason: "price too high".into(),
    };
    PurchaseService::reject_purchase_order(&pool, order.id, &req)
        .await
        .expect("reject must succeed");

    let (fetched, _) = PurchaseService::get_purchase_order(&pool, order.id)
        .await
        .unwrap();
    assert_eq!(fetched.status, "rejected");
    // Reject stores reason in notes
    assert!(fetched
        .notes
        .as_deref()
        .unwrap_or("")
        .contains("price too high"));
}

#[tokio::test]
async fn reject_purchase_order_fails_non_pending() {
    let pool = common::test_pool().await;

    let supplier_id = common::seed_supplier(&pool, "SUP-REJ2", "Supplier")
        .await
        .unwrap();
    let order = create_dummy_po(&pool, supplier_id).await;

    let req = PurchaseRejectReq {
        reason: "bad".into(),
    };
    let err = PurchaseService::reject_purchase_order(&pool, order.id, &req)
        .await
        .expect_err("reject from draft must fail");
    assert!(err.to_string().contains("Cannot reject"));
}

#[tokio::test]
async fn link_inbound_to_order_links() {
    let pool = common::test_pool().await;

    let supplier_id = common::seed_supplier(&pool, "SUP-LNK", "Link Supplier")
        .await
        .unwrap();
    let order = create_dummy_po(&pool, supplier_id).await;
    let order_id = order.id;

    // draft -> pending -> approved
    let t1 = PurchaseOrderStatusTransitionRequest {
        status: "pending".into(),
    };
    PurchaseService::transition_purchase_status(&pool, order_id, &t1)
        .await
        .unwrap();
    let req = PurchaseApproveReq { notes: None };
    PurchaseService::approve_purchase_order(&pool, order_id, &req)
        .await
        .unwrap();

    // Create an inbound record to link
    let inbound_id: i64 = sqlx::query_scalar(
        "INSERT INTO inbound_records (inbound_no, inbound_type, notes, approval_status, \
         created_at, updated_at) \
         VALUES (?, ?, ?, ?, datetime('now'), datetime('now')) RETURNING id",
    )
    .bind("INB-LNK-001")
    .bind("purchase")
    .bind("linked to PO")
    .bind("approved")
    .fetch_one(&pool)
    .await
    .unwrap();

    PurchaseService::link_inbound_to_order(&pool, order_id, inbound_id)
        .await
        .expect("link_inbound must succeed");

    // Verify the inbound record has the order_id set
    let linked_order_id: (Option<i64>,) =
        sqlx::query_as("SELECT order_id FROM inbound_records WHERE id = ?")
            .bind(inbound_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(linked_order_id.0, Some(order_id));
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Full PO lifecycle
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn full_purchase_order_lifecycle() {
    let pool = common::test_pool().await;
    let item_a = seed_item(&pool, "ITM-FULL_PURCHASE_ORDER_-A").await;

    let supplier_id = common::seed_supplier(&pool, "SUP-LIFE", "Lifecycle Supplier")
        .await
        .unwrap();

    // 1. Create (draft)
    let dto = CreatePurchaseOrderRequest {
        order_no: Some("PO-LIFE-001".into()),
        supplier_id,
        order_date: "2025-06-01".into(),
        notes: Some("initial".into()),
        items: vec![CreatePurchaseItemRequest {
            item_id: item_a,
            quantity: 100.0,
            unit_price: Some(dec!(120.0)),
            total_price: Some(dec!(12000.0)),
            notes: None,
        }],
    };
    let order = PurchaseService::create_purchase_order(&pool, &dto)
        .await
        .unwrap();
    assert_eq!(order.status, "draft");

    // 2. Update header
    let update = UpdatePurchaseOrderRequest {
        order_date: None,
        notes: Some("updated notes".into()),
        items: None,
    };
    let updated = PurchaseService::update_purchase_order(&pool, order.id, &update)
        .await
        .unwrap();
    assert_eq!(updated.notes.as_deref(), Some("updated notes"));

    // 3. Submit (draft -> pending)
    let t1 = PurchaseOrderStatusTransitionRequest {
        status: "pending".into(),
    };
    PurchaseService::transition_purchase_status(&pool, order.id, &t1)
        .await
        .unwrap();

    // 4. Approve (pending -> approved)
    let approve_req = PurchaseApproveReq { notes: None };
    PurchaseService::approve_purchase_order(&pool, order.id, &approve_req)
        .await
        .unwrap();

    let (fetched, _) = PurchaseService::get_purchase_order(&pool, order.id)
        .await
        .unwrap();
    assert_eq!(fetched.status, "approved");

    // 5. Link inbound
    let inbound_id: i64 = sqlx::query_scalar(
        "INSERT INTO inbound_records (inbound_no, inbound_type, notes, approval_status, \
         created_at, updated_at) \
         VALUES (?, ?, ?, ?, datetime('now'), datetime('now')) RETURNING id",
    )
    .bind("INB-LIFE-001")
    .bind("purchase")
    .bind("full lifecycle link")
    .bind("approved")
    .fetch_one(&pool)
    .await
    .unwrap();

    PurchaseService::link_inbound_to_order(&pool, order.id, inbound_id)
        .await
        .expect("link_inbound must succeed");
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Sales Order — create
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn create_sales_order_with_items() {
    let pool = common::test_pool().await;
    let item_a = seed_item(&pool, "ITM-CREATE_SALES_ORDER_W-A").await;

    let customer_id = common::seed_customer(&pool, "CUS-001", "Test Customer")
        .await
        .unwrap();

    let dto = CreateSalesOrderRequest {
        order_no: None,
        customer_id,
        order_date: "2025-06-15".into(),
        notes: Some("initial SO".into()),
        items: vec![CreateSalesItemRequest {
            item_id: item_a,
            quantity: 50.0,
            unit_price: Some(dec!(200.0)),
            total_price: Some(dec!(10000.0)),
            notes: None,
        }],
    };

    let order = SalesService::create_sales_order(&pool, &dto)
        .await
        .expect("create_sales_order must succeed");

    assert!(order.id > 0);
    assert!(order.order_no.starts_with("SO-"));
    assert_eq!(order.status, "draft");
    assert_eq!(order.customer_id, customer_id);
}

#[tokio::test]
async fn create_sales_order_fails_empty_items() {
    let pool = common::test_pool().await;

    let customer_id = common::seed_customer(&pool, "CUS-002", "Empty Items Customer")
        .await
        .unwrap();

    let dto = CreateSalesOrderRequest {
        order_no: None,
        customer_id,
        order_date: "2025-06-15".into(),
        notes: None,
        items: vec![],
    };

    let err = SalesService::create_sales_order(&pool, &dto)
        .await
        .expect_err("must fail with empty items");
    assert!(err.to_string().contains("At least one item"));
}

#[tokio::test]
async fn create_sales_order_fails_inactive_customer() {
    let pool = common::test_pool().await;
    let item_a = seed_item(&pool, "ITM-CREATE_SALES_ORDER_F-A").await;

    // Manually insert inactive customer
    let customer_id: i64 = sqlx::query_scalar(
        "INSERT INTO customers (customer_code, name, contact_person, phone, email, address, \
         is_active, notes, created_at, updated_at) \
         VALUES (?, ?, 'Contact', '13800138001', ?, 'Addr', 0, 'inactive', \
         datetime('now'), datetime('now')) RETURNING id",
    )
    .bind("CUS-003")
    .bind("Inactive Customer")
    .bind("cus003@test.local")
    .fetch_one(&pool)
    .await
    .unwrap();

    let dto = CreateSalesOrderRequest {
        order_no: None,
        customer_id,
        order_date: "2025-06-15".into(),
        notes: None,
        items: vec![CreateSalesItemRequest {
            item_id: item_a,
            quantity: 10.0,
            unit_price: Some(dec!(100.0)),
            total_price: Some(dec!(1000.0)),
            notes: None,
        }],
    };

    let err = SalesService::create_sales_order(&pool, &dto)
        .await
        .expect_err("must fail for inactive customer");
    assert!(err.to_string().contains("not active"));
}

#[tokio::test]
async fn create_sales_order_fails_nonexistent_customer() {
    let pool = common::test_pool().await;
    let item_a = seed_item(&pool, "ITM-CREATE_SALES_ORDER_F-A").await;

    let dto = CreateSalesOrderRequest {
        order_no: None,
        customer_id: 99999,
        order_date: "2025-06-15".into(),
        notes: None,
        items: vec![CreateSalesItemRequest {
            item_id: item_a,
            quantity: 10.0,
            unit_price: Some(dec!(100.0)),
            total_price: Some(dec!(1000.0)),
            notes: None,
        }],
    };

    let err = SalesService::create_sales_order(&pool, &dto)
        .await
        .expect_err("must fail for nonexistent customer");
    assert!(err.to_string().contains("Customer"));
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Sales Order — update / status / get / list / delete
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn update_sales_order_updates_header() {
    let pool = common::test_pool().await;

    let customer_id = common::seed_customer(&pool, "CUS-UPD", "Update Customer")
        .await
        .unwrap();
    let order = create_dummy_so(&pool, customer_id).await;

    let update = UpdateSalesOrderRequest {
        order_date: Some("2025-07-01".into()),
        notes: Some("updated SO notes".into()),
    };

    let updated = SalesService::update_sales_order(&pool, order.id, &update)
        .await
        .expect("update_sales_order must succeed");
    assert_eq!(updated.notes.as_deref(), Some("updated SO notes"));
}

#[tokio::test]
async fn update_sales_order_fails_non_draft() {
    let pool = common::test_pool().await;

    let customer_id = common::seed_customer(&pool, "CUS-UPD2", "Customer")
        .await
        .unwrap();
    let order = create_dummy_so(&pool, customer_id).await;

    // Transition to pending
    let trans = SalesOrderStatusTransitionRequest {
        status: "pending".into(),
    };
    SalesService::transition_sales_status(&pool, order.id, &trans)
        .await
        .unwrap();

    let update = UpdateSalesOrderRequest {
        order_date: None,
        notes: Some("should fail".into()),
    };
    let err = SalesService::update_sales_order(&pool, order.id, &update)
        .await
        .expect_err("update must fail for non-draft");
    assert!(err.to_string().contains("Cannot modify"));
}

#[tokio::test]
async fn transition_sales_status_draft_to_pending() {
    let pool = common::test_pool().await;

    let customer_id = common::seed_customer(&pool, "CUS-TRN", "Trans Customer")
        .await
        .unwrap();
    let order = create_dummy_so(&pool, customer_id).await;

    let trans = SalesOrderStatusTransitionRequest {
        status: "pending".into(),
    };
    SalesService::transition_sales_status(&pool, order.id, &trans)
        .await
        .expect("draft -> pending must succeed");

    let (fetched, _) = SalesService::get_sales_order(&pool, order.id)
        .await
        .unwrap();
    assert_eq!(fetched.status, "pending");
}

#[tokio::test]
async fn transition_sales_status_invalid_hop_fails() {
    let pool = common::test_pool().await;

    let customer_id = common::seed_customer(&pool, "CUS-TRN2", "Customer")
        .await
        .unwrap();
    let order = create_dummy_so(&pool, customer_id).await;

    let trans = SalesOrderStatusTransitionRequest {
        status: "approved".into(),
    };
    let err = SalesService::transition_sales_status(&pool, order.id, &trans)
        .await
        .expect_err("draft -> approved must fail");
    assert!(err.to_string().contains("Cannot transition"));
}

#[tokio::test]
async fn get_sales_order_with_items() {
    let pool = common::test_pool().await;
    let item_a = seed_item(&pool, "ITM-GET_SALES_ORDER_WITH-A").await;
    let item_b = seed_item(&pool, "ITM-GET_SALES_ORDER_WITH-B").await;

    let customer_id = common::seed_customer(&pool, "CUS-GET", "Get Customer")
        .await
        .unwrap();
    let dto = CreateSalesOrderRequest {
        order_no: Some("SO-GET-001".into()),
        customer_id,
        order_date: "2025-06-15".into(),
        notes: None,
        items: vec![
            CreateSalesItemRequest {
            item_id: item_a,
            quantity: 20.0,
                unit_price: Some(dec!(200.0)),
                total_price: Some(dec!(4000.0)),
                notes: None,
            },
            CreateSalesItemRequest {
            item_id: item_b,
            quantity: 30.0,
                unit_price: Some(dec!(150.0)),
                total_price: Some(dec!(4500.0)),
                notes: None,
            },
        ],
    };

    let order = SalesService::create_sales_order(&pool, &dto)
        .await
        .unwrap();

    let (fetched, items) = SalesService::get_sales_order(&pool, order.id)
        .await
        .expect("get_sales_order must succeed");
    assert_eq!(fetched.id, order.id);
    assert_eq!(items.len(), 2);
}

#[tokio::test]
async fn list_sales_orders_pagination() {
    let pool = common::test_pool().await;
    let item_a = seed_item(&pool, "ITM-LIST_SALES_ORDERS_PA-A").await;

    let customer_id = common::seed_customer(&pool, "CUS-LST", "List Customer")
        .await
        .unwrap();

    for i in 1..=3 {
        let dto = CreateSalesOrderRequest {
            order_no: Some(format!("SO-LST-{:03}", i)),
            customer_id,
            order_date: "2025-06-15".into(),
            notes: None,
            items: vec![CreateSalesItemRequest {
            item_id: item_a,
            quantity: 10.0,
                unit_price: Some(dec!(100.0)),
                total_price: Some(dec!(1000.0)),
                notes: None,
            }],
        };
        SalesService::create_sales_order(&pool, &dto)
            .await
            .unwrap();
    }

    let filter = SalesOrderFilterParams {
        q: None,
        status: None,
        customer_id: None,
        order_date_from: None,
        order_date_to: None,
        page: None,
        page_size: None,
        sort_by: None,
        sort_order: None,
    };
    let params = PaginationParams {
        page: Some(1),
        page_size: Some(2),
        sort_by: None,
        sort_order: None,
    };

    let (orders, total) = SalesService::list_sales_orders(&pool, &filter, &params)
        .await
        .expect("list_sales_orders must succeed");
    assert_eq!(orders.len(), 2);
    assert_eq!(total, 3);
}

#[tokio::test]
async fn delete_sales_order_draft() {
    let pool = common::test_pool().await;

    let customer_id = common::seed_customer(&pool, "CUS-DEL", "Del Customer")
        .await
        .unwrap();
    let order = create_dummy_so(&pool, customer_id).await;

    SalesService::delete_sales_order(&pool, order.id)
        .await
        .expect("delete draft SO must succeed");

    let deleted_at: (Option<DateTime<Utc>>,) =
        sqlx::query_as("SELECT deleted_at FROM sales_orders WHERE id = ?")
            .bind(order.id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(deleted_at.0.is_some());
}

#[tokio::test]
async fn delete_sales_order_fails_approved() {
    let pool = common::test_pool().await;

    let customer_id = common::seed_customer(&pool, "CUS-DEL2", "Customer")
        .await
        .unwrap();

    let order_id = common::seed_sales_order(&pool, "SO-DEL-002", customer_id, "approved")
        .await
        .unwrap();

    let err = SalesService::delete_sales_order(&pool, order_id)
        .await
        .expect_err("deleting approved SO must fail");
    assert!(err.to_string().contains("Cannot delete"));
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Sales Order — update / delete item
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn update_sales_item_changes_qty() {
    let pool = common::test_pool().await;
    let item_a = seed_item(&pool, "ITM-UPDATE_SALES_ITEM_CH-A").await;

    let customer_id = common::seed_customer(&pool, "CUS-SITM", "Item Customer")
        .await
        .unwrap();
    let dto = CreateSalesOrderRequest {
        order_no: Some("SO-SITM-001".into()),
        customer_id,
        order_date: "2025-06-15".into(),
        notes: None,
        items: vec![CreateSalesItemRequest {
            item_id: item_a,
            quantity: 30.0,
            unit_price: Some(dec!(200.0)),
            total_price: Some(dec!(6000.0)),
            notes: None,
        }],
    };
    let order = SalesService::create_sales_order(&pool, &dto)
        .await
        .unwrap();

    let (_order, items) = SalesService::get_sales_order(&pool, order.id)
        .await
        .unwrap();
    let item_id = items[0].id;

    let update = UpdateSalesItemRequest {
        item_id: None,
        quantity: Some(45.0),
        unit_price: Some(dec!(180.0)),
        notes: None,
    };

    let (_order, updated) =
        SalesService::update_sales_item(&pool, order.id, item_id, &update)
            .await
            .expect("update_sales_item must succeed");
    assert_eq!(updated.quantity, 45.0);
    assert_eq!(updated.unit_price, Some(180.0));
}

#[tokio::test]
async fn delete_sales_item_removes_item() {
    let pool = common::test_pool().await;
    let item_a = seed_item(&pool, "ITM-DELETE_SALES_ITEM_RE-A").await;
    let item_b = seed_item(&pool, "ITM-DELETE_SALES_ITEM_RE-B").await;

    let customer_id = common::seed_customer(&pool, "CUS-DSITM", "DelItem Customer")
        .await
        .unwrap();
    let dto = CreateSalesOrderRequest {
        order_no: Some("SO-DSITM-001".into()),
        customer_id,
        order_date: "2025-06-15".into(),
        notes: None,
        items: vec![
            CreateSalesItemRequest {
            item_id: item_a,
            quantity: 10.0,
                unit_price: Some(dec!(100.0)),
                total_price: Some(dec!(1000.0)),
                notes: None,
            },
            CreateSalesItemRequest {
            item_id: item_b,
            quantity: 20.0,
                unit_price: Some(dec!(100.0)),
                total_price: Some(dec!(1000.0)),
                notes: None,
            },
        ],
    };
    let order = SalesService::create_sales_order(&pool, &dto)
        .await
        .unwrap();

    let (_order, items) = SalesService::get_sales_order(&pool, order.id)
        .await
        .unwrap();
    assert_eq!(items.len(), 2);

    SalesService::delete_sales_item(&pool, order.id, items[0].id)
        .await
        .expect("delete_sales_item must succeed");

    let (_order, remaining) = SalesService::get_sales_order(&pool, order.id)
        .await
        .unwrap();
    assert_eq!(remaining.len(), 1);
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Sales Order — approve / reject / link / ATP
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn approve_sales_order_approves() {
    let pool = common::test_pool().await;
    let item_a = seed_item(&pool, "ITM-APPROVE_SALES_ORDER_-A").await;

    let customer_id = common::seed_customer(&pool, "CUS-SAPR", "SO Approve Customer")
        .await
        .unwrap();

    // Seed inbound stock to satisfy ATP
    seed_inventory_log(&pool, item_a, "inbound", 2.0).await;

    let dto = CreateSalesOrderRequest {
        order_no: Some("SO-SAPR-001".into()),
        customer_id,
        order_date: "2025-06-15".into(),
        notes: None,
        items: vec![CreateSalesItemRequest {
            item_id: item_a,
            quantity: 2.0,
            unit_price: Some(dec!(200.0)),
            total_price: Some(dec!(400.0)),
            notes: None,
        }],
    };
    let order = SalesService::create_sales_order(&pool, &dto)
        .await
        .unwrap();

    // draft -> pending
    let trans = SalesOrderStatusTransitionRequest {
        status: "pending".into(),
    };
    SalesService::transition_sales_status(&pool, order.id, &trans)
        .await
        .unwrap();

    // approve
    let req = SalesApproveReq { notes: None };
    SalesService::approve_sales_order(&pool, order.id, &req)
        .await
        .expect("approve_sales_order must succeed");

    let (fetched, _) = SalesService::get_sales_order(&pool, order.id)
        .await
        .unwrap();
    assert_eq!(fetched.status, "approved");
}

#[tokio::test]
async fn approve_sales_order_fails_insufficient_stock() {
    let pool = common::test_pool().await;
    let item_a = seed_item(&pool, "ITM-APPROVE_SALES_ORDER_-A").await;

    let customer_id = common::seed_customer(&pool, "CUS-NOATP", "No ATP Customer")
        .await
        .unwrap();

    // No in_stock pipes exist, so ATP should fail
    let dto = CreateSalesOrderRequest {
        order_no: Some("SO-NOATP-001".into()),
        customer_id,
        order_date: "2025-06-15".into(),
        notes: None,
        items: vec![CreateSalesItemRequest {
            item_id: item_a,
            quantity: 1.0,
            unit_price: Some(dec!(100.0)),
            total_price: Some(dec!(1000.0)),
            notes: None,
        }],
    };
    let order = SalesService::create_sales_order(&pool, &dto)
        .await
        .unwrap();

    let trans = SalesOrderStatusTransitionRequest {
        status: "pending".into(),
    };
    SalesService::transition_sales_status(&pool, order.id, &trans)
        .await
        .unwrap();

    let req = SalesApproveReq { notes: None };
    let err = SalesService::approve_sales_order(&pool, order.id, &req)
        .await
        .expect_err("approve must fail with insufficient stock");
    assert!(err.to_string().to_lowercase().contains("insufficient"));
}

#[tokio::test]
async fn approve_sales_order_fails_non_pending() {
    let pool = common::test_pool().await;

    let customer_id = common::seed_customer(&pool, "CUS-SAPR2", "Customer")
        .await
        .unwrap();
    let order = create_dummy_so(&pool, customer_id).await;

    let req = SalesApproveReq { notes: None };
    let err = SalesService::approve_sales_order(&pool, order.id, &req)
        .await
        .expect_err("approve from draft must fail");
    assert!(err.to_string().contains("Cannot approve"));
}

#[tokio::test]
async fn reject_sales_order_rejects() {
    let pool = common::test_pool().await;

    let customer_id = common::seed_customer(&pool, "CUS-SREJ", "SO Reject Customer")
        .await
        .unwrap();
    let order = create_dummy_so(&pool, customer_id).await;

    // draft -> pending
    let trans = SalesOrderStatusTransitionRequest {
        status: "pending".into(),
    };
    SalesService::transition_sales_status(&pool, order.id, &trans)
        .await
        .unwrap();

    let req = SalesRejectReq {
        reason: "customer changed mind".into(),
    };
    SalesService::reject_sales_order(&pool, order.id, &req)
        .await
        .expect("reject_sales_order must succeed");

    let (fetched, _) = SalesService::get_sales_order(&pool, order.id)
        .await
        .unwrap();
    assert_eq!(fetched.status, "rejected");
    assert!(fetched
        .notes
        .as_deref()
        .unwrap_or("")
        .contains("changed mind"));
}

#[tokio::test]
async fn link_outbound_to_order_links() {
    let pool = common::test_pool().await;
    let item_a = seed_item(&pool, "ITM-LINK_OUTBOUND_TO_ORD-A").await;

    let customer_id = common::seed_customer(&pool, "CUS-SLNK", "SO Link Customer")
        .await
        .unwrap();

    // Seed inbound stock
    seed_inventory_log(&pool, item_a, "inbound", 1.0).await;

    let dto = CreateSalesOrderRequest {
        order_no: Some("SO-SLNK-001".into()),
        customer_id,
        order_date: "2025-06-15".into(),
        notes: None,
        items: vec![CreateSalesItemRequest {
            item_id: item_a,
            quantity: 1.0,
            unit_price: Some(dec!(200.0)),
            total_price: Some(dec!(200.0)),
            notes: None,
        }],
    };
    let order = SalesService::create_sales_order(&pool, &dto)
        .await
        .unwrap();

    // draft -> pending -> approve
    let trans = SalesOrderStatusTransitionRequest {
        status: "pending".into(),
    };
    SalesService::transition_sales_status(&pool, order.id, &trans)
        .await
        .unwrap();
    let req = SalesApproveReq { notes: None };
    SalesService::approve_sales_order(&pool, order.id, &req)
        .await
        .unwrap();

    // Create outbound record to link
    let outbound_id: i64 = sqlx::query_scalar(
        "INSERT INTO outbound_records (outbound_no, outbound_type, notes, approval_status, \
         created_at, updated_at) \
         VALUES (?, ?, ?, ?, datetime('now'), datetime('now')) RETURNING id",
    )
    .bind("OUT-SLNK-001")
    .bind("sales")
    .bind("linked to SO")
    .bind("approved")
    .fetch_one(&pool)
    .await
    .unwrap();

    SalesService::link_outbound_to_order(&pool, order.id, outbound_id)
        .await
        .expect("link_outbound must succeed");

    let linked_order_id: (Option<i64>,) =
        sqlx::query_as("SELECT order_id FROM outbound_records WHERE id = ?")
            .bind(outbound_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(linked_order_id.0, Some(order.id));
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// ATP validation in sales order creation
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn sales_order_atp_validation_passes_with_stock() {
    let pool = common::test_pool().await;
    let item_a = seed_item(&pool, "ITM-SALES_ORDER_ATP_VALI-A").await;

    let customer_id = common::seed_customer(&pool, "CUS-ATP1", "ATP Customer 1")
        .await
        .unwrap();

    // Seed inbound stock: 3 units available
    seed_inventory_log(&pool, item_a, "inbound", 3.0).await;

    let dto = CreateSalesOrderRequest {
        order_no: Some("SO-ATP-001".into()),
        customer_id,
        order_date: "2025-06-15".into(),
        notes: None,
        items: vec![CreateSalesItemRequest {
            item_id: item_a,
            quantity: 3.0,
            unit_price: Some(dec!(200.0)),
            total_price: Some(dec!(600.0)),
            notes: None,
        }],
    };
    let order = SalesService::create_sales_order(&pool, &dto)
        .await
        .unwrap();

    // draft -> pending -> approve (should pass ATP with 3 in_stock pipes)
    let trans = SalesOrderStatusTransitionRequest {
        status: "pending".into(),
    };
    SalesService::transition_sales_status(&pool, order.id, &trans)
        .await
        .unwrap();

    let req = SalesApproveReq { notes: None };
    SalesService::approve_sales_order(&pool, order.id, &req)
        .await
        .expect("ATP check must pass with sufficient stock");

    let (fetched, _) = SalesService::get_sales_order(&pool, order.id)
        .await
        .unwrap();
    assert_eq!(fetched.status, "approved");
}

#[tokio::test]
async fn sales_order_atp_validation_fails_without_stock() {
    let pool = common::test_pool().await;
    let item_a = seed_item(&pool, "ITM-SALES_ORDER_ATP_VALI-A").await;

    let customer_id = common::seed_customer(&pool, "CUS-ATP2", "ATP Customer 2")
        .await
        .unwrap();

    // Seed only 1 unit inbound but order requests 10 — insufficient
    seed_inventory_log(&pool, item_a, "inbound", 1.0).await;

    let dto = CreateSalesOrderRequest {
        order_no: Some("SO-ATP-002".into()),
        customer_id,
        order_date: "2025-06-15".into(),
        notes: None,
        items: vec![CreateSalesItemRequest {
            item_id: item_a,
            quantity: 10.0,
            unit_price: Some(dec!(100.0)),
            total_price: Some(dec!(1000.0)),
            notes: None,
        }],
    };
    let order = SalesService::create_sales_order(&pool, &dto)
        .await
        .unwrap();

    let trans = SalesOrderStatusTransitionRequest {
        status: "pending".into(),
    };
    SalesService::transition_sales_status(&pool, order.id, &trans)
        .await
        .unwrap();

    let req = SalesApproveReq { notes: None };
    let err = SalesService::approve_sales_order(&pool, order.id, &req)
        .await
        .expect_err("ATP check must fail with insufficient stock");
    assert!(err.to_string().to_lowercase().contains("insufficient"));
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Helpers
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Create a minimal purchase order with a single item (draft status).
async fn create_dummy_po(
    pool: &sqlx::SqlitePool,
    supplier_id: i64,
) -> erp_server::models::purchase_order::PurchaseOrder {
    let item_a = seed_item(&pool, "ITM-CREATE_DUMMY_PO-A").await;
    let dto = CreatePurchaseOrderRequest {
        order_no: None,
        supplier_id,
        order_date: "2025-06-01".into(),
        notes: None,
        items: vec![CreatePurchaseItemRequest {
            item_id: item_a,
            quantity: 10.0,
            unit_price: Some(dec!(100.0)),
            total_price: Some(dec!(1000.0)),
            notes: None,
        }],
    };
    PurchaseService::create_purchase_order(pool, &dto)
        .await
        .expect("create_dummy_po must succeed")
}

/// Create a minimal sales order with a single item (draft status).
async fn create_dummy_so(
    pool: &sqlx::SqlitePool,
    customer_id: i64,
) -> erp_server::models::sales_order::SalesOrder {
    let item_a = seed_item(&pool, "ITM-CREATE_DUMMY_SO-A").await;
    let dto = CreateSalesOrderRequest {
        order_no: None,
        customer_id,
        order_date: "2025-06-15".into(),
        notes: None,
        items: vec![CreateSalesItemRequest {
            item_id: item_a,
            quantity: 10.0,
            unit_price: Some(dec!(100.0)),
            total_price: Some(dec!(1000.0)),
            notes: None,
        }],
    };
    SalesService::create_sales_order(pool, &dto)
        .await
        .expect("create_dummy_so must succeed")
}
