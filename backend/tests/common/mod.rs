//! Shared test utilities for integration tests.
//!
//! Provides:
//! - In-memory SQLite pool with migrations applied
//! - Pool setup/teardown helpers
//! - Seed data helpers for common test fixtures

use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};

/// JWT secret used in tests.
pub const TEST_JWT_SECRET: &str = "test-jwt-secret-for-integration-tests";
/// JWT expiry in hours for tests.
pub const TEST_JWT_EXPIRY_HOURS: i64 = 24;

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

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Seed helpers — Pipes
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

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

/// Create a seamless pipe with full custom spec for testing.
pub async fn seed_seamless_pipe_full(
    pool: &SqlitePool,
    pipe_number: &str,
    status: &str,
    grade: &str,
    od: f64,
    wt: f64,
    length: f64,
) -> sqlx::Result<i64> {
    let result = sqlx::query(
        r#"
        INSERT INTO seamless_pipes
          (pipe_number, batch_number, pipe_type, grade, od, wt, length, weight_per_unit,
           end_type, coupling_type, heat_number, manufacturer, location_id, status, notes,
           created_at, updated_at)
        VALUES
          ($1, 'BN-001', 'casing', $2, $4, $5, $6, 40.0,
           'BTC', 'N80Q', 'HN-001', 'test manufacturer', NULL, $3, 'test pipe',
           datetime('now'), datetime('now'))
        "#,
    )
    .bind(pipe_number)
    .bind(grade)
    .bind(status)
    .bind(od)
    .bind(wt)
    .bind(length)
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
        Argon2,
        PasswordHasher,
    };

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .expect("failed to hash password")
        .to_string();

    let result = sqlx::query(
        r#"
        INSERT INTO users
          (username, password_hash, display_name, role, email, is_active, created_at, updated_at)
        VALUES
          ($1, $2, $3, $4, $5, 1, datetime('now'), datetime('now'))
        "#,
    )
    .bind(username)
    .bind(&hash)
    .bind(username) // display_name = username
    .bind(role)
    .bind(format!("{}@test.local", username))
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Seed helpers — Suppliers
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Create a supplier row for testing (returns supplier ID).
pub async fn seed_supplier(
    pool: &SqlitePool,
    code: &str,
    name: &str,
) -> sqlx::Result<i64> {
    let result = sqlx::query(
        r#"
        INSERT INTO suppliers
          (supplier_code, name, contact_person, phone, email, address, is_active, notes,
           created_at, updated_at)
        VALUES
          ($1, $2, 'Contact', '13800138000', $3, 'Test Address', 1, 'test supplier',
           datetime('now'), datetime('now'))
        "#,
    )
    .bind(code)
    .bind(name)
    .bind(format!("{}@supplier.local", code))
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Seed helpers — Customers
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Create a customer row for testing (returns customer ID).
pub async fn seed_customer(
    pool: &SqlitePool,
    code: &str,
    name: &str,
) -> sqlx::Result<i64> {
    let result = sqlx::query(
        r#"
        INSERT INTO customers
          (customer_code, name, contact_person, phone, email, address, is_active, notes,
           created_at, updated_at)
        VALUES
          ($1, $2, 'Contact', '13800138001', $3, 'Test Address', 1, 'test customer',
           datetime('now'), datetime('now'))
        "#,
    )
    .bind(code)
    .bind(name)
    .bind(format!("{}@customer.local", code))
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
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
    let result = sqlx::query(
        r#"
        INSERT INTO purchase_orders
          (order_no, supplier_id, order_date, status, total_amount, notes, created_by,
           created_at, updated_at)
        VALUES
          ($1, $2, datetime('now'), $3, NULL, 'test PO', NULL,
           datetime('now'), datetime('now'))
        "#,
    )
    .bind(order_no)
    .bind(supplier_id)
    .bind(status)
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

/// Create a purchase order item row for testing (returns item ID).
pub async fn seed_purchase_order_item(
    pool: &SqlitePool,
    order_id: i64,
    pipe_type: &str,
    grade: &str,
    quantity: i64,
    unit_price: Option<f64>,
) -> sqlx::Result<i64> {
    let total_price = unit_price.map(|p| p * quantity as f64);
    let result = sqlx::query(
        r#"
        INSERT INTO purchase_order_items
          (order_id, pipe_type, grade, od, wt, quantity, received_quantity, unit_price, total_price, notes,
           created_at)
        VALUES
          ($1, $2, $3, 177.8, 9.19, $4, 0, $5, $6, NULL,
           datetime('now'))
        "#,
    )
    .bind(order_id)
    .bind(pipe_type)
    .bind(grade)
    .bind(quantity)
    .bind(unit_price)
    .bind(total_price)
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
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
    let result = sqlx::query(
        r#"
        INSERT INTO sales_orders
          (order_no, customer_id, order_date, status, total_amount, notes, created_by,
           created_at, updated_at)
        VALUES
          ($1, $2, datetime('now'), $3, NULL, 'test SO', NULL,
           datetime('now'), datetime('now'))
        "#,
    )
    .bind(order_no)
    .bind(customer_id)
    .bind(status)
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

/// Create a sales order item row for testing (returns item ID).
pub async fn seed_sales_order_item(
    pool: &SqlitePool,
    order_id: i64,
    pipe_type: &str,
    grade: &str,
    quantity: i64,
    unit_price: Option<f64>,
) -> sqlx::Result<i64> {
    let total_price = unit_price.map(|p| p * quantity as f64);
    let result = sqlx::query(
        r#"
        INSERT INTO sales_order_items
          (order_id, pipe_type, grade, od, wt, quantity, delivered_quantity, unit_price, total_price, notes,
           created_at)
        VALUES
          ($1, $2, $3, 177.8, 9.19, $4, 0, $5, $6, NULL,
           datetime('now'))
        "#,
    )
    .bind(order_id)
    .bind(pipe_type)
    .bind(grade)
    .bind(quantity)
    .bind(unit_price)
    .bind(total_price)
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Seed helpers — Quality
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Create a quality certificate row for testing (returns cert ID).
pub async fn seed_quality_cert(
    pool: &SqlitePool,
    cert_number: &str,
    pipe_type: &str,
    pipe_id: i64,
    result: &str,
) -> sqlx::Result<i64> {
    let result = sqlx::query(
        r#"
        INSERT INTO quality_certs
          (cert_number, pipe_type, pipe_id, cert_date, result, inspector, inspection_body, notes,
           created_at, updated_at)
        VALUES
          ($1, $2, $3, datetime('now'), $4, 'Test Inspector', 'Test Lab', NULL,
           datetime('now'), datetime('now'))
        "#,
    )
    .bind(cert_number)
    .bind(pipe_type)
    .bind(pipe_id)
    .bind(result)
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

/// Create an API 5CT grade reference row (returns ID).
pub async fn seed_api5ct_grade_ref(
    pool: &SqlitePool,
    grade: &str,
) -> sqlx::Result<i64> {
    let result = sqlx::query(
        r#"
        INSERT INTO api_5ct_grade_ref
          (grade, yield_strength_min, yield_strength_max, tensile_strength_min,
           hardness_max, carbon_content_max, notes)
        VALUES
          ($1, 379.0, 552.0, 517.0, 'HRC 22', 0.35, 'API 5CT grade reference')
        "#,
    )
    .bind(grade)
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

/// Create a pipe attachment row for testing (returns attachment ID).
pub async fn seed_pipe_attachment(
    pool: &SqlitePool,
    pipe_type: &str,
    pipe_id: i64,
    file_name: &str,
) -> sqlx::Result<i64> {
    let result = sqlx::query(
        r#"
        INSERT INTO pipe_attachments
          (pipe_type, pipe_id, file_name, file_path, file_size, content_type, uploaded_by,
           created_at)
        VALUES
          ($1, $2, $3, '/test/path/' || $3, 1024, 'application/pdf', NULL,
           datetime('now'))
        "#,
    )
    .bind(pipe_type)
    .bind(pipe_id)
    .bind(file_name)
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
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
    let result = sqlx::query(
        r#"
        INSERT INTO contracts
          (contract_no, contract_type, title, party_a, party_b, sign_date, start_date, end_date,
           total_amount, status, notes, created_by, created_at, updated_at)
        VALUES
          ($1, $2, $3, 'Party A Corp', 'Party B Corp',
           datetime('now'), datetime('now'), datetime('now', '+1 year'),
           0.0, $4, NULL, NULL,
           datetime('now'), datetime('now'))
        "#,
    )
    .bind(contract_no)
    .bind(contract_type)
    .bind(title)
    .bind(status)
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

/// Create a contract item row for testing (returns item ID).
pub async fn seed_contract_item(
    pool: &SqlitePool,
    contract_id: i64,
    pipe_type: &str,
    grade: &str,
    quantity: i64,
    unit_price: Option<f64>,
) -> sqlx::Result<i64> {
    let total_price = unit_price.map(|p| p * quantity as f64);
    let result = sqlx::query(
        r#"
        INSERT INTO contract_items
          (contract_id, pipe_type, grade, od, wt, quantity, unit_price, total_price, notes,
           created_at)
        VALUES
          ($1, $2, $3, 177.8, 9.19, $4, $5, $6, NULL,
           datetime('now'))
        "#,
    )
    .bind(contract_id)
    .bind(pipe_type)
    .bind(grade)
    .bind(quantity)
    .bind(unit_price)
    .bind(total_price)
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

/// Create a contract payment milestone row for testing (returns payment ID).
pub async fn seed_contract_payment(
    pool: &SqlitePool,
    contract_id: i64,
    payment_type: &str,
    amount: f64,
) -> sqlx::Result<i64> {
    let result = sqlx::query(
        r#"
        INSERT INTO contract_milestones
          (contract_id, due_date, amount, payment_type, is_paid, paid_date, notes,
           created_at)
        VALUES
          ($1, datetime('now'), $2, $3, 0, NULL, NULL,
           datetime('now'))
        "#,
    )
    .bind(contract_id)
    .bind(amount)
    .bind(payment_type)
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Seed helpers — Inventory
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Create an inventory log row for testing (returns log ID).
pub async fn seed_inventory_log(
    pool: &SqlitePool,
    pipe_type: &str,
    pipe_id: i64,
    change_type: &str,
    quantity_change: i64,
) -> sqlx::Result<i64> {
    let result = sqlx::query(
        r#"
        INSERT INTO inventory_logs
          (pipe_type, pipe_id, change_type, ref_type, ref_id, quantity_change, notes,
           created_at)
        VALUES
          ($1, $2, $3, 'test', 0, $4, 'test log entry',
           datetime('now'))
        "#,
    )
    .bind(pipe_type)
    .bind(pipe_id)
    .bind(change_type)
    .bind(quantity_change)
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
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
    let result = sqlx::query(
        r#"
        INSERT INTO operation_logs
          (action, entity_type, entity_id, user_id, details,
           created_at)
        VALUES
          ($1, $2, $3, $4, '{}',
           datetime('now'))
        "#,
    )
    .bind(action)
    .bind(entity_type)
    .bind(entity_id)
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}
