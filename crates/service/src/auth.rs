//! 认证用例：登录（双层限流 + 锁定审计）、刷新轮换、登出、请求认证。
//!
//! 规格：`docs/spec/A-auth.md` A9–A23。
//!
//! 限流语义（与旧实现的差异记在 A23）：
//! - IP 层：60 秒内失败次数 ≥5 → 下一次即 `429`（设计 §7「IP ≤5 次/60s」）
//! - 用户名层：300 秒内失败 10 次 → 锁 300 秒（从最后一次失败起算）→ `429`
//! - 只有用户名层「恰好第 10 次失败」写 1 条 `login_lockout` 审计（`user_id = NULL`）；
//!   前 9 次、锁定期内的 429、IP 层 429 都不写
//! - 登录成功清空该用户名的失败记录，不写审计

use rand_core::{OsRng, RngCore};
use serde::Serialize;
use steelpipe_audit::detail;
use steelpipe_auth::{AuthError, Claims, Keys, hash_password, verify_password};
use steelpipe_domain::Role;
use steelpipe_infra::{
    DbPool, InfraError,
    repo::{login, users},
    time,
    uow::Uow,
};

use crate::actions;

/// IP 层窗口与阈值。
pub const IP_WINDOW_SECS: i64 = 60;
pub const IP_MAX_FAILURES: i64 = 5;
/// 用户名层窗口、阈值与锁定时长。
pub const USER_WINDOW_SECS: i64 = 300;
pub const USER_MAX_FAILURES: i64 = 10;
pub const USER_LOCK_SECS: i64 = 300;

