//! Integration tests for `PurchaseSalesService` — purchase and sales order lifecycle.
//!
//! Covers PO/SO creation, status transitions, approval/rejection, item management,
//! soft-delete, linking to inbound/outbound, and ATP validation for sales orders.
//!
//! All tests use an in-memory SQLite database with fresh migrations.

mod common;

use steel_pipe_db::dto::common::PaginationParams;
use steel_pipe_db::dto::purchase_dto::{
    ApproveOrderRequest as PurchaseApproveReq, CreatePurchaseItemRequest,
    CreatePurchaseOrderRequest, PurchaseOrderFilterParams, PurchaseOrderStatusTransitionRequest,
    RejectOrderRequest as PurchaseRejectReq, UpdatePurchaseItemRequest, UpdatePurchaseOrderRequest,
};
use steel_pipe_db::dto::sales_dto::{
    ApproveOrderRequest as SalesApproveReq, CreateSalesItemRequest, CreateSalesOrderRequest,
    RejectOrderRequest as SalesRejectReq, SalesOrderFilterParams, SalesOrderStatusTransitionRequest,
    UpdateSalesItemRequest, UpdateSalesOrderRequest,
};
use steel_pipe_db::services::purchase_sales_service::PurchaseSalesService;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Purchase Order — create
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn create_purchase_order_with_items() {
    let pool = common::test_pool().await;

    let supplier_id = common::seed_supplier(&pool, "SUP-001", "Test Supplier")
        .await
        .unwrap();

    let dto = CreatePurchaseOrderRequest {
        order_no: None,
        supplier_id,
        order_date: "2025-06-01".into(),
        notes: Some("initial PO".into()),
        items: vec![CreatePurchaseItemRequest {
            pipe_type: "casing".into(),
            grade: "J55".into(),
            od: 177.8,
            wt: 9.19,
            quantity: 100,
            unit_price: Some(150.0),
            total_price: Some(15000.0),
            notes: None,
        }],
    };

    let order = PurchaseSalesService::create_purchase_order(&pool, &dto)
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

    let err = PurchaseSalesService::create_purchase_order(&pool, &dto)
        .await
        .expect_err("must fail with empty items");
    assert!(err.to_string().contains("At least one item"));
}

#[tokio::test]
async fn create_purchase_order_fails_inactive_supplier() {
    let pool = common::test_pool().await;

    // Manually insert an inactive supplier
    let result = sqlx::query(
        "INSERT INTO suppliers (supplier_code, name, contact_person, phone, email, address, \
         is_active, notes, created_at, updated_at) \
         VALUES ($1, $2, 'Contact', '13800138000', $3, 'Addr', 0, 'inactive', \
         datetime('now'), datetime('now'))",
    )
    .bind("SUP-003")
    .bind("Inactive Supplier")
    .bind("sup003@test.local")
    .execute(&pool)
    .await
    .unwrap();
    let supplier_id = result.last_insert_rowid();

    let dto = CreatePurchaseOrderRequest {
        order_no: None,
        supplier_id,
        order_date: "2025-06-01".into(),
        notes: None,
        items: vec![CreatePurchaseItemRequest {
            pipe_type: "casing".into(),
            grade: "J55".into(),
            od: 177.8,
            wt: 9.19,
            quantity: 10,
            unit_price: Some(100.0),
            total_price: Some(1000.0),
            notes: None,
        }],
    };

    let err = PurchaseSalesService::create_purchase_order(&pool, &dto)
        .await
        .expect_err("must fail for inactive supplier");
    assert!(err.to_string().contains("not active"));
}

#[tokio::test]
async fn create_purchase_order_fails_duplicate_order_no() {
    let pool = common::test_pool().await;

    let supplier_id = common::seed_supplier(&pool, "SUP-004", "Supplier Dup")
        .await
        .unwrap();

    let dto = CreatePurchaseOrderRequest {
        order_no: Some("PO-DUP-001".into()),
        supplier_id,
        order_date: "2025-06-01".into(),
        notes: None,
        items: vec![CreatePurchaseItemRequest {
            pipe_type: "casing".into(),
            grade: "J55".into(),
            od: 177.8,
            wt: 9.19,
            quantity: 10,
            unit_price: Some(100.0),
            total_price: Some(1000.0),
            notes: None,
        }],
    };

    // First creation should succeed
    PurchaseSalesService::create_purchase_order(&pool, &dto)
        .await
        .expect("first create must succeed");

    // Second creation with same order_no should fail
    let err = PurchaseSalesService::create_purchase_order(&pool, &dto)
        .await
        .expect_err("duplicate order_no must fail");
    assert!(err.to_string().contains("already exists"));
}

