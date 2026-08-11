//! Location service — 仓库/库位业务规则
//!
//! 对齐 catalog_service：free functions，`pool: &SqlitePool` 首参。
//! 业务规则：code/name 非空校验、仓库存在校验、库位 code 仓库内唯一校验。

use sqlx::SqlitePool;

use crate::error::{AppError, ErrorCode};
use crate::repos::location_repo::{
    self, LocationFilter, LocationRow, WarehouseFilter, WarehouseRow,
};

// —— Warehouses ——

pub async fn create_warehouse(
    pool: &SqlitePool,
    code: &str,
    name: &str,
    address: Option<&str>,
) -> Result<WarehouseRow, AppError> {
    if code.trim().is_empty() || name.trim().is_empty() {
        return Err(AppError::new(
            ErrorCode::Validation,
            "仓库编码和名称不能为空",
        ));
    }
    if location_repo::find_warehouse_by_code(pool, code)
        .await?
        .is_some()
    {
        return Err(AppError::new(ErrorCode::Validation, "仓库编码已存在"));
    }
    location_repo::create_warehouse(pool, code, name, address).await
}

pub async fn get_warehouse(pool: &SqlitePool, id: i64) -> Result<WarehouseRow, AppError> {
    location_repo::find_warehouse_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::NotFound, "仓库未找到"))
}

pub async fn list_warehouses(
    pool: &SqlitePool,
    filter: &WarehouseFilter<'_>,
    page: i64,
    page_size: i64,
) -> Result<(Vec<WarehouseRow>, i64), AppError> {
    location_repo::list_warehouses(pool, filter, page, page_size).await
}

pub async fn update_warehouse(
    pool: &SqlitePool,
    id: i64,
    code: &str,
    name: &str,
    address: Option<&str>,
) -> Result<WarehouseRow, AppError> {
    if code.trim().is_empty() || name.trim().is_empty() {
        return Err(AppError::new(
            ErrorCode::Validation,
            "仓库编码和名称不能为空",
        ));
    }
    let current = location_repo::find_warehouse_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::NotFound, "仓库未找到"))?;
    if code != current.code {
        if let Some(existing) = location_repo::find_warehouse_by_code(pool, code).await? {
            if existing.id != id {
                return Err(AppError::new(ErrorCode::Validation, "仓库编码已存在"));
            }
        }
    }
    location_repo::update_warehouse(pool, id, code, name, address).await?;
    location_repo::find_warehouse_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::NotFound, "仓库未找到"))
}

pub async fn delete_warehouse(pool: &SqlitePool, id: i64) -> Result<(), AppError> {
    location_repo::soft_delete_warehouse(pool, id).await
}

// —— Locations ——

pub async fn create_location(
    pool: &SqlitePool,
    warehouse_id: Option<i64>,
    code: &str,
    name: &str,
) -> Result<LocationRow, AppError> {
    if code.trim().is_empty() || name.trim().is_empty() {
        return Err(AppError::new(
            ErrorCode::Validation,
            "库位编码和名称不能为空",
        ));
    }
    // 仓库存在性校验（warehouse_id 提供时）
    if let Some(wid) = warehouse_id {
        if location_repo::find_warehouse_by_id(pool, wid)
            .await?
            .is_none()
        {
            return Err(AppError::new(ErrorCode::NotFound, "所属仓库未找到"));
        }
    }
    if location_repo::find_location_by_code_within_warehouse(pool, warehouse_id, code)
        .await?
        .is_some()
    {
        return Err(AppError::new(
            ErrorCode::Validation,
            "库位编码在该仓库内已存在",
        ));
    }
    location_repo::create_location(pool, warehouse_id, code, name).await
}

pub async fn get_location(pool: &SqlitePool, id: i64) -> Result<LocationRow, AppError> {
    location_repo::find_location_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::LocationNotFound, "库位未找到"))
}

pub async fn list_locations(
    pool: &SqlitePool,
    filter: &LocationFilter<'_>,
    page: i64,
    page_size: i64,
) -> Result<(Vec<LocationRow>, i64), AppError> {
    location_repo::list_locations(pool, filter, page, page_size).await
}

pub async fn update_location(
    pool: &SqlitePool,
    id: i64,
    warehouse_id: Option<i64>,
    code: &str,
    name: &str,
) -> Result<LocationRow, AppError> {
    if code.trim().is_empty() || name.trim().is_empty() {
        return Err(AppError::new(
            ErrorCode::Validation,
            "库位编码和名称不能为空",
        ));
    }
    if let Some(wid) = warehouse_id {
        if location_repo::find_warehouse_by_id(pool, wid)
            .await?
            .is_none()
        {
            return Err(AppError::new(ErrorCode::NotFound, "所属仓库未找到"));
        }
    }
    let current = location_repo::find_location_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::LocationNotFound, "库位未找到"))?;
    if code != current.code || current.warehouse_id != warehouse_id {
        if let Some(existing) =
            location_repo::find_location_by_code_within_warehouse(pool, warehouse_id, code).await?
        {
            if existing.id != id {
                return Err(AppError::new(
                    ErrorCode::Validation,
                    "库位编码在该仓库内已存在",
                ));
            }
        }
    }
    location_repo::update_location(pool, id, warehouse_id, code, name).await?;
    location_repo::find_location_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::LocationNotFound, "库位未找到"))
}

pub async fn delete_location(pool: &SqlitePool, id: i64) -> Result<(), AppError> {
    location_repo::soft_delete_location(pool, id).await
}