#[derive(Debug, thiserror::Error)]
pub enum AuthServiceError {
    #[error("用户名或口令错误")]
    InvalidCredentials,
    #[error("账号已停用")]
    Inactive,
    #[error("未认证")]
    Unauthenticated,
    #[error("refresh token 无效或已被轮换")]
    InvalidRefresh,
    #[error("尝试过于频繁（{scope:?}），{retry_after_secs} 秒后重试")]
    RateLimited {
        scope: LimitScope,
        retry_after_secs: i64,
    },
    #[error(transparent)]
    Infra(#[from] InfraError),
    #[error(transparent)]
    Auth(#[from] AuthError),
}

/// 兼容短名。
pub type LoginError = AuthServiceError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LimitScope {
    Ip,
    Username,
}

#[derive(Debug, Clone)]
pub struct Session {
    pub access_token: String,
    pub access_expires_at: u64,
    pub refresh_token: String,
    pub refresh_expires_at: u64,
    /// refresh token 的 jti，登录/轮换后写入 `users.last_refresh_jti`。
    pub refresh_jti: String,
    /// CSRF double-submit token（非 httpOnly cookie 分发给浏览器）。
    pub csrf_token: String,
}

#[derive(Debug, Clone)]
pub struct Identity {
    pub user_id: i64,
    pub username: String,
    pub display_name: String,
    pub role: Role,
    pub token_version: i64,
}

#[derive(Debug, Clone)]
pub struct LoginSuccess {
    pub identity: Identity,
    pub session: Session,
}

/// 认证用例集合。持有连接池与 JWT 密钥，无全局状态。
pub struct AuthService {
    pool: DbPool,
    keys: Keys,
}

impl AuthService {
    pub fn new(pool: DbPool, keys: Keys) -> Self {
        Self { pool, keys }
    }

    pub fn keys(&self) -> &Keys {
        &self.keys
    }

    fn issue_session(
        &self,
        user_id: i64,
        role: Role,
        token_version: i64,
    ) -> Result<Session, AuthServiceError> {
        let access = self.keys.issue_access(user_id, role, token_version)?;
        let refresh = self.keys.issue_refresh(user_id, token_version)?;
        let refresh_jti = refresh.jti.clone().ok_or(AuthError::InvalidToken)?;
        Ok(Session {
            access_token: access.token,
            access_expires_at: access.expires_at,
            refresh_token: refresh.token,
            refresh_expires_at: refresh.expires_at,
            refresh_jti,
            csrf_token: random_hex(),
        })
    }

    async fn ip_guard(&self, ip: &str) -> Result<(), AuthServiceError> {
        let since = time::secs_ago(IP_WINDOW_SECS);
        let failures = login::failures_since(&self.pool, login::SCOPE_IP, ip, &since).await?;
        if failures >= IP_MAX_FAILURES {
            return Err(AuthServiceError::RateLimited {
                scope: LimitScope::Ip,
                retry_after_secs: IP_WINDOW_SECS,
            });
        }
        Ok(())
    }

    async fn username_guard(&self, username: &str) -> Result<(), AuthServiceError> {
        let since = time::secs_ago(USER_WINDOW_SECS);
        let failures =
            login::failures_since(&self.pool, login::SCOPE_USERNAME, username, &since).await?;
        if failures < USER_MAX_FAILURES {
            return Ok(());
        }
        let locked_until =
            match login::last_failure_since(&self.pool, login::SCOPE_USERNAME, username, &since)
                .await?
            {
                Some(last) => lock_deadline(&last),
                None => String::new(),
            };
        if time::now_rfc3339() < locked_until {
            return Err(AuthServiceError::RateLimited {
                scope: LimitScope::Username,
                retry_after_secs: USER_LOCK_SECS,
            });
        }
        Ok(())
    }

    /// 记一次失败；恰好触发用户名层锁定阈值时，在同一事务里写 1 条审计。
    async fn record_failure(&self, username: &str, ip: &str) -> Result<(), AuthServiceError> {
        let mut uow = Uow::begin(&self.pool).await?;
        let now = time::now_rfc3339();
        login::record(uow.conn(), login::SCOPE_IP, ip, &now, false).await?;
        login::record(uow.conn(), login::SCOPE_USERNAME, username, &now, false).await?;

        let since = time::secs_ago(USER_WINDOW_SECS);
        let failures =
            login::failures_since(uow.conn(), login::SCOPE_USERNAME, username, &since).await?;
        if failures == USER_MAX_FAILURES {
            let payload = LockoutDetail {
                username,
                attempts: failures,
                ip,
                locked_secs: USER_LOCK_SECS,
            };
            let body = detail(&payload).map_err(|e| AuthError::Sign(e.to_string()))?;
            uow.audit(None, actions::LOGIN_LOCKOUT, &body).await?;
        }
        uow.commit().await?;
        Ok(())
    }

    /// 登录。失败路径都落限流记录，只有锁定阈值那一次写审计。
    pub async fn login(
        &self,
        username: &str,
        password: &str,
        ip: &str,
    ) -> Result<LoginSuccess, AuthServiceError> {
        let username = username.trim();
        self.ip_guard(ip).await?;
        self.username_guard(username).await?;

        let row = users::find_by_username(&self.pool, username).await?;

        // 用户不存在时也跑一次校验，避免用响应时间探测用户名。
        let password_ok = match &row {
            Some(user) => verify_password(password, &user.password_hash),
            None => {
                verify_password(password, dummy_hash());
                false
            }
        };

        let Some(user) = row else {
            self.record_failure(username, ip).await?;
            return Err(AuthServiceError::InvalidCredentials);
        };

        if !password_ok {
            self.record_failure(username, ip).await?;
            return Err(AuthServiceError::InvalidCredentials);
        }
        if !user.active {
            return Err(AuthServiceError::Inactive);
        }
        let Some(role) = user.role() else {
            return Err(AuthError::InvalidRole(user.role.clone()).into());
        };

        let session = self.issue_session(user.id, role, user.token_version)?;

        let mut uow = Uow::begin(&self.pool).await?;
        login::clear_failures(uow.conn(), username).await?;
        users::set_last_refresh_jti(uow.conn(), user.id, Some(&session.refresh_jti)).await?;
        uow.commit().await?;

        Ok(LoginSuccess {
            identity: Identity {
                user_id: user.id,
                username: user.username,
                display_name: user.display_name,
                role,
                token_version: user.token_version,
            },
            session,
        })
    }

    /// 刷新轮换（A15/A16）：presented refresh 必须与 DB 中的 jti 一致。
    pub async fn refresh(&self, presented_refresh: &str) -> Result<LoginSuccess, AuthServiceError> {
        let claims = self
            .keys
            .decode(presented_refresh)
            .map_err(|_| AuthServiceError::InvalidRefresh)?;
        let user_id = claims
            .user_id()
            .map_err(|_| AuthServiceError::InvalidRefresh)?;
        let presented_jti = claims.jti.clone().ok_or(AuthServiceError::InvalidRefresh)?;

        let user = users::find_by_id(&self.pool, user_id)
            .await?
            .ok_or(AuthServiceError::InvalidRefresh)?;

        // 三种失效：会话代数已吊销 / 该用户无 refresh / jti 已被轮换
        if claims.token_version != user.token_version
            || user.last_refresh_jti.as_deref() != Some(presented_jti.as_str())
        {
            return Err(AuthServiceError::InvalidRefresh);
        }
        if !user.active {
            return Err(AuthServiceError::Inactive);
        }
        let role = user.role().ok_or(AuthServiceError::InvalidRefresh)?;

        let session = self.issue_session(user.id, role, user.token_version)?;
        let mut uow = Uow::begin(&self.pool).await?;
        users::set_last_refresh_jti(uow.conn(), user.id, Some(&session.refresh_jti)).await?;
        uow.commit().await?;

        Ok(LoginSuccess {
            identity: Identity {
                user_id: user.id,
                username: user.username,
                display_name: user.display_name,
                role,
                token_version: user.token_version,
            },
            session,
        })
    }

    /// 登出（A14）：清空 `last_refresh_jti`，旧 refresh 立刻失效。
    /// 无 refresh cookie 视为已登出（幂等），不报错。
    pub async fn logout(&self, presented_refresh: Option<&str>) -> Result<(), AuthServiceError> {
        let Some(token) = presented_refresh else {
            return Ok(());
        };
        let Ok(claims) = self.keys.decode(token) else {
            return Ok(());
        };
        let Ok(user_id) = claims.user_id() else {
            return Ok(());
        };

        let mut uow = Uow::begin(&self.pool).await?;
        users::set_last_refresh_jti(uow.conn(), user_id, None).await?;
        uow.commit().await?;
        Ok(())
    }

    /// 请求认证（A9）：签名/过期/token_version/active 全部通过才算认证。
    pub async fn authenticate(&self, access_token: &str) -> Result<Identity, AuthServiceError> {
        let claims: Claims = self
            .keys
            .decode(access_token)
            .map_err(|_| AuthServiceError::Unauthenticated)?;
        let user_id = claims
            .user_id()
            .map_err(|_| AuthServiceError::Unauthenticated)?;

        let user = users::find_by_id(&self.pool, user_id)
            .await?
            .ok_or(AuthServiceError::Unauthenticated)?;
        if !user.active || claims.token_version != user.token_version {
            return Err(AuthServiceError::Unauthenticated);
        }
        let role = claims.role.ok_or(AuthServiceError::Unauthenticated)?;

        Ok(Identity {
            user_id: user.id,
            username: user.username,
            display_name: user.display_name,
            role,
            token_version: user.token_version,
        })
    }
}

/// 锁定截止时刻 = 最后一次失败 + 锁定时长（RFC3339 微秒格式可直接字典序比较）。
fn lock_deadline(last_failure: &str) -> String {
    use chrono::{DateTime, Utc};
    let parsed = DateTime::parse_from_rfc3339(last_failure)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now());
    (parsed + chrono::Duration::seconds(USER_LOCK_SECS))
        .to_rfc3339_opts(chrono::SecondsFormat::Micros, false)
}

/// 32 字节随机数的十六进制（CSRF token）。
fn random_hex() -> String {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// 用户不存在时用于对齐耗时的哑哈希（进程内只算一次）。
fn dummy_hash() -> &'static str {
    static DUMMY: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    DUMMY.get_or_init(|| {
        hash_password("steelpipe-dummy-password-for-timing-parity").unwrap_or_else(|_| {
            "$argon2id$v=19$m=19456,t=2,p=1$ZHVtbXlzYWx0$ZHVtbXloYXNo".to_owned()
        })
    })
}

#[derive(Debug, Serialize)]
struct LockoutDetail<'a> {
    username: &'a str,
    attempts: i64,
    ip: &'a str,
    locked_secs: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use steelpipe_audit::verify_chunk;
    use steelpipe_infra::repo::audit as audit_repo;

