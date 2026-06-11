use sqlx::SqlitePool;

use crate::models::refresh_token::RefreshToken;

/// CRUD for `refresh_tokens`. Handles token storage, lookup, and revocation.
pub struct RefreshTokenRepo;

impl RefreshTokenRepo {
    /// INSERT a new refresh token. `token_hash` is the SHA-256 of the opaque token.
    pub async fn create(
        pool: &SqlitePool,
        user_id: i64,
        token_hash: &str,
        expires_at: &str,
    ) -> Result<RefreshToken, sqlx::Error> {
        sqlx::query_as::<_, RefreshToken>(
            "INSERT INTO refresh_tokens (user_id, token_hash, expires_at)
             VALUES (?, ?, ?)
             RETURNING id, user_id, token_hash, expires_at, created_at, revoked_at",
        )
        .bind(user_id)
        .bind(token_hash)
        .bind(expires_at)
        .fetch_one(pool)
        .await
    }

    /// Find a refresh token by its hash. Returns None if not found, revoked, or expired.
    pub async fn find_by_token_hash(
        pool: &SqlitePool,
        token_hash: &str,
    ) -> Result<Option<RefreshToken>, sqlx::Error> {
        sqlx::query_as::<_, RefreshToken>(
            "SELECT id, user_id, token_hash, expires_at, created_at, revoked_at
             FROM refresh_tokens
             WHERE token_hash = ?
               AND revoked_at IS NULL
               AND expires_at > strftime('%Y-%m-%dT%H:%M:%SZ', 'now')",
        )
        .bind(token_hash)
        .fetch_optional(pool)
        .await
    }

    /// Revoke a single refresh token (sets revoked_at).
    pub async fn revoke(pool: &SqlitePool, token_hash: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE refresh_tokens SET revoked_at = datetime('now')
             WHERE token_hash = ? AND revoked_at IS NULL",
        )
        .bind(token_hash)
        .execute(pool)
        .await?;
        Ok(())
    }

    /// Revoke all refresh tokens for a user (logout / password change).
    pub async fn revoke_all_for_user(pool: &SqlitePool, user_id: i64) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            "UPDATE refresh_tokens SET revoked_at = datetime('now')
             WHERE user_id = ? AND revoked_at IS NULL",
        )
        .bind(user_id)
        .execute(pool)
        .await?;
        Ok(result.rows_affected())
    }

    /// Delete expired and revoked tokens older than `days` days (cleanup).
    pub async fn cleanup(pool: &SqlitePool, days: i64) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            "DELETE FROM refresh_tokens
             WHERE (revoked_at IS NOT NULL OR expires_at < datetime('now'))
               AND created_at < datetime('now', ?)",
        )
        .bind(format!("-{} days", days))
        .execute(pool)
        .await?;
        Ok(result.rows_affected())
    }
}
