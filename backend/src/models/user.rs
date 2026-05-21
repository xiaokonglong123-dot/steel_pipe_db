use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
    pub display_name: String,
    pub role: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: i64,
    pub username: String,
    pub display_name: String,
    pub role: String,
    pub email: Option<String>,
    pub phone: Option<String>,
}
