//! Integration tests for AuthService.
//!
//! Covers login, token refresh, user CRUD, password management, role management,
//! and soft-delete semantics.
//!
//! All tests use an in-memory SQLite database with fresh migrations.

mod common;

use steel_pipe_db::dto::auth_dto::{
    ChangePasswordRequest, CreateUserRequest, LoginRequest, RefreshTokenRequest, UpdateUserRequest,
};
use steel_pipe_db::dto::common::PaginationParams;
use steel_pipe_db::services::auth_service::AuthService;

/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
/// Login
/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn login_successful_returns_token_and_user_info() {
    let pool = common::test_pool().await;
    common::seed_user(&pool, "alice", "admin").await.unwrap();

    let req = LoginRequest {
        username: "alice".into(),
        password: "password123".into(),
    };

    let resp = AuthService::login(
        &pool,
        common::TEST_JWT_SECRET,
        common::TEST_JWT_EXPIRY_HOURS,
        &req,
    )
    .await
    .expect("login must succeed");

    assert!(!resp.token.is_empty(), "token must not be empty");
    assert_eq!(resp.user.username, "alice");
    assert_eq!(resp.user.role, "admin");
    assert!(resp.user.id > 0);
}

#[tokio::test]
async fn login_wrong_password_returns_unauthorized() {
    let pool = common::test_pool().await;
    common::seed_user(&pool, "bob", "warehouse").await.unwrap();

    let req = LoginRequest {
        username: "bob".into(),
        password: "wrongpassword".into(),
    };

    let err = AuthService::login(
        &pool,
        common::TEST_JWT_SECRET,
        common::TEST_JWT_EXPIRY_HOURS,
        &req,
    )
    .await
    .expect_err("must fail with wrong password");

    let msg = err.to_string();
    assert!(
        msg.contains("Invalid username or password") || msg.contains("Unauthorized"),
        "expected auth error, got: {msg}",
    );
}

#[tokio::test]
async fn login_inactive_user_returns_forbidden() {
    let pool = common::test_pool().await;
    let user_id = common::seed_user(&pool, "disabled", "admin").await.unwrap();

    sqlx::query("UPDATE users SET is_active = 0 WHERE id = ?")
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();

    let req = LoginRequest {
        username: "disabled".into(),
        password: "password123".into(),
    };

    let err = AuthService::login(
        &pool,
        common::TEST_JWT_SECRET,
        common::TEST_JWT_EXPIRY_HOURS,
        &req,
    )
    .await
    .expect_err("must fail for inactive user");

    let msg = err.to_string();
    assert!(
        msg.contains("Account is disabled") || msg.contains("Forbidden"),
        "expected disabled-account error, got: {msg}",
    );
}

#[tokio::test]
async fn login_nonexistent_user_returns_unauthorized() {
    let pool = common::test_pool().await;

    let req = LoginRequest {
        username: "nobody".into(),
        password: "password123".into(),
    };

    let err = AuthService::login(
        &pool,
        common::TEST_JWT_SECRET,
        common::TEST_JWT_EXPIRY_HOURS,
        &req,
    )
    .await
    .expect_err("must fail for nonexistent user");

    let msg = err.to_string();
    assert!(
        msg.contains("Invalid username or password") || msg.contains("Unauthorized"),
        "expected auth error, got: {msg}",
    );
}

/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
/// Refresh Token
/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn refresh_token_returns_new_token() {
    let pool = common::test_pool().await;
    common::seed_user(&pool, "refresh_me", "admin").await.unwrap();

    let login_req = LoginRequest {
        username: "refresh_me".into(),
        password: "password123".into(),
    };
    let login_resp = AuthService::login(
        &pool,
        common::TEST_JWT_SECRET,
        common::TEST_JWT_EXPIRY_HOURS,
        &login_req,
    )
    .await
    .expect("login must succeed");

    let refresh_req = RefreshTokenRequest {
        token: login_resp.token,
    };

    let resp = AuthService::refresh_token(
        common::TEST_JWT_SECRET,
        common::TEST_JWT_EXPIRY_HOURS,
        &refresh_req,
    )
    .await
    .expect("refresh_token must succeed");

    assert!(!resp.token.is_empty(), "refreshed token must not be empty");
}

#[tokio::test]
async fn refresh_token_invalid_format_returns_unauthorized() {
    let refresh_req = RefreshTokenRequest {
        token: "this.is.not.a.valid.jwt".into(),
    };

    let err = AuthService::refresh_token(
        common::TEST_JWT_SECRET,
        common::TEST_JWT_EXPIRY_HOURS,
        &refresh_req,
    )
    .await
    .expect_err("must fail with invalid token");

    let msg = err.to_string();
    assert!(
        msg.contains("Invalid token") || msg.contains("Unauthorized"),
        "expected invalid-token error, got: {msg}",
    );
}