#[tokio::test]
async fn create_purchase_order_fails_nonexistent_supplier() {
    let pool = common::test_pool().await;

    let dto = CreatePurchaseOrderRequest {
        order_no: None,
        supplier_id: 99999,
        order_date: "2025-06-01".into(),
        notes: None,
        items: vec![CreatePurchaseItemRequest {
            pipe_type: "casing".into(),
            grade: "J55".into(),
            od: 177.8,
            wt: 9.19,
            quantity: 10,
            unit_price: Some(100.0),
            total_price: Some(1000.0),
            notes: None,
        }],
    };

    let err = PurchaseSalesService::create_purchase_order(&pool, &dto)
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

    let supplier_id = common::seed_supplier(&pool, "SUP-UPD", "Update Supplier")
        .await
        .unwrap();

    let dto = CreatePurchaseOrderRequest {
        order_no: Some("PO-UPD-001".into()),
        supplier_id,
        order_date: "2025-06-01".into(),
        notes: Some("original".into()),
        items: vec![CreatePurchaseItemRequest {
            pipe_type: "casing".into(),
            grade: "J55".into(),
            od: 177.8,
            wt: 9.19,
            quantity: 50,
            unit_price: Some(100.0),
            total_price: Some(5000.0),
            notes: None,
        }],
    };

    let order = PurchaseSalesService::create_purchase_order(&pool, &dto)
        .await
        .unwrap();

    let update = UpdatePurchaseOrderRequest {
        order_date: Some("2025-07-01".into()),
        notes: Some("updated notes".into()),
    };

    let updated = PurchaseSalesService::update_purchase_order(&pool, order.id, &update)
        .await
        .expect("update_purchase_order must succeed");

    assert_eq!(updated.notes.as_deref(), Some("updated notes"));
}

#[tokio::test]
async fn update_purchase_order_fails_non_draft() {
    let pool = common::test_pool().await;

    let supplier_id = common::seed_supplier(&pool, "SUP-UPD2", "Supplier")
        .await
        .unwrap();

    let dto = CreatePurchaseOrderRequest {
        order_no: Some("PO-UPD-002".into()),
        supplier_id,
        order_date: "2025-06-01".into(),
        notes: None,
        items: vec![CreatePurchaseItemRequest {
            pipe_type: "casing".into(),
            grade: "J55".into(),
            od: 177.8,
            wt: 9.19,
            quantity: 10,
            unit_price: Some(100.0),
            total_price: Some(1000.0),
            notes: None,
        }],
    };

    let order = PurchaseSalesService::create_purchase_order(&pool, &dto)
        .await
        .unwrap();

    // Transition to pending
    let trans = PurchaseOrderStatusTransitionRequest {
        status: "pending".into(),
    };
    PurchaseSalesService::transition_purchase_status(&pool, order.id, &trans)
        .await
        .unwrap();

    // Now try to update — must fail
    let update = UpdatePurchaseOrderRequest {
        order_date: None,
        notes: Some("should fail".into()),
    };
    let err = PurchaseSalesService::update_purchase_order(&pool, order.id, &update)
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
    PurchaseSalesService::transition_purchase_status(&pool, order.id, &trans)
        .await
        .expect("draft -> pending must succeed");

    let (fetched, _items) = PurchaseSalesService::get_purchase_order(&pool, order.id)
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
    PurchaseSalesService::transition_purchase_status(&pool, order.id, &trans)
        .await
        .expect("draft -> cancelled must succeed");

    let (fetched, _) = PurchaseSalesService::get_purchase_order(&pool, order.id)
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
    let err = PurchaseSalesService::transition_purchase_status(&pool, order.id, &trans)
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
    PurchaseSalesService::transition_purchase_status(&pool, order.id, &t1)
        .await
        .unwrap();

    // pending -> approved
    let t2 = PurchaseOrderStatusTransitionRequest {
        status: "approved".into(),
    };
    PurchaseSalesService::transition_purchase_status(&pool, order.id, &t2)
        .await
        .unwrap();

    let (fetched, _) = PurchaseSalesService::get_purchase_order(&pool, order.id)
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
                pipe_type: "casing".into(),
                grade: "J55".into(),
                od: 177.8,
                wt: 9.19,
                quantity: 20,
                unit_price: Some(100.0),
                total_price: Some(2000.0),
                notes: None,
            },
            CreatePurchaseItemRequest {
                pipe_type: "tubing".into(),
                grade: "N80Q".into(),
                od: 88.9,
                wt: 6.45,
                quantity: 30,
                unit_price: Some(80.0),
                total_price: Some(2400.0),
                notes: None,
            },
        ],
    };

    let order = PurchaseSalesService::create_purchase_order(&pool, &dto)
        .await
        .unwrap();

    let (fetched, items) = PurchaseSalesService::get_purchase_order(&pool, order.id)
        .await
        .expect("get_purchase_order must succeed");

    assert_eq!(fetched.id, order.id);
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].order_id, order.id);
}

