//! 运行时配置。缺失关键配置时启动即失败（沿用原设计的 fail-fast 语义）。

use std::env;
use std::net::SocketAddr;

use anyhow::{Context, bail};

/// JWT 密钥最小长度（HMAC-SHA256 建议 ≥ 32 字节）。
const MIN_SECRET_LEN: usize = 32;

#[derive(Clone)]
pub struct Config {
    pub bind_addr: SocketAddr,
    pub database_url: String,
    pub jwt_secret: String,
    pub cookie_secure: bool,
    pub max_connections: u32,
    pub max_body_bytes: usize,
    pub admin_seed: Option<(String, String)>,
}

// Debug 手写：jwt_secret / admin 口令永不进日志。
impl std::fmt::Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Config")
            .field("bind_addr", &self.bind_addr)
            .field("database_url", &self.database_url)
            .field(
                "jwt_secret",
                &format_args!("<redacted, {} bytes>", self.jwt_secret.len()),
            )
            .field("cookie_secure", &self.cookie_secure)
            .field("max_connections", &self.max_connections)
            .field("max_body_bytes", &self.max_body_bytes)
            .field("admin_seed", &self.admin_seed.as_ref().map(|(u, _)| u))
            .finish()
    }
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let bind = env::var("STEELPIPE_BIND").unwrap_or_else(|_| "127.0.0.1:8080".to_owned());
        let bind_addr: SocketAddr = bind
            .parse()
            .with_context(|| format!("STEELPIPE_BIND 不是合法的监听地址：{bind}"))?;

        let database_url = env::var("STEELPIPE_DATABASE_URL")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "sqlite://data/steelpipe.db".to_owned());

        let jwt_secret = env::var("STEELPIPE_JWT_SECRET").unwrap_or_default();
        if jwt_secret.is_empty() {
            bail!("STEELPIPE_JWT_SECRET 未设置：本服务拒绝以空密钥启动（见设计 §9.4）");
        }
        if jwt_secret.len() < MIN_SECRET_LEN {
            bail!(
                "STEELPIPE_JWT_SECRET 长度 {} < {MIN_SECRET_LEN} 字节",
                jwt_secret.len()
            );
        }

        let cookie_secure = env::var("STEELPIPE_COOKIE_SECURE")
            .ok()
            .map(|value| matches!(value.trim(), "1" | "true" | "yes" | "on"))
            .unwrap_or(false);

        let max_connections = env::var("STEELPIPE_MAX_CONNECTIONS")
            .ok()
            .map(|value| value.parse::<u32>())
            .transpose()
            .context("STEELPIPE_MAX_CONNECTIONS 必须是整数")?
            .unwrap_or(10)
            .max(1);

        let max_body_bytes = env::var("STEELPIPE_MAX_BODY_BYTES")
            .ok()
            .map(|value| value.parse::<usize>())
            .transpose()
            .context("STEELPIPE_MAX_BODY_BYTES 必须是整数")?
            .unwrap_or(20 * 1024 * 1024);

        let admin_username = env::var("STEELPIPE_ADMIN_USERNAME").ok();
        let admin_password = env::var("STEELPIPE_ADMIN_PASSWORD").ok();
        let admin_seed = match (admin_username, admin_password) {
            (Some(username), Some(password))
                if !username.trim().is_empty() && !password.is_empty() =>
            {
                Some((username, password))
            }
            _ => None,
        };

        Ok(Self {
            bind_addr,
            database_url,
            jwt_secret,
            cookie_secure,
            max_connections,
            max_body_bytes,
            admin_seed,
        })
    }
}
