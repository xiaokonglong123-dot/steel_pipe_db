#![allow(dead_code)]

mod auth;
mod config;
mod db;
mod error;
mod handler;
mod middleware;
mod repository;
mod service;
mod domain;

use std::sync::Arc;
use axum::{
    middleware as axum_middleware,
    routing::{delete, get, post, put},
    Router,
};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

use config::AppConfig;
use db::init_pool;
use repository::user_repo::UserRepo;
use service::auth_service::AuthService;
use service::data_io_service::DataIoService;
use service::inventory_service::InventoryService;
use service::label_service::LabelService;
use service::purchase_service::PurchaseService;
use service::sales_service::SalesService;

pub struct AppState {
    pub config: AppConfig,
    pub db: sqlx::SqlitePool,
    pub auth_service: AuthService,
    pub purchase_service: PurchaseService,
    pub sales_service: SalesService,
    pub inventory_service: InventoryService,
    pub data_io_service: DataIoService,
    pub label_service: LabelService,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    dotenvy::dotenv().ok();
    let config = AppConfig::from_env();
    let pool = init_pool(&config.database_url)
        .await
        .expect("Failed to initialize database pool");

    db::migrations::run_migrations(&pool)
        .await
        .expect("Failed to run migrations");

    // Seed default admin if not exists
    seed_admin(&pool, &config).await;

    let state = Arc::new(AppState {
        auth_service: AuthService::new(UserRepo::new(pool.clone()), config.clone()),
        purchase_service: PurchaseService::new(pool.clone()),
        sales_service: SalesService::new(pool.clone()),
        inventory_service: InventoryService::new(pool.clone()),
        data_io_service: DataIoService::new(pool.clone()),
        label_service: LabelService::new(pool.clone()),
        config: config.clone(),
        db: pool,
    });

