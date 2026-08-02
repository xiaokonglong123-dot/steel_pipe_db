//! Fixed asset services — registration, straight-line depreciation, disposal.

use sqlx::PgPool;

use crate::assets::repos::AssetRepo;
use crate::dto::assets_dto::{CreateAssetRequest, DepreciateRequest, UpdateAssetRequest};
use crate::error::AppError;
use crate::models::assets::{DepreciationEntry, FixedAsset};

pub struct AssetService;

impl AssetService {
    pub async fn create_asset(pool: &PgPool, tenant_id: i64, dto: &CreateAssetRequest) -> Result<FixedAsset, AppError> {
        if dto.name.trim().is_empty() {
            return Err(AppError::Validation("Asset name is required".into()));
        }
        if dto.useful_life_months.unwrap_or(60) <= 0 {
            return Err(AppError::Validation("Useful life must be positive".into()));
        }
        let asset_no = format!("AST-{}-{}", chrono::Utc::now().format("%Y%m%d"), seq(pool, "fixed_assets").await?);
        let asset = FixedAsset {
            id: 0,
            tenant_id,
            asset_no,
            name: dto.name.trim().to_string(),
            category: dto.category.clone().unwrap_or_else(|| "equipment".into()),
            purchase_date: dto.purchase_date,
            purchase_cost: dto.purchase_cost,
            salvage_value: dto.salvage_value.unwrap_or_default(),
            useful_life_months: dto.useful_life_months.unwrap_or(60),
            current_value: dto.purchase_cost,
            status: "active".into(),
            location: dto.location.clone(),
            department_id: dto.department_id,
            notes: dto.notes.clone(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            deleted_at: None,
        };
        AssetRepo::create(pool, &asset).await.map_err(AppError::from)
    }

    pub async fn list_assets(pool: &PgPool, tenant_id: i64, status: Option<&str>) -> Result<Vec<FixedAsset>, AppError> {
        AssetRepo::list(pool, tenant_id, status).await.map_err(AppError::from)
    }

    pub async fn get_asset(pool: &PgPool, tenant_id: i64, id: i64) -> Result<FixedAsset, AppError> {
        AssetRepo::find_by_id(pool, tenant_id, id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Asset not found: {}", id)))
    }

    pub async fn update_asset(pool: &PgPool, tenant_id: i64, id: i64, dto: &UpdateAssetRequest) -> Result<FixedAsset, AppError> {
        Self::get_asset(pool, tenant_id, id).await?;
        AssetRepo::update(pool, tenant_id, id, dto.name.as_deref(), dto.location.as_deref(), dto.department_id, dto.notes.as_deref())
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Asset not found: {}", id)))
    }

    /// Straight-line depreciation for a period: monthly = (cost - salvage) / life.
    /// Idempotent per (asset, period); updates current_value.
    pub async fn depreciate(pool: &PgPool, tenant_id: i64, id: i64, dto: &DepreciateRequest) -> Result<DepreciationEntry, AppError> {
        let asset = Self::get_asset(pool, tenant_id, id).await?;
        if asset.status != "active" {
            return Err(AppError::Validation(format!("Asset is not active (status: {})", asset.status)));
        }
        let monthly = (asset.purchase_cost - asset.salvage_value) / rust_decimal::Decimal::from(asset.useful_life_months);
        let entry = AssetRepo::insert_depreciation(pool, id, &dto.period, monthly)
            .await
            .map_err(AppError::from)?;

        // Recompute current value from accumulated depreciation.
        let total_dep = AssetRepo::depreciation_total(pool, id).await.map_err(AppError::from)?;
        let new_value = (asset.purchase_cost - total_dep).max(asset.salvage_value);
        AssetRepo::update_value_and_status(pool, tenant_id, id, new_value, "active")
            .await
            .map_err(AppError::from)?;
        Ok(entry)
    }

    /// Dispose an asset (sale/scrap) → status disposed.
    pub async fn dispose_asset(pool: &PgPool, tenant_id: i64, id: i64) -> Result<FixedAsset, AppError> {
        let asset = Self::get_asset(pool, tenant_id, id).await?;
        if asset.status == "disposed" {
            return Err(AppError::Validation("Asset already disposed".into()));
        }
        AssetRepo::update_value_and_status(pool, tenant_id, id, asset.salvage_value, "disposed")
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Asset not found: {}", id)))
    }
}

/// Per-table sequence helper for document numbers.
async fn seq(pool: &PgPool, table: &str) -> Result<i64, AppError> {
    let n: i64 = sqlx::query_scalar(&format!("SELECT COALESCE(MAX(id), 0) + 1 FROM {}", table))
        .fetch_one(pool)
        .await
        .map_err(AppError::from)?;
    Ok(n)
}
