//! Warehouse & Location 数据访问 — warehouses / locations 表
//!
//! 纯 SQL（sqlx），无业务逻辑、无事务控制（事务在 service 层）。
//! locations 表前身在 004_inventory.sql 创建，010_warehouses.sql 补充 warehouse_id / deleted_at。

use sqlx::SqlitePool;

use crate::error::{AppError, ErrorCode};

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct WarehouseRow {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub address: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct LocationRow {
    pub id: i64,
    pub warehouse_id: Option<i64>,
    pub code: String,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}

/// 仓库列表筛选参数（与 WarehouseFilterQuery 对齐）
#[derive(Debug, Clone, Default)]
pub struct WarehouseFilter<'a> {
    pub code: Option<&'a str>,
    pub name: Option<&'a str>,
}

/// 库位列表筛选参数（与 LocationFilterQuery 对齐）
#[derive(Debug, Clone, Default)]
pub struct LocationFilter<'a> {
    pub warehouse_id: Option<i64>,
    pub code: Option<&'a str>,
    pub name: Option<&'a str>,
}

// —— Warehouses ——

pub async fn find_warehouse_by_id(
    pool: &SqlitePool,
    id: i64,
) -> Result<Option<WarehouseRow>, AppError> {
    let row = sqlx::query_as::<_, WarehouseRow>(
        "SELECT id, code, name, address, created_at, updated_at, deleted_at
         FROM warehouses WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn find_warehouse_by_code(
    pool: &SqlitePool,
    code: &str,
) -> Result<Option<WarehouseRow>, AppError> {
    let row = sqlx::query_as::<_, WarehouseRow>(
        "SELECT id, code, name, address, created_at, updated_at, deleted_at
         FROM warehouses WHERE code = ? AND deleted_at IS NULL",
    )
    .bind(code)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn create_warehouse(
    pool: &SqlitePool,
    code: &str,
    name: &str,
    address: Option<&str>,
) -> Result<WarehouseRow, AppError> {
    let result = sqlx::query("INSERT INTO warehouses (code, name, address) VALUES (?, ?, ?)")
        .bind(code)
        .bind(name)
        .bind(address)
        .execute(pool)
        .await?;
    let id = result.last_insert_rowid();
    find_warehouse_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::Internal, "仓库创建后读取失败"))
}

pub async fn list_warehouses(
    pool: &SqlitePool,
    filter: &WarehouseFilter<'_>,
    page: i64,
    page_size: i64,
) -> Result<(Vec<WarehouseRow>, i64), AppError> {
    let mut where_clauses: Vec<&'static str> = vec!["deleted_at IS NULL"];
    let mut count_sql = String::from("SELECT COUNT(*) FROM warehouses WHERE deleted_at IS NULL");
    let mut list_sql = String::from(
        "SELECT id, code, name, address, created_at, updated_at, deleted_at
         FROM warehouses WHERE deleted_at IS NULL",
    );

    if filter.code.is_some() {
        where_clauses.push("code = ?");
    }
    if filter.name.is_some() {
        where_clauses.push("name LIKE ?");
    }

    if where_clauses.len() > 1 {
        let extra = where_clauses[1..].join(" AND ");
        count_sql.push_str(" AND ");
        count_sql.push_str(&extra);
        list_sql.push_str(" AND ");
        list_sql.push_str(&extra);
    }

    list_sql.push_str(" ORDER BY id DESC LIMIT ? OFFSET ?");

    let mut count_q = sqlx::query_scalar::<_, i64>(&count_sql);
    if let Some(v) = filter.code {
        count_q = count_q.bind(v);
    }
    if let Some(v) = filter.name {
        let like = format!("%{v}%");
        count_q = count_q.bind(like);
    }
    let total: i64 = count_q.fetch_one(pool).await?;

    let offset = (page - 1).max(0) * page_size;
    let mut list_q = sqlx::query_as::<_, WarehouseRow>(&list_sql);
    if let Some(v) = filter.code {
        list_q = list_q.bind(v);
    }
    if let Some(v) = filter.name {
        let like = format!("%{v}%");
        list_q = list_q.bind(like);
    }
    list_q = list_q.bind(page_size).bind(offset);
    let rows = list_q.fetch_all(pool).await?;
    Ok((rows, total))
}

pub async fn update_warehouse(
    pool: &SqlitePool,
    id: i64,
    code: &str,
    name: &str,
    address: Option<&str>,
) -> Result<(), AppError> {
    let result = sqlx::query(
        "UPDATE warehouses SET code = ?, name = ?, address = ?,
         updated_at = datetime('now')
         WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(code)
    .bind(name)
    .bind(address)
    .bind(id)
    .execute(pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::new(ErrorCode::NotFound, "仓库未找到"));
    }
    Ok(())
}

pub async fn soft_delete_warehouse(pool: &SqlitePool, id: i64) -> Result<(), AppError> {
    let result = sqlx::query(
        "UPDATE warehouses SET deleted_at = datetime('now')
         WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(id)
    .execute(pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::new(ErrorCode::NotFound, "仓库未找到"));
    }
    Ok(())
}

// —— Locations ——

pub async fn find_location_by_id(
    pool: &SqlitePool,
    id: i64,
) -> Result<Option<LocationRow>, AppError> {
    let row = sqlx::query_as::<_, LocationRow>(
        "SELECT id, warehouse_id, code, name, created_at, updated_at, deleted_at
         FROM locations WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

/// 库位 code 唯一性检查 — 在指定仓库范围内（warehouse_id 可空则视为全局未关联）
pub async fn find_location_by_code_within_warehouse(
    pool: &SqlitePool,
    warehouse_id: Option<i64>,
    code: &str,
) -> Result<Option<LocationRow>, AppError> {
    let row = sqlx::query_as::<_, LocationRow>(
        "SELECT id, warehouse_id, code, name, created_at, updated_at, deleted_at
         FROM locations
         WHERE code = ? AND deleted_at IS NULL
           AND (warehouse_id IS ? OR warehouse_id = ?)",
    )
    .bind(code)
    .bind(warehouse_id)
    .bind(warehouse_id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn create_location(
    pool: &SqlitePool,
    warehouse_id: Option<i64>,
    code: &str,
    name: &str,
) -> Result<LocationRow, AppError> {
    let result = sqlx::query("INSERT INTO locations (warehouse_id, code, name) VALUES (?, ?, ?)")
        .bind(warehouse_id)
        .bind(code)
        .bind(name)
        .execute(pool)
        .await?;
    let id = result.last_insert_rowid();
    find_location_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::Internal, "库位创建后读取失败"))
}

pub async fn list_locations(
    pool: &SqlitePool,
    filter: &LocationFilter<'_>,
    page: i64,
    page_size: i64,
) -> Result<(Vec<LocationRow>, i64), AppError> {
    let mut where_clauses: Vec<&'static str> = vec!["deleted_at IS NULL"];
    let mut count_sql = String::from("SELECT COUNT(*) FROM locations WHERE deleted_at IS NULL");
    let mut list_sql = String::from(
        "SELECT id, warehouse_id, code, name, created_at, updated_at, deleted_at
         FROM locations WHERE deleted_at IS NULL",
    );

    if filter.warehouse_id.is_some() {
        where_clauses.push("warehouse_id = ?");
    }
    if filter.code.is_some() {
        where_clauses.push("code = ?");
    }
    if filter.name.is_some() {
        where_clauses.push("name LIKE ?");
    }

    if where_clauses.len() > 1 {
        let extra = where_clauses[1..].join(" AND ");
        count_sql.push_str(" AND ");
        count_sql.push_str(&extra);
        list_sql.push_str(" AND ");
        list_sql.push_str(&extra);
    }

    list_sql.push_str(" ORDER BY id DESC LIMIT ? OFFSET ?");

    let mut count_q = sqlx::query_scalar::<_, i64>(&count_sql);
    if let Some(v) = filter.warehouse_id {
        count_q = count_q.bind(v);
    }
    if let Some(v) = filter.code {
        count_q = count_q.bind(v);
    }
    if let Some(v) = filter.name {
        let like = format!("%{v}%");
        count_q = count_q.bind(like);
    }
    let total: i64 = count_q.fetch_one(pool).await?;

    let offset = (page - 1).max(0) * page_size;
    let mut list_q = sqlx::query_as::<_, LocationRow>(&list_sql);
    if let Some(v) = filter.warehouse_id {
        list_q = list_q.bind(v);
    }
    if let Some(v) = filter.code {
        list_q = list_q.bind(v);
    }
    if let Some(v) = filter.name {
        let like = format!("%{v}%");
        list_q = list_q.bind(like);
    }
    list_q = list_q.bind(page_size).bind(offset);
    let rows = list_q.fetch_all(pool).await?;
    Ok((rows, total))
}

pub async fn update_location(
    pool: &SqlitePool,
    id: i64,
    warehouse_id: Option<i64>,
    code: &str,
    name: &str,
) -> Result<(), AppError> {
    let result = sqlx::query(
        "UPDATE locations SET warehouse_id = ?, code = ?, name = ?,
         updated_at = datetime('now')
         WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(warehouse_id)
    .bind(code)
    .bind(name)
    .bind(id)
    .execute(pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::new(ErrorCode::LocationNotFound, "库位未找到"));
    }
    Ok(())
}

pub async fn soft_delete_location(pool: &SqlitePool, id: i64) -> Result<(), AppError> {
    let result = sqlx::query(
        "UPDATE locations SET deleted_at = datetime('now')
         WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(id)
    .execute(pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::new(ErrorCode::LocationNotFound, "库位未找到"));
    }
    Ok(())
}
