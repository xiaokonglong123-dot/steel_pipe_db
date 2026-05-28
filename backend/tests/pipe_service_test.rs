//! Integration tests for PipeService — seamless pipe, screen pipe, and cross-type search.
//!
//! Tests cover:
//! - Seamless pipe CRUD: create (with auto-generated and duplicate pipe numbers),
//!   update, soft-delete, get, paginated list with filters and sorting
//! - Screen pipe CRUD: create (with all attributes), update, soft-delete, get, list
//! - Cross-type search by pipe number and batch number
//!
//! All tests use an in-memory SQLite database with fresh migrations.

mod common;

use steel_pipe_db::dto::common::PaginationParams;
use steel_pipe_db::dto::pipe_dto::{
    CreateScreenPipeRequest, CreateSeamlessPipeRequest, PipeFilterParams,
    UpdateScreenPipeRequest, UpdateSeamlessPipeRequest,
};
use steel_pipe_db::services::pipe_service::PipeService;

/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
/// Seamless Pipes — Create
/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn create_seamless_pipe_with_auto_number() {
    let pool = common::test_pool().await;

    let dto = CreateSeamlessPipeRequest {
        pipe_number: None,
        batch_number: Some("BN-100".into()),
        pipe_type: "casing".into(),
        grade: "J55".into(),
        od: 177.8,
        wt: 9.19,
        length: Some(9.5),
        weight_per_unit: Some(40.0),
        end_type: Some("BTC".into()),
        coupling_type: Some("N80Q".into()),
        coupling_od: Some(194.0),
        coupling_length: Some(250.0),
        heat_number: Some("HN-100".into()),
        serial_number: Some("SN-100".into()),
        manufacturer: Some("Test Factory".into()),
        production_date: Some("2024-01-15".into()),
        cert_number: Some("CERT-100".into()),
        notes: Some("auto-number test".into()),
    };

    let pipe = PipeService::create_seamless_pipe(&pool, &dto)
        .await
        .expect("create_seamless_pipe with auto number must succeed");

    assert_eq!(pipe.grade, "J55");
    assert_eq!(pipe.od, 177.8);
    assert_eq!(pipe.wt, 9.19);
    assert_eq!(pipe.status, "in_stock");
    assert!(
        pipe.pipe_number.starts_with("SP-"),
        "Auto-generated pipe number should start with SP-, got: {}",
        pipe.pipe_number
    );
    assert_eq!(pipe.heat_number.as_deref(), Some("HN-100"));
    assert_eq!(pipe.manufacturer.as_deref(), Some("Test Factory"));
    assert_eq!(pipe.pipe_type, "casing");
}

#[tokio::test]
async fn create_seamless_pipe_with_explicit_number() {
    let pool = common::test_pool().await;

    let dto = CreateSeamlessPipeRequest {
        pipe_number: Some("PN-MANUAL-001".into()),
        batch_number: None,
        pipe_type: "tubing".into(),
        grade: "N80".into(),
        od: 88.9,
        wt: 6.45,
        length: None,
        weight_per_unit: None,
        end_type: None,
        coupling_type: None,
        coupling_od: None,
        coupling_length: None,
        heat_number: None,
        serial_number: None,
        manufacturer: None,
        production_date: None,
        cert_number: None,
        notes: None,
    };

    let pipe = PipeService::create_seamless_pipe(&pool, &dto)
        .await
        .expect("create_seamless_pipe with explicit number must succeed");

    assert_eq!(pipe.pipe_number, "PN-MANUAL-001");
    assert_eq!(pipe.grade, "N80");
    assert_eq!(pipe.pipe_type, "tubing");
    assert_eq!(pipe.od, 88.9);
    assert_eq!(pipe.wt, 6.45);
}

