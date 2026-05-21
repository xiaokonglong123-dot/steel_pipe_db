use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: super::super::models::user::UserInfo,
}

#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    pub token: String,
}

#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub token: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
    pub display_name: String,
    pub role: String,
    pub email: Option<String>,
    pub phone: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub display_name: Option<String>,
    pub role: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub old_password: String,
    pub new_password: String,
}
