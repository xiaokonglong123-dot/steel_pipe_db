use sqlx::{QueryBuilder, Sqlite, SqlitePool};

use crate::dto::common::PaginationParams;
use crate::dto::customer_dto::{CreateCustomerRequest, CustomerFilterParams, UpdateCustomerRequest};
use crate::models::customer::Customer;

/// CRUD for `customers`. All queries filter `deleted_at IS NULL`.
pub struct CustomerRepo;

impl CustomerRepo {
    /// INSERT a new customer with the given `code`. `is_active` defaults to `1`. Returns the created `Customer`.
    pub async fn create(
        pool: &SqlitePool,
        dto: &CreateCustomerRequest,
        code: &str,
    ) -> Result<Customer, sqlx::Error> {
        sqlx::query_as::<_, Customer>(
            "INSERT INTO customers (customer_code, name, contact_person, phone, email, address, \
             is_active, notes) \
             VALUES (?, ?, ?, ?, ?, ?, 1, ?) \
             RETURNING id, customer_code, name, contact_person, phone, email, address, \
               is_active, notes, created_at, updated_at, deleted_at",
        )
        .bind(code)
        .bind(&dto.name)
        .bind(&dto.contact_person)
        .bind(&dto.phone)
        .bind(&dto.email)
        .bind(&dto.address)
        .bind(&dto.notes)
        .fetch_one(pool)
        .await
    }

    /// Dynamic UPDATE of customer fields (name, contact_person, phone, email, is_active, etc.).
    /// Only supplied fields change. Returns the updated `Customer`.
    pub async fn update(
        pool: &SqlitePool,
        id: i64,
        dto: &UpdateCustomerRequest,
    ) -> Result<Customer, sqlx::Error> {
        let mut builder: QueryBuilder<Sqlite> =
            QueryBuilder::new("UPDATE customers SET updated_at = datetime('now')");

        if let Some(ref val) = dto.name {
            builder.push(", name = ");
            builder.push_bind(val);
        }
        if let Some(ref val) = dto.contact_person {
            builder.push(", contact_person = ");
            builder.push_bind(val);
        }
        if let Some(ref val) = dto.phone {
            builder.push(", phone = ");
            builder.push_bind(val);
        }
        if let Some(ref val) = dto.email {
            builder.push(", email = ");
            builder.push_bind(val);
        }
        if let Some(ref val) = dto.address {
            builder.push(", address = ");
            builder.push_bind(val);
        }
        if let Some(val) = dto.is_active {
            builder.push(", is_active = ");
            builder.push_bind(val);
        }
        if let Some(ref val) = dto.notes {
            builder.push(", notes = ");
            builder.push_bind(val);
        }

        builder.push(" WHERE id = ");
        builder.push_bind(id);
        builder.push(
            " AND deleted_at IS NULL RETURNING id, customer_code, name, contact_person, \
             phone, email, address, is_active, notes, created_at, updated_at, deleted_at",
        );

        builder.build_query_as::<Customer>().fetch_one(pool).await
    }

    /// SELECT by primary key. Returns `None` if soft-deleted or missing.
    pub async fn find_by_id(
        pool: &SqlitePool,
        id: i64,
    ) -> Result<Option<Customer>, sqlx::Error> {
        sqlx::query_as::<_, Customer>(
            "SELECT id, customer_code, name, contact_person, phone, email, address, \
             is_active, notes, created_at, updated_at, deleted_at \
             FROM customers WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    /// SELECT by unique `customer_code`. Returns `None` if soft-deleted or missing.
    pub async fn find_by_code(
        pool: &SqlitePool,
        code: &str,
    ) -> Result<Option<Customer>, sqlx::Error> {
        sqlx::query_as::<_, Customer>(
            "SELECT id, customer_code, name, contact_person, phone, email, address, \
             is_active, notes, created_at, updated_at, deleted_at \
             FROM customers WHERE customer_code = ? AND deleted_at IS NULL",
        )
        .bind(code)
        .fetch_optional(pool)
        .await
    }

    /// Soft-delete: sets `deleted_at` and `updated_at`.
    pub async fn delete(pool: &SqlitePool, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE customers SET deleted_at = datetime('now'), \
             updated_at = datetime('now') WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(id)
        .execute(pool)
        .await?;
        Ok(())
    }

    /// Paginated SELECT with dynamic filters (q, is_active).
    /// Supports sorting by customer_code, name, created_at. Returns `(items, total)`.
    pub async fn list(
        pool: &SqlitePool,
        filter: &CustomerFilterParams,
        params: &PaginationParams,
    ) -> Result<(Vec<Customer>, u64), sqlx::Error> {
        let page_size = params.page_size();
        let offset = params.offset();

        let mut conditions: Vec<String> = vec!["deleted_at IS NULL".into()];
        let mut bind_values: Vec<String> = Vec::new();

        if let Some(ref q) = filter.q {
            if !q.is_empty() {
                conditions.push("(name LIKE ? OR customer_code LIKE ? OR contact_person LIKE ?)".into());
                let pattern = format!("%{}%", q);
                bind_values.push(pattern.clone());
                bind_values.push(pattern.clone());
                bind_values.push(pattern);
            }
        }
        if let Some(val) = filter.is_active {
            conditions.push("is_active = ?".into());
            bind_values.push(if val { "1" } else { "0" }.into());
        }

        let where_clause = conditions.join(" AND ");

        let sort_by = match params.sort_by.as_deref() {
            Some("customer_code") => "customer_code",
            Some("name") => "name",
            Some("created_at") => "created_at",
            _ => "created_at",
        };
        let sort_order = params.sort_order_sql();

        let count_sql = format!("SELECT COUNT(*) as cnt FROM customers WHERE {}", where_clause);
        let mut count_q = sqlx::query_as::<_, (i64,)>(&count_sql);
        for val in &bind_values {
            count_q = count_q.bind(val.as_str());
        }
        let total: (i64,) = count_q.fetch_one(pool).await?;

        let list_sql = format!(
            "SELECT id, customer_code, name, contact_person, phone, email, address, \
             is_active, notes, created_at, updated_at, deleted_at \
             FROM customers WHERE {} ORDER BY {} {} LIMIT ? OFFSET ?",
            where_clause, sort_by, sort_order
        );
        let mut list_q = sqlx::query_as::<_, Customer>(&list_sql);
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

    /// Quick name/code search (LIKE) with LIMIT 50 results.
    pub async fn search(
        pool: &SqlitePool,
        query: &str,
    ) -> Result<Vec<Customer>, sqlx::Error> {
        let like = format!("%{}%", query);
        sqlx::query_as::<_, Customer>(
            "SELECT id, customer_code, name, contact_person, phone, email, address, \
             is_active, notes, created_at, updated_at, deleted_at \
             FROM customers \
             WHERE deleted_at IS NULL AND (name LIKE ? OR customer_code LIKE ?) \
             ORDER BY name ASC LIMIT 50",
        )
        .bind(&like)
        .bind(&like)
        .fetch_all(pool)
        .await
    }

    /// SELECT all active customers, ordered by `name ASC`. Used for dropdowns.
    pub async fn find_all_active(
        pool: &SqlitePool,
    ) -> Result<Vec<Customer>, sqlx::Error> {
        sqlx::query_as::<_, Customer>(
            "SELECT id, customer_code, name, contact_person, phone, email, address, \
             is_active, notes, created_at, updated_at, deleted_at \
             FROM customers WHERE deleted_at IS NULL AND is_active = 1 \
             ORDER BY name ASC",
        )
        .fetch_all(pool)
        .await
    }
}