#[tokio::test]
async fn get_purchase_order_fails_not_found() {
    let pool = common::test_pool().await;

    let err = PurchaseSalesService::get_purchase_order(&pool, 99999)
        .await
        .expect_err("must fail for nonexistent order");
    assert!(err.to_string().contains("not found"));
}

#[tokio::test]
async fn list_purchase_orders_pagination() {
    let pool = common::test_pool().await;

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
                pipe_type: "casing".into(),
                grade: "J55".into(),
                od: 177.8,
                wt: 9.19,
                quantity: 10,
                unit_price: Some(100.0),
            total_price: Some(1000.0),
                notes: None,
            }],
        };
        PurchaseSalesService::create_purchase_order(&pool, &dto)
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

    let (orders, total) = PurchaseSalesService::list_purchase_orders(&pool, &filter, &params)
        .await
        .expect("list must succeed");

    assert_eq!(orders.len(), 2);
    assert_eq!(total, 3);
}

#[tokio::test]
async fn list_purchase_orders_status_filter() {
    let pool = common::test_pool().await;

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
                pipe_type: "casing".into(),
                grade: "J55".into(),
                od: 177.8,
                wt: 9.19,
                quantity: 10,
                unit_price: Some(100.0),
            total_price: Some(1000.0),
                notes: None,
            }],
        };
        PurchaseSalesService::create_purchase_order(&pool, &dto)
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
                pipe_type: "casing".into(),
                grade: "J55".into(),
                od: 177.8,
                wt: 9.19,
                quantity: 10,
                unit_price: Some(100.0),
            total_price: Some(1000.0),
                notes: None,
            }],
        };
        let o = PurchaseSalesService::create_purchase_order(&pool, &dto)
            .await
            .unwrap();
        let trans = PurchaseOrderStatusTransitionRequest {
            status: "pending".into(),
        };
        PurchaseSalesService::transition_purchase_status(&pool, o.id, &trans)
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

    let (orders, total) = PurchaseSalesService::list_purchase_orders(&pool, &filter, &params)
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

    PurchaseSalesService::delete_purchase_order(&pool, order.id)
        .await
        .expect("delete draft PO must succeed");

    // Verify soft-deleted
    let deleted_at: (Option<String>,) =
        sqlx::query_as("SELECT deleted_at FROM purchase_orders WHERE id = ?")
            .bind(order.id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(deleted_at.0.is_some());

    // get should fail
    let err = PurchaseSalesService::get_purchase_order(&pool, order.id)
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

    let err = PurchaseSalesService::delete_purchase_order(&pool, order_id)
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
            pipe_type: "casing".into(),
            grade: "J55".into(),
            od: 177.8,
            wt: 9.19,
            quantity: 50,
            unit_price: Some(100.0),
            total_price: Some(5000.0),
            notes: None,
        }],
    };
    let order = PurchaseSalesService::create_purchase_order(&pool, &dto)
        .await
        .unwrap();

    let (_order, items) = PurchaseSalesService::get_purchase_order(&pool, order.id)
        .await
        .unwrap();
    let item_id = items[0].id;

    let update = UpdatePurchaseItemRequest {
        pipe_type: None,
        grade: None,
        od: None,
        wt: None,
        quantity: Some(75),
        unit_price: Some(90.0),
        notes: None,
    };

    let (_order, updated_item) =
        PurchaseSalesService::update_purchase_item(&pool, order.id, item_id, &update)
            .await
            .expect("update_purchase_item must succeed");

    assert_eq!(updated_item.quantity, 75);
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
    PurchaseSalesService::transition_purchase_status(&pool, order.id, &trans)
        .await
        .unwrap();

    let update = UpdatePurchaseItemRequest {
        pipe_type: None,
        grade: None,
        od: None,
        wt: None,
        quantity: Some(99),
        unit_price: None,
        notes: None,
    };

    let err = PurchaseSalesService::update_purchase_item(&pool, order.id, 0, &update)
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
                pipe_type: "casing".into(),
                grade: "J55".into(),
                od: 177.8,
                wt: 9.19,
                quantity: 10,
                unit_price: Some(100.0),
            total_price: Some(1000.0),
                notes: None,
            },
            CreatePurchaseItemRequest {
                pipe_type: "tubing".into(),
                grade: "N80Q".into(),
                od: 88.9,
                wt: 6.45,
                quantity: 20,
                unit_price: Some(100.0),
            total_price: Some(1000.0),
                notes: None,
            },
        ],
    };
    let order = PurchaseSalesService::create_purchase_order(&pool, &dto)
        .await
        .unwrap();

    let (_order, items) = PurchaseSalesService::get_purchase_order(&pool, order.id)
        .await
        .unwrap();
    assert_eq!(items.len(), 2);

    let item_id = items[0].id;
    PurchaseSalesService::delete_purchase_item(&pool, order.id, item_id)
        .await
        .expect("delete_purchase_item must succeed");

    let (_order, remaining) = PurchaseSalesService::get_purchase_order(&pool, order.id)
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
    PurchaseSalesService::transition_purchase_status(&pool, order.id, &trans)
        .await
        .unwrap();

    let err = PurchaseSalesService::delete_purchase_item(&pool, order.id, 0)
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
    PurchaseSalesService::transition_purchase_status(&pool, order.id, &trans)
        .await
        .unwrap();

    // approve
    let req = PurchaseApproveReq { notes: None };
    PurchaseSalesService::approve_purchase_order(&pool, order.id, &req)
        .await
        .expect("approve must succeed");

    let (fetched, _) = PurchaseSalesService::get_purchase_order(&pool, order.id)
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
    let err = PurchaseSalesService::approve_purchase_order(&pool, order.id, &req)
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
    PurchaseSalesService::transition_purchase_status(&pool, order.id, &trans)
        .await
        .unwrap();

    // reject
    let req = PurchaseRejectReq {
        reason: "price too high".into(),
    };
    PurchaseSalesService::reject_purchase_order(&pool, order.id, &req)
        .await
        .expect("reject must succeed");

    let (fetched, _) = PurchaseSalesService::get_purchase_order(&pool, order.id)
        .await
        .unwrap();
    assert_eq!(fetched.status, "rejected");
    // Reject stores reason in notes
    assert!(fetched.notes.as_deref().unwrap_or("").contains("price too high"));
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
    let err = PurchaseSalesService::reject_purchase_order(&pool, order.id, &req)
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
    PurchaseSalesService::transition_purchase_status(&pool, order_id, &t1)
        .await
        .unwrap();
    let req = PurchaseApproveReq { notes: None };
    PurchaseSalesService::approve_purchase_order(&pool, order_id, &req)
        .await
        .unwrap();

    // Create an inbound record to link
    let inbound_result = sqlx::query(
        "INSERT INTO inbound_records (inbound_no, inbound_type, notes, approval_status, \
         created_at, updated_at) \
         VALUES ($1, $2, $3, $4, datetime('now'), datetime('now'))",
    )
    .bind("INB-LNK-001")
    .bind("purchase")
    .bind("linked to PO")
    .bind("approved")
    .execute(&pool)
    .await
    .unwrap();
    let inbound_id = inbound_result.last_insert_rowid();

    PurchaseSalesService::link_inbound_to_order(&pool, order_id, inbound_id)
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
            pipe_type: "casing".into(),
            grade: "J55".into(),
            od: 177.8,
            wt: 9.19,
            quantity: 100,
            unit_price: Some(120.0),
            total_price: Some(12000.0),
            notes: None,
        }],
    };
    let order = PurchaseSalesService::create_purchase_order(&pool, &dto)
        .await
        .unwrap();
    assert_eq!(order.status, "draft");

    // 2. Update header
    let update = UpdatePurchaseOrderRequest {
        order_date: None,
        notes: Some("updated notes".into()),
    };
    let updated = PurchaseSalesService::update_purchase_order(&pool, order.id, &update)
        .await
        .unwrap();
    assert_eq!(updated.notes.as_deref(), Some("updated notes"));

    // 3. Submit (draft -> pending)
    let t1 = PurchaseOrderStatusTransitionRequest {
        status: "pending".into(),
    };
    PurchaseSalesService::transition_purchase_status(&pool, order.id, &t1)
        .await
        .unwrap();

    // 4. Approve (pending -> approved)
    let approve_req = PurchaseApproveReq { notes: None };
    PurchaseSalesService::approve_purchase_order(&pool, order.id, &approve_req)
        .await
        .unwrap();

    let (fetched, _) = PurchaseSalesService::get_purchase_order(&pool, order.id)
        .await
        .unwrap();
    assert_eq!(fetched.status, "approved");

    // 5. Link inbound
    let inbound_result = sqlx::query(
        "INSERT INTO inbound_records (inbound_no, inbound_type, notes, approval_status, \
         created_at, updated_at) \
         VALUES ($1, $2, $3, $4, datetime('now'), datetime('now'))",
    )
    .bind("INB-LIFE-001")
    .bind("purchase")
    .bind("full lifecycle link")
    .bind("approved")
    .execute(&pool)
    .await
    .unwrap();
    let inbound_id = inbound_result.last_insert_rowid();

    PurchaseSalesService::link_inbound_to_order(&pool, order.id, inbound_id)
        .await
        .expect("link_inbound must succeed");
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Sales Order — create
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn create_sales_order_with_items() {
    let pool = common::test_pool().await;

    let customer_id = common::seed_customer(&pool, "CUS-001", "Test Customer")
        .await
        .unwrap();

    let dto = CreateSalesOrderRequest {
        order_no: None,
        customer_id,
        order_date: "2025-06-15".into(),
        notes: Some("initial SO".into()),
        items: vec![CreateSalesItemRequest {
            pipe_type: "casing".into(),
            grade: "J55".into(),
            od: 177.8,
            wt: 9.19,
            quantity: 50,
            unit_price: Some(200.0),
            total_price: Some(10000.0),
            notes: None,
        }],
    };

    let order = PurchaseSalesService::create_sales_order(&pool, &dto)
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

    let err = PurchaseSalesService::create_sales_order(&pool, &dto)
        .await
        .expect_err("must fail with empty items");
    assert!(err.to_string().contains("At least one item"));
}

