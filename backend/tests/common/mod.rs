//! Shared test utilities for integration tests.
//!
//! Provides:
//! - SQLite test pool (tempfile-based file DB) with migrations applied
//! - Pool setup helpers
//! - Seed data helpers for common (schema-agnostic) test fixtures
//!
//! Note: pipe-specific seed helpers (seamless/screen pipes, quality certs,
//! API 5CT refs, pipe attachments, pipe-typed order/contract items) were
//! removed together with the steel-pipe domain. Per-module test files that
//! still reference them must be adapted by their module owners against the
//! generic ERP schema (items/inventory with item_id).

use std::sync::atomic::{AtomicU64, Ordering};

use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};

/// JWT secret used in tests.
pub const TEST_JWT_SECRET: &str = "test-jwt-secret-for-integration-tests";
/// JWT expiry in hours for tests.
pub const TEST_JWT_EXPIRY_HOURS: i64 = 24;
/// Refresh token expiry in days for tests.
pub const TEST_REFRESH_TOKEN_EXPIRY_DAYS: i64 = 30;

/// Monotonic counter for unique per-test database file names.
static DB_SEQ: AtomicU64 = AtomicU64::new(0);

/// Run all migrations against the given pool.
///
/// Migrations live in `backend/migrations` and are executed in filename order
/// by `sqlx::migrate!`. The migration files themselves are being rewritten
/// (PostgreSQL → SQLite) in parallel; this helper only invokes them at
/// runtime, so it is intentionally agnostic to their current contents.
pub async fn run_migrations(pool: &SqlitePool) {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .expect("failed to run migrations");
}

