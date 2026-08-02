//! Shared test utilities for integration tests.
//!
//! Provides:
//! - PostgreSQL test pool with migrations applied
//! - Pool setup helpers
//! - Seed data helpers for common test fixtures

use sqlx::postgres::{PgPool, PgPoolOptions};

/// JWT secret used in tests.
pub const TEST_JWT_SECRET: &str = "test-jwt-secret-for-integration-tests";
/// JWT expiry in hours for tests.
pub const TEST_JWT_EXPIRY_HOURS: i64 = 24;
/// Refresh token expiry in days for tests.
pub const TEST_REFRESH_TOKEN_EXPIRY_DAYS: i64 = 30;

/// Test database URL (local PostgreSQL 18.4 instance via /tmp socket).
pub const TEST_DATABASE_URL: &str = "postgres://postgres@localhost:5432/steel_pipe_test";

/// Create a test database pool against the local PostgreSQL test database.
/// Resets the `public` schema, runs all migrations from `./migrations` and
/// returns the pool.
///
/// ## Panics
/// Panics if pool creation or migration fails.
pub async fn test_pool() -> PgPool {
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .min_connections(1)
        .connect(TEST_DATABASE_URL)
        .await
        .expect("failed to connect to test database");

    // Reset the schema so every test run starts from a clean state.
    sqlx::query("DROP SCHEMA public CASCADE; CREATE SCHEMA public;")
        .execute(&pool)
        .await
        .expect("failed to reset test schema");

    let migrator = sqlx::migrate!("./migrations");
    migrator.run(&pool).await.expect("failed to run migrations");

    pool
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Seed helpers — Pipes
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Create a seamless pipe row for testing.
pub async fn seed_seamless_pipe(
    pool: &PgPool,
    pipe_number: &str,
    status: &str,
    grade: &str,
) -> sqlx::Result<i64> {
    let id = sqlx::query_scalar(
        r#"
        INSERT INTO seamless_pipes
          (pipe_number, batch_number, pipe_type, grade, od, wt, length, weight_per_unit,
           end_type, coupling_type, heat_number, manufacturer, location_id, status, notes,
           created_at, updated_at)
        VALUES
          ($1, 'BN-001', 'casing', $2, 177.8, 9.19, 9.5, 40.0,
           'BTC', 'N80Q', 'HN-001', 'test manufacturer', NULL, $3, 'test pipe',
           NOW(), NOW())
        RETURNING id
        "#,
    )
    .bind(pipe_number)
    .bind(grade)
    .bind(status)
    .fetch_one(pool)
    .await?;

    Ok(id)
}

/// Create a seamless pipe with full custom spec for testing.
pub async fn seed_seamless_pipe_full(
    pool: &PgPool,
    pipe_number: &str,
    status: &str,
    grade: &str,
    od: f64,
    wt: f64,
    length: f64,
) -> sqlx::Result<i64> {
    let id = sqlx::query_scalar(
        r#"
        INSERT INTO seamless_pipes
          (pipe_number, batch_number, pipe_type, grade, od, wt, length, weight_per_unit,
           end_type, coupling_type, heat_number, manufacturer, location_id, status, notes,
           created_at, updated_at)
        VALUES
          ($1, 'BN-001', 'casing', $2, $4, $5, $6, 40.0,
           'BTC', 'N80Q', 'HN-001', 'test manufacturer', NULL, $3, 'test pipe',
           NOW(), NOW())
        RETURNING id
        "#,
    )
    .bind(pipe_number)
    .bind(grade)
    .bind(status)
    .bind(od)
    .bind(wt)
    .bind(length)
    .fetch_one(pool)
    .await?;

    Ok(id)
}

/// Create a screen pipe row for testing.
pub async fn seed_screen_pipe(
    pool: &PgPool,
    pipe_number: &str,
    status: &str,
    grade: &str,
) -> sqlx::Result<i64> {
    let id = sqlx::query_scalar(
        r#"
        INSERT INTO screen_pipes
          (pipe_number, batch_number, screen_type, slot_size, filtration_grade,
           base_od, base_wt, base_grade, base_end_type, length, weight_per_unit,
           heat_number, manufacturer, location_id, status, notes, created_at, updated_at)
        VALUES
          ($1, 'BN-001', 'slotted', 0.02, 'standard',
            177.8, 9.19, $2, 'BTC', 9.5, 40.0,
            'HN-001', 'test manufacturer', NULL, $3, 'test screen pipe',
            NOW(), NOW())
        RETURNING id
        "#,
    )
    .bind(pipe_number)
    .bind(grade)
    .bind(status)
    .fetch_one(pool)
    .await?;

    Ok(id)
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Seed helpers — Locations
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Create a location row for testing.
pub async fn seed_location(
    pool: &PgPool,
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
          ($1, $2, $3, $4, 0, TRUE, NOW(), NOW())
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
pub async fn seed_user(pool: &PgPool, username: &str, role: &str) -> sqlx::Result<i64> {
    seed_user_with_password(pool, username, role, "password123").await
}

/// Create a user with a specific password (returns user_id).
pub async fn seed_user_with_password(
    pool: &PgPool,
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
          ($1, $2, $3, $4, $5, TRUE, NOW(), NOW())
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
pub async fn seed_supplier(pool: &PgPool, code: &str, name: &str) -> sqlx::Result<i64> {
    let id = sqlx::query_scalar(
        r#"
        INSERT INTO suppliers
          (supplier_code, name, contact_person, phone, email, address, is_active, notes,
           created_at, updated_at)
        VALUES
          ($1, $2, 'Contact', '13800138000', $3, 'Test Address', TRUE, 'test supplier',
           NOW(), NOW())
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
pub async fn seed_customer(pool: &PgPool, code: &str, name: &str) -> sqlx::Result<i64> {
    let id = sqlx::query_scalar(
        r#"
        INSERT INTO customers
          (customer_code, name, contact_person, phone, email, address, is_active, notes,
           created_at, updated_at)
        VALUES
          ($1, $2, 'Contact', '13800138001', $3, 'Test Address', TRUE, 'test customer',
           NOW(), NOW())
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
    pool: &PgPool,
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
          ($1, $2, NOW(), $3, NULL, 'test PO', NULL,
           NOW(), NOW())
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

/// Create a purchase order item row for testing (returns item ID).
pub async fn seed_purchase_order_item(
    pool: &PgPool,
    order_id: i64,
    pipe_type: &str,
    grade: &str,
    quantity: i64,
    unit_price: Option<f64>,
) -> sqlx::Result<i64> {
    let total_price = unit_price.map(|p| p * quantity as f64);
    let id = sqlx::query_scalar(
        r#"
        INSERT INTO purchase_order_items
          (order_id, pipe_type, grade, od, wt, quantity, received_quantity, unit_price, total_price, notes,
           created_at)
        VALUES
          ($1, $2, $3, 177.8, 9.19, $4, 0, $5, $6, NULL,
           NOW())
        RETURNING id
        "#,
    )
    .bind(order_id)
    .bind(pipe_type)
    .bind(grade)
    .bind(quantity)
    .bind(unit_price)
    .bind(total_price)
    .fetch_one(pool)
    .await?;

    Ok(id)
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Seed helpers — Sales Orders
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Create a sales order row for testing (returns order ID).
pub async fn seed_sales_order(
    pool: &PgPool,
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
          ($1, $2, NOW(), $3, NULL, 'test SO', NULL,
           NOW(), NOW())
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

/// Create a sales order item row for testing (returns item ID).
pub async fn seed_sales_order_item(
    pool: &PgPool,
    order_id: i64,
    pipe_type: &str,
    grade: &str,
    quantity: i64,
    unit_price: Option<f64>,
) -> sqlx::Result<i64> {
    let total_price = unit_price.map(|p| p * quantity as f64);
    let id = sqlx::query_scalar(
        r#"
        INSERT INTO sales_order_items
          (order_id, pipe_type, grade, od, wt, quantity, delivered_quantity, unit_price, total_price, notes,
           created_at)
        VALUES
          ($1, $2, $3, 177.8, 9.19, $4, 0, $5, $6, NULL,
           NOW())
        RETURNING id
        "#,
    )
    .bind(order_id)
    .bind(pipe_type)
    .bind(grade)
    .bind(quantity)
    .bind(unit_price)
    .bind(total_price)
    .fetch_one(pool)
    .await?;

    Ok(id)
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Seed helpers — Quality
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Create a quality certificate row for testing (returns cert ID).
pub async fn seed_quality_cert(
    pool: &PgPool,
    cert_number: &str,
    pipe_type: &str,
    pipe_id: i64,
    result: &str,
) -> sqlx::Result<i64> {
    let id = sqlx::query_scalar(
        r#"
        INSERT INTO quality_certs
          (cert_number, pipe_type, pipe_id, cert_date, result, inspector, inspection_body, notes,
           created_at, updated_at)
        VALUES
          ($1, $2, $3, NOW(), $4, 'Test Inspector', 'Test Lab', NULL,
           NOW(), NOW())
        RETURNING id
        "#,
    )
    .bind(cert_number)
    .bind(pipe_type)
    .bind(pipe_id)
    .bind(result)
    .fetch_one(pool)
    .await?;

    Ok(id)
}

/// Create an API 5CT grade reference row (returns ID).
pub async fn seed_api5ct_grade_ref(pool: &PgPool, grade: &str) -> sqlx::Result<i64> {
    let id = sqlx::query_scalar(
        r#"
        INSERT INTO api_5ct_grade_ref
          (grade, yield_strength_min, yield_strength_max, tensile_strength_min,
           hardness_max, carbon_content_max, notes)
        VALUES
          ($1, 379.0, 552.0, 517.0, 'HRC 22', 0.35, 'API 5CT grade reference')
        RETURNING id
        "#,
    )
    .bind(grade)
    .fetch_one(pool)
    .await?;

    Ok(id)
}

/// Create a pipe attachment row for testing (returns attachment ID).
pub async fn seed_pipe_attachment(
    pool: &PgPool,
    pipe_type: &str,
    pipe_id: i64,
    file_name: &str,
) -> sqlx::Result<i64> {
    let id = sqlx::query_scalar(
        r#"
        INSERT INTO pipe_attachments
          (pipe_type, pipe_id, file_name, file_path, file_size, content_type, uploaded_by,
           created_at)
        VALUES
          ($1, $2, $3, '/test/path/' || $3, 1024, 'application/pdf', NULL,
           NOW())
        RETURNING id
        "#,
    )
    .bind(pipe_type)
    .bind(pipe_id)
    .bind(file_name)
    .fetch_one(pool)
    .await?;

    Ok(id)
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Seed helpers — Contracts
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Create a contract row for testing (returns contract ID).
pub async fn seed_contract(
    pool: &PgPool,
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
          ($1, $2, $3, 'Party A Corp', 'Party B Corp',
           NOW(), NOW(), NOW() + INTERVAL '1 year',
           0.0, $4, NULL, NULL,
           NOW(), NOW())
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

/// Create a contract item row for testing (returns item ID).
pub async fn seed_contract_item(
    pool: &PgPool,
    contract_id: i64,
    pipe_type: &str,
    grade: &str,
    quantity: i64,
    unit_price: Option<f64>,
) -> sqlx::Result<i64> {
    let total_price = unit_price.map(|p| p * quantity as f64);
    let id = sqlx::query_scalar(
        r#"
        INSERT INTO contract_items
          (contract_id, pipe_type, grade, od, wt, quantity, unit_price, total_price, notes,
           created_at)
        VALUES
          ($1, $2, $3, 177.8, 9.19, $4, $5, $6, NULL,
           NOW())
        RETURNING id
        "#,
    )
    .bind(contract_id)
    .bind(pipe_type)
    .bind(grade)
    .bind(quantity)
    .bind(unit_price)
    .bind(total_price)
    .fetch_one(pool)
    .await?;

    Ok(id)
}

/// Create a contract payment milestone row for testing (returns payment ID).
pub async fn seed_contract_payment(
    pool: &PgPool,
    contract_id: i64,
    payment_type: &str,
    amount: f64,
) -> sqlx::Result<i64> {
    let id = sqlx::query_scalar(
        r#"
        INSERT INTO contract_milestones
          (contract_id, due_date, amount, payment_type, is_paid, paid_date, notes,
           created_at)
        VALUES
          ($1, NOW(), $2, $3, FALSE, NULL, NULL,
           NOW())
        RETURNING id
        "#,
    )
    .bind(contract_id)
    .bind(amount)
    .bind(payment_type)
    .fetch_one(pool)
    .await?;

    Ok(id)
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Seed helpers — Inventory
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Create an inventory log row for testing (returns log ID).
pub async fn seed_inventory_log(
    pool: &PgPool,
    pipe_type: &str,
    pipe_id: i64,
    change_type: &str,
    quantity_change: i64,
) -> sqlx::Result<i64> {
    let id = sqlx::query_scalar(
        r#"
        INSERT INTO inventory_logs
          (pipe_type, pipe_id, change_type, ref_type, ref_id, quantity_change, notes,
           created_at)
        VALUES
          ($1, $2, $3, 'test', 0, $4, 'test log entry',
           NOW())
        RETURNING id
        "#,
    )
    .bind(pipe_type)
    .bind(pipe_id)
    .bind(change_type)
    .bind(quantity_change)
    .fetch_one(pool)
    .await?;

    Ok(id)
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Seed helpers — Operation Logs
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Create an operation log row for testing (returns log ID).
pub async fn seed_operation_log(
    pool: &PgPool,
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
          ($1, $2, $3, $4, '{}',
           NOW())
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
