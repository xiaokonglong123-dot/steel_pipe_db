use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use jsonwebtoken::{encode, Header};
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::dto::auth_dto::{
    ChangePasswordRequest, CreateUserRequest, LoginRequest, LoginResponse, RefreshTokenRequest,
    UpdateUserRequest,
};
use crate::error::AppError;
use crate::middleware::auth::Claims;
use crate::models::user::{User, UserInfo};
use crate::repositories::refresh_token_repo::RefreshTokenRepo;
use crate::repositories::user_repo::UserRepo;

/// Auth service — handles login, token refresh, password management, and user CRUD.
/// Under the hood it delegates password verification to Argon2 and JWT generation to jsonwebtoken.
pub struct AuthService;

impl AuthService {
    /// Authenticate a user by username and password.
    ///
    /// Verifies credentials via Argon2, generates a JWT access token and an opaque
    /// refresh token, stores the refresh token hash in the DB, updates `last_login`,
    /// and returns both tokens + user profile.
    ///
    /// RBAC is resolved from the DB at token-issue time only: the token carries a
    /// snapshot of the user's current role/permissions, so authorization changes
    /// become effective once the access token expires (see `auth_middleware`).
    pub async fn login(
        pool: &SqlitePool,
        jwt_secret: &str,
        jwt_expiry_hours: i64,
        refresh_token_expiry_days: i64,
        req: &LoginRequest,
    ) -> Result<LoginResponse, AppError> {
        let user = UserRepo::find_by_username(pool, &req.username)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::Unauthorized("Invalid username or password".into()))?;

        if !user.is_active {
            return Err(AppError::Forbidden("Account is disabled".into()));
        }

        let parsed_hash = PasswordHash::new(&user.password_hash)
            .map_err(AppError::from)?;

        Argon2::default()
            .verify_password(req.password.as_bytes(), &parsed_hash)
            .map_err(|_| AppError::Unauthorized("Invalid username or password".into()))?;

        let permissions = crate::auth::services::IdentityService::user_permission_keys(pool, user.id).await?;
        let token = Self::generate_token(&user, jwt_secret, jwt_expiry_hours, &permissions)?;
        let (refresh_token, refresh_token_hash) = Self::generate_refresh_token();

        let expires_at = chrono::Utc::now()
            .checked_add_signed(chrono::Duration::days(refresh_token_expiry_days))
            .unwrap_or_else(|| {
                chrono::Utc::now() + chrono::Duration::days(refresh_token_expiry_days)
            });

        RefreshTokenRepo::create(pool, user.id, &refresh_token_hash, &expires_at)
            .await
            .map_err(AppError::from)?;

        UserRepo::update_last_login(pool, user.id)
            .await
            .map_err(AppError::from)?;