/// Create a fresh SQLite test pool backed by a unique temp-file database.
///
/// Each call creates a **dedicated temp file** (e.g. `/tmp/erp_test_<pid>_<n>.db`),
/// giving every test an isolated database. A file-based DB is used instead of
/// `sqlite::memory:` so pools may open more than one connection (several tests
/// rely on concurrent queries); an in-memory DB is per-connection and would
/// not be shared across a multi-connection pool.
///
/// ## Panics
/// Panics if pool creation or migration fails.
pub async fn create_test_pool() -> SqlitePool {
    let file = tempfile::NamedTempFile::new().expect("failed to create temp database file");
    let path = file.path().to_path_buf();
    // Keep the temp file alive for the lifetime of the pool (the OS cleans
    // it up on process exit; SQLite's -wal/-shm sidecars follow the same dir).
    std::mem::forget(file);

    let opts = SqliteConnectOptions::new()
        .filename(&path)
        .create_if_missing(true)
        .foreign_keys(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(opts)
        .await
        .expect("failed to connect to sqlite test database");

    run_migrations(&pool).await;
    pool
}

/// Backwards-compatible alias kept so existing per-module test files continue
/// to call `common::test_pool()`. New code should use [`create_test_pool`].
pub async fn test_pool() -> SqlitePool {
    create_test_pool().await
}

/// Counter helper kept for callers that need unique identifiers per test run.
pub fn next_seq() -> u64 {
    DB_SEQ.fetch_add(1, Ordering::Relaxed)
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Seed helpers — Locations
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Create a location row for testing.
pub async fn seed_location(
    pool: &SqlitePool,
    zone: &str,
    shelf: &str,
    level: &str,
) -> sqlx::Result<i64> {
    let full_code = format!("{}-{}-{}", zone, shelf, level);
    let id = sqlx::query_scalar(
        r#"
        INSERT INTO locations
          (zone_code, shelf_code, level_code, full_code, used_count, is_active,
           created_at, updated_at)
        VALUES
          (?, ?, ?, ?, 0, TRUE, datetime('now'), datetime('now'))
        RETURNING id
        "#,
    )
    .bind(zone)
    .bind(shelf)
    .bind(level)
    .bind(&full_code)
    .fetch_one(pool)
    .await?;

    Ok(id)
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Seed helpers — Users
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Create a user for testing with a known password "password123" (returns user_id).
pub async fn seed_user(pool: &SqlitePool, username: &str, role: &str) -> sqlx::Result<i64> {
    seed_user_with_password(pool, username, role, "password123").await
}

/// Create a user with a specific password (returns user_id).
pub async fn seed_user_with_password(
    pool: &SqlitePool,
    username: &str,
    role: &str,
    password: &str,
) -> sqlx::Result<i64> {
    use argon2::{
        password_hash::{rand_core::OsRng, SaltString},
        Argon2, PasswordHasher,
    };

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .expect("failed to hash password")
        .to_string();

    let id = sqlx::query_scalar(
        r#"
        INSERT INTO users
          (username, password_hash, display_name, role, email, is_active, created_at, updated_at)
        VALUES
          (?, ?, ?, ?, ?, TRUE, datetime('now'), datetime('now'))
        RETURNING id
        "#,
    )
    .bind(username)
    .bind(&hash)
    .bind(username) // display_name = username
    .bind(role)
    .bind(format!("{}@test.local", username))
    .fetch_one(pool)
    .await?;

    Ok(id)
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Seed helpers — Suppliers
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Create a supplier row for testing (returns supplier ID).
pub async fn seed_supplier(pool: &SqlitePool, code: &str, name: &str) -> sqlx::Result<i64> {
    let id = sqlx::query_scalar(
        r#"
        INSERT INTO suppliers
          (supplier_code, name, contact_person, phone, email, address, is_active, notes,
           created_at, updated_at)
        VALUES
          (?, ?, 'Contact', '13800138000', ?, 'Test Address', TRUE, 'test supplier',
           datetime('now'), datetime('now'))
        RETURNING id
        "#,
    )
    .bind(code)
    .bind(name)
    .bind(format!("{}@supplier.local", code))
    .fetch_one(pool)
    .await?;

    Ok(id)
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Seed helpers — Customers
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Create a customer row for testing (returns customer ID).
pub async fn seed_customer(pool: &SqlitePool, code: &str, name: &str) -> sqlx::Result<i64> {
    let id = sqlx::query_scalar(
        r#"
        INSERT INTO customers
          (customer_code, name, contact_person, phone, email, address, is_active, notes,
           created_at, updated_at)
        VALUES
          (?, ?, 'Contact', '13800138001', ?, 'Test Address', TRUE, 'test customer',
           datetime('now'), datetime('now'))
        RETURNING id
        "#,
    )
    .bind(code)
    .bind(name)
    .bind(format!("{}@customer.local", code))
    .fetch_one(pool)
    .await?;

    Ok(id)
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Seed helpers — Purchase Orders
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Create a purchase order row for testing (returns order ID).
pub async fn seed_purchase_order(
    pool: &SqlitePool,
    order_no: &str,
    supplier_id: i64,
    status: &str,
) -> sqlx::Result<i64> {
    let id = sqlx::query_scalar(
        r#"
        INSERT INTO purchase_orders
          (order_no, supplier_id, order_date, status, total_amount, notes, created_by,
           created_at, updated_at)
        VALUES
          (?, ?, datetime('now'), ?, NULL, 'test PO', NULL,
           datetime('now'), datetime('now'))
        RETURNING id
        "#,
    )
    .bind(order_no)
    .bind(supplier_id)
    .bind(status)
    .fetch_one(pool)
    .await?;

    Ok(id)
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Seed helpers — Sales Orders
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Create a sales order row for testing (returns order ID).
pub async fn seed_sales_order(
    pool: &SqlitePool,
    order_no: &str,
    customer_id: i64,
    status: &str,
) -> sqlx::Result<i64> {
    let id = sqlx::query_scalar(
        r#"
        INSERT INTO sales_orders
          (order_no, customer_id, order_date, status, total_amount, notes, created_by,
           created_at, updated_at)
        VALUES
          (?, ?, datetime('now'), ?, NULL, 'test SO', NULL,
           datetime('now'), datetime('now'))
        RETURNING id
        "#,
    )
    .bind(order_no)
    .bind(customer_id)
    .bind(status)
    .fetch_one(pool)
    .await?;

    Ok(id)
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Seed helpers — Contracts
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Create a contract row for testing (returns contract ID).
pub async fn seed_contract(
    pool: &SqlitePool,
    contract_no: &str,
    contract_type: &str,
    title: &str,
    status: &str,
) -> sqlx::Result<i64> {
    let id = sqlx::query_scalar(
        r#"
        INSERT INTO contracts
          (contract_no, contract_type, title, party_a, party_b, sign_date, start_date, end_date,
           total_amount, status, notes, created_by, created_at, updated_at)
        VALUES
          (?, ?, ?, 'Party A Corp', 'Party B Corp',
           datetime('now'), datetime('now'), datetime('now', '+1 year'),
           0.0, ?, NULL, NULL,
           datetime('now'), datetime('now'))
        RETURNING id
        "#,
    )
    .bind(contract_no)
    .bind(contract_type)
    .bind(title)
    .bind(status)
    .fetch_one(pool)
    .await?;

    Ok(id)
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Seed helpers — Operation Logs
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Create an operation log row for testing (returns log ID).
pub async fn seed_operation_log(
    pool: &SqlitePool,
    action: &str,
    entity_type: &str,
    entity_id: i64,
    user_id: i64,
) -> sqlx::Result<i64> {
    let id = sqlx::query_scalar(
        r#"
        INSERT INTO operation_logs
          (action, entity_type, entity_id, user_id, details,
           created_at)
        VALUES
          (?, ?, ?, ?, '{}',
           datetime('now'))
        RETURNING id
        "#,
    )
    .bind(action)
    .bind(entity_type)
    .bind(entity_id)
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(id)
}