    let app = create_router(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("Failed to bind");
    tracing::info!("Server starting on 0.0.0.0:8080");
    axum::serve(listener, app)
        .await
        .expect("Server failed");
}

#[allow(dead_code)]
async fn seed_admin(pool: &sqlx::SqlitePool, _config: &AppConfig) {
    let exists: bool = sqlx::query_scalar(
        "SELECT COUNT(*) > 0 FROM users WHERE username = 'admin'"
    )
    .fetch_one(pool)
    .await
    .unwrap_or(false);

    if !exists {
        use argon2::PasswordHasher;
        let salt = argon2::password_hash::SaltString::generate(&mut rand::rngs::OsRng);
        let password_hash = argon2::Argon2::default()
            .hash_password(b"admin123", &salt)
            .expect("Failed to hash password")
            .to_string();

        sqlx::query(
            "INSERT INTO users (id, username, password_hash, display_name, role) VALUES (?, ?, ?, ?, 'admin')"
        )
        .bind(uuid::Uuid::new_v4().to_string())
        .bind("admin")
        .bind(&password_hash)
        .bind("System Administrator")
        .execute(pool)
        .await
        .expect("Failed to seed admin user");

        tracing::info!("Default admin user created (username: admin, password: admin123)");
    }
}

fn create_router(state: Arc<AppState>) -> Router {
    let public_routes = Router::new()
        .route("/api/v1/auth/login", post(handler::auth::login))
        .route("/api/v1/auth/refresh", post(handler::auth::refresh_token));

    let protected_routes = Router::new()
        // Pipe management
        .route("/api/v1/seamless-pipes", get(handler::pipes::list_seamless_pipes))
        .route("/api/v1/seamless-pipes", post(handler::pipes::create_seamless_pipe))
        .route("/api/v1/seamless-pipes/{id}", get(handler::pipes::get_seamless_pipe))
        .route("/api/v1/seamless-pipes/{id}", put(handler::pipes::update_seamless_pipe))
        .route("/api/v1/seamless-pipes/{id}", delete(handler::pipes::delete_seamless_pipe))
        .route("/api/v1/screen-pipes", get(handler::pipes::list_screen_pipes))
        .route("/api/v1/screen-pipes", post(handler::pipes::create_screen_pipe))
        .route("/api/v1/screen-pipes/{id}", get(handler::pipes::get_screen_pipe))
        .route("/api/v1/screen-pipes/{id}", put(handler::pipes::update_screen_pipe))
        .route("/api/v1/screen-pipes/{id}", delete(handler::pipes::delete_screen_pipe))
        // Inventory
        .route("/api/v1/inventory/inbound", post(handler::inventory::create_inbound))
        .route("/api/v1/inventory/inbound", get(handler::inventory::list_inbound))
        .route("/api/v1/inventory/inbound/{id}", get(handler::inventory::get_inbound))
        .route("/api/v1/inventory/outbound", post(handler::inventory::create_outbound))
        .route("/api/v1/inventory/outbound", get(handler::inventory::list_outbound))
        .route("/api/v1/inventory/outbound/{id}", get(handler::inventory::get_outbound))
        .route("/api/v1/inventory/stock", get(handler::inventory::get_stock_summary))
        .route("/api/v1/inventory/inventory-check", post(handler::inventory::create_inventory_check))
        .route("/api/v1/inventory/inventory-check", get(handler::inventory::list_inventory_checks))
        // Tracing
        .route("/api/v1/trace/pipe-number/{pipe_no}", get(handler::pipes::trace_by_pipe_number))
        .route("/api/v1/trace/heat-number/{heat_no}", get(handler::pipes::trace_by_heat_number))
        // Quality
        .route("/api/v1/quality/certs", get(handler::quality::list_certs))
        .route("/api/v1/quality/certs", post(handler::quality::create_cert))
        .route("/api/v1/quality/certs/{id}", get(handler::quality::get_cert))
        .route("/api/v1/quality/certs/{id}", put(handler::quality::update_cert))
        .route("/api/v1/quality/trace/heat-number/{heat_no}", get(handler::quality::trace_by_heat_number))
        .route("/api/v1/quality/trace/pipe-number/{pipe_no}", get(handler::quality::trace_by_pipe_number))
        .route("/api/v1/quality/api5ct/grades", get(handler::quality::list_grades))
        .route("/api/v1/quality/api5ct/grades/{grade}", get(handler::quality::get_grade))
        .route("/api/v1/quality/attachments", post(handler::quality::upload_attachment))
        .route("/api/v1/quality/attachments/{id}", delete(handler::quality::delete_attachment))
        // Suppliers
        .route("/api/v1/suppliers", get(handler::purchases::list_suppliers))
        .route("/api/v1/suppliers", post(handler::purchases::create_supplier))
        .route("/api/v1/suppliers/{id}", put(handler::purchases::update_supplier))
        .route("/api/v1/suppliers/{id}", delete(handler::purchases::delete_supplier))
        // Customers
        .route("/api/v1/customers", get(handler::sales::list_customers))
        .route("/api/v1/customers", post(handler::sales::create_customer))
        .route("/api/v1/customers/{id}", put(handler::sales::update_customer))
        .route("/api/v1/customers/{id}", delete(handler::sales::delete_customer))
        // Purchase Orders
        .route("/api/v1/purchase-orders", get(handler::purchases::list_purchase_orders))
        .route("/api/v1/purchase-orders", post(handler::purchases::create_purchase_order))
        .route("/api/v1/purchase-orders/{id}", get(handler::purchases::get_purchase_order))
        .route("/api/v1/purchase-orders/{id}/approve", put(handler::purchases::approve_purchase_order))
        .route("/api/v1/purchase-orders/{id}/cancel", put(handler::purchases::cancel_purchase_order))
        .route("/api/v1/purchase-orders/{id}/link-inbound", put(handler::purchases::link_inbound_to_po))
        // Sales Orders
        .route("/api/v1/sales-orders", get(handler::sales::list_sales_orders))
        .route("/api/v1/sales-orders", post(handler::sales::create_sales_order))
        .route("/api/v1/sales-orders/{id}", get(handler::sales::get_sales_order))
        .route("/api/v1/sales-orders/{id}/approve", put(handler::sales::approve_sales_order))
        .route("/api/v1/sales-orders/{id}/cancel", put(handler::sales::cancel_sales_order))
        .route("/api/v1/sales-orders/atp", get(handler::sales::get_atp))
        .route("/api/v1/sales-orders/{id}/link-outbound", put(handler::sales::link_outbound_to_so))
        // Data Import/Export
        .route("/api/v1/import/seamless-pipes", post(handler::data_io::import_seamless_pipes))
        .route("/api/v1/import/screen-pipes", post(handler::data_io::import_screen_pipes))
        .route("/api/v1/import/template/seamless-pipes", get(handler::data_io::download_seamless_template))
        .route("/api/v1/import/template/screen-pipes", get(handler::data_io::download_screen_template))
        .route("/api/v1/export/inventory", post(handler::data_io::export_inventory))
        .route("/api/v1/export/inbound", post(handler::data_io::export_inbound))
        .route("/api/v1/export/outbound", post(handler::data_io::export_outbound))
        .route("/api/v1/export/pipes", post(handler::data_io::export_pipes))
        // Contracts
        .route("/api/v1/contracts", get(handler::contracts::list_contracts))
        .route("/api/v1/contracts", post(handler::contracts::create_contract))
        .route("/api/v1/contracts/{id}", get(handler::contracts::get_contract))
        .route("/api/v1/contracts/{id}", put(handler::contracts::update_contract))
        .route("/api/v1/contracts/{id}/status", put(handler::contracts::update_contract_status))
        .route("/api/v1/contracts/{id}", delete(handler::contracts::delete_contract))
        .route("/api/v1/contracts/{id}/payments", post(handler::contracts::add_payment))
        .route("/api/v1/contracts/{id}/payments/{payment_id}", put(handler::contracts::update_payment))
        .route("/api/v1/contracts/{id}/payments/{payment_id}", delete(handler::contracts::delete_payment))
        // Reports
        .route("/api/v1/reports/stock-summary", get(handler::reports::stock_summary))
        .route("/api/v1/reports/stock-by-grade", get(handler::reports::stock_by_grade))
        .route("/api/v1/reports/stock-by-location", get(handler::reports::stock_by_location))
        .route("/api/v1/reports/inbound-summary", get(handler::reports::inbound_summary))
        .route("/api/v1/reports/outbound-summary", get(handler::reports::outbound_summary))
        .route("/api/v1/reports/monthly-flow", get(handler::reports::monthly_flow))
        .route("/api/v1/reports/purchase-summary", get(handler::reports::purchase_summary))
        .route("/api/v1/reports/sales-summary", get(handler::reports::sales_summary))
        .route("/api/v1/reports/financial-monthly", get(handler::reports::financial_monthly))
        // Labels
        .route("/api/v1/label-templates", get(handler::labels::list_templates))
        .route("/api/v1/label-templates", post(handler::labels::create_template))
        .route("/api/v1/label-templates/{id}", put(handler::labels::update_template))
        .route("/api/v1/label-templates/{id}", delete(handler::labels::delete_template))
        .route("/api/v1/labels/generate", post(handler::labels::generate_labels))
        .route("/api/v1/labels/print-history", get(handler::labels::print_history))
        // Auth management
        .route("/api/v1/auth/me", get(handler::auth::me))
        .route("/api/v1/auth/users", get(handler::auth::list_users))
        .route("/api/v1/auth/users", post(handler::auth::create_user))
        .route("/api/v1/auth/users/{id}", put(handler::auth::update_user))
        .route("/api/v1/auth/users/{id}", delete(handler::auth::delete_user))
        .layer(axum_middleware::from_fn_with_state(state.clone(), middleware::auth_middleware));

    

    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