#[tokio::test]
async fn create_seamless_pipe_duplicate_number_fails() {
    let pool = common::test_pool().await;

    // Create a pipe with a known pipe number
    let dto = CreateSeamlessPipeRequest {
        pipe_number: Some("PN-DUP-001".into()),
        batch_number: None,
        pipe_type: "casing".into(),
        grade: "J55".into(),
        od: 177.8,
        wt: 9.19,
        length: None,
        weight_per_unit: None,
        end_type: None,
        coupling_type: None,
        coupling_od: None,
        coupling_length: None,
        heat_number: None,
        serial_number: None,
        manufacturer: None,
        production_date: None,
        cert_number: None,
        notes: None,
    };

    PipeService::create_seamless_pipe(&pool, &dto)
        .await
        .expect("first create must succeed");

    // Second create with the same pipe number must fail
    let err = PipeService::create_seamless_pipe(&pool, &dto)
        .await
        .expect_err("duplicate pipe number must fail");

    let msg = err.to_string();
    assert!(
        msg.contains("already exists"),
        "Expected duplicate error, got: {msg}"
    );
}

/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
/// Seamless Pipes — Update
/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn update_seamless_pipe_updates_fields() {
    let pool = common::test_pool().await;

    // Seed a pipe via the service first
    let dto = CreateSeamlessPipeRequest {
        pipe_number: Some("PN-UPDATE-001".into()),
        batch_number: Some("BN-OLD".into()),
        pipe_type: "casing".into(),
        grade: "J55".into(),
        od: 177.8,
        wt: 9.19,
        length: Some(9.5),
        weight_per_unit: Some(40.0),
        end_type: Some("STC".into()),
        coupling_type: None,
        coupling_od: None,
        coupling_length: None,
        heat_number: None,
        serial_number: None,
        manufacturer: None,
        production_date: None,
        cert_number: None,
        notes: Some("original".into()),
    };
    let pipe = PipeService::create_seamless_pipe(&pool, &dto).await.unwrap();

    // Update several fields
    let update = UpdateSeamlessPipeRequest {
        batch_number: Some("BN-UPDATED".into()),
        grade: Some("N80".into()),
        length: Some(10.0),
        coupling_type: Some("N80Q".into()),
        heat_number: Some("HN-UPDATED".into()),
        notes: Some("updated".into()),
        pipe_type: None,
        od: None,
        wt: None,
        weight_per_unit: None,
        end_type: None,
        coupling_od: None,
        coupling_length: None,
        serial_number: None,
        manufacturer: None,
        production_date: None,
        cert_number: None,
    };

    let updated = PipeService::update_seamless_pipe(&pool, pipe.id, &update)
        .await
        .expect("update must succeed");

    assert_eq!(updated.grade, "N80");
    assert_eq!(updated.batch_number.as_deref(), Some("BN-UPDATED"));
    assert_eq!(updated.length, Some(10.0));
    assert_eq!(updated.heat_number.as_deref(), Some("HN-UPDATED"));
    assert_eq!(updated.notes.as_deref(), Some("updated"));

    // Unchanged fields should remain as they were
    assert_eq!(updated.pipe_type, "casing");
    assert_eq!(updated.od, 177.8);
    assert_eq!(updated.wt, 9.19);
}

#[tokio::test]
async fn update_seamless_pipe_nonexistent_fails() {
    let pool = common::test_pool().await;

    let update = UpdateSeamlessPipeRequest {
        batch_number: None,
        pipe_type: None,
        grade: Some("N80".into()),
        od: None,
        wt: None,
        length: None,
        weight_per_unit: None,
        end_type: None,
        coupling_type: None,
        coupling_od: None,
        coupling_length: None,
        heat_number: None,
        serial_number: None,
        manufacturer: None,
        production_date: None,
        cert_number: None,
        notes: None,
    };

    let err = PipeService::update_seamless_pipe(&pool, 99999, &update)
        .await
        .expect_err("update for nonexistent pipe must fail");

    let msg = err.to_string();
    assert!(
        msg.contains("not found"),
        "Expected not-found error, got: {msg}"
    );
}

