use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// User DB row. Maps to the `users` table — system login accounts.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: i64,
    /// Login username — must be unique across the system.
    pub username: String,
    /// Argon2 password hash. Nobody gets to see this bad boy.
    pub password_hash: String,
    /// Display name — what people actually call you in the UI.
    pub display_name: String,
    /// Role: admin / warehouse / qc / sales. Determines what you can touch.
    pub role: String,
    /// Email address.
    pub email: Option<String>,
    /// Phone number.
    pub phone: Option<String>,
    /// Whether this account is active. Disabled users can't log in.
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
    /// Soft-delete timestamp. Null means this user is still alive.
    pub deleted_at: Option<String>,
}

/// Public user info — safe to share with the frontend. No password hash here.
#[derive(Debug, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: i64,
    pub username: String,
    pub display_name: String,
    pub role: String,
    pub email: Option<String>,
    pub phone: Option<String>,
}