#[tokio::test]
async fn create_sales_order_fails_inactive_customer() {
    let pool = common::test_pool().await;

    // Manually insert inactive customer
    let result = sqlx::query(
        "INSERT INTO customers (customer_code, name, contact_person, phone, email, address, \
         is_active, notes, created_at, updated_at) \
         VALUES ($1, $2, 'Contact', '13800138001', $3, 'Addr', 0, 'inactive', \
         datetime('now'), datetime('now'))",
    )
    .bind("CUS-003")
    .bind("Inactive Customer")
    .bind("cus003@test.local")
    .execute(&pool)
    .await
    .unwrap();
    let customer_id = result.last_insert_rowid();

    let dto = CreateSalesOrderRequest {
        order_no: None,
        customer_id,
        order_date: "2025-06-15".into(),
        notes: None,
        items: vec![CreateSalesItemRequest {
            pipe_type: "casing".into(),
            grade: "J55".into(),
            od: 177.8,
            wt: 9.19,
            quantity: 10,
            unit_price: Some(100.0),
            total_price: Some(1000.0),
            notes: None,
        }],
    };

    let err = PurchaseSalesService::create_sales_order(&pool, &dto)
        .await
        .expect_err("must fail for inactive customer");
    assert!(err.to_string().contains("not active"));
}

