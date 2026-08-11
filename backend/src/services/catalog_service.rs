//! Catalog service — 商品主数据业务规则
//!
//! 对齐 auth_service：free functions，`pool: &SqlitePool` 首参。
//! 业务规则：SKU 唯一校验、字段非空校验、状态机校验（draft/active/disabled）。

use sqlx::SqlitePool;

use crate::error::{AppError, ErrorCode};
use crate::repos::catalog_repo::{self, ItemFilter, ItemRow};

pub async fn create_item(
    pool: &SqlitePool,
    sku: &str,
    name: &str,
    category: Option<&str>,
    unit: Option<&str>,
    spec: Option<&str>,
) -> Result<ItemRow, AppError> {
    if sku.trim().is_empty() || name.trim().is_empty() {
        return Err(AppError::new(ErrorCode::Validation, "SKU 和名称不能为空"));
    }
    if catalog_repo::find_by_sku(pool, sku).await?.is_some() {
        return Err(AppError::new(ErrorCode::ItemDuplicateSku, "SKU 已存在"));
    }
    catalog_repo::create_item(pool, sku, name, category, unit, spec).await
}

pub async fn get_item(pool: &SqlitePool, id: i64) -> Result<ItemRow, AppError> {
    catalog_repo::find_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::ItemNotFound, "商品未找到"))
}

pub async fn list_items(
    pool: &SqlitePool,
    filter: &ItemFilter<'_>,
    page: i64,
    page_size: i64,
) -> Result<(Vec<ItemRow>, i64), AppError> {
    catalog_repo::list_items(pool, filter, page, page_size).await
}

pub async fn update_item(
    pool: &SqlitePool,
    id: i64,
    sku: &str,
    name: &str,
    category: Option<&str>,
    unit: Option<&str>,
    spec: Option<&str>,
    status: &str,
) -> Result<ItemRow, AppError> {
    if sku.trim().is_empty() || name.trim().is_empty() {
        return Err(AppError::new(ErrorCode::Validation, "SKU 和名称不能为空"));
    }
    if !matches!(status, "draft" | "active" | "disabled") {
        return Err(AppError::new(
            ErrorCode::Validation,
            "status 取值必须为 draft/active/disabled",
        ));
    }
    let current = catalog_repo::find_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::ItemNotFound, "商品未找到"))?;
    if sku != current.sku {
        if let Some(existing) = catalog_repo::find_by_sku(pool, sku).await? {
            if existing.id != id {
                return Err(AppError::new(ErrorCode::ItemDuplicateSku, "SKU 已存在"));
            }
        }
    }
    catalog_repo::update_item(pool, id, sku, name, category, unit, spec, status).await?;
    catalog_repo::find_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::ItemNotFound, "商品未找到"))
}

pub async fn delete_item(pool: &SqlitePool, id: i64) -> Result<(), AppError> {
    catalog_repo::soft_delete_item(pool, id).await
}

pub async fn list_categories(pool: &SqlitePool) -> Result<Vec<String>, AppError> {
    catalog_repo::list_categories(pool).await
}
