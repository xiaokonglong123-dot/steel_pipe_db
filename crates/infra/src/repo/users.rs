//! 用户仓储。写操作接 `&mut SqliteConnection`（由 [`crate::uow::Uow`] 提供）。

use sqlx::{FromRow, SqliteConnection};
use steelpipe_domain::Role;

use crate::{DbPool, InfraError};

#[derive(Debug, Clone, FromRow)]
pub struct UserRow {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
    pub display_name: String,
    pub role: String,
    pub token_version: i64,
    pub last_refresh_jti: Option<String>,
    pub active: bool,
    pub created_at: String,
}

impl UserRow {
    /// 角色字段是 TEXT，DB 层不信任它 —— 非法值返回 `None` 而不是猜。
    pub fn role(&self) -> Option<Role> {
        self.role.parse::<Role>().ok()
    }
}

const COLUMNS: &str = "id, username, password_hash, display_name, role, token_version, last_refresh_jti, active, created_at";

pub async fn find_by_username(
    pool: &DbPool,
    username: &str,
) -> Result<Option<UserRow>, InfraError> {
    let sql = format!("SELECT {COLUMNS} FROM users WHERE username = ?");
    sqlx::query_as::<_, UserRow>(&sql)
        .bind(username)
        .fetch_optional(pool)
        .await
        .map_err(InfraError::Connect)
}

pub async fn find_by_id(pool: &DbPool, id: i64) -> Result<Option<UserRow>, InfraError> {
    let sql = format!("SELECT {COLUMNS} FROM users WHERE id = ?");
    sqlx::query_as::<_, UserRow>(&sql)
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(InfraError::Connect)
}

pub async fn list(pool: &DbPool) -> Result<Vec<UserRow>, InfraError> {
    let sql = format!("SELECT {COLUMNS} FROM users ORDER BY id");
    sqlx::query_as::<_, UserRow>(&sql)
        .fetch_all(pool)
        .await
        .map_err(InfraError::Connect)
}

pub async fn count_admins(pool: &DbPool, excluding_id: Option<i64>) -> Result<i64, InfraError> {
    sqlx::query_scalar("SELECT count(*) FROM users WHERE role = 'admin' AND active = 1 AND id != ?")
        .bind(excluding_id.unwrap_or(-1))
        .fetch_one(pool)
        .await
        .map_err(InfraError::Connect)
}

/// 新建用户，返回 id。用户名重复 → [`InfraError::Conflict`]。
pub async fn insert(
    conn: &mut SqliteConnection,
    username: &str,
    password_hash: &str,
    display_name: &str,
    role: Role,
    created_at: &str,
) -> Result<i64, InfraError> {
    sqlx::query_scalar(
        "INSERT INTO users (username, password_hash, display_name, role, token_version, active, created_at)
         VALUES (?, ?, ?, ?, 0, 1, ?) RETURNING id",
    )
    .bind(username)
    .bind(password_hash)
    .bind(display_name)
    .bind(role.as_str())
    .bind(created_at)
    .fetch_one(conn)
    .await
    .map_err(|e| InfraError::from_unique(e, "用户名"))
}

/// 改密：同时递增 `token_version`（吊销该用户全部会话）。
pub async fn update_password(
    conn: &mut SqliteConnection,
    id: i64,
    password_hash: &str,
) -> Result<(), InfraError> {
    let affected = sqlx::query(
        "UPDATE users SET password_hash = ?, token_version = token_version + 1 WHERE id = ?",
    )
    .bind(password_hash)
    .bind(id)
    .execute(conn)
    .await
    .map_err(InfraError::Connect)?
    .rows_affected();
    if affected == 0 {
        return Err(InfraError::NotFound("用户"));
    }
    Ok(())
}

/// 吊销全部会话：递增 `token_version`，旧 access/refresh 立刻失效。
pub async fn revoke_all_sessions(conn: &mut SqliteConnection, id: i64) -> Result<i64, InfraError> {
    let version: Option<i64> = sqlx::query_scalar(
        "UPDATE users SET token_version = token_version + 1, last_refresh_jti = NULL
         WHERE id = ? RETURNING token_version",
    )
    .bind(id)
    .fetch_optional(conn)
    .await
    .map_err(InfraError::Connect)?;
    version.ok_or(InfraError::NotFound("用户"))
}

/// 单会话轮换：写入 refresh 的 jti（`None` 表示当前无有效 refresh）。
pub async fn set_last_refresh_jti(
    conn: &mut SqliteConnection,
    id: i64,
    jti: Option<&str>,
) -> Result<(), InfraError> {
    sqlx::query("UPDATE users SET last_refresh_jti = ? WHERE id = ?")
        .bind(jti)
        .bind(id)
        .execute(conn)
        .await
        .map_err(InfraError::Connect)?;
    Ok(())
}

pub async fn set_active(
    conn: &mut SqliteConnection,
    id: i64,
    active: bool,
) -> Result<(), InfraError> {
    let affected = sqlx::query("UPDATE users SET active = ? WHERE id = ?")
        .bind(active)
        .bind(id)
        .execute(conn)
        .await
        .map_err(InfraError::Connect)?
        .rows_affected();
    if affected == 0 {
        return Err(InfraError::NotFound("用户"));
    }
    Ok(())
}

pub async fn set_role(conn: &mut SqliteConnection, id: i64, role: Role) -> Result<(), InfraError> {
    let affected = sqlx::query("UPDATE users SET role = ? WHERE id = ?")
        .bind(role.as_str())
        .bind(id)
        .execute(conn)
        .await
        .map_err(InfraError::Connect)?
        .rows_affected();
    if affected == 0 {
        return Err(InfraError::NotFound("用户"));
    }
    Ok(())
}
