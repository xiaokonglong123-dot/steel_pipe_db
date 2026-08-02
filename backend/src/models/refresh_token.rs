use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Refresh token DB row. Server-side session token for issuing new access tokens.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RefreshToken {
    pub id: i64,
    /// FK to users table.
    pub user_id: i64,
    /// SHA-256 hash of the opaque refresh token.
    pub token_hash: String,
    /// ISO 8601 expiry timestamp.
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    /// Non-null if this token has been revoked (used, expired, or logged out).
    pub revoked_at: Option<String>,
}