#[tokio::test]
async fn create_sales_order_fails_nonexistent_customer() {
    let pool = common::test_pool().await;

    let dto = CreateSalesOrderRequest {
        order_no: None,
        customer_id: 99999,
        order_date: "2025-06-15".into(),
        notes: None,
        items: vec![CreateSalesItemRequest {
            pipe_type: "casing".into(),
            grade: "J55".into(),
            od: 177.8,
            wt: 9.19,
            quantity: 10,
            unit_price: Some(100.0),
            total_price: Some(1000.0),
            notes: None,
        }],
    };

    let err = PurchaseSalesService::create_sales_order(&pool, &dto)
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

    let updated = PurchaseSalesService::update_sales_order(&pool, order.id, &update)
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
    PurchaseSalesService::transition_sales_status(&pool, order.id, &trans)
        .await
        .unwrap();

    let update = UpdateSalesOrderRequest {
        order_date: None,
        notes: Some("should fail".into()),
    };
    let err = PurchaseSalesService::update_sales_order(&pool, order.id, &update)
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
    PurchaseSalesService::transition_sales_status(&pool, order.id, &trans)
        .await
        .expect("draft -> pending must succeed");

    let (fetched, _) = PurchaseSalesService::get_sales_order(&pool, order.id)
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
    let err = PurchaseSalesService::transition_sales_status(&pool, order.id, &trans)
        .await
        .expect_err("draft -> approved must fail");
    assert!(err.to_string().contains("Cannot transition"));
}

