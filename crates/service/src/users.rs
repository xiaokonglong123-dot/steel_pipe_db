//! 用户用例（全部要求 admin）+ 审计。

use serde::Serialize;
use steelpipe_audit::detail;
use steelpipe_auth::{AuthError, hash_password};
use steelpipe_domain::Role;
use steelpipe_infra::{DbPool, InfraError, repo::users, time, uow::Uow};

use crate::actions::{CREATE_USER, REVOKE_SESSIONS, TOGGLE_USER, UPDATE_USER};
use crate::auth::Identity;

#[derive(Debug, thiserror::Error)]
pub enum UserError {
    #[error("{0}")]
    Invalid(String),
    #[error("用户不存在")]
    NotFound,
    #[error("用户名已存在")]
    Conflict,
    #[error("不能移除最后一个管理员")]
    LastAdmin,
    #[error(transparent)]
    Infra(#[from] InfraError),
    #[error(transparent)]
    Auth(#[from] AuthError),
}

#[derive(Debug, Clone, Serialize)]
pub struct UserView {
    pub id: i64,
    pub username: String,
    pub display_name: String,
    pub role: Role,
    pub active: bool,
    pub created_at: String,
}

impl UserView {
    fn from_row(row: users::UserRow) -> Result<Self, UserError> {
        let role = row
            .role()
            .ok_or_else(|| UserError::Invalid(format!("数据库中的角色非法：{}", row.role)))?;
        Ok(Self {
            id: row.id,
            username: row.username,
            display_name: row.display_name,
            role,
            active: row.active,
            created_at: row.created_at,
        })
    }
}

#[derive(Debug, Clone)]
pub struct NewUser {
    pub username: String,
    pub password: String,
    pub display_name: String,
    pub role: Role,
}

pub struct UserService {
    pool: DbPool,
}

impl UserService {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self) -> Result<Vec<UserView>, UserError> {
        let rows = users::list(&self.pool).await?;
        rows.into_iter().map(UserView::from_row).collect()
    }

    pub async fn create(&self, actor: &Identity, input: NewUser) -> Result<UserView, UserError> {
        validate_username(&input.username)?;
        validate_password(&input.password)?;
        let display_name = input.display_name.trim();
        if display_name.is_empty() {
            return Err(UserError::Invalid("显示名不能为空".into()));
        }

        let hash = hash_password(&input.password)?;
        let created_at = time::now_rfc3339();

        let mut uow = Uow::begin(&self.pool).await?;
        let id = users::insert(
            uow.conn(),
            input.username.trim(),
            &hash,
            display_name,
            input.role,
            &created_at,
        )
        .await
        .map_err(|e| match e {
            InfraError::Conflict(_) => UserError::Conflict,
            other => UserError::Infra(other),
        })?;

        let payload = CreateUserDetail {
            user_id: id,
            username: input.username.trim(),
            role: input.role,
            by: &actor.username,
        };
        uow.audit(
            Some(actor.user_id),
            CREATE_USER,
            &detail(&payload).map_err(|e| AuthError::Sign(e.to_string()))?,
        )
        .await?;
        uow.commit().await?;

        Ok(UserView {
            id,
            username: input.username.trim().to_owned(),
            display_name: display_name.to_owned(),
            role: input.role,
            active: true,
            created_at,
        })
    }

    pub async fn set_active(
        &self,
        actor: &Identity,
        id: i64,
        active: bool,
    ) -> Result<(), UserError> {
        if id == actor.user_id && !active {
            return Err(UserError::Invalid("不能停用自己的账号".into()));
        }
        let mut uow = Uow::begin(&self.pool).await?;
        let target = users::find_by_id(uow.conn(), id)
            .await?
            .ok_or(UserError::NotFound)?;
        if !active
            && target.role().is_some_and(Role::is_admin)
            && users::count_admins(uow.conn(), Some(id)).await? == 0
        {
            return Err(UserError::LastAdmin);
        }
        users::set_active(uow.conn(), id, active).await?;
        let payload = ToggleUserDetail {
            user_id: id,
            username: &target.username,
            active,
            by: &actor.username,
        };
        uow.audit(
            Some(actor.user_id),
            TOGGLE_USER,
            &detail(&payload).map_err(|e| AuthError::Sign(e.to_string()))?,
        )
        .await?;
        uow.commit().await?;
        Ok(())
    }

    /// 改密：同时递增 `token_version`，该用户全部会话失效。
    pub async fn set_password(
        &self,
        actor: &Identity,
        id: i64,
        password: &str,
    ) -> Result<(), UserError> {
        validate_password(password)?;
        let target = users::find_by_id(&self.pool, id)
            .await?
            .ok_or(UserError::NotFound)?;
        let hash = hash_password(password)?;

        let mut uow = Uow::begin(&self.pool).await?;
        users::update_password(uow.conn(), id, &hash).await?;
        let payload = UpdateUserDetail {
            user_id: id,
            username: &target.username,
            field: "password",
            by: &actor.username,
        };
        uow.audit(
            Some(actor.user_id),
            UPDATE_USER,
            &detail(&payload).map_err(|e| AuthError::Sign(e.to_string()))?,
        )
        .await?;
        uow.commit().await?;
        Ok(())
    }

    /// 吊销全部会话（`token_version + 1` 且清 refresh jti）。
    pub async fn revoke_sessions(&self, actor: &Identity, id: i64) -> Result<i64, UserError> {
        let target = users::find_by_id(&self.pool, id)
            .await?
            .ok_or(UserError::NotFound)?;

        let mut uow = Uow::begin(&self.pool).await?;
        let version = users::revoke_all_sessions(uow.conn(), id).await?;
        let payload = RevokeDetail {
            user_id: id,
            username: &target.username,
            token_version: version,
            by: &actor.username,
        };
        uow.audit(
            Some(actor.user_id),
            REVOKE_SESSIONS,
            &detail(&payload).map_err(|e| AuthError::Sign(e.to_string()))?,
        )
        .await?;
        uow.commit().await?;
        Ok(version)
    }

    pub async fn set_role(&self, actor: &Identity, id: i64, role: Role) -> Result<(), UserError> {
        let mut uow = Uow::begin(&self.pool).await?;
        let target = users::find_by_id(uow.conn(), id)
            .await?
            .ok_or(UserError::NotFound)?;
        if !role.is_admin()
            && target.role().is_some_and(Role::is_admin)
            && users::count_admins(uow.conn(), Some(id)).await? == 0
        {
            return Err(UserError::LastAdmin);
        }
        users::set_role(uow.conn(), id, role).await?;
        let payload = SetRoleDetail {
            user_id: id,
            username: &target.username,
            role,
            by: &actor.username,
        };
        uow.audit(
            Some(actor.user_id),
            UPDATE_USER,
            &detail(&payload).map_err(|e| AuthError::Sign(e.to_string()))?,
        )
        .await?;
        uow.commit().await?;
        Ok(())
    }
}

fn validate_username(username: &str) -> Result<(), UserError> {
    let username = username.trim();
    if !(3..=32).contains(&username.chars().count()) {
        return Err(UserError::Invalid("用户名长度需在 3–32 之间".into()));
    }
    if !username
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'))
    {
        return Err(UserError::Invalid(
            "用户名只允许字母、数字、下划线、连字符与点".into(),
        ));
    }
    Ok(())
}

fn validate_password(password: &str) -> Result<(), UserError> {
    if password.chars().count() < 8 {
        return Err(UserError::Invalid("口令至少 8 位".into()));
    }
    if password.chars().count() > 256 {
        return Err(UserError::Invalid("口令过长（上限 256）".into()));
    }
    Ok(())
}

#[derive(Debug, Serialize)]
struct CreateUserDetail<'a> {
    user_id: i64,
    username: &'a str,
    role: Role,
    by: &'a str,
}

