//! Shared test utilities for integration tests.
//!
//! Provides:
//! - In-memory SQLite pool with migrations applied
//! - Pool setup/teardown helpers
//! - Seed data helpers for common test fixtures

use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};

/// Create a test database pool using an in-memory SQLite database.
/// Runs all migrations from `./migrations` and returns the pool.
///
/// ## Panics
/// Panics if pool creation or migration fails.
pub async fn test_pool() -> SqlitePool {
    test_pool_with_migrations("./migrations").await
}

/// Like `test_pool()` but accepts a custom migrations path.
pub async fn test_pool_with_migrations(_migrations_path: &str) -> SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .min_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("failed to create in-memory SQLite pool");

    // Run migrations
    let migrator = sqlx::migrate!("./migrations");
    migrator
        .run(&pool)
        .await
        .expect("failed to run migrations");

    // Enable foreign keys for integrity checks
    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&pool)
        .await
        .expect("failed to enable foreign_keys");

    pool
}

/// Like `test_pool()` but uses a file-based temporary SQLite database.
/// The file is deleted when the pool is dropped.
///
/// ## Panics
/// Panics if pool creation or migration fails.
pub async fn temp_file_pool() -> (SqlitePool, tempfile::NamedTempFile) {
    let temp_file = tempfile::NamedTempFile::new()
        .expect("failed to create temp file");
    let path = temp_file.path().to_str().unwrap();
    let database_url = format!("sqlite://{path}?mode=rwc");

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .min_connections(1)
        .connect(&database_url)
        .await
        .expect("failed to create file-based SQLite pool");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("failed to run migrations");

    // Enable foreign keys for integrity checks
    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&pool)
        .await
        .expect("failed to enable foreign_keys");

    (pool, temp_file)
}

/// Create a seamless pipe row for testing.
pub async fn seed_seamless_pipe(
    pool: &SqlitePool,
    pipe_number: &str,
    status: &str,
    grade: &str,
) -> sqlx::Result<i64> {
    let result = sqlx::query(
        r#"
        INSERT INTO seamless_pipes
          (pipe_number, batch_number, pipe_type, grade, od, wt, length, weight_per_unit,
           end_type, coupling_type, heat_number, manufacturer, location_id, status, notes,
           created_at, updated_at)
        VALUES
          ($1, 'BN-001', 'casing', $2, 177.8, 9.19, 9.5, 40.0,
           'BTC', 'N80Q', 'HN-001', 'test manufacturer', NULL, $3, 'test pipe',
           datetime('now'), datetime('now'))
        "#,
    )
    .bind(pipe_number)
    .bind(grade)
    .bind(status)
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

/// Create a screen pipe row for testing.
pub async fn seed_screen_pipe(
    pool: &SqlitePool,
    pipe_number: &str,
    status: &str,
    grade: &str,
) -> sqlx::Result<i64> {
    let result = sqlx::query(
        r#"
        INSERT INTO screen_pipes
          (pipe_number, batch_number, screen_type, slot_size, filtration_grade,
           base_od, base_wt, base_grade, base_end_type, length, weight_per_unit,
           heat_number, manufacturer, location_id, status, notes, created_at, updated_at)
        VALUES
          ($1, 'BN-001', 'slotted', 0.02, 'standard',
           177.8, 9.19, $2, 'BTC', 9.5, 40.0,
           'HN-001', 'test manufacturer', NULL, $3, 'test screen pipe',
           datetime('now'), datetime('now'))
        "#,
    )
    .bind(pipe_number)
    .bind(grade)
    .bind(status)
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

/// Create a location row for testing.
pub async fn seed_location(
    pool: &SqlitePool,
    zone: &str,
    shelf: &str,
    level: &str,
) -> sqlx::Result<i64> {
    let full_code = format!("{}-{}-{}", zone, shelf, level);
    let result = sqlx::query(
        r#"
        INSERT INTO locations
          (zone_code, shelf_code, level_code, full_code, used_count, is_active,
           created_at, updated_at)
        VALUES
          ($1, $2, $3, $4, 0, 1, datetime('now'), datetime('now'))
        "#,
    )
    .bind(zone)
    .bind(shelf)
    .bind(level)
    .bind(&full_code)
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

/// Create a user for testing (returns user_id).
pub async fn seed_user(pool: &SqlitePool, username: &str, role: &str) -> sqlx::Result<i64> {
    use argon2::{
        password_hash::{rand_core::OsRng, SaltString},
        Argon2,
        PasswordHasher,
    };

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(b"password123", &salt)
        .expect("failed to hash password")
        .to_string();

    let result = sqlx::query(
        r#"
        INSERT INTO users
          (username, password_hash, real_name, role, email, is_active, created_at, updated_at)
        VALUES
          ($1, $2, $3, $4, $5, 1, datetime('now'), datetime('now'))
        "#,
    )
    .bind(username)
    .bind(&hash)
    .bind(username) // real_name = username
    .bind(role)
    .bind(format!("{}@test.local", username))
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}