#[tokio::test]
async fn get_sales_order_with_items() {
    let pool = common::test_pool().await;

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
                pipe_type: "casing".into(),
                grade: "J55".into(),
                od: 177.8,
                wt: 9.19,
                quantity: 20,
                unit_price: Some(200.0),
                total_price: Some(4000.0),
                notes: None,
            },
            CreateSalesItemRequest {
                pipe_type: "tubing".into(),
                grade: "N80Q".into(),
                od: 88.9,
                wt: 6.45,
                quantity: 30,
                unit_price: Some(150.0),
                total_price: Some(4500.0),
                notes: None,
            },
        ],
    };

    let order = PurchaseSalesService::create_sales_order(&pool, &dto)
        .await
        .unwrap();

    let (fetched, items) = PurchaseSalesService::get_sales_order(&pool, order.id)
        .await
        .expect("get_sales_order must succeed");
    assert_eq!(fetched.id, order.id);
    assert_eq!(items.len(), 2);
}

#[tokio::test]
async fn list_sales_orders_pagination() {
    let pool = common::test_pool().await;

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
                pipe_type: "casing".into(),
                grade: "J55".into(),
                od: 177.8,
                wt: 9.19,
                quantity: 10,
                unit_price: Some(100.0),
            total_price: Some(1000.0),
                notes: None,
            }],
        };
        PurchaseSalesService::create_sales_order(&pool, &dto)
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

    let (orders, total) = PurchaseSalesService::list_sales_orders(&pool, &filter, &params)
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

    PurchaseSalesService::delete_sales_order(&pool, order.id)
        .await
        .expect("delete draft SO must succeed");

    let deleted_at: (Option<String>,) =
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

    let order_id =
        common::seed_sales_order(&pool, "SO-DEL-002", customer_id, "approved")
            .await
            .unwrap();

    let err = PurchaseSalesService::delete_sales_order(&pool, order_id)
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

    let customer_id = common::seed_customer(&pool, "CUS-SITM", "Item Customer")
        .await
        .unwrap();
    let dto = CreateSalesOrderRequest {
        order_no: Some("SO-SITM-001".into()),
        customer_id,
        order_date: "2025-06-15".into(),
        notes: None,
        items: vec![CreateSalesItemRequest {
            pipe_type: "casing".into(),
            grade: "J55".into(),
            od: 177.8,
            wt: 9.19,
            quantity: 30,
            unit_price: Some(200.0),
            total_price: Some(6000.0),
            notes: None,
        }],
    };
    let order = PurchaseSalesService::create_sales_order(&pool, &dto)
        .await
        .unwrap();

    let (_order, items) = PurchaseSalesService::get_sales_order(&pool, order.id)
        .await
        .unwrap();
    let item_id = items[0].id;

    let update = UpdateSalesItemRequest {
        pipe_type: None,
        grade: None,
        od: None,
        wt: None,
        quantity: Some(45),
        unit_price: Some(180.0),
        notes: None,
    };

    let (_order, updated) =
        PurchaseSalesService::update_sales_item(&pool, order.id, item_id, &update)
            .await
            .expect("update_sales_item must succeed");
    assert_eq!(updated.quantity, 45);
    assert_eq!(updated.unit_price, Some(180.0));
}

#[tokio::test]
async fn delete_sales_item_removes_item() {
    let pool = common::test_pool().await;

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
                pipe_type: "casing".into(),
                grade: "J55".into(),
                od: 177.8,
                wt: 9.19,
                quantity: 10,
                unit_price: Some(100.0),
            total_price: Some(1000.0),
                notes: None,
            },
            CreateSalesItemRequest {
                pipe_type: "tubing".into(),
                grade: "N80Q".into(),
                od: 88.9,
                wt: 6.45,
                quantity: 20,
                unit_price: Some(100.0),
            total_price: Some(1000.0),
                notes: None,
            },
        ],
    };
    let order = PurchaseSalesService::create_sales_order(&pool, &dto)
        .await
        .unwrap();

    let (_order, items) = PurchaseSalesService::get_sales_order(&pool, order.id)
        .await
        .unwrap();
    assert_eq!(items.len(), 2);

    PurchaseSalesService::delete_sales_item(&pool, order.id, items[0].id)
        .await
        .expect("delete_sales_item must succeed");

    let (_order, remaining) = PurchaseSalesService::get_sales_order(&pool, order.id)
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

    let customer_id = common::seed_customer(&pool, "CUS-SAPR", "SO Approve Customer")
        .await
        .unwrap();

    // Seed an in_stock pipe to satisfy ATP
    let pipe_id = common::seed_seamless_pipe(&pool, "PN-SAPR-001", "in_stock", "J55")
        .await
        .unwrap();
    // Also make a second one for quantity
    common::seed_seamless_pipe(&pool, "PN-SAPR-002", "in_stock", "J55")
        .await
        .unwrap();

    let dto = CreateSalesOrderRequest {
        order_no: Some("SO-SAPR-001".into()),
        customer_id,
        order_date: "2025-06-15".into(),
        notes: None,
        items: vec![CreateSalesItemRequest {
            pipe_type: "casing".into(),
            grade: "J55".into(),
            od: 177.8,
            wt: 9.19,
            quantity: 2,
            unit_price: Some(200.0),
            total_price: Some(400.0),
            notes: None,
        }],
    };
    let order = PurchaseSalesService::create_sales_order(&pool, &dto)
        .await
        .unwrap();

    // draft -> pending
    let trans = SalesOrderStatusTransitionRequest {
        status: "pending".into(),
    };
    PurchaseSalesService::transition_sales_status(&pool, order.id, &trans)
        .await
        .unwrap();

    // approve
    let req = SalesApproveReq { notes: None };
    PurchaseSalesService::approve_sales_order(&pool, order.id, &req)
        .await
        .expect("approve_sales_order must succeed");

    let (fetched, _) = PurchaseSalesService::get_sales_order(&pool, order.id)
        .await
        .unwrap();
    assert_eq!(fetched.status, "approved");
}