    struct Fixture {
        pool: DbPool,
        path: std::path::PathBuf,
        service: AuthService,
    }

    async fn fixture(tag: &str) -> Fixture {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("steelpipe-svc-{tag}-{nanos}.db"));
        let pool = steelpipe_infra::connect(&format!("sqlite://{}", path.display()), 8)
            .await
            .expect("打开临时库失败");
        steelpipe_infra::migrate(&pool).await.expect("迁移失败");
        let service = AuthService::new(
            pool.clone(),
            Keys::from_secret(b"service-test-secret-32-bytes-min!!"),
        );
        Fixture {
            pool,
            path,
            service,
        }
    }

    async fn seed_user(pool: &DbPool, username: &str, password: &str, role: Role) {
        let hash = hash_password(password).expect("哈希失败");
        let mut uow = Uow::begin(pool).await.unwrap();
        users::insert(
            uow.conn(),
            username,
            &hash,
            username,
            role,
            &time::now_rfc3339(),
        )
        .await
        .unwrap();
        uow.commit().await.unwrap();
    }

    async fn audit_count(pool: &DbPool) -> i64 {
        audit_repo::count(pool).await.unwrap()
    }

    #[tokio::test]
    async fn login_success_issues_session_and_records_jti() {
        let fx = fixture("login-ok").await;
        seed_user(&fx.pool, "alice", "pw-alice", Role::Operator).await;

        let out = fx
            .service
            .login("alice", "pw-alice", "10.0.0.1")
            .await
            .expect("登录应成功");
        assert_eq!(out.identity.username, "alice");
        assert_eq!(out.identity.role, Role::Operator);
        assert_eq!(out.session.csrf_token.len(), 64);
        assert!(!out.session.access_token.is_empty());

        let row = users::find_by_username(&fx.pool, "alice")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            row.last_refresh_jti.as_deref(),
            Some(out.session.refresh_jti.as_str())
        );
        assert_eq!(audit_count(&fx.pool).await, 0, "成功登录不写审计");

        let identity = fx
            .service
            .authenticate(&out.session.access_token)
            .await
            .expect("access 应有效");
        assert_eq!(identity.user_id, out.identity.user_id);

        fx.pool.close().await;
        let _ = std::fs::remove_file(&fx.path);
    }

    #[tokio::test]
    async fn wrong_password_records_failure_without_audit() {
        let fx = fixture("login-bad").await;
        seed_user(&fx.pool, "bob", "pw-bob", Role::Viewer).await;

        let err = fx
            .service
            .login("bob", "wrong", "10.0.0.2")
            .await
            .unwrap_err();
        assert!(matches!(err, AuthServiceError::InvalidCredentials));
        assert_eq!(audit_count(&fx.pool).await, 0);

        let since = time::secs_ago(USER_WINDOW_SECS);
        let failures = login::failures_since(&fx.pool, login::SCOPE_USERNAME, "bob", &since)
            .await
            .unwrap();
        assert_eq!(failures, 1);

        fx.pool.close().await;
        let _ = std::fs::remove_file(&fx.path);
    }

    #[tokio::test]
    async fn ninth_failure_writes_no_audit_tenth_writes_exactly_one() {
        let fx = fixture("lockout").await;
        seed_user(&fx.pool, "carol", "pw-carol", Role::Operator).await;

        // 每次用不同 IP，避免 IP 层先触发（A20 的场景前提）
        for i in 0..9 {
            let err = fx
                .service
                .login("carol", "nope", &format!("10.1.0.{i}"))
                .await
                .unwrap_err();
            assert!(matches!(err, AuthServiceError::InvalidCredentials));
        }
        assert_eq!(audit_count(&fx.pool).await, 0, "前 9 次失败不写审计");

        let err = fx
            .service
            .login("carol", "nope", "10.1.0.9")
            .await
            .unwrap_err();
        assert!(matches!(err, AuthServiceError::InvalidCredentials));
        assert_eq!(audit_count(&fx.pool).await, 1, "第 10 次恰好写 1 条");

        let row = audit_repo::page_after(&fx.pool, 0, 10)
            .await
            .unwrap()
            .remove(0);
        assert_eq!(row.action, actions::LOGIN_LOCKOUT);
        assert_eq!(row.user_id, None);
        assert!(row.detail.contains("carol"), "实际：{}", row.detail);

        fx.pool.close().await;
        let _ = std::fs::remove_file(&fx.path);
    }

    #[tokio::test]
    async fn locked_attempts_are_rate_limited_and_write_no_audit() {
        let fx = fixture("locked").await;
        seed_user(&fx.pool, "dave", "pw-dave", Role::Operator).await;
        for i in 0..10 {
            let _ = fx
                .service
                .login("dave", "nope", &format!("10.2.0.{i}"))
                .await;
        }
        assert_eq!(audit_count(&fx.pool).await, 1);

        // 锁定期内：即使口令正确也 429，且不再写审计
        for i in 0..3 {
            let err = fx
                .service
                .login("dave", "pw-dave", &format!("10.2.1.{i}"))
                .await
                .unwrap_err();
            assert!(
                matches!(
                    err,
                    AuthServiceError::RateLimited {
                        scope: LimitScope::Username,
                        ..
                    }
                ),
                "实际：{err:?}"
            );
        }
        assert_eq!(audit_count(&fx.pool).await, 1, "锁定期内不新增审计");

        fx.pool.close().await;
        let _ = std::fs::remove_file(&fx.path);
    }

    #[tokio::test]
    async fn ip_tier_trips_before_username_tier() {
        let fx = fixture("ip-tier").await;
        // 6 个不同用户名、同一 IP：第 6 次应被 IP 层拦下（用户名层各只有 1 次失败）
        for i in 0..5 {
            let err = fx
                .service
                .login(&format!("ghost{i}"), "nope", "10.3.0.1")
                .await
                .unwrap_err();
            assert!(matches!(err, AuthServiceError::InvalidCredentials));
        }
        let err = fx
            .service
            .login("ghost5", "nope", "10.3.0.1")
            .await
            .unwrap_err();
        assert!(
            matches!(
                err,
                AuthServiceError::RateLimited {
                    scope: LimitScope::Ip,
                    ..
                }
            ),
            "实际：{err:?}"
        );
        assert_eq!(audit_count(&fx.pool).await, 0, "IP 层 429 不写审计");

        fx.pool.close().await;
        let _ = std::fs::remove_file(&fx.path);
    }

    #[tokio::test]
    async fn successful_login_clears_failures_without_audit() {
        let fx = fixture("clear").await;
        seed_user(&fx.pool, "erin", "pw-erin", Role::Operator).await;
        for i in 0..3 {
            let _ = fx
                .service
                .login("erin", "nope", &format!("10.4.0.{i}"))
                .await;
        }
        fx.service
            .login("erin", "pw-erin", "10.4.0.9")
            .await
            .expect("登录应成功");
        let since = time::secs_ago(USER_WINDOW_SECS);
        let failures = login::failures_since(&fx.pool, login::SCOPE_USERNAME, "erin", &since)
            .await
            .unwrap();
        assert_eq!(failures, 0, "成功登录必须清空失败记录");
        assert_eq!(audit_count(&fx.pool).await, 0);

        fx.pool.close().await;
        let _ = std::fs::remove_file(&fx.path);
    }

    #[tokio::test]
    async fn inactive_user_is_rejected() {
        let fx = fixture("inactive").await;
        seed_user(&fx.pool, "frank", "pw-frank", Role::Viewer).await;
        let mut uow = Uow::begin(&fx.pool).await.unwrap();
        users::set_active(uow.conn(), 1, false).await.unwrap();
        uow.commit().await.unwrap();

        let err = fx
            .service
            .login("frank", "pw-frank", "10.5.0.1")
            .await
            .unwrap_err();
        assert!(matches!(err, AuthServiceError::Inactive));

        fx.pool.close().await;
        let _ = std::fs::remove_file(&fx.path);
    }

    #[tokio::test]
    async fn refresh_rotates_and_rejects_the_old_token() {
        let fx = fixture("rotate").await;
        seed_user(&fx.pool, "gina", "pw-gina", Role::Operator).await;
        let first = fx
            .service
            .login("gina", "pw-gina", "10.6.0.1")
            .await
            .unwrap();

        let rotated = fx
            .service
            .refresh(&first.session.refresh_token)
            .await
            .expect("刷新应成功");
        assert_ne!(rotated.session.refresh_jti, first.session.refresh_jti);

        let err = fx
            .service
            .refresh(&first.session.refresh_token)
            .await
            .unwrap_err();
        assert!(
            matches!(err, AuthServiceError::InvalidRefresh),
            "旧 refresh 必须失效"
        );

        let row = users::find_by_username(&fx.pool, "gina")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            row.last_refresh_jti.as_deref(),
            Some(rotated.session.refresh_jti.as_str())
        );

        fx.pool.close().await;
        let _ = std::fs::remove_file(&fx.path);
    }

    #[tokio::test]
    async fn logout_invalidates_refresh() {
        let fx = fixture("logout").await;
        seed_user(&fx.pool, "hank", "pw-hank", Role::Operator).await;
        let out = fx
            .service
            .login("hank", "pw-hank", "10.7.0.1")
            .await
            .unwrap();

        fx.service
            .logout(Some(&out.session.refresh_token))
            .await
            .expect("登出应成功");
        let err = fx
            .service
            .refresh(&out.session.refresh_token)
            .await
            .unwrap_err();
        assert!(matches!(err, AuthServiceError::InvalidRefresh));

        // 幂等：再次登出不报错
        fx.service
            .logout(Some(&out.session.refresh_token))
            .await
            .expect("重复登出应幂等");
        fx.service.logout(None).await.expect("无 cookie 登出应幂等");

        fx.pool.close().await;
        let _ = std::fs::remove_file(&fx.path);
    }

    #[tokio::test]
    async fn authenticate_rejects_stale_token_version() {
        let fx = fixture("version").await;
        seed_user(&fx.pool, "ivy", "pw-ivy", Role::Operator).await;
        let out = fx.service.login("ivy", "pw-ivy", "10.8.0.1").await.unwrap();

        let mut uow = Uow::begin(&fx.pool).await.unwrap();
        users::revoke_all_sessions(uow.conn(), out.identity.user_id)
            .await
            .unwrap();
        uow.commit().await.unwrap();

        let err = fx
            .service
            .authenticate(&out.session.access_token)
            .await
            .unwrap_err();
        assert!(
            matches!(err, AuthServiceError::Unauthenticated),
            "吊销后旧 access 必须失效"
        );

        fx.pool.close().await;
        let _ = std::fs::remove_file(&fx.path);
    }

    #[tokio::test]
    async fn chain_still_verifies_after_lockout() {
        let fx = fixture("chain").await;
        seed_user(&fx.pool, "joe", "pw-joe", Role::Operator).await;
        for i in 0..10 {
            let _ = fx
                .service
                .login("joe", "nope", &format!("10.9.0.{i}"))
                .await;
        }
        assert_eq!(audit_count(&fx.pool).await, 1);

        let rows = audit_repo::page_after(&fx.pool, 0, 100).await.unwrap();
        let entries: Vec<steelpipe_audit::Entry<'_>> =
            rows.iter().map(|row| row.as_entry()).collect();
        let tip =
            verify_chunk(steelpipe_audit::GENESIS_HASH, &entries).expect("锁定事件后链应可验证");
        assert_eq!(tip, rows.last().unwrap().current_hash);

        fx.pool.close().await;
        let _ = std::fs::remove_file(&fx.path);
    }
}
