//! Fixed asset repositories.

use sqlx::SqlitePool;
use crate::models::assets::{DepreciationEntry, FixedAsset};

pub struct AssetRepo;

impl AssetRepo {
    pub async fn create(pool: &SqlitePool, a: &FixedAsset) -> Result<FixedAsset, sqlx::Error> {
        sqlx::query_as::<_, FixedAsset>(
            "INSERT INTO fixed_assets \
             (tenant_id, asset_no, name, category, purchase_date, purchase_cost, salvage_value, \
              useful_life_months, current_value, location, department_id, notes) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) \
             RETURNING id, tenant_id, asset_no, name, category, purchase_date, purchase_cost, \
                       salvage_value, useful_life_months, current_value, status, location, \
                       department_id, notes, created_at, updated_at, deleted_at",
        )
        .bind(a.tenant_id)
        .bind(&a.asset_no)
        .bind(&a.name)
        .bind(&a.category)
        .bind(a.purchase_date)
        .bind(a.purchase_cost)
        .bind(a.salvage_value)
        .bind(a.useful_life_months)
        .bind(a.current_value)
        .bind(&a.location)
        .bind(a.department_id)
        .bind(&a.notes)
        .fetch_one(pool)
        .await
    }

    pub async fn list(pool: &SqlitePool, tenant_id: i64, status: Option<&str>) -> Result<Vec<FixedAsset>, sqlx::Error> {
        sqlx::query_as::<_, FixedAsset>(
            "SELECT id, tenant_id, asset_no, name, category, purchase_date, purchase_cost, \
                    salvage_value, useful_life_months, current_value, status, location, \
                    department_id, notes, created_at, updated_at, deleted_at \
             FROM fixed_assets WHERE tenant_id = ? AND deleted_at IS NULL \
             AND (? IS NULL OR status = ?) ORDER BY id DESC LIMIT 500",
        )
        .bind(tenant_id)
        .bind(status)
        .bind(status)
        .fetch_all(pool)
        .await
    }

    pub async fn find_by_id(pool: &SqlitePool, tenant_id: i64, id: i64) -> Result<Option<FixedAsset>, sqlx::Error> {
        sqlx::query_as::<_, FixedAsset>(
            "SELECT id, tenant_id, asset_no, name, category, purchase_date, purchase_cost, \
                    salvage_value, useful_life_months, current_value, status, location, \
                    department_id, notes, created_at, updated_at, deleted_at \
             FROM fixed_assets WHERE tenant_id = ? AND id = ? AND deleted_at IS NULL",
        )
        .bind(tenant_id)
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    pub async fn update(
        pool: &SqlitePool,
        tenant_id: i64,
        id: i64,
        name: Option<&str>,
        location: Option<&str>,
        department_id: Option<i64>,
        notes: Option<&str>,
    ) -> Result<Option<FixedAsset>, sqlx::Error> {
        sqlx::query_as::<_, FixedAsset>(
            "UPDATE fixed_assets SET name = COALESCE(?, name), \
                    location = COALESCE(?, location), \
                    department_id = COALESCE(?, department_id), \
                    notes = COALESCE(?, notes), updated_at = datetime('now') \
             WHERE tenant_id = ? AND id = ? AND deleted_at IS NULL \
             RETURNING id, tenant_id, asset_no, name, category, purchase_date, purchase_cost, \
                       salvage_value, useful_life_months, current_value, status, location, \
                       department_id, notes, created_at, updated_at, deleted_at",
        )
        .bind(name)
        .bind(location)
        .bind(department_id)
        .bind(notes)
        .bind(tenant_id)
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    pub async fn update_value_and_status(
        pool: &SqlitePool,
        tenant_id: i64,
        id: i64,
        current_value: f64,
        status: &str,
    ) -> Result<Option<FixedAsset>, sqlx::Error> {
        sqlx::query_as::<_, FixedAsset>(
            "UPDATE fixed_assets SET current_value = ?, status = ?, updated_at = datetime('now') \
             WHERE tenant_id = ? AND id = ? AND deleted_at IS NULL \
             RETURNING id, tenant_id, asset_no, name, category, purchase_date, purchase_cost, \
                       salvage_value, useful_life_months, current_value, status, location, \
                       department_id, notes, created_at, updated_at, deleted_at",
        )
        .bind(current_value)
        .bind(status)
        .bind(tenant_id)
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    pub async fn insert_depreciation(
        pool: &SqlitePool,
        asset_id: i64,
        period: &str,
        amount: f64,
    ) -> Result<DepreciationEntry, sqlx::Error> {
        sqlx::query_as::<_, DepreciationEntry>(
            "INSERT INTO depreciation_entries (asset_id, period, amount) \
             VALUES (?, ?, ?) ON CONFLICT (asset_id, period) DO UPDATE SET amount = EXCLUDED.amount \
             RETURNING id, asset_id, period, amount, created_at",
        )
        .bind(asset_id)
        .bind(period)
        .bind(amount)
        .fetch_one(pool)
        .await
    }

    pub async fn depreciation_total(pool: &SqlitePool, asset_id: i64) -> Result<f64, sqlx::Error> {
        let v: f64 = sqlx::query_scalar(
            "SELECT CAST(COALESCE(SUM(amount), 0.0) AS REAL) FROM depreciation_entries WHERE asset_id = ?",
        )
        .bind(asset_id)
        .fetch_one(pool)
        .await?;
        Ok(v)
    }
}