#[tokio::test]
async fn approve_sales_order_fails_insufficient_stock() {
    let pool = common::test_pool().await;

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
            pipe_type: "casing".into(),
            grade: "J55".into(),
            od: 177.8,
            wt: 9.19,
            quantity: 1,
            unit_price: Some(100.0),
            total_price: Some(1000.0),
            notes: None,
        }],
    };
    let order = PurchaseSalesService::create_sales_order(&pool, &dto)
        .await
        .unwrap();

    let trans = SalesOrderStatusTransitionRequest {
        status: "pending".into(),
    };
    PurchaseSalesService::transition_sales_status(&pool, order.id, &trans)
        .await
        .unwrap();

    let req = SalesApproveReq { notes: None };
    let err = PurchaseSalesService::approve_sales_order(&pool, order.id, &req)
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
    let err = PurchaseSalesService::approve_sales_order(&pool, order.id, &req)
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
    PurchaseSalesService::transition_sales_status(&pool, order.id, &trans)
        .await
        .unwrap();

    let req = SalesRejectReq {
        reason: "customer changed mind".into(),
    };
    PurchaseSalesService::reject_sales_order(&pool, order.id, &req)
        .await
        .expect("reject_sales_order must succeed");

    let (fetched, _) = PurchaseSalesService::get_sales_order(&pool, order.id)
        .await
        .unwrap();
    assert_eq!(fetched.status, "rejected");
    assert!(fetched.notes.as_deref().unwrap_or("").contains("changed mind"));
}

#[tokio::test]
async fn link_outbound_to_order_links() {
    let pool = common::test_pool().await;

    let customer_id = common::seed_customer(&pool, "CUS-SLNK", "SO Link Customer")
        .await
        .unwrap();

    // Create in_stock pipes for ATP
    common::seed_seamless_pipe(&pool, "PN-SLNK-001", "in_stock", "J55")
        .await
        .unwrap();

    let dto = CreateSalesOrderRequest {
        order_no: Some("SO-SLNK-001".into()),
        customer_id,
        order_date: "2025-06-15".into(),
        notes: None,
        items: vec![CreateSalesItemRequest {
            pipe_type: "casing".into(),
            grade: "J55".into(),
            od: 177.8,
            wt: 9.19,
            quantity: 1,
            unit_price: Some(200.0),
            total_price: Some(200.0),
            notes: None,
        }],
    };
    let order = PurchaseSalesService::create_sales_order(&pool, &dto)
        .await
        .unwrap();

    // draft -> pending -> approve
    let trans = SalesOrderStatusTransitionRequest {
        status: "pending".into(),
    };
    PurchaseSalesService::transition_sales_status(&pool, order.id, &trans)
        .await
        .unwrap();
    let req = SalesApproveReq { notes: None };
    PurchaseSalesService::approve_sales_order(&pool, order.id, &req)
        .await
        .unwrap();

    // Create outbound record to link
    let outbound_result = sqlx::query(
        "INSERT INTO outbound_records (outbound_no, outbound_type, notes, approval_status, \
         created_at, updated_at) \
         VALUES ($1, $2, $3, $4, datetime('now'), datetime('now'))",
    )
    .bind("OUT-SLNK-001")
    .bind("sales")
    .bind("linked to SO")
    .bind("approved")
    .execute(&pool)
    .await
    .unwrap();
    let outbound_id = outbound_result.last_insert_rowid();

    PurchaseSalesService::link_outbound_to_order(&pool, order.id, outbound_id)
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

    let customer_id = common::seed_customer(&pool, "CUS-ATP1", "ATP Customer 1")
        .await
        .unwrap();

    // Seed multiple in_stock pipes with matching grade
    common::seed_seamless_pipe(&pool, "PN-ATP-001", "in_stock", "J55")
        .await
        .unwrap();
    common::seed_seamless_pipe(&pool, "PN-ATP-002", "in_stock", "J55")
        .await
        .unwrap();
    common::seed_seamless_pipe(&pool, "PN-ATP-003", "in_stock", "J55")
        .await
        .unwrap();

    let dto = CreateSalesOrderRequest {
        order_no: Some("SO-ATP-001".into()),
        customer_id,
        order_date: "2025-06-15".into(),
        notes: None,
        items: vec![CreateSalesItemRequest {
            pipe_type: "casing".into(),
            grade: "J55".into(),
            od: 177.8,
            wt: 9.19,
            quantity: 3,
            unit_price: Some(200.0),
            total_price: Some(600.0),
            notes: None,
        }],
    };
    let order = PurchaseSalesService::create_sales_order(&pool, &dto)
        .await
        .unwrap();

    // draft -> pending -> approve (should pass ATP with 3 in_stock pipes)
    let trans = SalesOrderStatusTransitionRequest {
        status: "pending".into(),
    };
    PurchaseSalesService::transition_sales_status(&pool, order.id, &trans)
        .await
        .unwrap();

    let req = SalesApproveReq { notes: None };
    PurchaseSalesService::approve_sales_order(&pool, order.id, &req)
        .await
        .expect("ATP check must pass with sufficient stock");

    let (fetched, _) = PurchaseSalesService::get_sales_order(&pool, order.id)
        .await
        .unwrap();
    assert_eq!(fetched.status, "approved");
}

