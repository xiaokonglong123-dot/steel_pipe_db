//! Auth 数据访问 — users/roles/permissions/refresh_tokens/operation_logs
//!
//! 纯 SQL（sqlx），无业务逻辑、无事务控制（事务在 service 层）。

use sqlx::SqlitePool;

use crate::error::{AppError, ErrorCode};

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UserRow {
    pub id: i64,
    pub username: String,
    pub display_name: String,
    pub password_hash: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub deleted_at: Option<String>,
}

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct RoleRow {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub is_system: bool,
}

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct PermissionRow {
    pub id: i64,
    pub key: String,
    pub name: String,
}

pub async fn find_by_username(
    pool: &SqlitePool,
    username: &str,
) -> Result<Option<UserRow>, AppError> {
    let row = sqlx::query_as::<_, UserRow>(
        "SELECT id, username, display_name, password_hash, email, phone, is_active, created_at, deleted_at
         FROM users WHERE username = ? AND deleted_at IS NULL",
    )
    .bind(username)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn find_by_id(pool: &SqlitePool, id: i64) -> Result<Option<UserRow>, AppError> {
    let row = sqlx::query_as::<_, UserRow>(
        "SELECT id, username, display_name, password_hash, email, phone, is_active, created_at, deleted_at
         FROM users WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn create_user(
    pool: &SqlitePool,
    username: &str,
    password_hash: &str,
    display_name: &str,
) -> Result<UserRow, AppError> {
    let result =
        sqlx::query("INSERT INTO users (username, display_name, password_hash) VALUES (?, ?, ?)")
            .bind(username)
            .bind(display_name)
            .bind(password_hash)
            .execute(pool)
            .await?;
    let id = result.last_insert_rowid();
    find_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::Internal, "用户创建后读取失败"))
}

pub async fn list_users(
    pool: &SqlitePool,
    page: i64,
    page_size: i64,
) -> Result<(Vec<UserRow>, i64), AppError> {
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE deleted_at IS NULL")
        .fetch_one(pool)
        .await?;
    let offset = (page - 1).max(0) * page_size;
    let rows = sqlx::query_as::<_, UserRow>(
        "SELECT id, username, display_name, password_hash, email, phone, is_active, created_at, deleted_at
         FROM users WHERE deleted_at IS NULL ORDER BY id LIMIT ? OFFSET ?",
    )
    .bind(page_size)
    .bind(offset)
    .fetch_all(pool)
    .await?;
    Ok((rows, total))
}

/// 软删除用户
pub async fn soft_delete_user(pool: &SqlitePool, id: i64) -> Result<(), AppError> {
    let result = sqlx::query(
        "UPDATE users SET deleted_at = datetime('now') WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(id)
    .execute(pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::new(ErrorCode::NotFound, "用户未找到"));
    }
    Ok(())
}

/// 更新用户资料（display_name/email/phone/is_active；password_hash 可选）
pub async fn update_user(
    pool: &SqlitePool,
    id: i64,
    display_name: &str,
    email: Option<&str>,
    phone: Option<&str>,
    is_active: bool,
) -> Result<(), AppError> {
    let result = sqlx::query(
        "UPDATE users SET display_name = ?, email = ?, phone = ?, is_active = ?, updated_at = datetime('now')
         WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(display_name)
    .bind(email)
    .bind(phone)
    .bind(is_active)
    .bind(id)
    .execute(pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::new(ErrorCode::NotFound, "用户未找到"));
    }
    Ok(())
}

pub async fn update_password_hash(
    pool: &SqlitePool,
    id: i64,
    new_hash: &str,
) -> Result<(), AppError> {
    sqlx::query("UPDATE users SET password_hash = ?, updated_at = datetime('now') WHERE id = ?")
        .bind(new_hash)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

// —— RBAC ——

pub async fn list_roles(pool: &SqlitePool) -> Result<Vec<RoleRow>, AppError> {
    let rows = sqlx::query_as::<_, RoleRow>(
        "SELECT id, name, description, is_system FROM roles ORDER BY id",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn list_permissions(pool: &SqlitePool) -> Result<Vec<PermissionRow>, AppError> {
    let rows =
        sqlx::query_as::<_, PermissionRow>("SELECT id, key, name FROM permissions ORDER BY id")
            .fetch_all(pool)
            .await?;
    Ok(rows)
}

/// 查库实时取用户全部权限 key（user → user_roles → roles → role_permissions → permissions）
pub async fn list_permissions_for_user(
    pool: &SqlitePool,
    user_id: i64,
) -> Result<Vec<String>, AppError> {
    let rows: Vec<String> = sqlx::query_scalar(
        "SELECT DISTINCT p.key
         FROM permissions p
         JOIN role_permissions rp ON rp.permission_id = p.id
         JOIN user_roles ur ON ur.role_id = rp.role_id
         WHERE ur.user_id = ?",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn has_role(pool: &SqlitePool, user_id: i64, role_id: i64) -> Result<bool, AppError> {
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM user_roles WHERE user_id = ? AND role_id = ?")
            .bind(user_id)
            .bind(role_id)
            .fetch_one(pool)
            .await?;
    Ok(count > 0)
}

pub async fn assign_role(pool: &SqlitePool, user_id: i64, role_id: i64) -> Result<(), AppError> {
    sqlx::query("INSERT OR IGNORE INTO user_roles (user_id, role_id) VALUES (?, ?)")
        .bind(user_id)
        .bind(role_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// 替换用户的角色集合（先清空再插入）
pub async fn replace_user_roles(
    pool: &SqlitePool,
    user_id: i64,
    role_ids: &[i64],
) -> Result<(), AppError> {
    sqlx::query("DELETE FROM user_roles WHERE user_id = ?")
        .bind(user_id)
        .execute(pool)
        .await?;
    for role_id in role_ids {
        sqlx::query("INSERT OR IGNORE INTO user_roles (user_id, role_id) VALUES (?, ?)")
            .bind(user_id)
            .bind(role_id)
            .execute(pool)
            .await?;
    }
    Ok(())
}

// —— Refresh tokens（SHA-256 hash 存储 + 轮换）——

pub async fn insert_refresh_token(
    pool: &SqlitePool,
    user_id: i64,
    token_hash: &str,
    expires_at: &str,
) -> Result<(), AppError> {
    sqlx::query("INSERT INTO refresh_tokens (user_id, token_hash, expires_at) VALUES (?, ?, ?)")
        .bind(user_id)
        .bind(token_hash)
        .bind(expires_at)
        .execute(pool)
        .await?;
    Ok(())
}

/// 校验 refresh token 是否有效（未吊销 && 未过期），返回 user_id
pub async fn find_valid_refresh_token(
    pool: &SqlitePool,
    token_hash: &str,
) -> Result<Option<i64>, AppError> {
    let user_id: Option<i64> = sqlx::query_scalar(
        "SELECT user_id FROM refresh_tokens
         WHERE token_hash = ? AND revoked_at IS NULL AND expires_at > datetime('now')",
    )
    .bind(token_hash)
    .fetch_optional(pool)
    .await?;
    Ok(user_id)
}

/// 吊销指定 user 的全部 refresh tokens（登出/轮换）
pub async fn revoke_all_for_user(pool: &SqlitePool, user_id: i64) -> Result<(), AppError> {
    sqlx::query("UPDATE refresh_tokens SET revoked_at = datetime('now') WHERE user_id = ? AND revoked_at IS NULL")
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}

// —— Operation logs ——

pub async fn log_operation(
    pool: &SqlitePool,
    user_id: Option<i64>,
    action: &str,
    target_type: &str,
    target_id: Option<i64>,
    ip_address: &str,
) -> Result<(), AppError> {
    if user_id.is_none() {
        return Ok(());
    }
    sqlx::query(
        "INSERT INTO operation_logs (user_id, action, target_type, target_id, ip_address) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(user_id)
    .bind(action)
    .bind(target_type)
    .bind(target_id)
    .bind(ip_address)
    .execute(pool)
    .await?;
    Ok(())
}

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct OperationLogRow {
    pub id: i64,
    pub user_id: Option<i64>,
    pub action: String,
    pub target_type: Option<String>,
    pub target_id: Option<i64>,
    pub ip_address: Option<String>,
    pub created_at: String,
}

pub async fn list_operation_logs(
    pool: &SqlitePool,
    page: i64,
    page_size: i64,
) -> Result<(Vec<OperationLogRow>, i64), AppError> {
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM operation_logs")
        .fetch_one(pool)
        .await?;
    let offset = (page - 1).max(0) * page_size;
    let rows = sqlx::query_as::<_, OperationLogRow>(
        "SELECT id, user_id, action, target_type, target_id, ip_address, created_at
         FROM operation_logs ORDER BY id DESC LIMIT ? OFFSET ?",
    )
    .bind(page_size)
    .bind(offset)
    .fetch_all(pool)
    .await?;
    Ok((rows, total))
}
