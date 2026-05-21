use sqlx::{QueryBuilder, Sqlite, SqlitePool};

use crate::dto::common::PaginationParams;
use crate::dto::supplier_dto::{CreateSupplierRequest, SupplierFilterParams, UpdateSupplierRequest};
use crate::models::supplier::Supplier;

pub struct SupplierRepo;

impl SupplierRepo {
    pub async fn create(
        pool: &SqlitePool,
        dto: &CreateSupplierRequest,
        code: &str,
    ) -> Result<Supplier, sqlx::Error> {
        sqlx::query_as::<_, Supplier>(
            "INSERT INTO suppliers (supplier_code, name, contact_person, phone, email, address, \
             is_active, notes) \
             VALUES (?, ?, ?, ?, ?, ?, 1, ?) \
             RETURNING id, supplier_code, name, contact_person, phone, email, address, \
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

    pub async fn update(
        pool: &SqlitePool,
        id: i64,
        dto: &UpdateSupplierRequest,
    ) -> Result<Supplier, sqlx::Error> {
        let mut builder: QueryBuilder<Sqlite> =
            QueryBuilder::new("UPDATE suppliers SET updated_at = datetime('now')");

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
            " AND deleted_at IS NULL RETURNING id, supplier_code, name, contact_person, \
             phone, email, address, is_active, notes, created_at, updated_at, deleted_at",
        );

        builder.build_query_as::<Supplier>().fetch_one(pool).await
    }

    pub async fn find_by_id(
        pool: &SqlitePool,
        id: i64,
    ) -> Result<Option<Supplier>, sqlx::Error> {
        sqlx::query_as::<_, Supplier>(
            "SELECT id, supplier_code, name, contact_person, phone, email, address, \
             is_active, notes, created_at, updated_at, deleted_at \
             FROM suppliers WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    pub async fn find_by_code(
        pool: &SqlitePool,
        code: &str,
    ) -> Result<Option<Supplier>, sqlx::Error> {
        sqlx::query_as::<_, Supplier>(
            "SELECT id, supplier_code, name, contact_person, phone, email, address, \
             is_active, notes, created_at, updated_at, deleted_at \
             FROM suppliers WHERE supplier_code = ? AND deleted_at IS NULL",
        )
        .bind(code)
        .fetch_optional(pool)
        .await
    }

    pub async fn delete(pool: &SqlitePool, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE suppliers SET deleted_at = datetime('now'), \
             updated_at = datetime('now') WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(id)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn list(
        pool: &SqlitePool,
        filter: &SupplierFilterParams,
        params: &PaginationParams,
    ) -> Result<(Vec<Supplier>, u64), sqlx::Error> {
        let page_size = params.page_size();
        let offset = params.offset();

        let mut conditions: Vec<String> = vec!["deleted_at IS NULL".into()];
        let mut bind_values: Vec<String> = Vec::new();

        if let Some(ref q) = filter.q {
            if !q.is_empty() {
                conditions.push("(name LIKE ? OR supplier_code LIKE ? OR contact_person LIKE ?)".into());
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
            Some("supplier_code") => "supplier_code",
            Some("name") => "name",
            Some("created_at") => "created_at",
            _ => "created_at",
        };
        let sort_order = params.sort_order_sql();

        let count_sql = format!("SELECT COUNT(*) as cnt FROM suppliers WHERE {}", where_clause);
        let mut count_q = sqlx::query_as::<_, (i64,)>(&count_sql);
        for val in &bind_values {
            count_q = count_q.bind(val.as_str());
        }
        let total: (i64,) = count_q.fetch_one(pool).await?;

        let list_sql = format!(
            "SELECT id, supplier_code, name, contact_person, phone, email, address, \
             is_active, notes, created_at, updated_at, deleted_at \
             FROM suppliers WHERE {} ORDER BY {} {} LIMIT ? OFFSET ?",
            where_clause, sort_by, sort_order
        );
        let mut list_q = sqlx::query_as::<_, Supplier>(&list_sql);
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

    pub async fn search(
        pool: &SqlitePool,
        query: &str,
    ) -> Result<Vec<Supplier>, sqlx::Error> {
        let like = format!("%{}%", query);
        sqlx::query_as::<_, Supplier>(
            "SELECT id, supplier_code, name, contact_person, phone, email, address, \
             is_active, notes, created_at, updated_at, deleted_at \
             FROM suppliers \
             WHERE deleted_at IS NULL AND (name LIKE ? OR supplier_code LIKE ?) \
             ORDER BY name ASC LIMIT 50",
        )
        .bind(&like)
        .bind(&like)
        .fetch_all(pool)
        .await
    }

    pub async fn find_all_active(
        pool: &SqlitePool,
    ) -> Result<Vec<Supplier>, sqlx::Error> {
        sqlx::query_as::<_, Supplier>(
            "SELECT id, supplier_code, name, contact_person, phone, email, address, \
             is_active, notes, created_at, updated_at, deleted_at \
             FROM suppliers WHERE deleted_at IS NULL AND is_active = 1 \
             ORDER BY name ASC",
        )
        .fetch_all(pool)
        .await
    }
}