#[tokio::test]
async fn sales_order_atp_validation_fails_without_stock() {
    let pool = common::test_pool().await;

    let customer_id = common::seed_customer(&pool, "CUS-ATP2", "ATP Customer 2")
        .await
        .unwrap();

    // Seed one in_stock pipe but request 10 — insufficient
    common::seed_seamless_pipe(&pool, "PN-ATP-FAIL-001", "in_stock", "J55")
        .await
        .unwrap();

    let dto = CreateSalesOrderRequest {
        order_no: Some("SO-ATP-002".into()),
        customer_id,
        order_date: "2025-06-15".into(),
        notes: None,
        items: vec![CreateSalesItemRequest {
            pipe_type: "casing".into(),
            grade: "J55".into(),
            od: 177.8,
            wt: 9.19,
            quantity: 10,
            unit_price: Some(100.0),
            total_price: Some(1000.0),
            notes: None,
        }],
    };
    let order = PurchaseSalesService::create_sales_order(&pool, &dto)
        .await
        .unwrap();

    let trans = SalesOrderStatusTransitionRequest {
        status: "pending".into(),
    };
    PurchaseSalesService::transition_sales_status(&pool, order.id, &trans)
        .await
        .unwrap();

    let req = SalesApproveReq { notes: None };
    let err = PurchaseSalesService::approve_sales_order(&pool, order.id, &req)
        .await
        .expect_err("ATP check must fail with insufficient stock");
    assert!(err.to_string().to_lowercase().contains("insufficient"));
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Helpers
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Create a minimal purchase order with a single item (draft status).
async fn create_dummy_po(pool: &sqlx::SqlitePool, supplier_id: i64) -> steel_pipe_db::models::purchase_order::PurchaseOrder {
    let dto = CreatePurchaseOrderRequest {
        order_no: None,
        supplier_id,
        order_date: "2025-06-01".into(),
        notes: None,
        items: vec![CreatePurchaseItemRequest {
            pipe_type: "casing".into(),
            grade: "J55".into(),
            od: 177.8,
            wt: 9.19,
            quantity: 10,
            unit_price: Some(100.0),
            total_price: Some(1000.0),
            notes: None,
        }],
    };
    PurchaseSalesService::create_purchase_order(pool, &dto)
        .await
        .expect("create_dummy_po must succeed")
}

/// Create a minimal sales order with a single item (draft status).
async fn create_dummy_so(pool: &sqlx::SqlitePool, customer_id: i64) -> steel_pipe_db::models::sales_order::SalesOrder {
    let dto = CreateSalesOrderRequest {
        order_no: None,
        customer_id,
        order_date: "2025-06-15".into(),
        notes: None,
        items: vec![CreateSalesItemRequest {
            pipe_type: "casing".into(),
            grade: "J55".into(),
            od: 177.8,
            wt: 9.19,
            quantity: 10,
            unit_price: Some(100.0),
            total_price: Some(1000.0),
            notes: None,
        }],
    };
    PurchaseSalesService::create_sales_order(pool, &dto)
        .await
        .expect("create_dummy_so must succeed")
}
