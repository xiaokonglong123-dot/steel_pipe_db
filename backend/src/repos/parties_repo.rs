//! Parties 数据访问 — suppliers / customers 表（供应商 / 客户主数据）
//!
//! 纯 SQL（sqlx），无业务逻辑、无事务控制（事务在 service 层）。
//! SupplierRow / CustomerRow 字段对齐 migrations/003_parties.sql。

use sqlx::SqlitePool;

use crate::error::{AppError, ErrorCode};

// —— Suppliers ——

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct SupplierRow {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub contact: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct SupplierFilter<'a> {
    pub code: Option<&'a str>,
    pub name: Option<&'a str>,
    pub status: Option<&'a str>,
}

pub async fn find_supplier_by_id(
    pool: &SqlitePool,
    id: i64,
) -> Result<Option<SupplierRow>, AppError> {
    let row = sqlx::query_as::<_, SupplierRow>(
        "SELECT id, code, name, contact, phone, email, address, status,
                created_at, updated_at, deleted_at
         FROM suppliers WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn find_supplier_by_code(
    pool: &SqlitePool,
    code: &str,
) -> Result<Option<SupplierRow>, AppError> {
    let row = sqlx::query_as::<_, SupplierRow>(
        "SELECT id, code, name, contact, phone, email, address, status,
                created_at, updated_at, deleted_at
         FROM suppliers WHERE code = ? AND deleted_at IS NULL",
    )
    .bind(code)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn create_supplier(
    pool: &SqlitePool,
    code: &str,
    name: &str,
    contact: Option<&str>,
    phone: Option<&str>,
    email: Option<&str>,
    address: Option<&str>,
) -> Result<SupplierRow, AppError> {
    let result = sqlx::query(
        "INSERT INTO suppliers (code, name, contact, phone, email, address, status)
         VALUES (?, ?, ?, ?, ?, ?, 'active')",
    )
    .bind(code)
    .bind(name)
    .bind(contact)
    .bind(phone)
    .bind(email)
    .bind(address)
    .execute(pool)
    .await?;
    let id = result.last_insert_rowid();
    find_supplier_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::Internal, "供应商创建后读取失败"))
}

pub async fn list_suppliers(
    pool: &SqlitePool,
    filter: &SupplierFilter<'_>,
    page: i64,
    page_size: i64,
) -> Result<(Vec<SupplierRow>, i64), AppError> {
    let mut where_clauses: Vec<&'static str> = vec!["deleted_at IS NULL"];
    let mut count_sql = String::from("SELECT COUNT(*) FROM suppliers WHERE deleted_at IS NULL");
    let mut list_sql = String::from(
        "SELECT id, code, name, contact, phone, email, address, status,
                created_at, updated_at, deleted_at
         FROM suppliers WHERE deleted_at IS NULL",
    );

    if filter.code.is_some() {
        where_clauses.push("code = ?");
    }
    if filter.name.is_some() {
        where_clauses.push("name LIKE ?");
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
    if let Some(v) = filter.code {
        count_q = count_q.bind(v);
    }
    if let Some(v) = filter.name {
        let like = format!("%{v}%");
        count_q = count_q.bind(like);
    }
    if let Some(v) = filter.status {
        count_q = count_q.bind(v);
    }
    let total: i64 = count_q.fetch_one(pool).await?;

    let offset = (page - 1).max(0) * page_size;
    let mut list_q = sqlx::query_as::<_, SupplierRow>(&list_sql);
    if let Some(v) = filter.code {
        list_q = list_q.bind(v);
    }
    if let Some(v) = filter.name {
        let like = format!("%{v}%");
        list_q = list_q.bind(like);
    }
    if let Some(v) = filter.status {
        list_q = list_q.bind(v);
    }
    list_q = list_q.bind(page_size).bind(offset);
    let rows = list_q.fetch_all(pool).await?;
    Ok((rows, total))
}

pub async fn update_supplier(
    pool: &SqlitePool,
    id: i64,
    code: &str,
    name: &str,
    contact: Option<&str>,
    phone: Option<&str>,
    email: Option<&str>,
    address: Option<&str>,
    status: &str,
) -> Result<(), AppError> {
    let result = sqlx::query(
        "UPDATE suppliers SET code = ?, name = ?, contact = ?, phone = ?, email = ?,
           address = ?, status = ?, updated_at = datetime('now')
         WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(code)
    .bind(name)
    .bind(contact)
    .bind(phone)
    .bind(email)
    .bind(address)
    .bind(status)
    .bind(id)
    .execute(pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::new(ErrorCode::SupplierNotFound, "供应商未找到"));
    }
    Ok(())
}

pub async fn soft_delete_supplier(pool: &SqlitePool, id: i64) -> Result<(), AppError> {
    let result = sqlx::query(
        "UPDATE suppliers SET deleted_at = datetime('now')
         WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(id)
    .execute(pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::new(ErrorCode::SupplierNotFound, "供应商未找到"));
    }
    Ok(())
}

// —— Customers ——

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct CustomerRow {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub contact: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct CustomerFilter<'a> {
    pub code: Option<&'a str>,
    pub name: Option<&'a str>,
    pub status: Option<&'a str>,
}

pub async fn find_customer_by_id(
    pool: &SqlitePool,
    id: i64,
) -> Result<Option<CustomerRow>, AppError> {
    let row = sqlx::query_as::<_, CustomerRow>(
        "SELECT id, code, name, contact, phone, email, address, status,
                created_at, updated_at, deleted_at
         FROM customers WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn find_customer_by_code(
    pool: &SqlitePool,
    code: &str,
) -> Result<Option<CustomerRow>, AppError> {
    let row = sqlx::query_as::<_, CustomerRow>(
        "SELECT id, code, name, contact, phone, email, address, status,
                created_at, updated_at, deleted_at
         FROM customers WHERE code = ? AND deleted_at IS NULL",
    )
    .bind(code)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn create_customer(
    pool: &SqlitePool,
    code: &str,
    name: &str,
    contact: Option<&str>,
    phone: Option<&str>,
    email: Option<&str>,
    address: Option<&str>,
) -> Result<CustomerRow, AppError> {
    let result = sqlx::query(
        "INSERT INTO customers (code, name, contact, phone, email, address, status)
         VALUES (?, ?, ?, ?, ?, ?, 'active')",
    )
    .bind(code)
    .bind(name)
    .bind(contact)
    .bind(phone)
    .bind(email)
    .bind(address)
    .execute(pool)
    .await?;
    let id = result.last_insert_rowid();
    find_customer_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::Internal, "客户创建后读取失败"))
}

pub async fn list_customers(
    pool: &SqlitePool,
    filter: &CustomerFilter<'_>,
    page: i64,
    page_size: i64,
) -> Result<(Vec<CustomerRow>, i64), AppError> {
    let mut where_clauses: Vec<&'static str> = vec!["deleted_at IS NULL"];
    let mut count_sql = String::from("SELECT COUNT(*) FROM customers WHERE deleted_at IS NULL");
    let mut list_sql = String::from(
        "SELECT id, code, name, contact, phone, email, address, status,
                created_at, updated_at, deleted_at
         FROM customers WHERE deleted_at IS NULL",
    );

    if filter.code.is_some() {
        where_clauses.push("code = ?");
    }
    if filter.name.is_some() {
        where_clauses.push("name LIKE ?");
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
    if let Some(v) = filter.code {
        count_q = count_q.bind(v);
    }
    if let Some(v) = filter.name {
        let like = format!("%{v}%");
        count_q = count_q.bind(like);
    }
    if let Some(v) = filter.status {
        count_q = count_q.bind(v);
    }
    let total: i64 = count_q.fetch_one(pool).await?;

    let offset = (page - 1).max(0) * page_size;
    let mut list_q = sqlx::query_as::<_, CustomerRow>(&list_sql);
    if let Some(v) = filter.code {
        list_q = list_q.bind(v);
    }
    if let Some(v) = filter.name {
        let like = format!("%{v}%");
        list_q = list_q.bind(like);
    }
    if let Some(v) = filter.status {
        list_q = list_q.bind(v);
    }
    list_q = list_q.bind(page_size).bind(offset);
    let rows = list_q.fetch_all(pool).await?;
    Ok((rows, total))
}

pub async fn update_customer(
    pool: &SqlitePool,
    id: i64,
    code: &str,
    name: &str,
    contact: Option<&str>,
    phone: Option<&str>,
    email: Option<&str>,
    address: Option<&str>,
    status: &str,
) -> Result<(), AppError> {
    let result = sqlx::query(
        "UPDATE customers SET code = ?, name = ?, contact = ?, phone = ?, email = ?,
           address = ?, status = ?, updated_at = datetime('now')
         WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(code)
    .bind(name)
    .bind(contact)
    .bind(phone)
    .bind(email)
    .bind(address)
    .bind(status)
    .bind(id)
    .execute(pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::new(ErrorCode::CustomerNotFound, "客户未找到"));
    }
    Ok(())
}

pub async fn soft_delete_customer(pool: &SqlitePool, id: i64) -> Result<(), AppError> {
    let result = sqlx::query(
        "UPDATE customers SET deleted_at = datetime('now')
         WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(id)
    .execute(pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::new(ErrorCode::CustomerNotFound, "客户未找到"));
    }
    Ok(())
}
