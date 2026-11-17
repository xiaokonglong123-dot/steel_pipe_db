//! 认证：argon2id 密码哈希 + HS256 JWT 签发/校验。
//!
//! 搬自旧 `rust-core/src/auth.rs`（739/776 行可搬率的一部分），差异：
//!
//! - 不再用 `static OnceLock` 存密钥。密钥由 [`Keys`] 显式持有并注入，
//!   消除 `init()` 调用顺序依赖，也让测试可以构造任意密钥。
//! - 返回 `Result<_, AuthError>` 而不是 pyo3 异常。
//! - `token_version` 只做解码透出，是否匹配由调用方（中间件）比对 DB 值 —— 与原实现一致。
//!
//! 密码学参数与旧实现一致：Argon2id 默认参数（m=19456KiB, t=2, p=1）；
//! access 15 分钟，refresh 30 天。

#![forbid(unsafe_code)]

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use argon2::Argon2;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use steelpipe_domain::Role;

/// access token 有效期（设计：15 分钟）。
pub const ACCESS_TTL: Duration = Duration::from_secs(15 * 60);
/// refresh token 有效期（设计：30 天）。
pub const REFRESH_TTL: Duration = Duration::from_secs(30 * 24 * 60 * 60);

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("密码哈希失败：{0}")]
    Hash(String),
    #[error("token 签发失败：{0}")]
    Sign(String),
    #[error("token 无效：签名、格式或已过期")]
    InvalidToken,
    #[error("token 的 sub 不是合法用户 id")]
    InvalidSubject,
    #[error("token 的角色非法：{0}")]
    InvalidRole(String),
}

/// 签发与校验 token 所需的密钥对。
#[derive(Clone)]
pub struct Keys {
    encoding: EncodingKey,
    decoding: DecodingKey,
}

impl Keys {
    pub fn from_secret(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }
}

/// 手写 Debug：密钥永不进日志。
impl std::fmt::Debug for Keys {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Keys(<secret>)")
    }
}

/// JWT 载荷。access 与 refresh 共用同一结构（与原实现一致）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// 用户 id，按 JWT 规范序列化为字符串。
    pub sub: String,
    /// 角色；refresh token 为空。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<Role>,
    /// 会话代数；吊销全体会话时递增。
    pub token_version: i64,
    /// refresh token 的会话标识；access token 无。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jti: Option<String>,
    /// 过期时间（Unix 秒）。
    pub exp: u64,
}

impl Claims {
    pub fn user_id(&self) -> Result<i64, AuthError> {
        self.sub
            .parse::<i64>()
            .map_err(|_| AuthError::InvalidSubject)
    }

    pub fn expires_at(&self) -> u64 {
        self.exp
    }
}

/// 签发结果：token 字符串 + 需要落库/回写的元数据。
#[derive(Debug, Clone)]
pub struct IssuedToken {
    pub token: String,
    pub expires_at: u64,
    /// 仅 refresh token 有值，用于 `users.last_refresh_jti` 单会话轮换。
    pub jti: Option<String>,
}

/// argon2id 哈希口令。
pub fn hash_password(password: &str) -> Result<String, AuthError> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| AuthError::Hash(e.to_string()))
}

