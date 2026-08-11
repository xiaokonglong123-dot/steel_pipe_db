//! Parties service — 供应商 / 客户业务规则
//!
//! 对齐 catalog_service：free functions，`pool: &SqlitePool` 首参。
//! 业务规则：code 唯一校验（Validation 10002）、code/name 非空校验、
//! status 取值校验（active/inactive）、存在性校验（15001/15002）。
//!
//! 注：error.rs 未提供 Supplier/Customer Duplicate 变体，重复 code 复用
//! ErrorCode::Validation(10002) — 与 Validation 同域，状态码 400。

use sqlx::SqlitePool;

use crate::error::{AppError, ErrorCode};
use crate::repos::parties_repo::{self, CustomerFilter, CustomerRow, SupplierFilter, SupplierRow};

// —— Suppliers ——

pub async fn create_supplier(
    pool: &SqlitePool,
    code: &str,
    name: &str,
    contact: Option<&str>,
    phone: Option<&str>,
    email: Option<&str>,
    address: Option<&str>,
) -> Result<SupplierRow, AppError> {
    if code.trim().is_empty() || name.trim().is_empty() {
        return Err(AppError::new(ErrorCode::Validation, "code 和名称不能为空"));
    }
    if parties_repo::find_supplier_by_code(pool, code)
        .await?
        .is_some()
    {
        return Err(AppError::new(ErrorCode::Validation, "供应商 code 已存在"));
    }
    parties_repo::create_supplier(pool, code, name, contact, phone, email, address).await
}

pub async fn get_supplier(pool: &SqlitePool, id: i64) -> Result<SupplierRow, AppError> {
    parties_repo::find_supplier_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::SupplierNotFound, "供应商未找到"))
}

pub async fn list_suppliers(
    pool: &SqlitePool,
    filter: &SupplierFilter<'_>,
    page: i64,
    page_size: i64,
) -> Result<(Vec<SupplierRow>, i64), AppError> {
    parties_repo::list_suppliers(pool, filter, page, page_size).await
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
) -> Result<SupplierRow, AppError> {
    if code.trim().is_empty() || name.trim().is_empty() {
        return Err(AppError::new(ErrorCode::Validation, "code 和名称不能为空"));
    }
    if !matches!(status, "active" | "inactive") {
        return Err(AppError::new(
            ErrorCode::Validation,
            "status 取值必须为 active/inactive",
        ));
    }
    let current = parties_repo::find_supplier_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::SupplierNotFound, "供应商未找到"))?;
    if code != current.code {
        if let Some(existing) = parties_repo::find_supplier_by_code(pool, code).await? {
            if existing.id != id {
                return Err(AppError::new(ErrorCode::Validation, "供应商 code 已存在"));
            }
        }
    }
    parties_repo::update_supplier(pool, id, code, name, contact, phone, email, address, status)
        .await?;
    parties_repo::find_supplier_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::SupplierNotFound, "供应商未找到"))
}

pub async fn delete_supplier(pool: &SqlitePool, id: i64) -> Result<(), AppError> {
    parties_repo::soft_delete_supplier(pool, id).await
}

// —— Customers ——

pub async fn create_customer(
    pool: &SqlitePool,
    code: &str,
    name: &str,
    contact: Option<&str>,
    phone: Option<&str>,
    email: Option<&str>,
    address: Option<&str>,
) -> Result<CustomerRow, AppError> {
    if code.trim().is_empty() || name.trim().is_empty() {
        return Err(AppError::new(ErrorCode::Validation, "code 和名称不能为空"));
    }
    if parties_repo::find_customer_by_code(pool, code)
        .await?
        .is_some()
    {
        return Err(AppError::new(ErrorCode::Validation, "客户 code 已存在"));
    }
    parties_repo::create_customer(pool, code, name, contact, phone, email, address).await
}

pub async fn get_customer(pool: &SqlitePool, id: i64) -> Result<CustomerRow, AppError> {
    parties_repo::find_customer_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::CustomerNotFound, "客户未找到"))
}

pub async fn list_customers(
    pool: &SqlitePool,
    filter: &CustomerFilter<'_>,
    page: i64,
    page_size: i64,
) -> Result<(Vec<CustomerRow>, i64), AppError> {
    parties_repo::list_customers(pool, filter, page, page_size).await
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
) -> Result<CustomerRow, AppError> {
    if code.trim().is_empty() || name.trim().is_empty() {
        return Err(AppError::new(ErrorCode::Validation, "code 和名称不能为空"));
    }
    if !matches!(status, "active" | "inactive") {
        return Err(AppError::new(
            ErrorCode::Validation,
            "status 取值必须为 active/inactive",
        ));
    }
    let current = parties_repo::find_customer_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::CustomerNotFound, "客户未找到"))?;
    if code != current.code {
        if let Some(existing) = parties_repo::find_customer_by_code(pool, code).await? {
            if existing.id != id {
                return Err(AppError::new(ErrorCode::Validation, "客户 code 已存在"));
            }
        }
    }
    parties_repo::update_customer(pool, id, code, name, contact, phone, email, address, status)
        .await?;
    parties_repo::find_customer_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::CustomerNotFound, "客户未找到"))
}

pub async fn delete_customer(pool: &SqlitePool, id: i64) -> Result<(), AppError> {
    parties_repo::soft_delete_customer(pool, id).await
}
