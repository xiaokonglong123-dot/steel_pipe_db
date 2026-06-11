use serde::{Deserialize, Serialize};
use validator::Validate;

/// Login request DTO.
#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    /// Username.
    #[validate(length(min = 1))]
    pub username: String,
    /// Password (plaintext — server hashes with Argon2).
    #[validate(length(min = 1))]
    pub password: String,
}

/// Login success response DTO.
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    /// JWT access token (Bearer format).
    pub token: String,
    /// Opaque refresh token for obtaining new access tokens.
    pub refresh_token: String,
    /// Currently logged-in user info.
    pub user: super::super::models::user::UserInfo,
}

/// Token refresh request DTO.
#[derive(Debug, Deserialize, Validate)]
pub struct RefreshTokenRequest {
    /// The opaque refresh token (not a JWT).
    #[validate(length(min = 1))]
    pub refresh_token: String,
}

/// Token response DTO.
#[derive(Debug, Serialize)]
pub struct TokenResponse {
    /// The refreshed JWT access token.
    pub token: String,
    /// New opaque refresh token (rotation).
    pub refresh_token: String,
}

/// Create user request DTO.
#[derive(Debug, Deserialize, Validate)]
pub struct CreateUserRequest {
    /// Login username (unique).
    #[validate(length(min = 1))]
    pub username: String,
    /// Initial password.
    #[validate(length(min = 1))]
    pub password: String,
    /// Display name.
    #[validate(length(min = 1))]
    pub display_name: String,
    /// Role: admin / warehouse / qc / sales.
    #[validate(length(min = 1))]
    pub role: String,
    /// Email address.
    #[validate(email)]
    pub email: Option<String>,
    /// Phone number.
    pub phone: Option<String>,
}

/// Update user info request DTO.
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateUserRequest {
    /// Display name.
    pub display_name: Option<String>,
    /// Role.
    pub role: Option<String>,
    /// Email address.
    #[validate(email)]
    pub email: Option<String>,
    /// Phone number.
    pub phone: Option<String>,
    /// Whether the user is active.
    pub is_active: Option<bool>,
}

/// Change user role request DTO.
#[derive(Debug, Deserialize, Validate)]
pub struct ChangeUserRoleRequest {
    /// New role: admin / warehouse / qc / sales.
    #[validate(length(min = 1))]
    pub role: String,
}

/// Change password request DTO.
#[derive(Debug, Deserialize, Validate)]
pub struct ChangePasswordRequest {
    /// Current password.
    #[validate(length(min = 1))]
    pub old_password: String,
    /// New password.
    #[validate(length(min = 1))]
    pub new_password: String,
}
