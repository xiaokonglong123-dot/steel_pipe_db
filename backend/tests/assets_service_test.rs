//! Fixed asset integration tests — registration, straight-line depreciation,
//! disposal.

mod common;

use erp_server::assets::services::AssetService;
use erp_server::dto::assets_dto::{CreateAssetRequest, DepreciateRequest, UpdateAssetRequest};

fn asset_dto(cost: f64, life: i32) -> CreateAssetRequest {
    CreateAssetRequest {
        name: "车床".into(),
        category: Some("equipment".into()),
        purchase_date: chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        purchase_cost: cost,
        salvage_value: Some(0.0),
        useful_life_months: Some(life),
        location: Some("一号车间".into()),
        department_id: None,
        notes: None,
    }
}

#[tokio::test]
async fn asset_creation_and_update() {
    let pool = common::test_pool().await;
    let asset = AssetService::create_asset(&pool, 1, &asset_dto(120000.0, 60)).await.unwrap();
    assert_eq!(asset.status, "active");
    assert_eq!(asset.current_value, 120000.0, "initial value = purchase cost");

    let updated = AssetService::update_asset(
        &pool, 1, asset.id,
        &UpdateAssetRequest { name: Some("数控车床".into()), location: Some("二号车间".into()), department_id: None, notes: None },
    )
    .await
    .unwrap();
    assert_eq!(updated.name, "数控车床");
}

#[tokio::test]
async fn straight_line_depreciation() {
    let pool = common::test_pool().await;
    // 120000 / 60 months = 2000/month.
    let asset = AssetService::create_asset(&pool, 1, &asset_dto(120000.0, 60)).await.unwrap();
    let entry = AssetService::depreciate(
        &pool, 1, asset.id,
        &DepreciateRequest { period: "2026-08".into() },
    )
    .await
    .unwrap();
    assert_eq!(entry.amount, 2000.0);

    // Current value reduced.
    let after = AssetService::get_asset(&pool, 1, asset.id).await.unwrap();
    assert_eq!(after.current_value, 118000.0);

    // Idempotent re-depreciate same period.
    let again = AssetService::depreciate(&pool, 1, asset.id, &DepreciateRequest { period: "2026-08".into() }).await.unwrap();
    assert_eq!(again.amount, 2000.0);
    let after2 = AssetService::get_asset(&pool, 1, asset.id).await.unwrap();
    assert_eq!(after2.current_value, 118000.0, "same period must not double-depreciate");
}

#[tokio::test]
async fn disposal_sets_status() {
    let pool = common::test_pool().await;
    let asset = AssetService::create_asset(&pool, 1, &asset_dto(50000.0, 24)).await.unwrap();
    let disposed = AssetService::dispose_asset(&pool, 1, asset.id).await.unwrap();
    assert_eq!(disposed.status, "disposed");
    assert_eq!(disposed.current_value, 0.0, "disposed value = salvage");

    // Depreciating a disposed asset fails.
    let err = AssetService::depreciate(&pool, 1, asset.id, &DepreciateRequest { period: "2026-09".into() }).await;
    assert!(err.is_err(), "disposed asset must not depreciate");
}
