use sqlx::SqlitePool;
use crate::domain::User;
use crate::error::AppResult;

pub struct UserRepo {
    pool: SqlitePool,
}

impl UserRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn find_by_username(&self, username: &str) -> AppResult<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            "SELECT id, username, password_hash, display_name, role, email, phone, \
             is_active, created_at, updated_at \
             FROM users WHERE username = ? AND is_active = 1",
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await?;
        Ok(user)
    }

    pub async fn find_by_id(&self, id: &str) -> AppResult<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            "SELECT id, username, password_hash, display_name, role, email, phone, \
             is_active, created_at, updated_at \
             FROM users WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(user)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn create(
        &self,
        id: &str,
        username: &str,
        password_hash: &str,
        display_name: &str,
        role: &str,
        email: Option<&str>,
        phone: Option<&str>,
    ) -> AppResult<()> {
        sqlx::query(
            "INSERT INTO users (id, username, password_hash, display_name, role, email, phone) \
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(id)
        .bind(username)
        .bind(password_hash)
        .bind(display_name)
        .bind(role)
        .bind(email)
        .bind(phone)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn list(&self) -> AppResult<Vec<User>> {
        let users = sqlx::query_as::<_, User>(
            "SELECT id, username, password_hash, display_name, role, email, phone, \
             is_active, created_at, updated_at \
             FROM users ORDER BY created_at DESC",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(users)
    }

    pub async fn list_active(&self) -> AppResult<Vec<User>> {
        let users = sqlx::query_as::<_, User>(
            "SELECT id, username, password_hash, display_name, role, email, phone, \
             is_active, created_at, updated_at \
             FROM users WHERE is_active = 1 ORDER BY created_at DESC",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(users)
    }

    pub async fn update(
        &self,
        id: &str,
        display_name: &str,
        role: &str,
        email: Option<&str>,
        phone: Option<&str>,
        is_active: bool,
    ) -> AppResult<()> {
        sqlx::query(
            "UPDATE users SET display_name = ?, role = ?, email = ?, phone = ?, \
             is_active = ?, updated_at = datetime('now') WHERE id = ?",
        )
        .bind(display_name)
        .bind(role)
        .bind(email)
        .bind(phone)
        .bind(is_active as i32)
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn delete(&self, id: &str) -> AppResult<()> {
        sqlx::query("UPDATE users SET is_active = 0, updated_at = datetime('now') WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn update_password(&self, id: &str, password_hash: &str) -> AppResult<()> {
        sqlx::query("UPDATE users SET password_hash = ?, updated_at = datetime('now') WHERE id = ?")
            .bind(password_hash)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
