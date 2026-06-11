use axum::http::HeaderValue;
use std::env;
use std::sync::Once;

static WEAK_SECRET_WARNING: Once = Once::new();

/// Placeholder JWT secret shipped in `.env.example`. Treated as "unset" — never
/// allowed to sign tokens in production (fail-closed) and warned about in development.
const PLACEHOLDER_JWT_SECRET: &str = "change-this-to-a-long-random-secret";

/// Minimum acceptable JWT secret length (bytes) enforced in production.
const MIN_JWT_SECRET_LEN: usize = 32;

/// Application configuration sourced from environment variables at startup.
/// All fields have sensible defaults for development — override via `.env` file.
#[derive(Clone, Debug)]
pub struct Config {
    /// Deployment environment: `development` (default) or `production`.
    /// In production, missing/placeholder/weak secrets cause a fail-closed panic.
    pub app_env: String,
    /// SQLite connection string (e.g., `sqlite://./data/steel_pipe.db?mode=rwc`).
    /// Default: `sqlite://./data/steel_pipe.db?mode=rwc` (auto-creates DB file).
    pub database_url: String,
    /// HMAC secret for signing and verifying JWT tokens.
    /// Default is a placeholder — must be changed in production.
    pub jwt_secret: String,
    /// Number of hours before issued JWT tokens expire.
    /// Default: 24 (one day).
    pub jwt_expiry_hours: i64,
    /// Number of days before refresh tokens expire.
    /// Default: 30.
    pub refresh_token_expiry_days: i64,
    /// Initial admin username for first-run bootstrap.
    /// Default: "admin". Only used when no users exist yet.
    pub admin_username: String,
    /// Initial admin password for first-run bootstrap.
    /// Default: "admin123". Only used when no users exist yet.
    pub admin_password: String,
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
        let app_env = env::var("APP_ENV")
            .unwrap_or_else(|_| "development".to_string())
            .to_lowercase();
        let is_production = app_env == "production" || app_env == "prod";

        let jwt_secret = Self::resolve_jwt_secret(is_production);

        Self {
            app_env,
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite://./data/steel_pipe.db?mode=rwc".to_string()),
            jwt_secret,
            jwt_expiry_hours: env::var("JWT_EXPIRY_HOURS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(24),
            refresh_token_expiry_days: env::var("REFRESH_TOKEN_EXPIRY_DAYS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(30),
            admin_username: env::var("ADMIN_USERNAME").unwrap_or_else(|_| "admin".to_string()),
            admin_password: env::var("ADMIN_PASSWORD").unwrap_or_else(|_| "admin123".to_string()),
            server_host: env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            server_port: env::var("SERVER_PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(3000),
            cors_origins: env::var("CORS_ORIGINS")
                .unwrap_or_else(|_| "http://localhost:5173".to_string()),
        }
    }

    /// Resolves the JWT secret with fail-closed semantics in production.
    ///
    /// Production (`APP_ENV=production`) panics if the secret is missing, equal to
    /// the `.env.example` placeholder, or shorter than [`MIN_JWT_SECRET_LEN`] bytes —
    /// a weak signing key lets attackers forge auth tokens, so refusing to boot is
    /// safer than silently running insecure. Development falls back to the placeholder
    /// with a warning to preserve `cp .env.example .env && cargo run` ergonomics.
    fn resolve_jwt_secret(is_production: bool) -> String {
        match env::var("JWT_SECRET") {
            Ok(secret)
                if secret != PLACEHOLDER_JWT_SECRET && secret.len() >= MIN_JWT_SECRET_LEN =>
            {
                secret
            }
            Ok(secret) if is_production => {
                let reason = if secret == PLACEHOLDER_JWT_SECRET {
                    "is set to the .env.example placeholder"
                } else {
                    "is shorter than the 32-byte minimum"
                };
                panic!(
                    "Refusing to start: JWT_SECRET {reason} while APP_ENV=production. \
                     Generate a strong secret (e.g. `openssl rand -base64 48`) and set JWT_SECRET."
                );
            }
            Err(_) if is_production => {
                panic!(
                    "Refusing to start: JWT_SECRET is not set while APP_ENV=production. \
                     Generate a strong secret (e.g. `openssl rand -base64 48`) and set JWT_SECRET."
                );
            }
            Ok(weak) => {
                WEAK_SECRET_WARNING.call_once(|| {
                    tracing::warn!(
                        "JWT_SECRET is weak or uses the placeholder value — acceptable for \
                         development only. Set APP_ENV=production with a strong secret before deploying."
                    );
                });
                weak
            }
            Err(_) => {
                WEAK_SECRET_WARNING.call_once(|| {
                    tracing::warn!(
                        "JWT_SECRET is not set — falling back to the insecure development placeholder. \
                         Set APP_ENV=production with a strong secret before deploying."
                    );
                });
                PLACEHOLDER_JWT_SECRET.to_string()
            }
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
