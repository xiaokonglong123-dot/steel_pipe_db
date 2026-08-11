//! JWT 签发/校验 + refresh token 轮换 + bootstrap_admin
//!
//! 对齐 detailed-design：access token（httpOnly cookie 或 Bearer）+ refresh token
//! （服务器端 SHA-256 hash 存储 + 轮换）。token 只带 user_id + username + display_name，
//! 权限由 middleware 查库实时取。

use argon2::password_hash::{rand_core::OsRng, PasswordHasher, PasswordVerifier, SaltString};
use argon2::{Argon2, PasswordHash};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::error::{AppError, ErrorCode};
use crate::repos::auth_repo::{self, UserRow};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i64, // user_id
    pub username: String,
    pub display_name: String,
    pub exp: usize, // 过期时间(秒)
    pub iat: usize,
}

pub fn issue_token(claims: &Claims, jwt_secret: &str) -> Result<String, AppError> {
    encode(
        &Header::default(),
        claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
    .map_err(|e| {
        tracing::error!(error = %e, "token encode failed");
        AppError::new(ErrorCode::Internal, "token 生成失败")
    })
}

pub fn create_access_token(
    user: &UserRow,
    jwt_secret: &str,
    expiry_hours: u64,
) -> Result<String, AppError> {
    let now = Utc::now();
    let exp = (now + Duration::hours(expiry_hours as i64)).timestamp() as usize;
    let claims = Claims {
        sub: user.id,
        username: user.username.clone(),
        display_name: user.display_name.clone(),
        exp,
        iat: now.timestamp() as usize,
    };
    issue_token(&claims, jwt_secret)
}

/// 校验 access token（fail-closed：任何错误返回 Unauthorized/TokenExpired）
pub fn verify_token(token: &str, jwt_secret: &str) -> Result<Claims, AppError> {
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|e| {
        if *e.kind() == jsonwebtoken::errors::ErrorKind::ExpiredSignature {
            AppError::new(ErrorCode::TokenExpired, "登录已过期，请重新登录")
        } else {
            AppError::new(ErrorCode::Unauthorized, "无效的认证令牌")
        }
    })?;
    Ok(data.claims)
}

/// 生成 refresh token 原文（随机 hex）
pub fn generate_refresh_token() -> String {
    use rand_core::RngCore;
    let mut bytes = [0u8; 32];
    let mut rng = rand_core::OsRng;
    rng.fill_bytes(&mut bytes);
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// SHA-256 hash（DB 只存 hash，不存原文）
pub fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    let out = hasher.finalize();
    out.iter().map(|b| format!("{b:02x}")).collect()
}

/// Argon2id 哈希密码
pub fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    Ok(argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| {
            tracing::error!(error = %e, "password hash failed");
            AppError::new(ErrorCode::Internal, "密码处理错误")
        })?
        .to_string())
}

/// 校验密码（Argon2id）
pub fn verify_password(password: &str, hash: &str) -> bool {
    let Ok(parsed) = PasswordHash::new(hash) else {
        return false;
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok()
}

/// 启动时幂等创建初始管理员（对齐 seed：role_id=1 admin）
pub async fn bootstrap_admin(
    pool: &sqlx::SqlitePool,
    username: &str,
    password: &str,
) -> Result<(), AppError> {
    if let Some(user) = auth_repo::find_by_username(pool, username).await? {
        // 用户已存在：确保已分配 admin 角色
        if !auth_repo::has_role(pool, user.id, 1).await? {
            auth_repo::assign_role(pool, user.id, 1).await?;
        }
        return Ok(());
    }

    let hash = hash_password(password)?;
    let user = auth_repo::create_user(pool, username, &hash, username).await?;
    auth_repo::assign_role(pool, user.id, 1).await?;
    Ok(())
}
