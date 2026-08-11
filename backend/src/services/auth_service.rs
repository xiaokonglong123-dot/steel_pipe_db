//! Authentication & RBAC service
//!
//! Login (Argon2 verify + JWT issue + refresh rotation),
//! Refresh (validate + reissue),
//! Logout (revoke all refresh tokens for user),
//! User manage (create / list / update / disable).

use argon2::password_hash::{
    rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
};
use argon2::Argon2;
use chrono::{Duration, Utc};
use sqlx::SqlitePool;

use crate::auth::{self, Claims};
use crate::error::{AppError, ErrorCode};
use crate::repos::auth_repo;

/// Argon2id 哈希
pub fn hash_password(plain: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon = Argon2::default();
    argon
        .hash_password(plain.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| AppError::new(ErrorCode::Internal, &format!("password hash failed: {e}")))
}

/// Argon2id 校验
pub fn verify_password(plain: &str, hash: &str) -> Result<bool, AppError> {
    let parsed = PasswordHash::new(hash)
        .map_err(|e| AppError::new(ErrorCode::Internal, &format!("invalid password hash: {e}")))?;
    Ok(Argon2::default()
        .verify_password(plain.as_bytes(), &parsed)
        .is_ok())
}

/// SHA-256 用于 refresh token 哈希存储
pub fn sha256_hex(s: &str) -> String {
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(s.as_bytes());
    // 转为 hex 字符串
    let mut out = String::with_capacity(64);
    for b in digest {
        use std::fmt::Write;
        let _ = write!(&mut out, "{:02x}", b);
    }
    out
}

pub fn compute_access_expiry(hours: u64) -> usize {
    (Utc::now() + Duration::hours(hours as i64)).timestamp() as usize
}

pub fn compute_refresh_expiry(days: u64) -> chrono::DateTime<Utc> {
    Utc::now() + Duration::days(days as i64)
}

pub struct LoginResult {
    pub access_token: String,
    pub refresh_token: String,
    pub user_id: i64,
    pub username: String,
    pub display_name: String,
}

pub async fn login(
    pool: &SqlitePool,
    username: &str,
    password: &str,
    jwt_secret: &str,
    access_hours: u64,
    refresh_days: u64,
) -> Result<LoginResult, AppError> {
    let user = auth_repo::find_by_username(pool, username)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::Unauthorized, "用户名或密码错误"))?;

    if !user.is_active {
        return Err(AppError::new(ErrorCode::Unauthorized, "用户已禁用"));
    }

    if !verify_password(password, &user.password_hash)? {
        return Err(AppError::new(ErrorCode::Unauthorized, "用户名或密码错误"));
    }

    let access_token = auth::issue_token(
        &Claims {
            sub: user.id,
            username: user.username.clone(),
            display_name: user.display_name.clone(),
            exp: compute_access_expiry(access_hours),
            iat: Utc::now().timestamp() as usize,
        },
        jwt_secret,
    )?;

    let refresh_raw = uuid::Uuid::new_v4().to_string();
    let refresh_hash = sha256_hex(&refresh_raw);
    let expires_at = compute_refresh_expiry(refresh_days).to_rfc3339();
    auth_repo::insert_refresh_token(pool, user.id, &refresh_hash, &expires_at).await?;

    Ok(LoginResult {
        access_token,
        refresh_token: refresh_raw,
        user_id: user.id,
        username: user.username,
        display_name: user.display_name,
    })
}

pub struct RefreshResult {
    pub access_token: String,
    pub refresh_token: String,
    pub user_id: i64,
}

/// 轮换 refresh token：旧 token 吊销 + 新 token 颁发
pub async fn refresh(
    pool: &SqlitePool,
    refresh_token_plain: &str,
    jwt_secret: &str,
    access_hours: u64,
    refresh_days: u64,
) -> Result<RefreshResult, AppError> {
    let hash = sha256_hex(refresh_token_plain);
    let user_id = auth_repo::find_valid_refresh_token(pool, &hash)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::TokenExpired, "refresh token 无效或已过期"))?;

    // 吊销该 token
    sqlx::query("UPDATE refresh_tokens SET revoked_at = datetime('now') WHERE token_hash = ?")
        .bind(&hash)
        .execute(pool)
        .await?;

    let user = auth_repo::find_by_id(pool, user_id)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::Unauthorized, "用户不存在"))?;

    let access_token = auth::issue_token(
        &Claims {
            sub: user.id,
            username: user.username.clone(),
            display_name: user.display_name.clone(),
            exp: compute_access_expiry(access_hours),
            iat: Utc::now().timestamp() as usize,
        },
        jwt_secret,
    )?;

    let refresh_raw = uuid::Uuid::new_v4().to_string();
    let refresh_hash = sha256_hex(&refresh_raw);
    let expires_at = compute_refresh_expiry(refresh_days).to_rfc3339();
    auth_repo::insert_refresh_token(pool, user.id, &refresh_hash, &expires_at).await?;

    Ok(RefreshResult {
        access_token,
        refresh_token: refresh_raw,
        user_id,
    })
}

pub async fn logout(pool: &SqlitePool, user_id: i64) -> Result<(), AppError> {
    auth_repo::revoke_all_for_user(pool, user_id).await
}

pub async fn create_user(
    pool: &SqlitePool,
    username: &str,
    password: &str,
    display_name: &str,
    role_ids: &[i64],
) -> Result<i64, AppError> {
    if username.trim().is_empty() || password.is_empty() {
        return Err(AppError::new(ErrorCode::Validation, "用户名和密码不能为空"));
    }
    if auth_repo::find_by_username(pool, username).await?.is_some() {
        return Err(AppError::new(ErrorCode::Validation, "用户名已存在"));
    }
    let hash = hash_password(password)?;
    let user = auth_repo::create_user(pool, username, &hash, display_name).await?;
    for role_id in role_ids {
        auth_repo::assign_role(pool, user.id, *role_id).await?;
    }
    Ok(user.id)
}

pub async fn list_users(
    pool: &SqlitePool,
    page: i64,
    page_size: i64,
) -> Result<(Vec<auth_repo::UserRow>, i64), AppError> {
    auth_repo::list_users(pool, page, page_size).await
}

pub async fn update_user(
    pool: &SqlitePool,
    id: i64,
    display_name: &str,
    email: Option<&str>,
    phone: Option<&str>,
    is_active: bool,
    role_ids: Option<&[i64]>,
) -> Result<(), AppError> {
    auth_repo::update_user(pool, id, display_name, email, phone, is_active).await?;
    if let Some(roles) = role_ids {
        auth_repo::replace_user_roles(pool, id, roles).await?;
    }
    Ok(())
}

pub async fn disable_user(pool: &SqlitePool, id: i64) -> Result<(), AppError> {
    auth_repo::soft_delete_user(pool, id).await
}

pub async fn list_roles(pool: &SqlitePool) -> Result<Vec<auth_repo::RoleRow>, AppError> {
    auth_repo::list_roles(pool).await
}

pub async fn list_permissions(
    pool: &SqlitePool,
) -> Result<Vec<auth_repo::PermissionRow>, AppError> {
    auth_repo::list_permissions(pool).await
}

pub async fn list_operation_logs(
    pool: &SqlitePool,
    page: i64,
    page_size: i64,
) -> Result<(Vec<auth_repo::OperationLogRow>, i64), AppError> {
    auth_repo::list_operation_logs(pool, page, page_size).await
}