#[derive(Debug, Serialize)]
struct ToggleUserDetail<'a> {
    user_id: i64,
    username: &'a str,
    active: bool,
    by: &'a str,
}

#[derive(Debug, Serialize)]
struct UpdateUserDetail<'a> {
    user_id: i64,
    username: &'a str,
    field: &'a str,
    by: &'a str,
}

#[derive(Debug, Serialize)]
struct SetRoleDetail<'a> {
    user_id: i64,
    username: &'a str,
    role: Role,
    by: &'a str,
}

#[derive(Debug, Serialize)]
struct RevokeDetail<'a> {
    user_id: i64,
    username: &'a str,
    token_version: i64,
    by: &'a str,
}

#[cfg(test)]
mod tests {
    use super::*;
    use steelpipe_auth::Keys;
    use steelpipe_infra::repo::audit as audit_repo;

    use crate::auth::AuthService;

    struct Fixture {
        pool: DbPool,
        path: std::path::PathBuf,
        users: UserService,
        auth: AuthService,
    }

    async fn fixture(tag: &str) -> Fixture {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("steelpipe-users-{tag}-{nanos}.db"));
        let pool = steelpipe_infra::connect(&format!("sqlite://{}", path.display()), 8)
            .await
            .unwrap();
        steelpipe_infra::migrate(&pool).await.unwrap();
        let keys = Keys::from_secret(b"users-test-secret-at-least-32-bytes!!");
        let auth = AuthService::new(pool.clone(), keys.clone());
        Fixture {
            users: UserService::new(pool.clone()),
            auth,
            pool,
            path,
        }
    }

    async fn admin_identity(fx: &Fixture) -> Identity {
        let hash = hash_password("admin-pw-123").unwrap();
        let mut uow = Uow::begin(&fx.pool).await.unwrap();
        let id = users::insert(
            uow.conn(),
            "root",
            &hash,
            "Root",
            Role::Admin,
            &time::now_rfc3339(),
        )
        .await
        .unwrap();
        uow.commit().await.unwrap();
        fx.auth
            .login("root", "admin-pw-123", "127.0.0.1")
            .await
            .map(|out| {
                assert_eq!(out.identity.user_id, id);
                out.identity
            })
            .expect("管理员登录失败")
    }

    #[tokio::test]
    async fn create_writes_one_audit_row_with_actor() {
        let fx = fixture("create").await;
        let admin = admin_identity(&fx).await;

        let view = fx
            .users
            .create(
                &admin,
                NewUser {
                    username: "warehouse".into(),
                    password: "warehouse-pw".into(),
                    display_name: "仓库".into(),
                    role: Role::Operator,
                },
            )
            .await
            .expect("创建失败");
        assert_eq!(view.role, Role::Operator);

        let rows = audit_repo::page_after(&fx.pool, 0, 10).await.unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].action, CREATE_USER);
        assert!(rows[0].detail.contains("\"warehouse\""));
        assert!(rows[0].detail.contains("\"by\":\"root\""));

        fx.pool.close().await;
        let _ = std::fs::remove_file(&fx.path);
    }

    #[tokio::test]
    async fn duplicate_username_is_conflict_not_500() {
        let fx = fixture("dup").await;
        let admin = admin_identity(&fx).await;
        let input = NewUser {
            username: "dup".into(),
            password: "password-1".into(),
            display_name: "Dup".into(),
            role: Role::Viewer,
        };
        fx.users.create(&admin, input.clone()).await.unwrap();
        let err = fx.users.create(&admin, input).await.unwrap_err();
        assert!(matches!(err, UserError::Conflict));
        assert_eq!(
            audit_repo::count(&fx.pool).await.unwrap(),
            1,
            "失败创建不该写审计"
        );

        fx.pool.close().await;
        let _ = std::fs::remove_file(&fx.path);
    }

    #[tokio::test]
    async fn rejects_weak_password_and_bad_username() {
        let fx = fixture("validate").await;
        let admin = admin_identity(&fx).await;
        let err = fx
            .users
            .create(
                &admin,
                NewUser {
                    username: "ok_name".into(),
                    password: "short".into(),
                    display_name: "X".into(),
                    role: Role::Viewer,
                },
            )
            .await
            .unwrap_err();
        assert!(matches!(err, UserError::Invalid(_)));

        let err = fx
            .users
            .create(
                &admin,
                NewUser {
                    username: "bad name!".into(),
                    password: "long-enough".into(),
                    display_name: "X".into(),
                    role: Role::Viewer,
                },
            )
            .await
            .unwrap_err();
        assert!(matches!(err, UserError::Invalid(_)));

        fx.pool.close().await;
        let _ = std::fs::remove_file(&fx.path);
    }

    #[tokio::test]
    async fn cannot_remove_the_last_admin() {
        let fx = fixture("last-admin").await;
        let admin = admin_identity(&fx).await;
        let err = fx
            .users
            .set_active(&admin, admin.user_id, false)
            .await
            .unwrap_err();
        assert!(matches!(err, UserError::Invalid(_)), "不允许停用自己");

        let err = fx
            .users
            .set_role(&admin, admin.user_id, Role::Operator)
            .await
            .unwrap_err();
        assert!(matches!(err, UserError::LastAdmin));

        fx.pool.close().await;
        let _ = std::fs::remove_file(&fx.path);
    }

    #[tokio::test]
    async fn password_change_revokes_sessions() {
        let fx = fixture("pw").await;
        let admin = admin_identity(&fx).await;
        let err = fx
            .users
            .set_password(&admin, admin.user_id, "new-password-9")
            .await;
        assert!(err.is_ok());

        let version = users::find_by_id(&fx.pool, admin.user_id)
            .await
            .unwrap()
            .unwrap()
            .token_version;
        assert_eq!(version, admin.token_version + 1);

        fx.pool.close().await;
        let _ = std::fs::remove_file(&fx.path);
    }

    #[tokio::test]
    async fn revoke_sessions_audits_token_version() {
        let fx = fixture("revoke").await;
        let admin = admin_identity(&fx).await;
        let version = fx
            .users
            .revoke_sessions(&admin, admin.user_id)
            .await
            .unwrap();
        assert_eq!(version, admin.token_version + 1);

        let rows = audit_repo::page_after(&fx.pool, 0, 10).await.unwrap();
        assert_eq!(rows[0].action, REVOKE_SESSIONS);
        assert!(
            rows[0]
                .detail
                .contains(&format!("\"token_version\":{version}"))
        );

        fx.pool.close().await;
        let _ = std::fs::remove_file(&fx.path);
    }
}
