//! Inventory ATP integration tests — reservations, transfers, cycle counts.

mod common;

use rust_decimal_macros::dec;
use steel_pipe_db::dto::inventory_atp_dto::{
    CompleteCountSessionRequest, CreateCountTemplateRequest, CreateReservationRequest,
    CreateTransferRequest,
};
use steel_pipe_db::inventory_atp::services::InventoryAtpService;

/// Seed a location + seamless pipes (one row per pipe, status='in_stock')
/// so ATP/transfer tests have stock to work with.
async fn seed_stock(pool: &sqlx::PgPool, location_id: i64, count: i64) {
    sqlx::query(
        "INSERT INTO locations (id, full_code, zone_code, shelf_code, level_code, capacity) \
         VALUES ($1, $2, 'Z', 'S', 'L', 1000) ON CONFLICT (id) DO NOTHING",
    )
    .bind(location_id)
    .bind(format!("LOC-{}", location_id))
    .execute(pool)
    .await
    .unwrap();
    for i in 0..count {
        sqlx::query(
            "INSERT INTO seamless_pipes \
             (pipe_number, pipe_type, grade, od, wt, location_id, status) \
             VALUES ($1, 'casing', 'J55', 244.5, 11.05, $2, 'in_stock') \
             ON CONFLICT (pipe_number) DO NOTHING",
        )
        .bind(format!("PN-ATP-{}-{}", location_id, i))
        .bind(location_id)
        .execute(pool)
        .await
        .unwrap();
    }
}

#[tokio::test]
async fn reserve_and_release() {
    let pool = common::test_pool().await;
    seed_stock(&pool, 1, 50).await;

    let slot = InventoryAtpService::reserve(
        &pool,
        1,
        &CreateReservationRequest {
            pipe_type: "seamless".into(),
            pipe_number: Some("PN-ATP-1-0".into()),
            quantity: dec!(1),
            sales_order_id: Some(7),
        },
    )
    .await
    .unwrap();
    assert_eq!(slot.status, "reserved");

    // Per-pipe ATP reflects the reservation.
    let overview = InventoryAtpService::pipe_atp(&pool, 1, "seamless", "PN-ATP-1-0").await.unwrap();
    assert_eq!(overview.on_hand, dec!(1));
    assert_eq!(overview.reserved, dec!(1));
    assert_eq!(overview.available, dec!(0));

    let released = InventoryAtpService::release(&pool, 1, slot.id).await.unwrap();
    assert_eq!(released.status, "released");
}

#[tokio::test]
async fn over_reservation_rejected() {
    let pool = common::test_pool().await;
    seed_stock(&pool, 2, 5).await;

    let err = InventoryAtpService::reserve(
        &pool,
        1,
        &CreateReservationRequest {
            pipe_type: "seamless".into(),
            pipe_number: Some("PN-ATP-2-0".into()),
            quantity: dec!(2),
            sales_order_id: None,
        },
    )
    .await;
    assert!(err.is_err(), "reserving more than available must fail");
}

#[tokio::test]
async fn internal_transfer_moves_stock() {
    let pool = common::test_pool().await;
    seed_stock(&pool, 3, 30).await;
    // Destination location exists but has no stock yet.
    sqlx::query(
        "INSERT INTO locations (id, full_code, zone_code, shelf_code, level_code, capacity) \
         VALUES (4, 'LOC-4', 'Z', 'S', 'L', 1000) ON CONFLICT (id) DO NOTHING",
    )
    .execute(&pool)
    .await
    .unwrap();

    let transfer = InventoryAtpService::create_transfer(
        &pool,
        1,
        Some(1),
        &CreateTransferRequest {
            from_location_id: 3,
            to_location_id: 4,
            pipe_id: None,
            pipe_number: Some("PN-ATP-3-0".into()),
            quantity: dec!(1),
            notes: Some("移库".into()),
        },
    )
    .await
    .unwrap();
    assert_eq!(transfer.status, "completed");

    // The pipe's location_id moved from 3 to 4.
    let loc: Option<i64> = sqlx::query_scalar(
        "SELECT location_id FROM seamless_pipes WHERE pipe_number = 'PN-ATP-3-0'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(loc, Some(4), "pipe must now live at the destination location");
}

#[tokio::test]
async fn transfer_over_stock_rejected() {
    let pool = common::test_pool().await;
    seed_stock(&pool, 5, 3).await;
    sqlx::query(
        "INSERT INTO locations (id, full_code, zone_code, shelf_code, level_code, capacity) \
         VALUES (6, 'LOC-6', 'Z', 'S', 'L', 1000) ON CONFLICT (id) DO NOTHING",
    )
    .execute(&pool)
    .await
    .unwrap();

    let err = InventoryAtpService::create_transfer(
        &pool,
        1,
        None,
        &CreateTransferRequest {
            from_location_id: 5,
            to_location_id: 6,
            pipe_id: None,
            pipe_number: Some("PN-ATP-5-NOPE".into()),
            quantity: dec!(1),
            notes: None,
        },
    )
    .await;
    assert!(err.is_err(), "transfer exceeding source stock must fail atomically");
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

    let session = InventoryAtpService::start_count_session(&pool, 1, template.id).await.unwrap();
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
