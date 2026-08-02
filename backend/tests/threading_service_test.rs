//! Threading integration tests — records and API 5CT engineering math.

mod common;

use steel_pipe_db::dto::threading_dto::{CreateThreadingRecordRequest, DesignCheckRequest, ThreadCalcRequest};
use steel_pipe_db::threading::services::ThreadingService;

#[tokio::test]
async fn threading_record_crud() {
    let pool = common::test_pool().await;
    let rec = ThreadingService::create_record(
        &pool,
        1,
        &CreateThreadingRecordRequest {
            pipe_id: Some(1),
            pipe_number: Some("PN-TH-1".into()),
            thread_type: "API 5B round".into(),
            od: 244.5,
            wt: 11.05,
            grade: Some("J55".into()),
            threads_per_inch: Some(8.0),
            pitch_diameter: Some(242.0),
            makeup_torque: Some(2500.0),
            notes: None,
        },
        Some(1),
    )
    .await
    .unwrap();
    assert_eq!(rec.thread_type, "API 5B round");
    assert_eq!(rec.makeup_torque, Some(2500.0));

    let list = ThreadingService::list_records(&pool, 1, Some(1)).await.unwrap();
    assert_eq!(list.len(), 1);
}

#[tokio::test]
async fn invalid_geometry_rejected() {
    let pool = common::test_pool().await;
    // wt >= od/2 is physically impossible.
    let err = ThreadingService::create_record(
        &pool,
        1,
        &CreateThreadingRecordRequest {
            pipe_id: None,
            pipe_number: None,
            thread_type: "round".into(),
            od: 100.0,
            wt: 60.0,
            grade: None,
            threads_per_inch: None,
            pitch_diameter: None,
            makeup_torque: None,
            notes: None,
        },
        None,
    )
    .await;
    assert!(err.is_err(), "invalid geometry must be rejected");
}

#[tokio::test]
async fn calc_matches_barlow_and_caches() {
    let pool = common::test_pool().await;
    // 244.5mm OD, 11.05mm WT, N80 → burst ≈ 0.875 * 2 * 80000 * 11.05 / 244.5
    let result = ThreadingService::calc(
        &pool,
        1,
        &ThreadCalcRequest {
            od: 244.5,
            wt: 11.05,
            grade: "N80".into(),
            connection_type: "premium".into(),
        },
    )
    .await
    .unwrap();
    let expected_burst = 0.875 * 2.0 * 80_000.0 * 11.05 / 244.5;
    assert!((result.burst_pressure - expected_burst).abs() < 1.0, "burst must follow Barlow");
    assert_eq!(result.joint_efficiency, 1.0, "premium connection");
    assert!(!result.cached, "first call must compute");

    // Second call hits the cache.
    let cached = ThreadingService::calc(
        &pool,
        1,
        &ThreadCalcRequest {
            od: 244.5,
            wt: 11.05,
            grade: "N80".into(),
            connection_type: "premium".into(),
        },
    )
    .await
    .unwrap();
    assert!(cached.cached, "second call must hit the geometry cache");
    assert_eq!(cached.burst_pressure, result.burst_pressure);
}

#[tokio::test]
async fn design_check_safety_factors() {
    let pool = common::test_pool().await;
    let out = ThreadingService::design_check(
        &pool,
        1,
        &DesignCheckRequest {
            od: 244.5,
            wt: 11.05,
            grade: "N80".into(),
            connection_type: "premium".into(),
            depth: 500.0,
            fluid_density: 1025.0,
        },
    )
    .await
    .unwrap();
    assert!(out.external_pressure_psi > 0.0);
    assert!(out.burst_safety_factor > 1.0, "N80 at 500m must clear burst SF");
    assert!(out.collapse_safety_factor > 1.0);
    assert!(matches!(out.verdict.as_str(), "safe" | "unsafe"));
}

#[tokio::test]
async fn round_thread_lower_efficiency() {
    let pool = common::test_pool().await;
    let round = ThreadingService::calc(
        &pool,
        1,
        &ThreadCalcRequest { od: 177.8, wt: 9.19, grade: "P110".into(), connection_type: "round".into() },
    )
    .await
    .unwrap();
    let premium = ThreadingService::calc(
        &pool,
        1,
        &ThreadCalcRequest { od: 177.8, wt: 9.19, grade: "P110".into(), connection_type: "premium".into() },
    )
    .await
    .unwrap();
    assert!(round.joint_efficiency < premium.joint_efficiency, "round thread must be weaker than premium");
    assert!(round.tension_capacity < premium.tension_capacity);
}
