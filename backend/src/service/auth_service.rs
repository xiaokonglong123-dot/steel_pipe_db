use argon2::PasswordHasher;
use argon2::PasswordVerifier;
use argon2::password_hash::SaltString;
use serde::{Deserialize, Serialize};

use crate::auth::{create_access_token, create_refresh_token};
use crate::config::AppConfig;
use crate::domain::User;
use crate::error::{AppError, AppResult};
use crate::repository::user_repo::UserRepo;

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub user: UserInfo,
}

#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub id: String,
    pub username: String,
    pub display_name: String,
    pub role: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub is_active: bool,
}

impl From<User> for UserInfo {
    fn from(u: User) -> Self {
        Self {
            id: u.id,
            username: u.username,
            display_name: u.display_name,
            role: u.role,
            email: u.email,
            phone: u.phone,
            is_active: u.is_active,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct LoginDto {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserDto {
    pub username: String,
    pub password: String,
    pub display_name: String,
    pub role: String,
    pub email: Option<String>,
    pub phone: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserDto {
    pub display_name: Option<String>,
    pub role: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub is_active: Option<bool>,
    pub password: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RefreshTokenDto {
    pub refresh_token: String,
}

pub struct AuthService {
    repo: UserRepo,
    config: AppConfig,
}

impl AuthService {
    pub fn new(repo: UserRepo, config: AppConfig) -> Self {
        Self { repo, config }
    }

    pub async fn login(&self, dto: LoginDto) -> AppResult<LoginResponse> {
        let user = self
            .repo
            .find_by_username(&dto.username)
            .await?
            .ok_or_else(|| AppError::Unauthorized("Invalid username or password".into()))?;

        if !user.is_active {
            return Err(AppError::Forbidden("Account is disabled".into()));
        }

        // Verify password
        let parsed_hash = argon2::PasswordHash::new(&user.password_hash)
            .map_err(|_| AppError::Internal("Invalid password hash format".into()))?;

        argon2::Argon2::default()
            .verify_password(dto.password.as_bytes(), &parsed_hash)
            .map_err(|_| AppError::Unauthorized("Invalid username or password".into()))?;

        // Generate tokens
        let access_token = create_access_token(
            &user.id,
            &user.username,
            &user.role,
            &self.config.jwt_secret,
            self.config.jwt_access_expiry,
        )
        .map_err(|e| AppError::Internal(format!("Failed to create access token: {e}")))?;

        let refresh_token = create_refresh_token(
            &user.id,
            &user.username,
            &user.role,
            &self.config.jwt_secret,
            self.config.jwt_refresh_expiry,
        )
        .map_err(|e| AppError::Internal(format!("Failed to create refresh token: {e}")))?;

        Ok(LoginResponse {
            access_token,
            refresh_token,
            user: user.into(),
        })
    }

    pub async fn refresh_token(&self, dto: RefreshTokenDto) -> AppResult<LoginResponse> {
        let claims = crate::auth::validate_token(&dto.refresh_token, &self.config.jwt_secret)
            .map_err(|_| AppError::Unauthorized("Invalid or expired refresh token".into()))?;

        let user = self
            .repo
            .find_by_id(&claims.sub)
            .await?
            .ok_or_else(|| AppError::Unauthorized("User not found".into()))?;

        if !user.is_active {
            return Err(AppError::Forbidden("Account is disabled".into()));
        }

        let access_token = create_access_token(
            &user.id,
            &user.username,
            &user.role,
            &self.config.jwt_secret,
            self.config.jwt_access_expiry,
        )
        .map_err(|e| AppError::Internal(format!("Failed to create access token: {e}")))?;

        let refresh_token = create_refresh_token(
            &user.id,
            &user.username,
            &user.role,
            &self.config.jwt_secret,
            self.config.jwt_refresh_expiry,
        )
        .map_err(|e| AppError::Internal(format!("Failed to create refresh token: {e}")))?;

        Ok(LoginResponse {
            access_token,
            refresh_token,
            user: user.into(),
        })
    }

    pub async fn get_current_user(&self, user_id: &str) -> AppResult<UserInfo> {
        let user = self
            .repo
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".into()))?;
        Ok(user.into())
    }

    pub async fn list_users(&self) -> AppResult<Vec<UserInfo>> {
        let users = self.repo.list_active().await?;
        Ok(users.into_iter().map(|u| u.into()).collect())
    }

    pub async fn create_user(&self, dto: CreateUserDto) -> AppResult<UserInfo> {
        // Check if username already exists
        if self.repo.find_by_username(&dto.username).await?.is_some() {
            return Err(AppError::Conflict("Username already exists".into()));
        }

        let salt = SaltString::generate(&mut rand::rngs::OsRng);
        let password_hash = argon2::Argon2::default()
            .hash_password(dto.password.as_bytes(), &salt)
            .map_err(|e| AppError::Internal(format!("Failed to hash password: {e}")))?
            .to_string();

        let id = uuid::Uuid::new_v4().to_string();
        self.repo
            .create(
                &id,
                &dto.username,
                &password_hash,
                &dto.display_name,
                &dto.role,
                dto.email.as_deref(),
                dto.phone.as_deref(),
            )
            .await?;

        let user = self
            .repo
            .find_by_id(&id)
            .await?
            .ok_or_else(|| AppError::Internal("Failed to retrieve created user".into()))?;

        Ok(user.into())
    }

    pub async fn update_user(&self, id: &str, dto: UpdateUserDto) -> AppResult<UserInfo> {
        let existing = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".into()))?;

        let display_name = dto.display_name.unwrap_or(existing.display_name);
        let role = dto.role.unwrap_or(existing.role);
        let email = dto.email.or(existing.email);
        let phone = dto.phone.or(existing.phone);
        let is_active = dto.is_active.unwrap_or(existing.is_active);

        self.repo
            .update(id, &display_name, &role, email.as_deref(), phone.as_deref(), is_active)
            .await?;

        // Optionally update password
        if let Some(new_password) = dto.password {
            let salt = SaltString::generate(&mut rand::rngs::OsRng);
            let password_hash = argon2::Argon2::default()
                .hash_password(new_password.as_bytes(), &salt)
                .map_err(|e| AppError::Internal(format!("Failed to hash password: {e}")))?
                .to_string();
            self.repo.update_password(id, &password_hash).await?;
        }

        let user = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::Internal("Failed to retrieve updated user".into()))?;

        Ok(user.into())
    }

    pub async fn delete_user(&self, id: &str) -> AppResult<()> {
        let exists = self.repo.find_by_id(id).await?.is_some();
        if !exists {
            return Err(AppError::NotFound("User not found".into()));
        }
        self.repo.delete(id).await?;
        Ok(())
    }
}
