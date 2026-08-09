//! Inventory ATP integration tests — reservations, transfers, cycle counts.
//!
//! Quantity-based generic ERP: stock lives in `inventory_logs`, reservations in
//! `atp_slots`, transfers in `internal_transfers`.

mod common;

use erp_server::dto::inventory_atp_dto::{
    CompleteCountSessionRequest, CreateCountTemplateRequest, CreateReservationRequest,
    CreateTransferRequest,
};
use erp_server::inventory_atp::services::InventoryAtpService;

/// Seed a generic item (商品) row; returns item id.
async fn seed_item(pool: &sqlx::SqlitePool, sku: &str) -> i64 {
    sqlx::query_scalar(
        "INSERT INTO items (sku, name, category, unit, spec, price, status) \
         VALUES (?, '测试商品', '原材料', '件', 'SPC', 10.0, 'active') RETURNING id",
    )
    .bind(sku)
    .fetch_one(pool)
    .await
    .unwrap()
}

/// Seed a location row; returns location id.
async fn seed_location(pool: &sqlx::SqlitePool, location_id: i64, full_code: &str) {
    sqlx::query(
        "INSERT INTO locations (id, full_code, zone_code, shelf_code, level_code, capacity, is_active) \
         VALUES (?, ?, 'Z', 'S', 'L', 1000, 1) ON CONFLICT (id) DO NOTHING",
    )
    .bind(location_id)
    .bind(full_code)
    .execute(pool)
    .await
    .unwrap();
}

/// Put an item quantity into stock at a given location (inbound log).
async fn seed_stock_at(pool: &sqlx::SqlitePool, item_id: i64, location_id: i64, quantity: f64) {
    sqlx::query(
        "INSERT INTO inventory_logs (item_id, quantity, change_type, ref_type, ref_id, \
         from_location_id, to_location_id, notes, created_by) \
         VALUES (?, ?, 'inbound', 'purchase', 1, NULL, ?, 'seed', NULL)",
    )
    .bind(item_id)
    .bind(quantity)
    .bind(location_id)
    .execute(pool)
    .await
    .unwrap();
}

#[tokio::test]
async fn reserve_and_release() {
    let pool = common::test_pool().await;
    let item_id = seed_item(&pool, "ATP-001").await;
    seed_location(&pool, 1, "LOC-1").await;
    seed_stock_at(&pool, item_id, 1, 50.0).await;

    let slot = InventoryAtpService::reserve(
        &pool,
        1,
        &CreateReservationRequest {
            item_id,
            quantity: 1.0,
            sales_order_id: Some(7),
        },
    )
    .await
    .unwrap();
    assert_eq!(slot.status, "reserved");

    // Per-item ATP reflects the reservation.
    let overview = InventoryAtpService::item_atp(&pool, 1, item_id).await.unwrap();
    assert_eq!(overview.on_hand, 50.0);
    assert_eq!(overview.reserved, 1.0);
    assert_eq!(overview.available, 49.0);

    let released = InventoryAtpService::release(&pool, 1, slot.id).await.unwrap();
    assert_eq!(released.status, "released");
}

#[tokio::test]
async fn over_reservation_rejected() {
    let pool = common::test_pool().await;
    let item_id = seed_item(&pool, "ATP-002").await;
    seed_location(&pool, 2, "LOC-2").await;
    seed_stock_at(&pool, item_id, 2, 5.0).await;

    let err = InventoryAtpService::reserve(
        &pool,
        1,
        &CreateReservationRequest {
            item_id,
            quantity: 6.0,
            sales_order_id: None,
        },
    )
    .await;
    assert!(err.is_err(), "reserving more than available must fail");
}

#[tokio::test]
async fn internal_transfer_moves_stock() {
    let pool = common::test_pool().await;
    let item_id = seed_item(&pool, "ATP-003").await;
    seed_location(&pool, 3, "LOC-3").await;
    seed_stock_at(&pool, item_id, 3, 30.0).await;
    // Destination location exists but has no stock yet.
    seed_location(&pool, 4, "LOC-4").await;

    let transfer = InventoryAtpService::create_transfer(
        &pool,
        1,
        Some(1),
        &CreateTransferRequest {
            from_location_id: 3,
            to_location_id: 4,
            item_id,
            quantity: 10.0,
            notes: Some("移库".into()),
        },
    )
    .await
    .unwrap();
    assert_eq!(transfer.status, "completed");

    // Source lost stock, destination gained it.
    let source: f64 = sqlx::query_scalar(
        "SELECT CAST(COALESCE(SUM(
             CASE WHEN to_location_id = ? THEN quantity
                  WHEN from_location_id = ? THEN -quantity
                  ELSE 0 END), 0.0) AS REAL)
         FROM inventory_logs WHERE item_id = ?",
    )
    .bind(3)
    .bind(3)
    .bind(item_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(source, 20.0, "source location must have 20 left");

    let dest: f64 = sqlx::query_scalar(
        "SELECT CAST(COALESCE(SUM(
             CASE WHEN to_location_id = ? THEN quantity
                  WHEN from_location_id = ? THEN -quantity
                  ELSE 0 END), 0.0) AS REAL)
         FROM inventory_logs WHERE item_id = ?",
    )
    .bind(4)
    .bind(4)
    .bind(item_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(dest, 10.0, "destination location must hold 10");
}

#[tokio::test]
async fn transfer_over_stock_rejected() {
    let pool = common::test_pool().await;
    let item_id = seed_item(&pool, "ATP-005").await;
    seed_location(&pool, 5, "LOC-5").await;
    seed_stock_at(&pool, item_id, 5, 3.0).await;
    seed_location(&pool, 6, "LOC-6").await;

    let err = InventoryAtpService::create_transfer(
        &pool,
        1,
        None,
        &CreateTransferRequest {
            from_location_id: 5,
            to_location_id: 6,
            item_id,
            quantity: 99.0,
            notes: None,
        },
    )
    .await;
    assert!(
        err.is_err(),
        "transfer exceeding source stock must fail atomically"
    );
}

#[tokio::test]
async fn count_template_and_session() {
    let pool = common::test_pool().await;
    let template = InventoryAtpService::create_count_template(
        &pool,
        1,
        &CreateCountTemplateRequest {
            name: "月度盘点".into(),
            description: Some("全库".into()),
            location_ids: vec![1, 2],
        },
    )
    .await
    .unwrap();
    assert!(template.id > 0);

    let session = InventoryAtpService::start_count_session(&pool, 1, template.id)
        .await
        .unwrap();
    assert_eq!(session.status, "inprogress");

    let done = InventoryAtpService::complete_count_session(
        &pool,
        1,
        &CompleteCountSessionRequest {
            session_id: session.id,
            result: serde_json::json!({"matches": 10, "mismatches": 1}),
        },
    )
    .await
    .unwrap();
    assert_eq!(done.status, "completed");
    assert!(done.completed_at.is_some());
}
