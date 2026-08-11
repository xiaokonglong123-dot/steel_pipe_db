//! 环境配置 — 对齐 detailed-design §8
//!
//! 所有配置从环境变量读取（支持 .env 文件，dotenvy）。

use std::net::SocketAddr;

use crate::error::{AppError, ErrorCode};

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_expiry_hours: u64,
    pub refresh_expiry_days: u64,
    pub server_host: String,
    pub server_port: u16,
    pub cors_origins: Vec<String>,
    pub admin_username: String,
    pub admin_password: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            database_url: "sqlite://data/erp.db?mode=rwc".to_string(),
            jwt_secret: "dev-only-insecure-secret-change-me".to_string(),
            jwt_expiry_hours: 24,
            refresh_expiry_days: 7,
            server_host: "0.0.0.0".to_string(),
            server_port: 3000,
            cors_origins: vec!["http://localhost:5173".to_string()],
            admin_username: "admin".to_string(),
            admin_password: "admin123".to_string(),
        }
    }
}

impl Config {
    pub fn from_env() -> Result<Self, AppError> {
        let _ = dotenvy::dotenv();

        let c = Self::default();

        let database_url = std::env::var("DATABASE_URL").unwrap_or(c.database_url.clone());
        let default_jwt = c.jwt_secret.clone();
        let jwt_secret = std::env::var("JWT_SECRET").unwrap_or(default_jwt.clone());
        let jwt_expiry_hours = std::env::var("JWT_EXPIRY_HOURS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(c.jwt_expiry_hours);
        let refresh_expiry_days = std::env::var("REFRESH_EXPIRY_DAYS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(c.refresh_expiry_days);
        let server_host = std::env::var("SERVER_HOST").unwrap_or(c.server_host.clone());
        let server_port = std::env::var("SERVER_PORT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(c.server_port);
        let default_cors = c.cors_origins.clone();
        let cors_origins: Vec<String> = std::env::var("CORS_ORIGINS")
            .map(|v| v.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_else(|_| default_cors.clone());
        let admin_username = std::env::var("ADMIN_USERNAME").unwrap_or(c.admin_username.clone());
        let admin_password = std::env::var("ADMIN_PASSWORD").unwrap_or(c.admin_password.clone());

        if jwt_secret == default_jwt {
            tracing::warn!("JWT_SECRET 未设置，使用不安全的开发默认值");
        }

        Ok(Self {
            database_url,
            jwt_secret,
            jwt_expiry_hours,
            refresh_expiry_days,
            server_host,
            server_port,
            cors_origins: if cors_origins.is_empty() {
                default_cors
            } else {
                cors_origins
            },
            admin_username,
            admin_password,
        })
    }

    pub fn server_addr(&self) -> Result<SocketAddr, AppError> {
        format!("{}:{}", self.server_host, self.server_port)
            .parse()
            .map_err(|e| AppError::new(ErrorCode::Config, format!("invalid server addr: {e}")))
    }
}
