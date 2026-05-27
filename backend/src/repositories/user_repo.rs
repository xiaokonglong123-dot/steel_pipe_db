use sqlx::SqlitePool;

use crate::dto::auth_dto::{CreateUserRequest, UpdateUserRequest};
use crate::dto::common::PaginationParams;
use crate::models::user::User;

/// CRUD for `users`. All queries filter `deleted_at IS NULL`.
pub struct UserRepo;

impl UserRepo {
    /// SELECT user by username, including `password_hash` for auth verification.
    pub async fn find_by_username(
        pool: &SqlitePool,
        username: &str,
    ) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            "SELECT id, username, password_hash, display_name, role, email, phone,
                    is_active, created_at, updated_at, deleted_at
             FROM users WHERE username = ? AND deleted_at IS NULL",
        )
        .bind(username)
        .fetch_optional(pool)
        .await
    }

    /// SELECT user by primary key. Returns `None` if soft-deleted or missing.
    pub async fn find_by_id(
        pool: &SqlitePool,
        id: i64,
    ) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            "SELECT id, username, password_hash, display_name, role, email, phone,
                    is_active, created_at, updated_at, deleted_at
             FROM users WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    /// INSERT a new user with hashed password. Returns the created `User`.
    pub async fn create(
        pool: &SqlitePool,
        dto: &CreateUserRequest,
        password_hash: &str,
    ) -> Result<User, sqlx::Error> {
        sqlx::query_as::<_, User>(
            "INSERT INTO users (username, password_hash, display_name, role, email, phone)
             VALUES (?, ?, ?, ?, ?, ?)
             RETURNING id, username, password_hash, display_name, role, email, phone,
                       is_active, created_at, updated_at, deleted_at",
        )
        .bind(&dto.username)
        .bind(password_hash)
        .bind(&dto.display_name)
        .bind(&dto.role)
        .bind(&dto.email)
        .bind(&dto.phone)
        .fetch_one(pool)
        .await
    }

    /// Dynamic UPDATE of user fields (display_name, role, email, phone, is_active).
    /// Uses positional parameter numbering. Returns the updated `User`.
    pub async fn update(
        pool: &SqlitePool,
        id: i64,
        dto: &UpdateUserRequest,
    ) -> Result<User, sqlx::Error> {
        let mut updates = Vec::new();
        let mut params: Vec<String> = Vec::new();

        updates.push("updated_at = datetime('now')".to_string());

        if let Some(ref display_name) = dto.display_name {
            params.push(display_name.clone());
            updates.push(format!("display_name = ?{}", params.len()));
        }
        if let Some(ref role) = dto.role {
            params.push(role.clone());
            updates.push(format!("role = ?{}", params.len()));
        }
        if let Some(ref email) = dto.email {
            params.push(email.clone());
            updates.push(format!("email = ?{}", params.len()));
        }
        if let Some(ref phone) = dto.phone {
            params.push(phone.clone());
            updates.push(format!("phone = ?{}", params.len()));
        }
        if let Some(is_active) = dto.is_active {
            let val = if is_active { "1" } else { "0" };
            params.push(val.to_string());
            updates.push(format!("is_active = ?{}", params.len()));
        }

        let set_clause = updates.join(", ");
        let sql = format!(
            "UPDATE users SET {} WHERE id = ?{} AND deleted_at IS NULL
             RETURNING id, username, password_hash, display_name, role, email, phone,
                       is_active, created_at, updated_at, deleted_at",
            set_clause,
            params.len() + 1,
        );

        let mut query = sqlx::query_as::<_, User>(&sql);
        for p in &params {
            query = query.bind(p);
        }
        query = query.bind(id);
        query.fetch_one(pool).await
    }

    /// Paginated user list with optional search (username, display_name, email, phone). Returns `(items, total)`.
    pub async fn list(
        pool: &SqlitePool,
        params: &PaginationParams,
        q: Option<&str>,
    ) -> Result<(Vec<User>, u64), sqlx::Error> {
        let page_size = params.page_size() as i64;
        let offset = params.offset() as i64;

        if let Some(search) = q {
            let like = format!("%{}%", search);
            let total: (i64,) = sqlx::query_as(
                "SELECT COUNT(*) as cnt FROM users WHERE deleted_at IS NULL
                   AND (username LIKE ?1 OR display_name LIKE ?1 OR email LIKE ?1 OR phone LIKE ?1)",
            )
            .bind(&like)
            .fetch_one(pool)
            .await?;

            let items = sqlx::query_as::<_, User>(
                "SELECT id, username, password_hash, display_name, role, email, phone,
                        is_active, created_at, updated_at, deleted_at
                 FROM users WHERE deleted_at IS NULL
                   AND (username LIKE ?1 OR display_name LIKE ?1 OR email LIKE ?1 OR phone LIKE ?1)
                 ORDER BY created_at DESC LIMIT ?2 OFFSET ?3",
            )
            .bind(&like)
            .bind(page_size as i64)
            .bind(offset as i64)
            .fetch_all(pool)
            .await?;

            Ok((items, total.0 as u64))
        } else {
            let total: (i64,) = sqlx::query_as(
                "SELECT COUNT(*) as cnt FROM users WHERE deleted_at IS NULL",
            )
            .fetch_one(pool)
            .await?;

            let items = sqlx::query_as::<_, User>(
                "SELECT id, username, password_hash, display_name, role, email, phone,
                        is_active, created_at, updated_at, deleted_at
                 FROM users WHERE deleted_at IS NULL
                 ORDER BY created_at DESC LIMIT ?1 OFFSET ?2",
            )
            .bind(page_size as i64)
            .bind(offset as i64)
            .fetch_all(pool)
            .await?;

            Ok((items, total.0 as u64))
        }
    }

    /// UPDATE `password_hash` for a user.
    pub async fn update_password(
        pool: &SqlitePool,
        id: i64,
        password_hash: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE users SET password_hash = ?, updated_at = datetime('now')
             WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(password_hash)
        .bind(id)
        .execute(pool)
        .await?;
        Ok(())
    }

    /// Touch `updated_at` on login (tracks last login time).
    pub async fn update_last_login(
        pool: &SqlitePool,
        id: i64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE users SET updated_at = datetime('now')
             WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(id)
        .execute(pool)
        .await?;
        Ok(())
    }

    /// UPDATE user role. Returns the updated `User`.
    pub async fn update_role(
        pool: &SqlitePool,
        user_id: i64,
        role: &str,
    ) -> Result<User, sqlx::Error> {
        sqlx::query_as::<_, User>(
            "UPDATE users SET role = ?1, updated_at = datetime('now')
             WHERE id = ?2 AND deleted_at IS NULL
             RETURNING id, username, password_hash, display_name, role, email, phone,
                       is_active, created_at, updated_at, deleted_at",
        )
        .bind(role)
        .bind(user_id)
        .fetch_one(pool)
        .await
    }

    /// Soft-delete a user: sets `deleted_at`. Returns the deleted `User` or `None` if already gone.
    pub async fn delete_soft(
        pool: &SqlitePool,
        user_id: i64,
    ) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            "UPDATE users SET deleted_at = datetime('now')
             WHERE id = ? AND deleted_at IS NULL
             RETURNING id, username, password_hash, display_name, role, email, phone,
                       is_active, created_at, updated_at, deleted_at",
        )
        .bind(user_id)
        .fetch_optional(pool)
        .await
    }
}
