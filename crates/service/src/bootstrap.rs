//! 启动引导：确保存在一个可登录的管理员。
//!
//! 语义沿用设计 §9.4：只有同时提供 `STEELPIPE_ADMIN_USERNAME` 与
//! `STEELPIPE_ADMIN_PASSWORD` 才生效；同名用户存在则重置其口令并把角色提为
//! admin（旧实现重名即重置口令）。不写审计：这是部署动作，发生在任何会话之前。

use steelpipe_auth::hash_password;
use steelpipe_domain::Role;
use steelpipe_infra::{DbPool, InfraError, repo::users, time, uow::Uow};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdminSeed {
    Created,
    Reset,
    Invalid(String),
}

pub async fn ensure_admin(
    pool: &DbPool,
    username: &str,
    password: &str,
) -> Result<AdminSeed, InfraError> {
    let username = username.trim();
    if username.is_empty() || password.is_empty() {
        return Ok(AdminSeed::Invalid("用户名或口令为空".to_owned()));
    }
    if password.chars().count() < 8 {
        return Ok(AdminSeed::Invalid("管理员口令至少 8 位".to_owned()));
    }

    let hash = hash_password(password).map_err(|e| InfraError::Other(e.to_string()))?;
    let existing = users::find_by_username(pool, username).await?;

    let mut uow = Uow::begin(pool).await?;
    let outcome = match existing {
        Some(row) => {
            users::update_password(uow.conn(), row.id, &hash).await?;
            users::set_role(uow.conn(), row.id, Role::Admin).await?;
            users::set_active(uow.conn(), row.id, true).await?;
            AdminSeed::Reset
        }
        None => {
            users::insert(
                uow.conn(),
                username,
                &hash,
                username,
                Role::Admin,
                &time::now_rfc3339(),
            )
            .await?;
            AdminSeed::Created
        }
    };
    uow.commit().await?;
    Ok(outcome)
}