#[tokio::test]
async fn refresh_token_garbage_string_returns_unauthorized() {
    let refresh_req = RefreshTokenRequest {
        token: "complete-garbage-string-not-a-jwt-at-all".into(),
    };

    let err = AuthService::refresh_token(
        common::TEST_JWT_SECRET,
        common::TEST_JWT_EXPIRY_HOURS,
        &refresh_req,
    )
    .await
    .expect_err("must fail with garbage token");

    let msg = err.to_string();
    assert!(
        msg.contains("Invalid token") || msg.contains("Unauthorized"),
        "expected invalid-token error, got: {msg}",
    );
}

/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
/// Create User
/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn create_user_creates_and_returns_user_info() {
    let pool = common::test_pool().await;

    let dto = CreateUserRequest {
        username: "newguy".into(),
        password: "SecurePass1".into(),
        display_name: "New Guy".into(),
        role: "warehouse".into(),
        email: Some("newguy@test.local".into()),
        phone: Some("13800138002".into()),
    };

    let info = AuthService::create_user(&pool, &dto)
        .await
        .expect("create_user must succeed");

    assert_eq!(info.username, "newguy");
    assert_eq!(info.display_name, "New Guy");
    assert_eq!(info.role, "warehouse");
    assert_eq!(info.email.as_deref(), Some("newguy@test.local"));
    assert_eq!(info.phone.as_deref(), Some("13800138002"));
    assert!(info.id > 0);
}

#[tokio::test]
async fn create_user_with_minimal_fields_succeeds() {
    let pool = common::test_pool().await;

    let dto = CreateUserRequest {
        username: "minimal".into(),
        password: "minimal1".into(),
        display_name: "Minimal".into(),
        role: "qc".into(),
        email: None,
        phone: None,
    };

    let info = AuthService::create_user(&pool, &dto)
        .await
        .expect("create_user with minimal fields must succeed");

    assert_eq!(info.username, "minimal");
    assert_eq!(info.role, "qc");
    assert!(info.email.is_none());
    assert!(info.phone.is_none());
}

#[tokio::test]
async fn create_user_duplicate_username_returns_validation_error() {
    let pool = common::test_pool().await;
    common::seed_user(&pool, "existing", "admin").await.unwrap();

    let dto = CreateUserRequest {
        username: "existing".into(),
        password: "Another1Pass".into(),
        display_name: "Duplicate".into(),
        role: "warehouse".into(),
        email: None,
        phone: None,
    };

    let err = AuthService::create_user(&pool, &dto)
        .await
        .expect_err("must fail for duplicate username");

    let msg = err.to_string();
    assert!(
        msg.contains("Username already exists") || msg.contains("Validation"),
        "expected duplicate-username error, got: {msg}",
    );
}

/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
/// Update User
/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn update_user_changes_display_name_and_email() {
    let pool = common::test_pool().await;
    let user_id = common::seed_user(&pool, "updatable", "warehouse").await.unwrap();

    let dto = UpdateUserRequest {
        display_name: Some("Updated Name".into()),
        role: Some("admin".into()),
        email: Some("updated@test.local".into()),
        phone: Some("13900139000".into()),
        is_active: None,
    };

    let info = AuthService::update_user(&pool, user_id, &dto)
        .await
        .expect("update_user must succeed");

    assert_eq!(info.display_name, "Updated Name");
    assert_eq!(info.role, "admin");
    assert_eq!(info.email.as_deref(), Some("updated@test.local"));
    assert_eq!(info.phone.as_deref(), Some("13900139000"));
}

#[tokio::test]
async fn update_user_partial_fields_only_changes_specified_fields() {
    let pool = common::test_pool().await;
    let user_id = common::seed_user(&pool, "partial", "warehouse").await.unwrap();

    let dto = UpdateUserRequest {
        display_name: Some("Just Name".into()),
        role: None,
        email: None,
        phone: None,
        is_active: None,
    };

    let info = AuthService::update_user(&pool, user_id, &dto)
        .await
        .expect("partial update must succeed");

    assert_eq!(info.display_name, "Just Name");
    assert_eq!(info.role, "warehouse"); // unchanged
}

#[tokio::test]
async fn update_user_nonexistent_returns_not_found() {
    let pool = common::test_pool().await;

    let dto = UpdateUserRequest {
        display_name: Some("Ghost".into()),
        role: None,
        email: None,
        phone: None,
        is_active: None,
    };

    let err = AuthService::update_user(&pool, 99999, &dto)
        .await
        .expect_err("must fail for nonexistent user");

    let msg = err.to_string();
    assert!(
        msg.contains("User not found") || msg.contains("NotFound"),
        "expected not-found error, got: {msg}",
    );
}

