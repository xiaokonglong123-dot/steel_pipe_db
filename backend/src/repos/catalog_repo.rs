//! Catalog 数据访问 — items 表（商品主数据）
//!
//! 纯 SQL（sqlx），无业务逻辑、无事务控制（事务在 service 层）。

use sqlx::SqlitePool;

use crate::error::{AppError, ErrorCode};

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct ItemRow {
    pub id: i64,
    pub sku: String,
    pub name: String,
    pub category: Option<String>,
    pub unit: Option<String>,
    pub spec: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}

/// 列表筛选参数（与 ItemFilter query 对齐）
#[derive(Debug, Clone, Default)]
pub struct ItemFilter<'a> {
    pub sku: Option<&'a str>,
    pub name: Option<&'a str>,
    pub category: Option<&'a str>,
    pub status: Option<&'a str>,
}

pub async fn find_by_id(pool: &SqlitePool, id: i64) -> Result<Option<ItemRow>, AppError> {
    let row = sqlx::query_as::<_, ItemRow>(
        "SELECT id, sku, name, category, unit, spec, status, created_at, updated_at, deleted_at
         FROM items WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn find_by_sku(pool: &SqlitePool, sku: &str) -> Result<Option<ItemRow>, AppError> {
    let row = sqlx::query_as::<_, ItemRow>(
        "SELECT id, sku, name, category, unit, spec, status, created_at, updated_at, deleted_at
         FROM items WHERE sku = ? AND deleted_at IS NULL",
    )
    .bind(sku)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn create_item(
    pool: &SqlitePool,
    sku: &str,
    name: &str,
    category: Option<&str>,
    unit: Option<&str>,
    spec: Option<&str>,
) -> Result<ItemRow, AppError> {
    let result = sqlx::query(
        "INSERT INTO items (sku, name, category, unit, spec, status)
         VALUES (?, ?, ?, ?, ?, 'draft')",
    )
    .bind(sku)
    .bind(name)
    .bind(category)
    .bind(unit)
    .bind(spec)
    .execute(pool)
    .await?;
    let id = result.last_insert_rowid();
    find_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::Internal, "商品创建后读取失败"))
}

/// 动态筛选列表 — 按条件拼 WHERE，bind 顺序与占位符顺序一致
pub async fn list_items(
    pool: &SqlitePool,
    filter: &ItemFilter<'_>,
    page: i64,
    page_size: i64,
) -> Result<(Vec<ItemRow>, i64), AppError> {
    let mut where_clauses: Vec<&'static str> = vec!["deleted_at IS NULL"];
    let mut count_sql = String::from("SELECT COUNT(*) FROM items WHERE deleted_at IS NULL");
    let mut list_sql = String::from(
        "SELECT id, sku, name, category, unit, spec, status, created_at, updated_at, deleted_at
         FROM items WHERE deleted_at IS NULL",
    );

    if filter.sku.is_some() {
        where_clauses.push("sku = ?");
    }
    if filter.name.is_some() {
        where_clauses.push("name LIKE ?");
    }
    if filter.category.is_some() {
        where_clauses.push("category = ?");
    }
    if filter.status.is_some() {
        where_clauses.push("status = ?");
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
    if let Some(v) = filter.sku {
        count_q = count_q.bind(v);
    }
    if let Some(v) = filter.name {
        let like = format!("%{v}%");
        count_q = count_q.bind(like);
    }
    if let Some(v) = filter.category {
        count_q = count_q.bind(v);
    }
    if let Some(v) = filter.status {
        count_q = count_q.bind(v);
    }
    let total: i64 = count_q.fetch_one(pool).await?;

    let offset = (page - 1).max(0) * page_size;
    let mut list_q = sqlx::query_as::<_, ItemRow>(&list_sql);
    if let Some(v) = filter.sku {
        list_q = list_q.bind(v);
    }
    if let Some(v) = filter.name {
        let like = format!("%{v}%");
        list_q = list_q.bind(like);
    }
    if let Some(v) = filter.category {
        list_q = list_q.bind(v);
    }
    if let Some(v) = filter.status {
        list_q = list_q.bind(v);
    }
    list_q = list_q.bind(page_size).bind(offset);
    let rows = list_q.fetch_all(pool).await?;
    Ok((rows, total))
}

/// 更新商品（全字段更新；sku/name/category/unit/spec/status）
pub async fn update_item(
    pool: &SqlitePool,
    id: i64,
    sku: &str,
    name: &str,
    category: Option<&str>,
    unit: Option<&str>,
    spec: Option<&str>,
    status: &str,
) -> Result<(), AppError> {
    let result = sqlx::query(
        "UPDATE items SET sku = ?, name = ?, category = ?, unit = ?, spec = ?, status = ?,
         updated_at = datetime('now')
         WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(sku)
    .bind(name)
    .bind(category)
    .bind(unit)
    .bind(spec)
    .bind(status)
    .bind(id)
    .execute(pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::new(ErrorCode::ItemNotFound, "商品未找到"));
    }
    Ok(())
}

/// 软删除商品
pub async fn soft_delete_item(pool: &SqlitePool, id: i64) -> Result<(), AppError> {
    let result = sqlx::query(
        "UPDATE items SET deleted_at = datetime('now')
         WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(id)
    .execute(pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::new(ErrorCode::ItemNotFound, "商品未找到"));
    }
    Ok(())
}

/// 去重类别列表
pub async fn list_categories(pool: &SqlitePool) -> Result<Vec<String>, AppError> {
    let rows: Vec<String> = sqlx::query_scalar(
        "SELECT DISTINCT category FROM items
         WHERE category IS NOT NULL AND deleted_at IS NULL
         ORDER BY category",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}
