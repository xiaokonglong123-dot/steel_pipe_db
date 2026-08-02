//! Manufacturing integration tests — BOMs, work orders (step machine),
//! inspections, NCRs.

mod common;

use rust_decimal_macros::dec;
use steel_pipe_db::dto::manufacturing_dto::{
    CreateBomRequest, CreateInspectionRequest, CreateNcrRequest, CreateWorkOrderRequest,
    BomItemInput,
};
use steel_pipe_db::manufacturing::services::ManufacturingService;

#[tokio::test]
async fn bom_lifecycle() {
    let pool = common::test_pool().await;
    let bom = ManufacturingService::create_bom(
        &pool,
        1,
        &CreateBomRequest {
            name: "N80 套管 BOM".into(),
            product_type: "seamless".into(),
            notes: None,
            items: vec![
                BomItemInput { material: "steel_billet".into(), quantity: dec!(1), unit: Some("t".into()), notes: None },
                BomItemInput { material: "thread_compound".into(), quantity: dec!(0.5), unit: Some("kg".into()), notes: None },
            ],
        },
    )
    .await
    .unwrap();
    assert!(bom.id > 0);
    assert_eq!(bom.version, 1);

    let (got, items) = ManufacturingService::get_bom(&pool, 1, bom.id).await.unwrap();
    assert_eq!(got.name, "N80 套管 BOM");
    assert_eq!(items.len(), 2);
}

#[tokio::test]
async fn empty_bom_rejected() {
    let pool = common::test_pool().await;
    let err = ManufacturingService::create_bom(
        &pool,
        1,
        &CreateBomRequest { name: "空 BOM".into(), product_type: "seamless".into(), notes: None, items: vec![] },
    )
    .await;
    assert!(err.is_err(), "BOM without items must be rejected");
}

#[tokio::test]
async fn work_order_step_machine() {
    let pool = common::test_pool().await;
    let wo = ManufacturingService::create_work_order(
        &pool,
        1,
        &CreateWorkOrderRequest {
            bom_id: None,
            product_type: "seamless".into(),
            quantity: dec!(10),
            assigned_to: None,
            due_date: None,
            notes: None,
        },
    )
    .await
    .unwrap();
    assert_eq!(wo.status, "pending");
    assert_eq!(wo.current_step, 0);

    // Can't advance a pending order.
    let err = ManufacturingService::complete_step(&pool, 1, wo.id).await;
    assert!(err.is_err(), "pending work order must not advance");

    // Start → in_progress.
    let started = ManufacturingService::start_work_order(&pool, 1, wo.id).await.unwrap();
    assert_eq!(started.status, "in_progress");

    // Complete step 0 → current_step 1.
    let after1 = ManufacturingService::complete_step(&pool, 1, wo.id).await.unwrap();
    assert_eq!(after1.current_step, 1);
    assert_eq!(after1.status, "in_progress");

    // Complete step 1 → current_step 2.
    let after2 = ManufacturingService::complete_step(&pool, 1, wo.id).await.unwrap();
    assert_eq!(after2.current_step, 2);

    // Complete final step → completed.
    let done = ManufacturingService::complete_step(&pool, 1, wo.id).await.unwrap();
    assert_eq!(done.status, "completed");
}

#[tokio::test]
async fn work_order_from_bom_steps() {
    let pool = common::test_pool().await;
    let bom = ManufacturingService::create_bom(
        &pool,
        1,
        &CreateBomRequest {
            name: "BOM-2".into(),
            product_type: "welded".into(),
            notes: None,
            items: vec![
                BomItemInput { material: "coil".into(), quantity: dec!(1), unit: None, notes: None },
                BomItemInput { material: "coupling".into(), quantity: dec!(2), unit: None, notes: None },
            ],
        },
    )
    .await
    .unwrap();
    let wo = ManufacturingService::create_work_order(
        &pool,
        1,
        &CreateWorkOrderRequest { bom_id: Some(bom.id), product_type: "welded".into(), quantity: dec!(5), assigned_to: None, due_date: None, notes: None },
    )
    .await
    .unwrap();
    let (_, steps) = ManufacturingService::get_work_order(&pool, 1, wo.id).await.unwrap();
    assert_eq!(steps.len(), 2, "BOM items become work order steps");
}

#[tokio::test]
async fn inspection_and_ncr_flow() {
    let pool = common::test_pool().await;
    // Create a work order first so the inspection FK is satisfied.
    let wo = ManufacturingService::create_work_order(
        &pool,
        1,
        &CreateWorkOrderRequest { bom_id: None, product_type: "seamless".into(), quantity: dec!(1), assigned_to: None, due_date: None, notes: None },
    )
    .await
    .unwrap();
    let insp = ManufacturingService::create_inspection(
        &pool,
        1,
        &CreateInspectionRequest {
            work_order_id: Some(wo.id),
            pipe_id: None,
            inspection_type: "hydrostatic".into(),
            result: "fail".into(),
            notes: Some("压力不足".into()),
        },
        Some(1),
    )
    .await
    .unwrap();
    assert_eq!(insp.result, "fail");

    // NCR from the failed inspection.
    let ncr = ManufacturingService::create_ncr(
        &pool,
        1,
        &CreateNcrRequest {
            work_order_id: Some(wo.id),
            pipe_id: None,
            description: "水压试验不合格".into(),
            severity: Some("major".into()),
        },
        Some(1),
    )
    .await
    .unwrap();
    assert_eq!(ncr.status, "open");
    assert_eq!(ncr.severity, "major");

    // Resolve with disposition.
    let resolved = ManufacturingService::resolve_ncr(&pool, 1, ncr.id, &serde_json::from_value(serde_json::json!({"disposition": "rework"})).unwrap())
        .await
        .unwrap();
    assert_eq!(resolved.status, "resolved");
    assert_eq!(resolved.disposition.as_deref(), Some("rework"));

    // Resolving twice fails.
    let err = ManufacturingService::resolve_ncr(&pool, 1, ncr.id, &serde_json::from_value(serde_json::json!({"disposition": "scrap"})).unwrap()).await;
    assert!(err.is_err(), "resolved NCR must not resolve again");
}
