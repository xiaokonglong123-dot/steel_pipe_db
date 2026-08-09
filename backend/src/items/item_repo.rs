//! Item (商品) repository — CRUD for the `items` master table.
//!
//! All queries filter `deleted_at IS NULL` (soft delete).

use sqlx::{QueryBuilder, Sqlite, SqlitePool};

use crate::dto::common::PaginationParams;
use crate::dto::item_dto::{CreateItemRequest, ItemFilter, UpdateItemRequest};
use crate::models::item::Item;

pub struct ItemRepo;

impl ItemRepo {
    /// INSERT into `items`. Returns the newly created row with generated `id`.
    pub async fn create(
        pool: &SqlitePool,
        dto: &CreateItemRequest,
    ) -> Result<Item, sqlx::Error> {
        sqlx::query_as::<_, Item>(
            "INSERT INTO items (sku, name, category, unit, spec, price, status) \
             VALUES (?, ?, ?, ?, ?, ?, 'active') \
             RETURNING id, sku, name, category, unit, spec, price, status, \
               created_at, updated_at, deleted_at",
        )
        .bind(dto.sku.trim())
        .bind(dto.name.trim())
        .bind(&dto.category)
        .bind(&dto.unit)
        .bind(&dto.spec)
        .bind(dto.price)
        .fetch_one(pool)
        .await
    }

    /// SELECT by primary key. Returns `None` if not found or soft-deleted.
    pub async fn find_by_id(pool: &SqlitePool, id: i64) -> Result<Option<Item>, sqlx::Error> {
        sqlx::query_as::<_, Item>(
            "SELECT id, sku, name, category, unit, spec, price, status, \
             created_at, updated_at, deleted_at \
             FROM items WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    /// SELECT by unique SKU (exact). Returns `None` if not found or soft-deleted.
    pub async fn find_by_sku(pool: &SqlitePool, sku: &str) -> Result<Option<Item>, sqlx::Error> {
        sqlx::query_as::<_, Item>(
            "SELECT id, sku, name, category, unit, spec, price, status, \
             created_at, updated_at, deleted_at \
             FROM items WHERE sku = ? AND deleted_at IS NULL",
        )
        .bind(sku)
        .fetch_optional(pool)
        .await
    }

    /// Paginated SELECT with optional filters (`q`, `category`, `status`).
    /// Returns `(items, total)`.
    pub async fn list(
        pool: &SqlitePool,
        filter: &ItemFilter,
    ) -> Result<(Vec<Item>, u64), sqlx::Error> {
        let pagination = PaginationParams {
            page: filter.page,
            page_size: filter.page_size,
            sort_by: None,
            sort_order: None,
        };
        let page_size = pagination.page_size();
        let offset = pagination.offset();

        let mut conditions: Vec<String> = vec!["deleted_at IS NULL".into()];
        let mut bind_values: Vec<String> = Vec::new();

        if let Some(ref q) = filter.q {
            if !q.is_empty() {
                conditions.push(format!(
                    "(sku LIKE ? OR name LIKE ? OR category LIKE ? OR spec LIKE ?)"
                ));
                let pattern = format!("%{}%", q);
                bind_values.push(pattern.clone());
                bind_values.push(pattern.clone());
                bind_values.push(pattern.clone());
                bind_values.push(pattern);
            }
        }
        if let Some(ref category) = filter.category {
            conditions.push(format!("category = ?"));
            bind_values.push(category.clone());
        }
        if let Some(ref status) = filter.status {
            conditions.push(format!("status = ?"));
            bind_values.push(status.clone());
        }

        let where_clause = conditions.join(" AND ");

        let count_sql = format!("SELECT COUNT(*) as cnt FROM items WHERE {}", where_clause);
        let mut count_q = sqlx::query_as::<_, (i64,)>(&count_sql);
        for val in &bind_values {
            count_q = count_q.bind(val.as_str());
        }
        let total: (i64,) = count_q.fetch_one(pool).await?;

        let list_sql = format!(
            "SELECT id, sku, name, category, unit, spec, price, status, \
             created_at, updated_at, deleted_at FROM items WHERE {} \
             ORDER BY id DESC LIMIT ? OFFSET ?",
            where_clause
        );
        let mut list_q = sqlx::query_as::<_, Item>(&list_sql);
        for val in &bind_values {
            list_q = list_q.bind(val.as_str());
        }
        let items = list_q
            .bind(page_size as i64)
            .bind(offset as i64)
            .fetch_all(pool)
            .await?;

        Ok((items, total.0 as u64))
    }

    /// UPDATE editable fields on an item. Returns the updated row.
    pub async fn update(
        pool: &SqlitePool,
        id: i64,
        dto: &UpdateItemRequest,
    ) -> Result<Item, sqlx::Error> {
        let mut builder: QueryBuilder<Sqlite> =
            QueryBuilder::new("UPDATE items SET updated_at = datetime('now')");

        if let Some(ref val) = dto.name {
            builder.push(", name = ");
            builder.push_bind(val.trim());
        }
        if let Some(ref val) = dto.category {
            builder.push(", category = ");
            builder.push_bind(val);
        }
        if let Some(ref val) = dto.unit {
            builder.push(", unit = ");
            builder.push_bind(val);
        }
        if let Some(ref val) = dto.spec {
            builder.push(", spec = ");
            builder.push_bind(val);
        }
        if let Some(val) = dto.price {
            builder.push(", price = ");
            builder.push_bind(val);
        }
        if let Some(ref val) = dto.status {
            builder.push(", status = ");
            builder.push_bind(val);
        }

        builder.push(" WHERE id = ");
        builder.push_bind(id);
        builder.push(
            " AND deleted_at IS NULL RETURNING id, sku, name, category, unit, spec, price, \
             status, created_at, updated_at, deleted_at",
        );

        builder.build_query_as::<Item>().fetch_one(pool).await
    }

    /// Soft-delete by setting `deleted_at`. No-op if already deleted.
    pub async fn delete(pool: &SqlitePool, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE items SET deleted_at = datetime('now'), \
             updated_at = datetime('now') WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(id)
        .execute(pool)
        .await?;
        Ok(())
    }

    /// Checks whether an SKU is already used by a non-deleted item.
    /// Returns `true` when the SKU is taken.
    pub async fn sku_exists(pool: &SqlitePool, sku: &str) -> Result<bool, sqlx::Error> {
        let row: Option<(i64,)> = sqlx::query_as(
            "SELECT id FROM items WHERE sku = ? AND deleted_at IS NULL LIMIT 1",
        )
        .bind(sku)
        .fetch_optional(pool)
        .await?;
        Ok(row.is_some())
    }
}
