use axum::http::HeaderValue;
use std::env;

/// Application configuration sourced from environment variables at startup.
/// All fields have sensible defaults for development — override via `.env` file.
#[derive(Clone, Debug)]
pub struct Config {
    /// SQLite connection string (e.g., `sqlite://./data/steel_pipe.db?mode=rwc`).
    /// Default: `sqlite://./data/steel_pipe.db?mode=rwc` (auto-creates DB file).
    pub database_url: String,
    /// HMAC secret for signing and verifying JWT tokens.
    /// Default is a placeholder — must be changed in production.
    pub jwt_secret: String,
    /// Number of hours before issued JWT tokens expire.
    /// Default: 24 (one day).
    pub jwt_expiry_hours: i64,
    /// Network interface to bind the HTTP server to.
    /// Default: `0.0.0.0` (all interfaces).
    pub server_host: String,
    /// TCP port for the HTTP server.
    /// Default: 3000.
    pub server_port: u16,
    /// Comma-separated list of allowed CORS origins.
    /// Default: `http://localhost:5173` (Vite dev server).
    /// Production example: `https://pipe.example.com,https://pipe2.example.com`
    pub cors_origins: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite://./data/steel_pipe.db?mode=rwc".to_string()),
            jwt_secret: env::var("JWT_SECRET")
                .unwrap_or_else(|_| "change-this-to-a-long-random-secret".to_string()),
            jwt_expiry_hours: env::var("JWT_EXPIRY_HOURS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(24),
            server_host: env::var("SERVER_HOST")
                .unwrap_or_else(|_| "0.0.0.0".to_string()),
            server_port: env::var("SERVER_PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(3000),
            cors_origins: env::var("CORS_ORIGINS")
                .unwrap_or_else(|_| "http://localhost:5173".to_string()),
        }
    }

    pub fn server_addr(&self) -> String {
        format!("{}:{}", self.server_host, self.server_port)
    }

    /// Parse the comma-separated `cors_origins` into a `Vec<HeaderValue>`.
    /// Invalid origins are logged as warnings and skipped.
    pub fn parse_cors_origins(&self) -> Vec<HeaderValue> {
        self.cors_origins
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .filter_map(|origin| {
                origin.parse::<HeaderValue>().ok().or_else(|| {
                    tracing::warn!("Invalid CORS origin skipped: {}", origin);
                    None
                })
            })
            .collect()
    }
}
