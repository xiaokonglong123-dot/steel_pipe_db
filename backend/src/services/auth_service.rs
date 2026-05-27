use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use uuid::Uuid;
use jsonwebtoken::{encode, Header};
use sqlx::SqlitePool;

use crate::dto::auth_dto::{CreateUserRequest, LoginRequest, LoginResponse, RefreshTokenRequest, ChangePasswordRequest, UpdateUserRequest};
use crate::error::AppError;
use crate::middleware::auth::Claims;
use crate::models::user::{User, UserInfo};
use crate::repositories::user_repo::UserRepo;

/// Auth service — handles login, token refresh, password management, and user CRUD.
/// Under the hood it delegates password verification to Argon2 and JWT generation to jsonwebtoken.
pub struct AuthService;

impl AuthService {
    /// Authenticate a user by username and password.
    ///
    /// Verifies credentials via Argon2, generates a JWT access token,
    /// updates the `last_login` timestamp, and returns the token + user profile.
    ///
    /// # Errors
    /// - `AppError::Unauthorized` — invalid username or password
    /// - `AppError::Forbidden` — account is disabled
    /// - `AppError::Internal` — password hash corrupted or token generation failed
    pub async fn login(
        pool: &SqlitePool,
        jwt_secret: &str,
        jwt_expiry_hours: i64,
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
            .map_err(|_| AppError::Internal("Invalid stored password hash".into()))?;

        Argon2::default()
            .verify_password(req.password.as_bytes(), &parsed_hash)
            .map_err(|_| AppError::Unauthorized("Invalid username or password".into()))?;

        let token = Self::generate_token(&user, jwt_secret, jwt_expiry_hours)?;

        UserRepo::update_last_login(pool, user.id)
            .await
            .map_err(AppError::from)?;

        Ok(LoginResponse {
            token,
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

    /// Issue a new JWT by decoding and re-encoding the existing (possibly expired) token.
    ///
    /// Reads the original claims, discards the old expiry, and assigns a fresh `exp` + `iat`.
    /// Tokens expired within the leeway grace period (7 days) are accepted for refresh;
    /// tokens expired beyond that are rejected to prevent indefinite token reuse.
    ///
    /// # Errors
    /// - `AppError::TokenExpired` — the token has been expired for too long (beyond leeway)
    /// - `AppError::Unauthorized` — the token is structurally invalid
    /// - `AppError::Internal` — system time is misconfigured or signing failed
    pub async fn refresh_token(
        jwt_secret: &str,
        jwt_expiry_hours: i64,
        req: &RefreshTokenRequest,
    ) -> Result<crate::dto::auth_dto::TokenResponse, AppError> {
        use jsonwebtoken::{decode, DecodingKey, Validation};

        // Allow tokens expired within the last 7 days to be refreshed.
        // Tokens expired beyond this window are permanently rejected.
        let mut validation = Validation::default();
        validation.leeway = 7 * 24 * 3600; // 7 days in seconds

        let token_data = decode::<Claims>(
            &req.token,
            &DecodingKey::from_secret(jwt_secret.as_bytes()),
            &validation,
        )
        .map_err(|e| match e.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => AppError::TokenExpired,
            _ => AppError::Unauthorized("Invalid token".into()),
        })?;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| AppError::Internal(format!("System time error: {}", e)))?
            .as_secs() as usize;

        let claims = Claims {
            sub: token_data.claims.sub,
            username: token_data.claims.username.clone(),
            role: token_data.claims.role.clone(),
            iat: now,
            exp: now + (jwt_expiry_hours as usize * 3600),
        };

        let token = encode(
            &Header::default(),
            &claims,
            &jsonwebtoken::EncodingKey::from_secret(jwt_secret.as_bytes()),
        )
        .map_err(|_| AppError::Internal("Failed to generate token".into()))?;

        Ok(crate::dto::auth_dto::TokenResponse { token })
    }

    /// Creates a new user and returns the basic profile.
    /// Hashes the password with Argon2 before storing it in the DB.
    ///
    /// # Errors
    /// - `AppError::Validation` — username already exists
    /// - `AppError::Internal` — password hashing failure
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
    ///
    /// # Errors
    /// - `AppError::NotFound` — user ID does not exist
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
    ///
    /// # Errors
    /// - `AppError::NotFound` — user ID does not exist
    /// - `AppError::Unauthorized` — old password does not match (non-admin)
    /// - `AppError::Internal` — password hashing failure
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
            let parsed_hash = PasswordHash::new(&user.password_hash)
                .map_err(|_| AppError::Internal("Invalid stored password hash".into()))?;

            Argon2::default()
                .verify_password(req.old_password.as_bytes(), &parsed_hash)
                .map_err(|_| AppError::Unauthorized("Current password is incorrect".into()))?;
        }

        let new_hash = Self::hash_password(&req.new_password)?;

        UserRepo::update_password(pool, user.id, &new_hash)
            .await
            .map_err(AppError::from)?;

        Ok(())
    }

    /// Fetches the currently logged-in user's own profile.
    ///
    /// # Errors
    /// - `AppError::NotFound` — user ID does not exist
    pub async fn get_me(
        pool: &SqlitePool,
        user_id: i64,
    ) -> Result<UserInfo, AppError> {
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

    fn hash_password(password: &str) -> Result<String, AppError> {
        let uuid = Uuid::new_v4();
        let salt = SaltString::encode_b64(uuid.as_bytes())
            .map_err(|_| AppError::Internal("Failed to generate salt".into()))?;
        let argon2 = Argon2::default();
        argon2
            .hash_password(password.as_bytes(), &salt)
            .map(|h| h.to_string())
            .map_err(|_| AppError::Internal("Failed to hash password".into()))
    }

    fn generate_token(
        user: &User,
        jwt_secret: &str,
        jwt_expiry_hours: i64,
    ) -> Result<String, AppError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| AppError::Internal(format!("System time error: {}", e)))?
            .as_secs() as usize;

        let claims = Claims {
            sub: user.id,
            username: user.username.clone(),
            role: user.role.clone(),
            iat: now,
            exp: now + (jwt_expiry_hours as usize * 3600),
        };

        encode(
            &Header::default(),
            &claims,
            &jsonwebtoken::EncodingKey::from_secret(jwt_secret.as_bytes()),
        )
        .map_err(|_| AppError::Internal("Failed to generate token".into()))
    }

    /// Swaps a user's role — only accepts admin/warehouse/qc/sales, no-BS.
    ///
    /// # Errors
    /// - `AppError::NotFound` — user ID does not exist
    /// - `AppError::Validation` — role value is not in the allowed set
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
    ///
    /// # Errors
    /// - `AppError::NotFound` — user ID does not exist
    pub async fn delete_user(
        pool: &SqlitePool,
        user_id: i64,
    ) -> Result<(), AppError> {
        let user = UserRepo::find_by_id(pool, user_id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::NotFound("User not found".into()))?;

        UserRepo::delete_soft(pool, user.id)
            .await
            .map_err(AppError::from)?;

        Ok(())
    }
}