        Ok(LoginResponse {
            token,
            refresh_token,
            user: UserInfo {
                id: user.id,
                username: user.username,
                display_name: user.display_name,
                role: user.role,
                email: user.email,
                phone: user.phone,
            },
        })
    }

    /// Issue new access + refresh tokens by validating and rotating a refresh token.
    ///
    /// The incoming refresh token is revoked (rotation), and a new pair is issued.
    /// Tokens that are expired, revoked, or not found are rejected.
    pub async fn refresh_token(
        pool: &SqlitePool,
        jwt_secret: &str,
        jwt_expiry_hours: i64,
        refresh_token_expiry_days: i64,
        req: &RefreshTokenRequest,
    ) -> Result<crate::dto::auth_dto::TokenResponse, AppError> {
        let token_hash = Self::hash_refresh_token(&req.refresh_token);

        let existing = RefreshTokenRepo::find_by_token_hash(pool, &token_hash)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::Unauthorized("Invalid or expired refresh token".into()))?;

        // Revoke the old token (rotation)
        RefreshTokenRepo::revoke(pool, &token_hash)
            .await
            .map_err(AppError::from)?;

        // Look up user to generate new tokens
        let user = UserRepo::find_by_id(pool, existing.user_id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::Unauthorized("User not found".into()))?;

        if !user.is_active {
            return Err(AppError::Forbidden("Account is disabled".into()));
        }

        let permissions = crate::auth::services::IdentityService::user_permission_keys(pool, user.id).await?;
        let token = Self::generate_token(&user, jwt_secret, jwt_expiry_hours, &permissions)?;
        let (new_refresh_token, new_refresh_token_hash) = Self::generate_refresh_token();

        let expires_at = chrono::Utc::now()
            .checked_add_signed(chrono::Duration::days(refresh_token_expiry_days))
            .unwrap_or_else(|| {
                chrono::Utc::now() + chrono::Duration::days(refresh_token_expiry_days)
            });

        RefreshTokenRepo::create(pool, user.id, &new_refresh_token_hash, &expires_at)
            .await
            .map_err(AppError::from)?;

        Ok(crate::dto::auth_dto::TokenResponse {
            token,
            refresh_token: new_refresh_token,
        })
    }

    /// Revoke all refresh tokens for a user (logout / forced session invalidation).
    pub async fn logout(pool: &SqlitePool, user_id: i64) -> Result<(), AppError> {
        RefreshTokenRepo::revoke_all_for_user(pool, user_id)
            .await
            .map_err(AppError::from)?;
        Ok(())
    }

    /// Creates a new user and returns the basic profile.
    /// Hashes the password with Argon2 before storing it in the DB.
    pub async fn create_user(
        pool: &SqlitePool,
        dto: &CreateUserRequest,
    ) -> Result<UserInfo, AppError> {
        let existing = UserRepo::find_by_username(pool, &dto.username)
            .await
            .map_err(AppError::from)?;

        if existing.is_some() {
            return Err(AppError::Validation("Username already exists".into()));
        }

        let password_hash = Self::hash_password(&dto.password)?;

        let user = UserRepo::create(pool, dto, &password_hash)
            .await
            .map_err(AppError::from)?;

        Ok(UserInfo {
            id: user.id,
            username: user.username,
            display_name: user.display_name,
            role: user.role,
            email: user.email,
            phone: user.phone,
        })
    }

    /// Updates the user's profile — display name, email, phone, etc.
    /// Returns the updated `UserInfo`.
    pub async fn update_user(
        pool: &SqlitePool,
        id: i64,
        dto: &UpdateUserRequest,
    ) -> Result<UserInfo, AppError> {
        let user = UserRepo::find_by_id(pool, id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::NotFound("User not found".into()))?;

        let updated = UserRepo::update(pool, user.id, dto)
            .await
            .map_err(AppError::from)?;

        Ok(UserInfo {
            id: updated.id,
            username: updated.username,
            display_name: updated.display_name,
            role: updated.role,
            email: updated.email,
            phone: updated.phone,
        })
    }

    /// Changes the user's password.
    /// Admins bypass the old-password check; everyone else must provide their current password.
    pub async fn change_password(
        pool: &SqlitePool,
        user_id: i64,
        current_user_role: &str,
        req: &ChangePasswordRequest,
    ) -> Result<(), AppError> {
        let user = UserRepo::find_by_id(pool, user_id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::NotFound("User not found".into()))?;

        if current_user_role != "admin" {
            let old_password = req.old_password.as_deref().ok_or_else(|| {
                AppError::Validation("old_password is required for self-service password change".into())
            })?;
            let parsed_hash = PasswordHash::new(&user.password_hash)
                .map_err(AppError::from)?;

            Argon2::default()
                .verify_password(old_password.as_bytes(), &parsed_hash)
                .map_err(|_| AppError::Unauthorized("Current password is incorrect".into()))?;
        }

        let new_hash = Self::hash_password(&req.new_password)?;

        UserRepo::update_password(pool, user.id, &new_hash)
            .await
            .map_err(AppError::from)?;

        // Revoke all refresh tokens on password change
        RefreshTokenRepo::revoke_all_for_user(pool, user_id)
            .await
            .map_err(AppError::from)?;

        Ok(())
    }

    /// Fetches the currently logged-in user's own profile.
    pub async fn get_me(pool: &SqlitePool, user_id: i64) -> Result<UserInfo, AppError> {
        let user = UserRepo::find_by_id(pool, user_id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::NotFound("User not found".into()))?;

        Ok(UserInfo {
            id: user.id,
            username: user.username,
            display_name: user.display_name,
            role: user.role,
            email: user.email,
            phone: user.phone,
        })
    }

    /// Paginated user list with fuzzy username search.
    /// Returns a tuple of `(user_infos, total_count)`.
    pub async fn list_users(
        pool: &SqlitePool,
        params: &crate::dto::common::PaginationParams,
        q: Option<&str>,
    ) -> Result<(Vec<UserInfo>, u64), AppError> {
        let (users, total) = UserRepo::list(pool, params, q)
            .await
            .map_err(AppError::from)?;

        let infos: Vec<UserInfo> = users
            .into_iter()
            .map(|u| UserInfo {
                id: u.id,
                username: u.username,
                display_name: u.display_name,
                role: u.role,
                email: u.email,
                phone: u.phone,
            })
            .collect();

        Ok((infos, total))
    }

    pub(crate) fn hash_password(password: &str) -> Result<String, AppError> {
        let uuid = Uuid::new_v4();
        let salt = SaltString::encode_b64(uuid.as_bytes())
            .map_err(AppError::from)?;
        let argon2 = Argon2::default();
        argon2
            .hash_password(password.as_bytes(), &salt)
            .map(|h| h.to_string())
            .map_err(AppError::from)
    }

    fn generate_token(
        user: &User,
        jwt_secret: &str,
        jwt_expiry_hours: i64,
        permissions: &[String],
    ) -> Result<String, AppError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| AppError::Internal(format!("System time error: {}", e)))?
            .as_secs() as usize;

        let claims = Claims {
            sub: user.id,
            tenant_id: user.tenant_id,
            username: user.username.clone(),
            role: user.role.clone(),
            permissions: permissions.to_vec(),
            iat: now,
            exp: now + (jwt_expiry_hours as usize * 3600),
        };

        encode(
            &Header::default(),
            &claims,
            &jsonwebtoken::EncodingKey::from_secret(jwt_secret.as_bytes()),
        )
        .map_err(AppError::from)
    }

    /// Generate an opaque refresh token and its SHA-256 hash.
    fn generate_refresh_token() -> (String, String) {
        let token = Uuid::new_v4().to_string();
        let hash = Self::hash_refresh_token(&token);
        (token, hash)
    }

    /// SHA-256 hash a refresh token for safe storage.
    fn hash_refresh_token(token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Swaps a user's role — only accepts admin/warehouse/qc/sales, no-BS.
    pub async fn change_role(
        pool: &SqlitePool,
        user_id: i64,
        new_role: &str,
    ) -> Result<UserInfo, AppError> {
        let user = UserRepo::find_by_id(pool, user_id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::NotFound("User not found".into()))?;

        match new_role {
            "admin" | "warehouse" | "qc" | "sales" => {}
            _ => {
                return Err(AppError::Validation(
                    "Invalid role. Must be one of: admin, warehouse, qc, sales".into(),
                ))
            }
        }

        let updated = UserRepo::update_role(pool, user.id, new_role)
            .await
            .map_err(AppError::from)?;

        Ok(UserInfo {
            id: updated.id,
            username: updated.username,
            display_name: updated.display_name,
            role: updated.role,
            email: updated.email,
            phone: updated.phone,
        })
    }

    /// Soft-deletes a user by flipping on the `deleted_at` flag.
    pub async fn delete_user(pool: &SqlitePool, user_id: i64) -> Result<(), AppError> {
        let user = UserRepo::find_by_id(pool, user_id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::NotFound("User not found".into()))?;

        UserRepo::delete_soft(pool, user.id)
            .await
            .map_err(AppError::from)?;

        // Revoke all refresh tokens on user deletion
        RefreshTokenRepo::revoke_all_for_user(pool, user_id)
            .await
            .map_err(AppError::from)?;

        Ok(())
    }
}