/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
/// Change Password
/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn change_password_with_correct_old_password_succeeds() {
    let pool = common::test_pool().await;
    let user_id = common::seed_user(&pool, "chpass", "warehouse").await.unwrap();

    let req = ChangePasswordRequest {
        old_password: "password123".into(),
        new_password: "NewSecure1Pass".into(),
    };

    AuthService::change_password(&pool, user_id, "warehouse", &req)
        .await
        .expect("change_password must succeed");

    // Verify we can log in with the new password
    let login_req = LoginRequest {
        username: "chpass".into(),
        password: "NewSecure1Pass".into(),
    };
    let resp = AuthService::login(
        &pool,
        common::TEST_JWT_SECRET,
        common::TEST_JWT_EXPIRY_HOURS,
        &login_req,
    )
    .await
    .expect("login with new password must succeed");

    assert_eq!(resp.user.username, "chpass");
}

#[tokio::test]
async fn change_password_with_wrong_old_password_returns_unauthorized() {
    let pool = common::test_pool().await;
    let user_id = common::seed_user(&pool, "chpass2", "warehouse").await.unwrap();

    let req = ChangePasswordRequest {
        old_password: "wrongOldPassword".into(),
        new_password: "NewSecure1Pass".into(),
    };

    let err = AuthService::change_password(&pool, user_id, "warehouse", &req)
        .await
        .expect_err("must fail with wrong old password");

    let msg = err.to_string();
    assert!(
        msg.contains("Current password is incorrect") || msg.contains("Unauthorized"),
        "expected wrong-password error, got: {msg}",
    );
}

#[tokio::test]
async fn change_password_admin_bypasses_old_password_check() {
    let pool = common::test_pool().await;
    let user_id = common::seed_user(&pool, "chpass3", "warehouse").await.unwrap();

    // Admin can change any user's password without knowing the old one
    let req = ChangePasswordRequest {
        old_password: "does_not_matter".into(),
        new_password: "AdminSetPass1".into(),
    };

    AuthService::change_password(&pool, user_id, "admin", &req)
        .await
        .expect("admin bypass must succeed");

    // Verify login with new password works
    let login_req = LoginRequest {
        username: "chpass3".into(),
        password: "AdminSetPass1".into(),
    };
    AuthService::login(
        &pool,
        common::TEST_JWT_SECRET,
        common::TEST_JWT_EXPIRY_HOURS,
        &login_req,
    )
    .await
    .expect("login with admin-set password must succeed");
}

/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
/// Get Me
/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn get_me_returns_user_info_for_valid_user() {
    let pool = common::test_pool().await;
    let user_id = common::seed_user(&pool, "getme_user", "qc").await.unwrap();

    let info = AuthService::get_me(&pool, user_id)
        .await
        .expect("get_me must succeed");

    assert_eq!(info.username, "getme_user");
    assert_eq!(info.role, "qc");
    assert_eq!(info.display_name, "getme_user");
}

#[tokio::test]
async fn get_me_nonexistent_user_returns_not_found() {
    let pool = common::test_pool().await;

    let err = AuthService::get_me(&pool, 99999)
        .await
        .expect_err("must fail for nonexistent user");

    let msg = err.to_string();
    assert!(
        msg.contains("User not found") || msg.contains("NotFound"),
        "expected not-found error, got: {msg}",
    );
}

#[tokio::test]
async fn get_me_soft_deleted_user_returns_not_found() {
    let pool = common::test_pool().await;
    let user_id = common::seed_user(&pool, "deleted_getme", "admin").await.unwrap();
    AuthService::delete_user(&pool, user_id).await.unwrap();

    let err = AuthService::get_me(&pool, user_id)
        .await
        .expect_err("must fail for soft-deleted user");

    let msg = err.to_string();
    assert!(
        msg.contains("User not found") || msg.contains("NotFound"),
        "expected not-found for deleted user, got: {msg}",
    );
}

/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
/// List Users
/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn list_users_returns_paginated_results() {
    let pool = common::test_pool().await;

    // Seed users (the seed from test setup also counts)
    common::seed_user(&pool, "list_a", "admin").await.unwrap();
    common::seed_user(&pool, "list_b", "warehouse").await.unwrap();
    common::seed_user(&pool, "list_c", "qc").await.unwrap();

    let params = PaginationParams {
        page: Some(1),
        page_size: Some(2),
        sort_by: None,
        sort_order: None,
    };

    let (users, total) = AuthService::list_users(&pool, &params, None)
        .await
        .expect("list_users must succeed");

    // Migration 001 creates a seed admin user, so total is 4 (seed admin + 3 seeded)
    assert_eq!(total, 4, "total should include seed admin + 3 seeded users");
    assert_eq!(users.len(), 2, "page_size=2 should return 2 users");
}

