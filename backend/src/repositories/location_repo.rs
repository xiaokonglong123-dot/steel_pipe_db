use sqlx::{QueryBuilder, Sqlite, PgPool};

use crate::dto::common::PaginationParams;
use crate::dto::inventory_dto::{CreateLocationRequest, UpdateLocationRequest};
use crate::models::inventory::Location;

/// CRUD for `locations` table (warehouse bin locations). All queries filter `deleted_at IS NULL`.
pub struct LocationRepo;

impl LocationRepo {
    pub async fn refresh_used_count(
        pool: &PgPool,
        location_id: i64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE locations SET used_count = (
                 SELECT COUNT(*) FROM seamless_pipes
                 WHERE location_id = $1 AND status = 'in_stock' AND deleted_at IS NULL
             ) + (
                 SELECT COUNT(*) FROM screen_pipes
                 WHERE location_id = $2 AND status = 'in_stock' AND deleted_at IS NULL
             ) + (
                 SELECT COUNT(*) FROM welded_pipes
                 WHERE location_id = $3 AND status = 'in_stock' AND deleted_at IS NULL
             ), updated_at = NOW()
             WHERE id = $4 AND deleted_at IS NULL",
        )
        .bind(location_id)
        .bind(location_id)
        .bind(location_id)
        .bind(location_id)
        .execute(pool)
        .await?;
        Ok(())
    }

    /// INSERT into `locations`. Returns the newly created row with generated `id`.
    pub async fn create(
        pool: &PgPool,
        dto: &CreateLocationRequest,
        full_code: &str,
    ) -> Result<Location, sqlx::Error> {
        sqlx::query_as::<_, Location>(
            "INSERT INTO locations (zone_code, shelf_code, level_code, full_code, description, capacity) \
             VALUES ($1, $2, $3, $4, $5, $6) \
             RETURNING id, zone_code, shelf_code, level_code, full_code, description, capacity, \
               used_count, is_active, created_at, updated_at, deleted_at",
        )
        .bind(&dto.zone_code)
        .bind(&dto.shelf_code)
        .bind(&dto.level_code)
        .bind(full_code)
        .bind(&dto.description)
        .bind(dto.capacity)
        .fetch_one(pool)
        .await
    }

    /// UPDATE `locations` by id. Supports optional `description`, `capacity`, `is_active` fields.
    /// Returns the updated row.
    pub async fn update(
        pool: &PgPool,
        id: i64,
        dto: &UpdateLocationRequest,
    ) -> Result<Location, sqlx::Error> {
        let mut builder: QueryBuilder<Sqlite> =
            QueryBuilder::new("UPDATE locations SET updated_at = NOW()");

        if let Some(ref val) = dto.description {
            builder.push(", description = ");
            builder.push_bind(val);
        }
        if let Some(val) = dto.capacity {
            builder.push(", capacity = ");
            builder.push_bind(val);
        }
        if let Some(val) = dto.is_active {
            builder.push(", is_active = ");
            builder.push_bind(val);
        }

        builder.push(" WHERE id = ");
        builder.push_bind(id);
        builder.push(
            " AND deleted_at IS NULL RETURNING id, zone_code, shelf_code, level_code, \
             full_code, description, capacity, used_count, is_active, created_at, \
             updated_at, deleted_at",
        );

        builder.build_query_as::<Location>().fetch_one(pool).await
    }

    /// SELECT by primary key from `locations`. Returns `None` if not found or soft-deleted.
    pub async fn find_by_id(pool: &PgPool, id: i64) -> Result<Option<Location>, sqlx::Error> {
        sqlx::query_as::<_, Location>(
            "SELECT id, zone_code, shelf_code, level_code, full_code, description, capacity, \
             used_count, is_active, created_at, updated_at, deleted_at \
             FROM locations WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    /// SELECT by unique `full_code` (e.g. `A-01-01`). Returns `None` if not found or soft-deleted.
    pub async fn find_by_full_code(
        pool: &PgPool,
        code: &str,
    ) -> Result<Option<Location>, sqlx::Error> {
        sqlx::query_as::<_, Location>(
            "SELECT id, zone_code, shelf_code, level_code, full_code, description, capacity, \
             used_count, is_active, created_at, updated_at, deleted_at \
             FROM locations WHERE full_code = $1 AND deleted_at IS NULL",
        )
        .bind(code)
        .fetch_optional(pool)
        .await
    }

    /// Soft-delete by setting `deleted_at` timestamp. No-op if already deleted.
    pub async fn delete(pool: &PgPool, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE locations SET deleted_at = NOW(), \
             updated_at = NOW() WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(id)
        .execute(pool)
        .await?;
        Ok(())
    }

    /// Paginated SELECT from `locations`. Optionally filters to only active locations.
    /// Returns `(items, total)`.
    pub async fn list(
        pool: &PgPool,
        params: &PaginationParams,
        active_only: bool,
    ) -> Result<(Vec<Location>, u64), sqlx::Error> {
        let page_size = params.page_size();
        let offset = params.offset();

        let mut conditions: Vec<String> = vec!["deleted_at IS NULL".into()];
        if active_only {
            conditions.push("is_active = TRUE".into());
        }
        let where_clause = conditions.join(" AND ");

        let count_sql = format!(
            "SELECT COUNT(*) as cnt FROM locations WHERE {}",
            where_clause
        );
        let total: (i64,) = sqlx::query_as(&count_sql).fetch_one(pool).await?;

        let list_sql = format!(
            "SELECT id, zone_code, shelf_code, level_code, full_code, description, capacity, \
             used_count, is_active, created_at, updated_at, deleted_at \
             FROM locations WHERE {} ORDER BY zone_code ASC, shelf_code ASC, level_code ASC \
             LIMIT $1 OFFSET $2",
            where_clause
        );

        let items = sqlx::query_as::<_, Location>(&list_sql)
            .bind(page_size as i64)
            .bind(offset as i64)
            .fetch_all(pool)
            .await?;

        Ok((items, total.0 as u64))
    }
}
