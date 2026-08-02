//! Fixed asset repositories.

use sqlx::PgPool;
use crate::models::assets::{DepreciationEntry, FixedAsset};

pub struct AssetRepo;

impl AssetRepo {
    pub async fn create(pool: &PgPool, a: &FixedAsset) -> Result<FixedAsset, sqlx::Error> {
        sqlx::query_as::<_, FixedAsset>(
            "INSERT INTO fixed_assets \
             (tenant_id, asset_no, name, category, purchase_date, purchase_cost, salvage_value, \
              useful_life_months, current_value, location, department_id, notes) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12) \
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

    pub async fn list(pool: &PgPool, tenant_id: i64, status: Option<&str>) -> Result<Vec<FixedAsset>, sqlx::Error> {
        sqlx::query_as::<_, FixedAsset>(
            "SELECT id, tenant_id, asset_no, name, category, purchase_date, purchase_cost, \
                    salvage_value, useful_life_months, current_value, status, location, \
                    department_id, notes, created_at, updated_at, deleted_at \
             FROM fixed_assets WHERE tenant_id = $1 AND deleted_at IS NULL \
             AND ($2::text IS NULL OR status = $2) ORDER BY id DESC LIMIT 500",
        )
        .bind(tenant_id)
        .bind(status)
        .fetch_all(pool)
        .await
    }

    pub async fn find_by_id(pool: &PgPool, tenant_id: i64, id: i64) -> Result<Option<FixedAsset>, sqlx::Error> {
        sqlx::query_as::<_, FixedAsset>(
            "SELECT id, tenant_id, asset_no, name, category, purchase_date, purchase_cost, \
                    salvage_value, useful_life_months, current_value, status, location, \
                    department_id, notes, created_at, updated_at, deleted_at \
             FROM fixed_assets WHERE tenant_id = $1 AND id = $2 AND deleted_at IS NULL",
        )
        .bind(tenant_id)
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    pub async fn update(
        pool: &PgPool,
        tenant_id: i64,
        id: i64,
        name: Option<&str>,
        location: Option<&str>,
        department_id: Option<i64>,
        notes: Option<&str>,
    ) -> Result<Option<FixedAsset>, sqlx::Error> {
        sqlx::query_as::<_, FixedAsset>(
            "UPDATE fixed_assets SET name = COALESCE($3, name), \
                    location = COALESCE($4, location), \
                    department_id = COALESCE($5, department_id), \
                    notes = COALESCE($6, notes), updated_at = NOW() \
             WHERE tenant_id = $1 AND id = $2 AND deleted_at IS NULL \
             RETURNING id, tenant_id, asset_no, name, category, purchase_date, purchase_cost, \
                       salvage_value, useful_life_months, current_value, status, location, \
                       department_id, notes, created_at, updated_at, deleted_at",
        )
        .bind(tenant_id)
        .bind(id)
        .bind(name)
        .bind(location)
        .bind(department_id)
        .bind(notes)
        .fetch_optional(pool)
        .await
    }

    pub async fn update_value_and_status(
        pool: &PgPool,
        tenant_id: i64,
        id: i64,
        current_value: rust_decimal::Decimal,
        status: &str,
    ) -> Result<Option<FixedAsset>, sqlx::Error> {
        sqlx::query_as::<_, FixedAsset>(
            "UPDATE fixed_assets SET current_value = $3, status = $4, updated_at = NOW() \
             WHERE tenant_id = $1 AND id = $2 AND deleted_at IS NULL \
             RETURNING id, tenant_id, asset_no, name, category, purchase_date, purchase_cost, \
                       salvage_value, useful_life_months, current_value, status, location, \
                       department_id, notes, created_at, updated_at, deleted_at",
        )
        .bind(tenant_id)
        .bind(id)
        .bind(current_value)
        .bind(status)
        .fetch_optional(pool)
        .await
    }

    pub async fn insert_depreciation(
        pool: &PgPool,
        asset_id: i64,
        period: &str,
        amount: rust_decimal::Decimal,
    ) -> Result<DepreciationEntry, sqlx::Error> {
        sqlx::query_as::<_, DepreciationEntry>(
            "INSERT INTO depreciation_entries (asset_id, period, amount) \
             VALUES ($1, $2, $3) ON CONFLICT (asset_id, period) DO UPDATE SET amount = EXCLUDED.amount \
             RETURNING id, asset_id, period, amount, created_at",
        )
        .bind(asset_id)
        .bind(period)
        .bind(amount)
        .fetch_one(pool)
        .await
    }

    pub async fn depreciation_total(pool: &PgPool, asset_id: i64) -> Result<rust_decimal::Decimal, sqlx::Error> {
        let v: rust_decimal::Decimal = sqlx::query_scalar(
            "SELECT COALESCE(SUM(amount), 0) FROM depreciation_entries WHERE asset_id = $1",
        )
        .bind(asset_id)
        .fetch_one(pool)
        .await?;
        Ok(v)
    }
}