#[tokio::test]
async fn list_users_second_page_returns_remaining() {
    let pool = common::test_pool().await;

    common::seed_user(&pool, "page_a", "admin").await.unwrap();
    common::seed_user(&pool, "page_b", "warehouse").await.unwrap();
    common::seed_user(&pool, "page_c", "qc").await.unwrap();

    let params = PaginationParams {
        page: Some(2),
        page_size: Some(2),
        sort_by: None,
        sort_order: None,
    };

    let (users, total) = AuthService::list_users(&pool, &params, None)
        .await
        .expect("list_users must succeed");

    // Migration 001 creates a seed admin user, so total is 4 (seed admin + 3 seeded)
    assert_eq!(total, 4, "total must reflect all users");
    assert_eq!(users.len(), 2, "page 2 with total=4 and page_size=2 should return 2 users");
}

#[tokio::test]
async fn list_users_filters_by_search_query() {
    let pool = common::test_pool().await;

    common::seed_user(&pool, "alice", "admin").await.unwrap();
    common::seed_user(&pool, "bob", "warehouse").await.unwrap();
    common::seed_user(&pool, "charlie", "qc").await.unwrap();

    let params = PaginationParams {
        page: Some(1),
        page_size: Some(20),
        sort_by: None,
        sort_order: None,
    };

    let (users, total) = AuthService::list_users(&pool, &params, Some("alice"))
        .await
        .expect("list_users with search must succeed");

    assert_eq!(total, 1, "search should match one user");
    assert_eq!(users[0].username, "alice");
}

/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
/// Change Role
/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn change_role_updates_role_successfully() {
    let pool = common::test_pool().await;
    let user_id = common::seed_user(&pool, "role_switch", "warehouse").await.unwrap();

    let info = AuthService::change_role(&pool, user_id, "admin")
        .await
        .expect("change_role must succeed");

    assert_eq!(info.role, "admin");

    // Verify persistence via get_me
    let fetched = AuthService::get_me(&pool, user_id).await.unwrap();
    assert_eq!(fetched.role, "admin");
}

#[tokio::test]
async fn change_role_invalid_role_returns_validation_error() {
    let pool = common::test_pool().await;
    let user_id = common::seed_user(&pool, "role_bad", "warehouse").await.unwrap();

    let err = AuthService::change_role(&pool, user_id, "superadmin")
        .await
        .expect_err("must fail with invalid role");

    let msg = err.to_string();
    assert!(
        msg.contains("Invalid role") || msg.contains("Validation"),
        "expected invalid-role error, got: {msg}",
    );
}

#[tokio::test]
async fn change_role_nonexistent_user_returns_not_found() {
    let pool = common::test_pool().await;

    let err = AuthService::change_role(&pool, 99999, "admin")
        .await
        .expect_err("must fail for nonexistent user");

    let msg = err.to_string();
    assert!(
        msg.contains("User not found") || msg.contains("NotFound"),
        "expected not-found error, got: {msg}",
    );
}

/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
/// Delete User
/// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn delete_user_sets_deleted_at_timestamp() {
    let pool = common::test_pool().await;
    let user_id = common::seed_user(&pool, "deletable", "admin").await.unwrap();

    AuthService::delete_user(&pool, user_id)
        .await
        .expect("delete_user must succeed");

    let row: (Option<String>,) =
        sqlx::query_as("SELECT deleted_at FROM users WHERE id = ?")
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .expect("user row must still exist");
    assert!(row.0.is_some(), "deleted_at must be set after soft delete");
}

#[tokio::test]
async fn delete_user_already_deleted_returns_not_found() {
    let pool = common::test_pool().await;
    let user_id = common::seed_user(&pool, "double_del", "admin").await.unwrap();

    AuthService::delete_user(&pool, user_id)
        .await
        .expect("first delete must succeed");

    let err = AuthService::delete_user(&pool, user_id)
        .await
        .expect_err("second delete must fail");

    let msg = err.to_string();
    assert!(
        msg.contains("User not found") || msg.contains("NotFound"),
        "expected not-found error on second delete, got: {msg}",
    );
}

#[tokio::test]
async fn delete_user_nonexistent_returns_not_found() {
    let pool = common::test_pool().await;

    let err = AuthService::delete_user(&pool, 99999)
        .await
        .expect_err("must fail for nonexistent user");

    let msg = err.to_string();
    assert!(
        msg.contains("User not found") || msg.contains("NotFound"),
        "expected not-found error, got: {msg}",
    );
}