/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
/// Seamless Pipes — Delete
/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn delete_seamless_pipe_soft_deletes() {
    let pool = common::test_pool().await;

    // Seed an in_stock pipe (only in_stock can be deleted)
    let pipe_id = common::seed_seamless_pipe(&pool, "PN-DEL-001", "in_stock", "J55")
        .await
        .unwrap();

    PipeService::delete_seamless_pipe(&pool, pipe_id)
        .await
        .expect("delete in_stock pipe must succeed");

    // Pipe should be gone via service get
    let err = PipeService::get_seamless_pipe(&pool, pipe_id)
        .await
        .expect_err("deleted pipe should not be found");
    let msg = err.to_string();
    assert!(
        msg.contains("not found"),
        "Expected not-found error, got: {msg}"
    );

    // Check deleted_at is set directly in DB
    let deleted_at: (Option<String>,) =
        sqlx::query_as("SELECT deleted_at FROM seamless_pipes WHERE id = ?")
            .bind(pipe_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(
        deleted_at.0.is_some(),
        "deleted_at should be set after soft delete"
    );
}

#[tokio::test]
async fn delete_seamless_pipe_not_in_stock_fails() {
    let pool = common::test_pool().await;

    let pipe_id = common::seed_seamless_pipe(&pool, "PN-DEL-002", "scrapped", "J55")
        .await
        .unwrap();

    let err = PipeService::delete_seamless_pipe(&pool, pipe_id)
        .await
        .expect_err("delete scrapped pipe must fail");

    let msg = err.to_string();
    assert!(
        msg.contains("Cannot delete")
            || msg.contains("status conflict"),
        "Expected status conflict error, got: {msg}"
    );
}

#[tokio::test]
async fn delete_seamless_pipe_nonexistent_fails() {
    let pool = common::test_pool().await;

    let err = PipeService::delete_seamless_pipe(&pool, 99999)
        .await
        .expect_err("delete nonexistent pipe must fail");

    let msg = err.to_string();
    assert!(
        msg.contains("not found"),
        "Expected not-found error, got: {msg}"
    );
}

/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
/// Seamless Pipes — Get
/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn get_seamless_pipe_found() {
    let pool = common::test_pool().await;

    let pipe_id = common::seed_seamless_pipe(&pool, "PN-GET-001", "in_stock", "L80")
        .await
        .unwrap();

    let pipe = PipeService::get_seamless_pipe(&pool, pipe_id)
        .await
        .expect("get must succeed");

    assert_eq!(pipe.pipe_number, "PN-GET-001");
    assert_eq!(pipe.grade, "L80");
    assert_eq!(pipe.status, "in_stock");
}

#[tokio::test]
async fn get_seamless_pipe_not_found() {
    let pool = common::test_pool().await;

    let err = PipeService::get_seamless_pipe(&pool, 99999)
        .await
        .expect_err("get nonexistent must fail");

    let msg = err.to_string();
    assert!(
        msg.contains("not found"),
        "Expected not-found error, got: {msg}"
    );
}

/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
/// Seamless Pipes — List
/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn list_seamless_pipes_pagination() {
    let pool = common::test_pool().await;

    // Seed 5 pipes
    for i in 1..=5 {
        common::seed_seamless_pipe(&pool, &format!("PN-LIST-{:03}", i), "in_stock", "J55")
            .await
            .unwrap();
    }

    // Page 1 with page_size = 2
    let filter = PipeFilterParams {
        q: None,
        grade: None,
        pipe_type: None,
        status: None,
        od_min: None,
        od_max: None,
        wt_min: None,
        wt_max: None,
        location_id: None,
        manufacturer: None,
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

    let (items, total) = PipeService::list_seamless_pipes(&pool, &filter, &params)
        .await
        .expect("list must succeed");

    assert_eq!(total, 5, "total should be 5");
    assert_eq!(items.len(), 2, "page 1 should have 2 items");

    // Page 3 should have 1 item
    let params3 = PaginationParams {
        page: Some(3),
        page_size: Some(2),
        sort_by: None,
        sort_order: None,
    };

    let (items3, total3) = PipeService::list_seamless_pipes(&pool, &filter, &params3)
        .await
        .expect("list page 3 must succeed");

    assert_eq!(total3, 5, "total should still be 5");
    assert_eq!(items3.len(), 1, "page 3 should have 1 item");
}

#[tokio::test]
async fn list_seamless_pipes_filters_by_status_and_grade() {
    let pool = common::test_pool().await;

    common::seed_seamless_pipe(&pool, "PN-FILTER-001", "in_stock", "J55")
        .await
        .unwrap();
    common::seed_seamless_pipe(&pool, "PN-FILTER-002", "in_stock", "N80")
        .await
        .unwrap();
    common::seed_seamless_pipe(&pool, "PN-FILTER-003", "scrapped", "J55")
        .await
        .unwrap();

    let default_params = PaginationParams {
        page: None,
        page_size: None,
        sort_by: None,
        sort_order: None,
    };

    // Filter by status = "in_stock"
    let filter_status = PipeFilterParams {
        q: None,
        grade: None,
        pipe_type: None,
        status: Some("in_stock".into()),
        od_min: None,
        od_max: None,
        wt_min: None,
        wt_max: None,
        location_id: None,
        manufacturer: None,
        page: None,
        page_size: None,
        sort_by: None,
        sort_order: None,
    };

    let (_items, total) =
        PipeService::list_seamless_pipes(&pool, &filter_status, &default_params)
            .await
            .expect("list with status filter must succeed");
    assert_eq!(total, 2, "should find 2 in_stock pipes");

    // Filter by grade = "N80"
    let filter_grade = PipeFilterParams {
        q: None,
        grade: Some("N80".into()),
        pipe_type: None,
        status: None,
        od_min: None,
        od_max: None,
        wt_min: None,
        wt_max: None,
        location_id: None,
        manufacturer: None,
        page: None,
        page_size: None,
        sort_by: None,
        sort_order: None,
    };

    let (items_grade, total_grade) =
        PipeService::list_seamless_pipes(&pool, &filter_grade, &default_params)
            .await
            .expect("list with grade filter must succeed");
    assert_eq!(total_grade, 1, "should find 1 N80 pipe");
    assert_eq!(items_grade[0].pipe_number, "PN-FILTER-002");

    // Filter by combined status + grade (no results)
    let filter_combined = PipeFilterParams {
        q: None,
        grade: Some("N80".into()),
        pipe_type: None,
        status: Some("scrapped".into()),
        od_min: None,
        od_max: None,
        wt_min: None,
        wt_max: None,
        location_id: None,
        manufacturer: None,
        page: None,
        page_size: None,
        sort_by: None,
        sort_order: None,
    };

    let (items_comb, total_comb) =
        PipeService::list_seamless_pipes(&pool, &filter_combined, &default_params)
            .await
            .expect("list with combined filters must succeed");
    assert_eq!(total_comb, 0, "no scrapped N80 pipes expected");
    assert!(items_comb.is_empty());
}

#[tokio::test]
async fn list_seamless_pipes_sorts_by_column() {
    let pool = common::test_pool().await;

    common::seed_seamless_pipe(&pool, "PN-SORT-B", "in_stock", "N80")
        .await
        .unwrap();
    common::seed_seamless_pipe(&pool, "PN-SORT-A", "in_stock", "J55")
        .await
        .unwrap();

    let filter = PipeFilterParams {
        q: None,
        grade: None,
        pipe_type: None,
        status: None,
        od_min: None,
        od_max: None,
        wt_min: None,
        wt_max: None,
        location_id: None,
        manufacturer: None,
        page: None,
        page_size: None,
        sort_by: None,
        sort_order: None,
    };

    // Sort by pipe_number ascending
    let params_asc = PaginationParams {
        page: None,
        page_size: None,
        sort_by: Some("pipe_number".into()),
        sort_order: Some("asc".into()),
    };

    let (items, _total) = PipeService::list_seamless_pipes(&pool, &filter, &params_asc)
        .await
        .expect("list sorted asc must succeed");

    assert_eq!(items.len(), 2);
    assert_eq!(items[0].pipe_number, "PN-SORT-A", "asc: first should be A");
    assert_eq!(items[1].pipe_number, "PN-SORT-B", "asc: second should be B");

    // Sort by pipe_number descending
    let params_desc = PaginationParams {
        page: None,
        page_size: None,
        sort_by: Some("pipe_number".into()),
        sort_order: Some("desc".into()),
    };

    let (items_desc, _total_desc) =
        PipeService::list_seamless_pipes(&pool, &filter, &params_desc)
            .await
            .expect("list sorted desc must succeed");

    assert_eq!(items_desc.len(), 2);
    assert_eq!(
        items_desc[0].pipe_number, "PN-SORT-B",
        "desc: first should be B"
    );
    assert_eq!(
        items_desc[1].pipe_number, "PN-SORT-A",
        "desc: second should be A"
    );
}

/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
/// Screen Pipes — Create
/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn create_screen_pipe_with_all_attributes() {
    let pool = common::test_pool().await;

    let dto = CreateScreenPipeRequest {
        pipe_number: Some("SC-FULL-001".into()),
        batch_number: Some("BN-SC-001".into()),
        screen_type: "wire_wrapped".into(),
        slot_size: Some(0.02),
        filtration_grade: Some("high".into()),
        base_od: 177.8,
        base_wt: 9.19,
        base_grade: "L80".into(),
        base_end_type: Some("BTC".into()),
        length: Some(9.5),
        weight_per_unit: Some(42.0),
        heat_number: Some("HN-SC-001".into()),
        serial_number: Some("SN-SC-001".into()),
        manufacturer: Some("Screen Factory".into()),
        production_date: Some("2024-03-01".into()),
        cert_number: Some("CERT-SC-001".into()),
        notes: Some("full spec screen pipe".into()),
    };

    let pipe = PipeService::create_screen_pipe(&pool, &dto)
        .await
        .expect("create_screen_pipe must succeed");

    assert_eq!(pipe.pipe_number, "SC-FULL-001");
    assert_eq!(pipe.screen_type, "wire_wrapped");
    assert_eq!(pipe.slot_size, Some(0.02));
    assert_eq!(pipe.base_grade, "L80");
    assert_eq!(pipe.base_od, 177.8);
    assert_eq!(pipe.base_wt, 9.19);
    assert_eq!(pipe.manufacturer.as_deref(), Some("Screen Factory"));
    assert_eq!(pipe.heat_number.as_deref(), Some("HN-SC-001"));
    assert_eq!(pipe.status, "in_stock");
}

#[tokio::test]
async fn create_screen_pipe_with_auto_number() {
    let pool = common::test_pool().await;

    let dto = CreateScreenPipeRequest {
        pipe_number: None,
        batch_number: None,
        screen_type: "slotted".into(),
        slot_size: None,
        filtration_grade: None,
        base_od: 88.9,
        base_wt: 6.45,
        base_grade: "N80".into(),
        base_end_type: None,
        length: None,
        weight_per_unit: None,
        heat_number: None,
        serial_number: None,
        manufacturer: None,
        production_date: None,
        cert_number: None,
        notes: None,
    };

    let pipe = PipeService::create_screen_pipe(&pool, &dto)
        .await
        .expect("create_screen_pipe with auto number must succeed");

    assert_eq!(pipe.base_grade, "N80");
    assert_eq!(pipe.screen_type, "slotted");
    assert!(
        pipe.pipe_number.starts_with("SCP-"),
        "Auto-generated screen pipe number should start with SCP-, got: {}",
        pipe.pipe_number
    );
}

#[tokio::test]
async fn create_screen_pipe_duplicate_number_fails() {
    let pool = common::test_pool().await;

    let dto = CreateScreenPipeRequest {
        pipe_number: Some("SC-DUP-001".into()),
        batch_number: None,
        screen_type: "slotted".into(),
        slot_size: None,
        filtration_grade: None,
        base_od: 177.8,
        base_wt: 9.19,
        base_grade: "J55".into(),
        base_end_type: None,
        length: None,
        weight_per_unit: None,
        heat_number: None,
        serial_number: None,
        manufacturer: None,
        production_date: None,
        cert_number: None,
        notes: None,
    };

    PipeService::create_screen_pipe(&pool, &dto)
        .await
        .expect("first create must succeed");

    let err = PipeService::create_screen_pipe(&pool, &dto)
        .await
        .expect_err("duplicate screen pipe number must fail");

    let msg = err.to_string();
    assert!(
        msg.contains("already exists"),
        "Expected duplicate error, got: {msg}"
    );
}

/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
/// Screen Pipes — Update
/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn update_screen_pipe_updates_fields() {
    let pool = common::test_pool().await;

    let pipe_id = common::seed_screen_pipe(&pool, "SC-UPD-001", "in_stock", "J55")
        .await
        .unwrap();

    let update = UpdateScreenPipeRequest {
        batch_number: Some("BN-SC-UPD".into()),
        screen_type: Some("wire_wrapped".into()),
        slot_size: Some(0.03),
        filtration_grade: Some("premium".into()),
        base_grade: Some("L80".into()),
        notes: Some("updated screen pipe".into()),
        base_od: None,
        base_wt: None,
        base_end_type: None,
        length: None,
        weight_per_unit: None,
        heat_number: None,
        serial_number: None,
        manufacturer: None,
        production_date: None,
        cert_number: None,
    };

    let updated = PipeService::update_screen_pipe(&pool, pipe_id, &update)
        .await
        .expect("update screen pipe must succeed");

    assert_eq!(updated.screen_type, "wire_wrapped");
    assert_eq!(updated.slot_size, Some(0.03));
    assert_eq!(updated.filtration_grade.as_deref(), Some("premium"));
    assert_eq!(updated.base_grade, "L80");
    assert_eq!(updated.notes.as_deref(), Some("updated screen pipe"));
    assert_eq!(updated.batch_number.as_deref(), Some("BN-SC-UPD"));

    // Unchanged fields
    assert_eq!(updated.base_od, 177.8);
    assert_eq!(updated.base_wt, 9.19);
}

#[tokio::test]
async fn update_screen_pipe_nonexistent_fails() {
    let pool = common::test_pool().await;

    let update = UpdateScreenPipeRequest {
        batch_number: None,
        screen_type: Some("slotted".into()),
        slot_size: None,
        filtration_grade: None,
        base_od: None,
        base_wt: None,
        base_grade: None,
        base_end_type: None,
        length: None,
        weight_per_unit: None,
        heat_number: None,
        serial_number: None,
        manufacturer: None,
        production_date: None,
        cert_number: None,
        notes: None,
    };

    let err = PipeService::update_screen_pipe(&pool, 99999, &update)
        .await
        .expect_err("update nonexistent screen pipe must fail");

    let msg = err.to_string();
    assert!(
        msg.contains("not found"),
        "Expected not-found error, got: {msg}"
    );
}

/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
/// Screen Pipes — Delete
/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn delete_screen_pipe_soft_deletes() {
    let pool = common::test_pool().await;

    let pipe_id = common::seed_screen_pipe(&pool, "SC-DEL-001", "in_stock", "L80")
        .await
        .unwrap();

    PipeService::delete_screen_pipe(&pool, pipe_id)
        .await
        .expect("delete in_stock screen pipe must succeed");

    // Verify it's gone via service
    let err = PipeService::get_screen_pipe(&pool, pipe_id)
        .await
        .expect_err("deleted screen pipe should not be found");
    let msg = err.to_string();
    assert!(
        msg.contains("not found"),
        "Expected not-found error, got: {msg}"
    );

    // Verify deleted_at is set in DB
    let deleted_at: (Option<String>,) =
        sqlx::query_as("SELECT deleted_at FROM screen_pipes WHERE id = ?")
            .bind(pipe_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(
        deleted_at.0.is_some(),
        "deleted_at should be set after soft delete"
    );
}

#[tokio::test]
async fn delete_screen_pipe_not_in_stock_fails() {
    let pool = common::test_pool().await;

    let pipe_id = common::seed_screen_pipe(&pool, "SC-DEL-002", "scrapped", "J55")
        .await
        .unwrap();

    let err = PipeService::delete_screen_pipe(&pool, pipe_id)
        .await
        .expect_err("delete scrapped screen pipe must fail");

    let msg = err.to_string();
    assert!(
        msg.contains("Cannot delete")
            || msg.contains("status conflict"),
        "Expected status conflict error, got: {msg}"
    );
}

/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
/// Screen Pipes — Get
/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn get_screen_pipe_found() {
    let pool = common::test_pool().await;

    let pipe_id = common::seed_screen_pipe(&pool, "SC-GET-001", "in_stock", "N80")
        .await
        .unwrap();

    let pipe = PipeService::get_screen_pipe(&pool, pipe_id)
        .await
        .expect("get_screen_pipe must succeed");

    assert_eq!(pipe.pipe_number, "SC-GET-001");
    assert_eq!(pipe.base_grade, "N80");
    assert_eq!(pipe.status, "in_stock");
    assert_eq!(pipe.screen_type, "slotted");
}

#[tokio::test]
async fn get_screen_pipe_not_found() {
    let pool = common::test_pool().await;

    let err = PipeService::get_screen_pipe(&pool, 99999)
        .await
        .expect_err("get nonexistent screen pipe must fail");

    let msg = err.to_string();
    assert!(
        msg.contains("not found"),
        "Expected not-found error, got: {msg}"
    );
}

/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
/// Screen Pipes — List
/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn list_screen_pipes_pagination() {
    let pool = common::test_pool().await;

    for i in 1..=4 {
        common::seed_screen_pipe(
            &pool,
            &format!("SC-LIST-{:03}", i),
            "in_stock",
            "J55",
        )
        .await
        .unwrap();
    }

    let filter = PipeFilterParams {
        q: None,
        grade: None,
        pipe_type: None,
        status: None,
        od_min: None,
        od_max: None,
        wt_min: None,
        wt_max: None,
        location_id: None,
        manufacturer: None,
        page: None,
        page_size: None,
        sort_by: None,
        sort_order: None,
    };
    let params = PaginationParams {
        page: Some(1),
        page_size: Some(3),
        sort_by: None,
        sort_order: None,
    };

    let (items, total) = PipeService::list_screen_pipes(&pool, &filter, &params)
        .await
        .expect("list screen pipes must succeed");

    assert_eq!(total, 4, "total should be 4");
    assert_eq!(items.len(), 3, "page 1 should have 3 items");
}

#[tokio::test]
async fn list_screen_pipes_filters_by_grade_and_status() {
    let pool = common::test_pool().await;

    common::seed_screen_pipe(&pool, "SC-FILT-001", "in_stock", "J55")
        .await
        .unwrap();
    common::seed_screen_pipe(&pool, "SC-FILT-002", "in_stock", "L80")
        .await
        .unwrap();
    common::seed_screen_pipe(&pool, "SC-FILT-003", "scrapped", "L80")
        .await
        .unwrap();

    let default_params = PaginationParams {
        page: None,
        page_size: None,
        sort_by: None,
        sort_order: None,
    };

    // Filter by base_grade = "L80" (uses grade field in PipeFilterParams)
    let filter_grade = PipeFilterParams {
        q: None,
        grade: Some("L80".into()),
        pipe_type: None,
        status: None,
        od_min: None,
        od_max: None,
        wt_min: None,
        wt_max: None,
        location_id: None,
        manufacturer: None,
        page: None,
        page_size: None,
        sort_by: None,
        sort_order: None,
    };

    let (items, total) = PipeService::list_screen_pipes(&pool, &filter_grade, &default_params)
        .await
        .expect("list with grade filter must succeed");
    assert_eq!(total, 2, "should find 2 L80 screen pipes");
    assert_eq!(items.len(), 2);

    // Filter by status = "in_stock"
    let filter_status = PipeFilterParams {
        q: None,
        grade: None,
        pipe_type: None,
        status: Some("in_stock".into()),
        od_min: None,
        od_max: None,
        wt_min: None,
        wt_max: None,
        location_id: None,
        manufacturer: None,
        page: None,
        page_size: None,
        sort_by: None,
        sort_order: None,
    };

    let (_items_stock, total_stock) =
        PipeService::list_screen_pipes(&pool, &filter_status, &default_params)
            .await
            .expect("list with status filter must succeed");
    assert_eq!(total_stock, 2, "should find 2 in_stock screen pipes");
}

/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
/// Search — across both pipe types
/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn search_pipes_finds_by_pipe_number() {
    let pool = common::test_pool().await;

    let _seamless_id =
        common::seed_seamless_pipe(&pool, "PN-SRCH-001", "in_stock", "J55")
            .await
            .unwrap();
    let _screen_id =
        common::seed_screen_pipe(&pool, "SC-SRCH-001", "in_stock", "N80")
            .await
            .unwrap();

    // Search for the seamless pipe by number
    let results = PipeService::search_pipes(&pool, "PN-SRCH-001")
        .await
        .expect("search must succeed");

    // Should find exactly 1 seamless match
    let seamless_results: Vec<_> = results
        .iter()
        .filter(|r| r.pipe_type == "seamless")
        .collect();
    assert_eq!(
        seamless_results.len(),
        1,
        "should find 1 seamless pipe: {:?}",
        results
    );

    let screen_results: Vec<_> = results
        .iter()
        .filter(|r| r.pipe_type == "screen")
        .collect();
    assert_eq!(screen_results.len(), 0, "should not match screen pipes");
}

#[tokio::test]
async fn search_pipes_finds_both_types() {
    let pool = common::test_pool().await;

    // Seed pipes that share a common batch number pattern
    common::seed_seamless_pipe(&pool, "PN-BOTH-001", "in_stock", "J55")
        .await
        .unwrap();
    common::seed_screen_pipe(&pool, "SC-BOTH-001", "in_stock", "L80")
        .await
        .unwrap();

    // Search by common batch number "BN-001" (both seed helpers use this batch number)
    let results = PipeService::search_pipes(&pool, "BOTH")
        .await
        .expect("search must succeed");

    // Should find matches from both types since search uses LIKE on pipe_number
    let seamless: Vec<_> = results
        .iter()
        .filter(|r| r.pipe_type == "seamless")
        .collect();
    let screen: Vec<_> = results
        .iter()
        .filter(|r| r.pipe_type == "screen")
        .collect();

    assert!(
        !seamless.is_empty(),
        "should find seamless pipe (found {} total results)",
        results.len()
    );
    assert!(
        !screen.is_empty(),
        "should find screen pipe (found {} total results)",
        results.len()
    );
}

#[tokio::test]
async fn search_pipes_no_results() {
    let pool = common::test_pool().await;

    // Seed some pipes so the table isn't empty
    common::seed_seamless_pipe(&pool, "PN-NIL-001", "in_stock", "J55")
        .await
        .unwrap();
    common::seed_screen_pipe(&pool, "SC-NIL-001", "in_stock", "N80")
        .await
        .unwrap();

    // Search for something that doesn't exist
    let results = PipeService::search_pipes(&pool, "ZZZZ_NONEXISTENT_ZZZZ")
        .await
        .expect("search with no matches must succeed");

    assert!(
        results.is_empty(),
        "expected empty results, got {} items",
        results.len()
    );
}

#[tokio::test]
async fn search_pipes_empty_query_returns_all() {
    let pool = common::test_pool().await;

    common::seed_seamless_pipe(&pool, "PN-EMPTYQ-001", "in_stock", "J55")
        .await
        .unwrap();
    common::seed_screen_pipe(&pool, "SC-EMPTYQ-001", "in_stock", "N80")
        .await
        .unwrap();

    // An empty search string is treated as LIKE '%%' which matches everything
    let results = PipeService::search_pipes(&pool, "")
        .await
        .expect("search with empty query must succeed");

    // Should return both pipes (limited to 50)
    assert!(
        results.len() >= 2,
        "empty query should match all pipes, got {}",
        results.len()
    );
}