/// 校验口令。任何异常（错误口令、损坏的哈希）都返回 `false`，绝不 panic。
pub fn verify_password(password: &str, phc_hash: &str) -> bool {
    let Ok(parsed) = PasswordHash::new(phc_hash) else {
        return false;
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok()
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

impl Keys {
    /// 签发任意载荷（`exp` 由调用方给定）。`issue_*` 之外的唯一低层入口。
    pub fn sign(&self, claims: &Claims) -> Result<String, AuthError> {
        encode(&Header::new(Algorithm::HS256), claims, &self.encoding)
            .map_err(|e| AuthError::Sign(e.to_string()))
    }

    /// 校验签名与过期时间，返回载荷。不接受 `alg=none` 或其他算法。
    ///
    /// `leeway` 显式设为 0：jsonwebtoken 默认给 60 秒宽限，会让「已过期」
    /// 的语义漂移，也让边界测不出来。
    pub fn decode(&self, token: &str) -> Result<Claims, AuthError> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.leeway = 0;
        let data = decode::<Claims>(token, &self.decoding, &validation)
            .map_err(|_| AuthError::InvalidToken)?;
        Ok(data.claims)
    }

    /// 签发 access token（15 分钟）。
    pub fn issue_access(
        &self,
        user_id: i64,
        role: Role,
        token_version: i64,
    ) -> Result<IssuedToken, AuthError> {
        self.issue_access_with_ttl(user_id, role, token_version, ACCESS_TTL)
    }

    /// 签发 refresh token（30 天，带 jti）。
    pub fn issue_refresh(
        &self,
        user_id: i64,
        token_version: i64,
    ) -> Result<IssuedToken, AuthError> {
        self.issue_refresh_with_ttl(user_id, token_version, REFRESH_TTL)
    }

    pub fn issue_access_with_ttl(
        &self,
        user_id: i64,
        role: Role,
        token_version: i64,
        ttl: Duration,
    ) -> Result<IssuedToken, AuthError> {
        let exp = now_secs() + ttl.as_secs();
        let claims = Claims {
            sub: user_id.to_string(),
            role: Some(role),
            token_version,
            jti: None,
            exp,
        };
        Ok(IssuedToken {
            token: self.sign(&claims)?,
            expires_at: exp,
            jti: None,
        })
    }

    pub fn issue_refresh_with_ttl(
        &self,
        user_id: i64,
        token_version: i64,
        ttl: Duration,
    ) -> Result<IssuedToken, AuthError> {
        let exp = now_secs() + ttl.as_secs();
        let jti = Uuid::new_v4().to_string();
        let claims = Claims {
            sub: user_id.to_string(),
            role: None,
            token_version,
            jti: Some(jti.clone()),
            exp,
        };
        Ok(IssuedToken {
            token: self.sign(&claims)?,
            expires_at: exp,
            jti: Some(jti),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keys() -> Keys {
        Keys::from_secret(b"test-secret-at-least-32-bytes-long!!")
    }

    #[test]
    fn phc_hash_is_argon2id_with_design_parameters() {
        let phc = hash_password("correct horse battery staple").expect("哈希失败");
        assert!(
            phc.starts_with("$argon2id$v=19$m=19456,t=2,p=1$"),
            "实际：{phc}"
        );
    }

    #[test]
    fn verify_accepts_correct_and_rejects_everything_else() {
        let phc = hash_password("s3cret").expect("哈希失败");
        assert!(verify_password("s3cret", &phc));
        assert!(!verify_password("s3cret ", &phc));
        assert!(!verify_password("", &phc));
        assert!(!verify_password("s3cret", "not-a-phc-hash"));
        assert!(!verify_password("s3cret", ""));
    }

    #[test]
    fn hashes_are_salted_independently() {
        let a = hash_password("same").expect("哈希失败");
        let b = hash_password("same").expect("哈希失败");
        assert_ne!(a, b, "同口令两次哈希必须不同（随机盐）");
        assert!(verify_password("same", &a) && verify_password("same", &b));
    }

    #[test]
    fn access_token_roundtrip() {
        let issued = keys()
            .issue_access(42, Role::Operator, 7)
            .expect("签发失败");
        let claims = keys().decode(&issued.token).expect("解码失败");
        assert_eq!(claims.user_id().unwrap(), 42);
        assert_eq!(claims.role, Some(Role::Operator));
        assert_eq!(claims.token_version, 7);
        assert_eq!(claims.jti, None);
        assert_eq!(issued.jti, None);
        assert!(claims.expires_at() >= now_secs() + ACCESS_TTL.as_secs() - 1);
    }

    #[test]
    fn refresh_token_carries_jti_and_no_role() {
        let issued = keys().issue_refresh(42, 3).expect("签发失败");
        let claims = keys().decode(&issued.token).expect("解码失败");
        assert_eq!(claims.role, None);
        assert!(claims.jti.is_some());
        assert_eq!(claims.jti, issued.jti);
        assert!(claims.expires_at() >= now_secs() + REFRESH_TTL.as_secs() - 1);
    }

    #[test]
    fn expired_token_is_rejected() {
        let expired = Claims {
            sub: "1".into(),
            role: Some(Role::Admin),
            token_version: 0,
            jti: None,
            exp: now_secs() - 60,
        };
        let token = keys().sign(&expired).expect("签发失败");
        assert!(matches!(
            keys().decode(&token),
            Err(AuthError::InvalidToken)
        ));
    }

    #[test]
    fn tampered_or_foreign_signed_token_is_rejected() {
        let issued = keys().issue_access(1, Role::Admin, 0).expect("签发失败");
        let mut tampered = issued.token.clone().into_bytes();
        let last = tampered.len() - 2;
        tampered[last] = if tampered[last] == b'A' { b'B' } else { b'A' };
        let tampered = String::from_utf8(tampered).expect("utf8");
        assert!(keys().decode(&tampered).is_err());

        let foreign = Keys::from_secret(b"another-secret-at-least-32-bytes!!")
            .issue_access(1, Role::Admin, 0)
            .expect("签发失败");
        assert!(keys().decode(&foreign.token).is_err());
    }

    #[test]
    fn garbage_tokens_are_rejected_not_panicked() {
        for token in ["", "a.b.c", "not-a-token", "eyJhbGciOiJub25lIn0.e30."] {
            assert!(keys().decode(token).is_err(), "token 不应通过：{token}");
        }
    }

    #[test]
    fn invalid_subject_is_reported() {
        let claims = Claims {
            sub: "not-a-number".into(),
            role: Some(Role::Viewer),
            token_version: 0,
            jti: None,
            exp: now_secs() + 60,
        };
        let token = keys().sign(&claims).expect("签发失败");
        let decoded = keys().decode(&token).expect("解码失败");
        assert!(matches!(decoded.user_id(), Err(AuthError::InvalidSubject)));
    }
}
